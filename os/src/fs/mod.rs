mod pipe;
mod stdio;
mod inode;
mod dir;

use crate::mm::UserBuffer;

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