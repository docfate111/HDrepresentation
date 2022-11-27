pub use serde::{Deserialize, Serialize};
pub use std::ffi::CStr;
pub use std::fmt;

/// wrap the value in a type
/// include a size for UCharPtr
#[derive(Serialize, Eq, Clone, Deserialize, Debug, PartialEq)]
pub enum VariableType {
    Long(i64),
    Str(String),
    UCharPtr(Option<Vec<u8>>, u32),
    VoidPtr,
    MmapBase,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum FileType {
    None,
    File,
    Dir,
    Symlink,
    Fifo,
    Mmap,
    Unknown,
}
