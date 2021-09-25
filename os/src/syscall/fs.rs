use crate::fs::{make_pipe, open, DiskInodeType, File, FileDescriptor, FileType, OpenFlags};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let f: Arc<dyn File + Send + Sync> = match &file.ftype {
            FileType::Abstr(f) => f.clone(),
            FileType::File(f) => f.clone(),
            _ => return -1,
        };
        if !f.writable() {
            return -1;
        }

        drop(inner);
        let size = f.write(UserBuffer::new(translated_byte_buffer(token, buf, len)));
        if fd == 2 {
            let str = str::replace(translated_str(token, buf).as_str(), "\n", "\\n");
            println!(
                "sys_write(fd: {}, buf: {}, len: {}) = {}",
                fd, str, len, size
            );
        } else if fd > 2 {
            println!("sys_write(fd: {}, buf: {}, len: {}", fd, len, size);
        }
        size as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file: Arc<dyn File + Send + Sync> = match &file.ftype {
            FileType::Abstr(f) => f.clone(),
            FileType::File(f) => f.clone(),
            _ => return -1,
        };
        if !file.readable() {
            return -1;
        }
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    let open_flags = OpenFlags::from_bits(flags).unwrap();
    let mut inner = task.acquire_inner_lock();
    if let Some(inode) = open(
        inner.get_work_path().as_str(),
        path.as_str(),
        open_flags,
        DiskInodeType::File,
    ) {
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(FileDescriptor::new(
            open_flags.contains(OpenFlags::CLOEXEC),
            FileType::File(inode),
        ));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(FileDescriptor::new(false, FileType::Abstr(pipe_read)));
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(FileDescriptor::new(false, FileType::Abstr(pipe_write)));
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(inner.fd_table[fd].as_ref().unwrap().clone());
    new_fd as isize
}
