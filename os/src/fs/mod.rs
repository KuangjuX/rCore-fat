mod pipe;
mod stdio;
mod inode;
mod dir;

use crate::mm::UserBuffer;
use alloc::sync::Arc;

pub struct FileDescriptor {
    pub ftype: FileType
}

/// 文件类型
pub enum FileType {
    File(Arc<OSInode>),
    Abstr(Arc<dyn File>)
}

pub trait File : Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
pub use inode::{OSInode, open_file, OpenFlags, list_apps};
pub use dir::{ DirEntry, DT_DIR, DT_REG, DT_UNKNOWN };