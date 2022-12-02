pub use crate::arg::*;

#[derive(Debug, Copy, Eq, Serialize, Deserialize, PartialEq, Clone)]
pub enum SysNo {
    Open,
    Read,
    Write,
    Lseek,
    Getdents,
    Pread,
    Pwrite,
    Fstat,
    Stat, // need to go add these
    Lstat,
    Rename,
    Fsync,
    Fdatasync,
    Syncfs,
    Sendfile,
    Access,
    Ftruncate,
    Truncate,
    Mkdir,
    Rmdir,
    Link,
    Unlink,
    Symlink,
    Setxattr,
    Getxattr,
    Removexattr,
    Listxattr,
}

pub fn num_to_name(nr: SysNo) -> String {
    let x = match nr {
        SysNo::Open => "SYS_open",
        SysNo::Write => "SYS_write",
        SysNo::Lseek => "SYS_lseek",
        SysNo::Pread => "SYS_pread64",
        SysNo::Getdents => "SYS_getdents64",
        SysNo::Pwrite => "SYS_pwrite64",
        SysNo::Stat => "SYS_stat",
        SysNo::Fstat => "SYS_fstat",
        SysNo::Lstat => "SYS_lstat",
        SysNo::Rename => "SYS_rename",
        SysNo::Fdatasync => "SYS_fdatasync",
        SysNo::Fsync => "SYS_fsync",
        SysNo::Syncfs => "SYS_syncfs",
        SysNo::Sendfile => "SYS_sendfile",
        SysNo::Access => "SYS_access",
        SysNo::Ftruncate => "SYS_ftruncate",
        SysNo::Truncate => "SYS_truncate",
        SysNo::Mkdir => "SYS_mkdir",
        SysNo::Rmdir => "SYS_rmdir",
        SysNo::Link => "SYS_link",
        SysNo::Unlink => "SYS_unlink",
        SysNo::Symlink => "SYS_symlink",
        SysNo::Read => "SYS_read",
        SysNo::Setxattr => "SYS_setxattr",
        SysNo::Getxattr => "SYS_getxattr",
        SysNo::Listxattr => "SYS_listxattr",
        SysNo::Removexattr => "SYS_removexattr",
    };
    String::from(x)
}

impl fmt::Display for SysNo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", num_to_name(*self))
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub struct Syscall {
    pub nr: SysNo,
    pub args: Vec<Arg>,
    // if we save return value, it must be a
    // variable;
    // if we do not care the return value,
    // by default it is -1.
    pub ret_index: i64,
}

impl Syscall {
    pub fn new_with_index(nr: SysNo, ret_index: i64) -> Self {
        Self {
            nr,
            ret_index,
            args: Vec::<Arg>::new(),
        }
    }

    pub fn new(nr: SysNo) -> Self {
        Self::new_with_index(nr, -1)
    }

    pub fn add_arg(&mut self, value: i64, is_variable: bool) {
        self.args.push(Arg::new(value, is_variable));
    }
}


