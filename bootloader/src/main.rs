#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn efi_main(_handle: Handle, _st: SystemTable<Boot>) -> Status {
    Status::SUCCESS
}