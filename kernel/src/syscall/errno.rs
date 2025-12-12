// File: kernel/src/syscall/errno.rs
//! Errno definitions for Bulldog kernel.
//! Provides constants, encoding helper, and strerror mapping.

/// Canonical errno constants (Linux-compatible).
pub mod errno {
    pub const EPERM:   u64 = 1;   // Operation not permitted
    pub const ENOENT:  u64 = 2;   // No such file or directory
    pub const ESRCH:   u64 = 3;   // No such process
    pub const EINTR:   u64 = 4;   // Interrupted system call
    pub const EIO:     u64 = 5;   // I/O error
    pub const ENXIO:   u64 = 6;   // No such device or address
    pub const E2BIG:   u64 = 7;   // Argument list too long
    pub const ENOEXEC: u64 = 8;   // Exec format error
    pub const EBADF:   u64 = 9;   // Bad file descriptor
    pub const ECHILD:  u64 = 10;  // No child processes
    pub const EAGAIN:  u64 = 11;  // Try again
    pub const ENOMEM:  u64 = 12;  // Out of memory
    pub const EACCES:  u64 = 13;  // Permission denied
    pub const EFAULT:  u64 = 14;  // Bad address
    pub const ENOTBLK: u64 = 15;  // Block device required
    pub const EBUSY:   u64 = 16;  // Device or resource busy
    pub const EEXIST:  u64 = 17;  // File exists
    pub const EXDEV:   u64 = 18;  // Cross-device link
    pub const ENODEV:  u64 = 19;  // No such device
    pub const ENOTDIR: u64 = 20;  // Not a directory
    pub const EISDIR:  u64 = 21;  // Is a directory
    pub const EINVAL:  u64 = 22;  // Invalid argument
    pub const ENFILE:  u64 = 23;  // File table overflow
    pub const EMFILE:  u64 = 24;  // Too many open files
    pub const ENOTTY:  u64 = 25;  // Not a typewriter
    pub const ETXTBSY: u64 = 26;  // Text file busy
    pub const EFBIG:   u64 = 27;  // File too large
    pub const ENOSPC:  u64 = 28;  // No space left on device
    pub const ESPIPE:  u64 = 29;  // Illegal seek
    pub const EROFS:   u64 = 30;  // Read-only file system
    pub const EMLINK:  u64 = 31;  // Too many
    pub const EPIPE:   u64 = 32;  // Broken pipe
    pub const EDOM:    u64 = 33;  // Math argument out of domain of func
    pub const ERANGE:  u64 = 34;  // Math result not representable
    pub const EDEADLK: u64 = 35;  // Resource deadlock would occur
    pub const ENAMETOOLONG:u64 = 36; // File name too long
    pub const ENOLCK:  u64 = 37;  // No record locks available
    pub const ENOSYS:  u64 = 38;  // Function not implemented
    pub const ENOTEMPTY:u64 = 39;  // Directory not empty
    pub const ELOOP:   u64 = 40;  // Too many symbolic links encountered
    pub const ENOMSG:  u64 = 42;  // No message of desired type
    pub const EIDRM:   u64 = 43;  // Identifier removed
    pub const ECHRNG:  u64 = 44;  // Channel number out of range
    pub const EL2NSYNC:u64 = 45;  // Level 2 not synchronized
    pub const EL3HLT:  u64 = 46;  // Level 3 halted
    pub const EL3RST:  u64 = 47;  // Level 3 reset
    pub const ELNRNG: u64 = 48;  // Link number out of range
    pub const EUNATCH:u64 = 49;  // Protocol driver not attached
    pub const ENOCSI: u64 = 50;  // No CSI structure available
    pub const EL2HLT:u64 = 51;  // Level 2 halted
    pub const EBADE: u64 = 52;  // Invalid exchange
    pub const EBADR:  u64 = 53;  // Invalid request descriptor
    pub const EXFULL: u64 = 54;  // Exchange full
    pub const ENOANO: u64 = 55;  // No anode
    pub const EBADRQC:u64 = 56;  // Invalid request code
    pub const EBADSLT:u64 = 57;  // Invalid slot
    pub const EBFONT: u64 = 59;  // Bad font file format
    pub const ENOSTR: u64 = 60;  // Device not a stream
    pub const ENODATA:u64 = 61;  // No data available
    pub const ETIME:  u64 = 62;  // Timer expired
    pub const ENOSR:  u64 = 63;  // Out of streams
    pub const ENONET: u64 = 64;  // Machine is not on the network
    pub const ENOPKG: u64 = 65;  // Package not installed
    pub const EREMOTE:u64 = 66;  // Object is remote
    pub const ENOLINK:u64 = 67;  // Link has been severed
    pub const EADV:   u64 = 68;  // Advertise error
    pub const ESRMNT: u64 = 69;  // Srmount error
    pub const ECOMM:  u64 = 70;  // Communication error on send
    pub const EPROTO: u64 = 71;  // Protocol error
    pub const EMULTIHOP:u64 = 72; // Multihop attempted
    pub const EDOTDOT:u64 = 73;  // RFS specific error
    pub const EBADMSG:u64 = 74;  // Not a data message
    pub const EOVERFLOW:u64 = 75; // Value too large for defined data type
    pub const ENOTUNIQ:u64 = 76;  // Name not unique
    pub const EBADFD: u64 = 77;  // File descriptor in bad state
    pub const EREMCHG:u64 = 78;  // Remote address changed
    pub const ELIBACC:u64 = 79;  // Can not access a needed shared library
    pub const ELIBBAD:u64 = 80;  // Accessing a corrupted shared library
    pub const ELIBSCN:u64 = 81;  // .lib section in a.out corrupted
    pub const ELIBMAX:u64 = 82;  // Attempting to link in too many shared libraries
    pub const ELIBEXEC:u64 = 83; // Cannot exec a shared library directly
    pub const EILSEQ: u64 = 84;  // Illegal byte sequence
    pub const ERESTART:u64 = 85; // Interrupted system call should be restarted
    pub const ESTRPIPE:u64 = 86;  // Streams pipe error
    pub const EUSERS: u64 = 87;  // Too many users
    pub const ENOTSOCK:u64 = 88;  // Socket operation on non-socket
    pub const EDESTADDRREQ:u64 = 89; // Destination address required
    pub const EMSGSIZE:u64 = 90;  // Message too long
    pub const EPROTOTYPE:u64 = 91; // Protocol wrong type for socket
    pub const ENOPROTOOPT:u64 = 92; // Protocol not available
    pub const EPROTONOSUPPORT:u64 = 93; // Protocol not supported
    pub const ESOCKTNOSUPPORT:u64 = 94; // Socket type not supported
    pub const EOPNOTSUPP:u64 = 95; // Operation not supported on transport endpoint
    pub const EPFNOSUPPORT:u64 = 96; // Protocol family not supported
    pub const EAFNOSUPPORT:u64 = 97; // Address family not supported by protocol
    pub const EADDRINUSE:u64 = 98; // Address already in use
    pub const EADDRNOTAVAIL:u64 = 99; // Cannot assign requested address
    pub const ENETDOWN:u64 = 100; // Network is down
    pub const ENETUNREACH:u64 = 101; // Network is unreachable
    pub const ENETRESET:u64 = 102; // Network dropped connection because of reset
    pub const ECONNABORTED:u64 = 103; // Software caused connection abort
    pub const ECONNRESET:u64 = 104; // Connection reset by peer
    pub const ENOBUFS:u64 = 105; // No buffer space available
    pub const EISCONN:u64 = 106; // Transport endpoint is already connected
    pub const ENOTCONN:u64 = 107; // Transport endpoint is not connected
    pub const ESHUTDOWN:u64 = 108; // Cannot send after transport endpoint shutdown
    pub const ETOOMANYREFS:u64 = 109; // Too many references: cannot splice
    pub const ETIMEDOUT:u64 = 110; // Connection timed out
    pub const ECONNREFUSED:u64 = 111; // Connection refused
    pub const EHOSTDOWN:u64 = 112; // Host is down
    pub const EHOSTUNREACH:u64 = 113; // No route to host
    pub const EALREADY:u64 = 114; // Operation already in progress
    pub const EINPROGRESS:u64 = 115; // Operation now in progress
    pub const ESTALE:u64 = 116; // Stale file handle
    pub const EUCLEAN:u64 = 117; // Structure needs cleaning
    pub const ENOTNAM:u64 = 118; // Not a XENIX named type file
    pub const ENAVAIL:u64 = 119; // No XENIX semaphores available
    pub const EISNAM:u64 = 120; // Is a XENIX named type file
    pub const EREMOTEIO:u64 = 121; // Remote I/O error
    pub const EDQUOT:u64 = 122; // Quota exceeded
    pub const ENOMEDIUM:u64 = 123; // No medium found
    pub const EMEDIUMTYPE:u64 = 124; // Wrong medium type
    pub const ECANCELED:u64 = 125; // Operation canceled
    pub const ENOKEY:u64 = 126; // Required key not available
    pub const EKEYEXPIRED:u64 = 127; // Key has expired
    pub const EKEYREVOKED:u64 = 128; // Key has been revoked
    pub const EKEYREJECTED:u64 = 129; // Key was rejected by service
    pub const EOWNERDEAD:u64 = 130; // Owner died
    pub const ENOTRECOVERABLE:u64 = 131; // State not recoverable
    pub const ERFKILL:u64 = 132; // Operation not possible due to RFKill  
    pub const EHWPOISON:u64 = 133; // Memory page has hardware error
}

/// Encode errno as a negative return value (Linux convention).
#[inline(always)]
pub fn err(errno: u64) -> u64 {
    (-(errno as i64)) as u64
}

/// Minimal strerror mapping for common errors.
/// Extend incrementally as Bulldog grows.
pub fn strerror(errno: u64) -> &'static str {
    match errno {
        errno::EPERM   => "Operation not permitted",
        errno::ENOENT  => "No such file or directory",
        errno::EBADF   => "Bad file descriptor",
        errno::EAGAIN  => "Try again",
        errno::ENOMEM  => "Out of memory",
        errno::EACCES  => "Permission denied",
        errno::EFAULT  => "Bad address",
        errno::EINVAL  => "Invalid argument",
        errno::ENOSYS  => "Function not implemented",
        _              => "Unknown error",
    }
}


