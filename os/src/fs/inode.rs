use crate::drivers::BLOCK_DEVICE;
use crate::mm::UserBuffer;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::*;
use lazy_static::*;
use spin::Mutex;

use super::{DirEntry, File, DT_DIR, DT_REG, DT_UNKNOWN};
use FAT32::{FAT32Manager, VFile, ATTRIBUTE_ARCHIVE, ATTRIBUTE_DIRECTORY};

pub enum DiskInodeType {
    File,
    Directory,
}

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: Mutex<OSInodeInner>,
}

pub struct OSInodeInner {
    /// 当前读写位置
    offset: usize,
    inode: Arc<VFile>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<VFile>) -> Self {
        Self {
            readable,
            writable,
            inner: Mutex::new(OSInodeInner { offset: 0, inode }),
        }
    }

    pub fn is_dir(&self) -> bool {
        let inner = self.inner.lock();
        inner.inode.is_dir()
    }

    pub fn read_vec(&self, offset: isize, len: usize) -> Vec<u8> {
        let mut inner = self.inner.lock();
        let mut len = len;
        let ori_off = inner.offset;
        if offset >= 0 {
            inner.offset = offset as usize;
        }
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let rlen = inner.inode.read_at(inner.offset, &mut buffer);
            if rlen == 0 {
                break;
            }
            inner.offset += rlen;
            v.extend_from_slice(&buffer[..rlen.min(len)]);
            if len > rlen {
                len -= rlen;
            } else {
                break;
            }
        }
        if offset >= 0 {
            inner.offset = ori_off;
        }
        v
    }

    /// 从文件中读出信息放入缓冲区中
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.lock();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }

    pub fn write_all(&self, str_vec: &Vec<u8>) -> usize {
        let mut inner = self.inner.lock();
        let mut remain = str_vec.len();
        let mut base = 0;
        loop {
            let len = remain.min(512);
            inner
                .inode
                .write_at(inner.offset, &str_vec.as_slice()[base..base + len]);
            inner.offset += len;
            base += len;
            remain -= len;
            if remain == 0 {
                break;
            }
        }
        return base;
    }

    pub fn find(&self, path: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
        let inner = self.inner.lock();
        let mut new_path: Vec<&str> = path.split('/').collect();
        let vfile = inner.inode.find_vfile_bypath(new_path);
        if vfile.is_none() {
            return None;
        } else {
            let (readable, writable) = flags.read_write();
            return Some(Arc::new(OSInode::new(readable, writable, vfile.unwrap())));
        }
    }

    pub fn get_dirent(&self, dir_entry: &mut DirEntry) -> Option<usize> {
        let mut inner = self.inner.lock();
        let offset = inner.offset as u32;
        if let Some((name, off, first_clu, attri)) = inner.inode.dirent_info(offset as usize) {
            let mut dtype: u8 = 0;
            if attri & ATTRIBUTE_DIRECTORY != 0 {
                dtype = DT_DIR;
            } else if attri & ATTRIBUTE_ARCHIVE != 0 {
                dtype = DT_REG;
            } else {
                dtype = DT_UNKNOWN;
            }
            dir_entry.set(
                name.as_str(),
                first_clu as usize,
                (off - offset) as isize,
                name.len() as u16,
                dtype,
            );
            inner.offset = off as usize;
            let len = name.len() + 8 * 4;
            Some(len)
        } else {
            None
        }
    }

    pub fn get_size(&self) -> usize {
        let inner = self.inner.lock();
        let (size, _, mt_me, _, _) = inner.inode.stat();
        return size as usize;
    }
}

// lazy_static! {
//     pub static ref ROOT_INODE: Arc<Inode> = {
//         let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
//         Arc::new(EasyFileSystem::root_inode(&efs))
//     };
// }

lazy_static! {
    pub static ref ROOT_INODE: Arc<VFile> = {
        let fat32_manager = FAT32Manager::open(BLOCK_DEVICE.clone());
        let manager_reader = fat32_manager.read();
        Arc::new(manager_reader.get_root_vfile(&fat32_manager))
    };
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls_lite().unwrap() {
        if app.1 & ATTRIBUTE_DIRECTORY == 0 {
            println!("{}", app.0);
        }
    }
    println!("**************/")
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
        const DIRECTROY = 0200000;
        const LARGEFILE  = 0100000;
        const CLOEXEC = 02000000;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

// pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
//     let (readable, writable) = flags.read_write();
//     if flags.contains(OpenFlags::CREATE) {
//         if let Some(inode) = ROOT_INODE.find(name) {
//             // clear size
//             inode.clear();
//             Some(Arc::new(OSInode::new(
//                 readable,
//                 writable,
//                 inode,
//             )))
//         } else {
//             // create file
//             ROOT_INODE.create(name)
//                 .map(|inode| {
//                     Arc::new(OSInode::new(
//                         readable,
//                         writable,
//                         inode,
//                     ))
//                 })
//         }
//     } else {
//         ROOT_INODE.find(name)
//             .map(|inode| {
//                 if flags.contains(OpenFlags::TRUNC) {
//                     inode.clear();
//                 }
//                 Arc::new(OSInode::new(
//                     readable,
//                     writable,
//                     inode
//                 ))
//             })
//     }
// }

pub fn open(
    work_path: &str,
    path: &str,
    flags: OpenFlags,
    dtype: DiskInodeType,
) -> Option<Arc<OSInode>> {
    // 找到当前路径的inode(file, directory)
    let cur_inode = {
        if work_path == "/" {
            ROOT_INODE.clone()
        } else {
            let wpath: Vec<&str> = work_path.split('/').collect();
            ROOT_INODE.find_vfile_bypath(wpath).unwrap()
        }
    };
    let mut pathv: Vec<&str> = path.split('/').collect();

    let (readable, writeable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = cur_inode.find_vfile_bypath(pathv.clone()) {
            inode.remove();
        }
        {
            // create file
            let name = pathv.pop().unwrap();
            if let Some(temp_inode) = cur_inode.find_vfile_bypath(pathv.clone()) {
                let attribute = {
                    match dtype {
                        DiskInodeType::Directory => ATTRIBUTE_DIRECTORY,
                        DiskInodeType::File => ATTRIBUTE_ARCHIVE,
                    }
                };
                temp_inode
                    .create(name, attribute)
                    .map(|inode| Arc::new(OSInode::new(readable, writeable, inode)))
            } else {
                None
            }
        }
    } else {
        cur_inode.find_vfile_bypath(pathv).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writeable, inode))
        })
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}
