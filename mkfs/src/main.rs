use std::fs::{File, OpenOptions, read_dir};
use std::io::{Read, Write, Seek, SeekFrom};
use std::sync::{ Mutex, Arc };
use std::ptr;
use clap::{Arg, App};

pub mod fs;
pub use fs::*;

// use fs::{
//     BlockDevice,
//     FAT32Manager,
//     VFile,
//     ShortDirEntry,
//     ATTRIBUTE_ARCHIVE,
//     ATTRIBUTE_DIRECTORY
// };

use crate::init::{init_boot, init_fat, init_fsinfo, init_root};

mod init;


const BSIZE: usize = 512;


// [Boot | FAT | Root Dir Sector | Data ]

pub struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let offset = (block_id * BSIZE) as u64;
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BSIZE, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let offset = (block_id * BSIZE) as u64;
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BSIZE, "Not a complete block!");
    }
}

fn main() {
    make().unwrap();
}

fn make() -> std::io::Result<()> {
    let matches = App::new("EasyFileSystem packer")
        .arg(Arg::with_name("source")
            .short("s") // 对应输入的 -s
            .long("source")//对应输入 --source
            .takes_value(true)
            .help("Executable source dir(with backslash)")
        )
        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .takes_value(true)
            .help("Executable target dir(with backslash)")    
        )
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    println!("src_path = {}\ntarget_path = {}", src_path, target_path);
    
    // 打开U盘
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}{}", target_path, "fs.img"))?;
        f.set_len(8192 * 512).unwrap();
        f
    })));

    // 初始化文件系统
    // 应当首先初始化 boot_secotr, fs_info, FAT 表，根目录目录项
    // 随后将根目录下的文件或目录作为 entry 写入根目录 sector 中

    init_boot(block_file.clone());
    init_fsinfo(block_file.clone());
    init_fat(block_file.clone());
    init_root(block_file.clone());
    
    let fs_manager = FAT32Manager::create(block_file.clone());
    let fs_reader = fs_manager.read();
    let root_vfile = fs_reader.get_root_vfile(&fs_manager);
    println!("first date sec = {}", fs_reader.first_data_sector());
    drop(fs_reader);

    // 从host获取应用名
    let apps: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            // 丢弃后缀 从'.'到末尾(len-1)
            println!("name_with_ext: {}", name_with_ext);
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    for app in apps {
       
        // 获取所有用户可执行程序
        let mut host_file = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        // 将用户程序写入缓冲区
        host_file.read_to_end(&mut all_data).unwrap();

        // 创建一个FAT32文件
        let o_vfile = root_vfile.create(app.as_str(), ATTRIBUTE_ARCHIVE);
        if o_vfile.is_none() {
            // println!("vfile is none.\n");
            continue;
        }
        let vfile = o_vfile.unwrap();
        println!("vfile: name {}, short_sector {}, short_offset {}", vfile.get_name(), vfile.short_sector, vfile.short_offset);
        println!("vfile is dir: {}", vfile.is_dir());

        // 向文件镜像中写入数据
        println!("file_len = {}", all_data.len());
        
        vfile.write_at(0, all_data.as_slice());
        fs_manager.read().cache_write_back();
        
        
    }
    // list apps

    for app in root_vfile.ls_lite().unwrap() {
        println!("{}", app.0);
    }
    Ok(())
}
