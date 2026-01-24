// These errno values must match kernel/src/syscall/errno.rs.
// They are duplicated intentionally because kernel and user space
// cannot share Rust code or crates.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Errno {
    EBADF,
    EFAULT,
    EINVAL,
    ENOSYS,
    ENOMEM,
    EMFILE,
    Unknown(i64),
}

impl Errno {
    pub fn from_raw(raw: i64) -> Self {
        match raw {
            9  => Errno::EBADF,
            14 => Errno::EFAULT,
            22 => Errno::EINVAL,
            38 => Errno::ENOSYS,
            12 => Errno::ENOMEM,
            24 => Errno::EMFILE,
            x  => Errno::Unknown(x),
        }
    }
}

pub type SysResult<T> = Result<T, Errno>;