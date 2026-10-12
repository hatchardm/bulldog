use crate::fs::block::RamDisk;
use alloc::vec::Vec;
use spin::Once;

static GLOBAL_BPB: Once<FatBpb> = Once::new();
static GLOBAL_FAT: Once<FatTable> = Once::new();


pub struct FatBpb {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    pub fat_size_sectors: u32,
    pub root_cluster: u32,
}

impl FatBpb {
    pub fn read(disk: &RamDisk) -> Self {
        let mut sector = [0u8; 512];
        disk.read_sector(0, &mut sector);

        let bytes_per_sector = u16::from_le_bytes([sector[11], sector[12]]);
        let sectors_per_cluster = sector[13];
        let reserved_sectors = u16::from_le_bytes([sector[14], sector[15]]);
        let num_fats = sector[16];
        let fat_size_sectors = u32::from_le_bytes([
            sector[36], sector[37], sector[38], sector[39]
        ]);
        let root_cluster = u32::from_le_bytes([
            sector[44], sector[45], sector[46], sector[47]
        ]);

        FatBpb {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            num_fats,
            fat_size_sectors,
            root_cluster,
        }
    }
}

pub struct FatTable {
    pub entries: Vec<u32>,
}

impl FatTable {
    pub fn read(disk: &RamDisk, bpb: &FatBpb) -> Self {
        let fat_start_lba = bpb.reserved_sectors as u64;
        let fat_sectors = bpb.fat_size_sectors as usize;

        let mut entries = Vec::with_capacity((fat_sectors * 512) / 4);

        let mut sector = [0u8; 512];

        for i in 0..fat_sectors {
            disk.read_sector(fat_start_lba + i as u64, &mut sector);

            // FAT32 entries are 32-bit little-endian
            for chunk in sector.chunks_exact(4) {
                let val = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                entries.push(val);
            }
        }

        FatTable { entries }
    }

    pub fn next_cluster(&self, cluster: u32) -> Option<u32> {
        let val = self.entries[cluster as usize];

        // FAT32 end-of-chain markers are >= 0x0FFFFFF8
        if val >= 0x0FFFFFF8 {
            None
        } else {
            Some(val)
        }
    }
}


pub fn set_global_fat(bpb: FatBpb, fat: FatTable) {
    GLOBAL_BPB.call_once(|| bpb);
    GLOBAL_FAT.call_once(|| fat);
}

pub fn get_global_bpb() -> &'static FatBpb {
    GLOBAL_BPB.get().expect("GLOBAL_BPB not initialized")
}

pub fn get_global_fat() -> &'static FatTable {
    GLOBAL_FAT.get().expect("GLOBAL_FAT not initialized")
}

#[derive(Debug, Clone)]
pub struct FatDirEntry {
    pub name: [u8; 11],     // 8.3 name
    pub attr: u8,           // attribute byte
    pub cluster_low: u16,
    pub cluster_high: u16,
    pub size: u32,
}

impl FatDirEntry {
    pub fn cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) | (self.cluster_low as u32)
    }
}

pub fn read_directory(disk: &RamDisk, bpb: &FatBpb, fat: &FatTable, start_cluster: u32) -> Vec<FatDirEntry> {
    let mut entries = Vec::new();

    let mut cluster = start_cluster;

    loop {
        // Compute LBA of this cluster
        let first_data_sector =
            bpb.reserved_sectors as u32 +
            (bpb.num_fats as u32 * bpb.fat_size_sectors);

        let sector_of_cluster =
            first_data_sector +
            ((cluster - 2) * bpb.sectors_per_cluster as u32);

        // Read all sectors in this cluster
        for s in 0..bpb.sectors_per_cluster {
            let mut buf = [0u8; 512];
            disk.read_sector((sector_of_cluster + s as u32) as u64, &mut buf);

            // Each directory entry is 32 bytes
            for chunk in buf.chunks_exact(32) {
                let name = <[u8; 11]>::try_from(&chunk[0..11]).unwrap();
                let attr = chunk[11];

                // 0x00 = end of directory
                if name[0] == 0x00 {
                    return entries;
                }

                // 0xE5 = deleted entry
                if name[0] == 0xE5 {
                    continue;
                }

                // Skip LFN entries (attr == 0x0F)
                if attr == 0x0F {
                    continue;
                }

                let cluster_low = u16::from_le_bytes([chunk[26], chunk[27]]);
                let cluster_high = u16::from_le_bytes([chunk[20], chunk[21]]);
                let size = u32::from_le_bytes([chunk[28], chunk[29], chunk[30], chunk[31]]);

                entries.push(FatDirEntry {
                    name,
                    attr,
                    cluster_low,
                    cluster_high,
                    size,
                });
            }
        }

        // Follow cluster chain
        match fat.next_cluster(cluster) {
            Some(next) => cluster = next,
            None => break,
        }
    }

    entries
}


/// Read a file starting at `start_cluster` into a `Vec<u8>`.
/// `size` is the file size from the directory entry (bytes).
pub fn read_file(
    disk: &RamDisk,
    bpb: &FatBpb,
    fat: &FatTable,
    start_cluster: u32,
    size: usize,
) -> Result<Vec<u8>, ()> {
    let mut out = Vec::with_capacity(size);
    let mut cluster = start_cluster;

    let first_data_sector =
        bpb.reserved_sectors as u32 + (bpb.num_fats as u32 * bpb.fat_size_sectors);

    let bytes_per_cluster =
        bpb.bytes_per_sector as usize * bpb.sectors_per_cluster as usize;

    loop {
        let sector_of_cluster =
            first_data_sector + ((cluster - 2) * bpb.sectors_per_cluster as u32);

        for s in 0..bpb.sectors_per_cluster {
            let mut buf = [0u8; 512];
            disk.read_sector((sector_of_cluster + s as u32) as u64, &mut buf);

            let remaining = size.saturating_sub(out.len());
            if remaining == 0 {
                return Ok(out);
            }

            let take = remaining.min(buf.len());
            out.extend_from_slice(&buf[..take]);
        }

        if out.len() >= size {
            out.truncate(size);
            return Ok(out);
        }

        match fat.next_cluster(cluster) {
            Some(next) => cluster = next,
            None => {
                out.truncate(size);
                return Ok(out);
            }
        }
    }
}