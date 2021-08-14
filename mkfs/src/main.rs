use std::fs::{File, OpenOptions, read_dir};
use std::io::{Read, Write, Seek, SeekFrom};
use std::sync::{ Mutex, Arc };
use std::ptr;
use clap::{Arg, App};

use fat32::BlockDevice;
use fat32::BIOSParameterBlock;
use fat32::{ reverse_u16, reverse_u32 };
use fat32::FAT;

const BSIZE: usize = 512;

const FSSIZE: usize = 8192;

const FATSIZE: usize = 32;

// [Boot | FAT | Root Dir Sector | Data]

#[derive(Clone)]
struct BlockFile(Arc<Mutex<File>>);

impl BlockDevice for BlockFile {
    fn read(&self, block_id: usize, buf: &mut [u8]) {
        let offset = (block_id * BSIZE) as u64;
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BSIZE, "Not a complete block!");
    }

    fn write(&self, block_id: usize, buf: &[u8]) {
        let offset = (block_id * BSIZE) as u64;
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BSIZE, "Not a complete block!");
    }
}

impl BlockFile {
    fn write_bpb(&self, bpb: &BIOSParameterBlock) {
        let mut buf: Vec<u8> = vec![0; BSIZE];
        unsafe{
            ptr::write(buf.as_mut_ptr() as *mut BIOSParameterBlock, *bpb);
        }
    }
}

impl BlockFile {
    fn zero(&self) {
        let buf = vec![0;BSIZE];
        for i in 0..FSSIZE {
            self.write(i, &buf);
        }
    }
}

fn main() {
    let matches = App::new("EasyFileSystem packer")
    .arg(Arg::with_name("source")
        .short("s")
        .long("source")
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

    let device = BlockFile(
        Arc::new(
            Mutex::new(
                    OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(format!("{}{}", target_path, "fs.img"))
                    .expect("Fail to open fs.img")
            )
        )
    );
    
    // Initialize fs.img
    device.zero();

    // Initialize bpb in fs.img
    let mut bpb = BIOSParameterBlock::uninit();
    bpb.reserved_sector = 1;
    bpb.num_fat = 1;
    bpb.total_sector = FSSIZE as u32;
    bpb.root_cluster = 2;
    device.write_bpb(&bpb);


    let fat1 = bpb.fat1();
    let clusters = bpb.count_of_clusters();

    // Initlizate File Allocation Table
    let mut fat = FAT::new(1, 0, Arc::new(device.clone()));
    fat.fat_offset = fat1;
    for cluster in 0..= clusters {
        fat.write(cluster as u32, 0xFFFFFFFF);
    }

    // Initialize Root Dir Sector

    // initialize Data Sector

}
