use std::sync::Arc;
use std::ptr;
use FAT32::{ATTRIBUTE_DIRECTORY, BLOCK_SZ, BlockDevice, DATA_SIZE, FAT_SIZE, FatBS, FatExtBS, LEAD_SIGNATURE, SECOND_SIGNATURE, SECTOR_SIZE, ShortDirEntry};
use super::BlockFile;

// Fat32文件系统，block大小（即 sector 大小为 512 bytes）
// 我们暂时将 1 cluster 设定为 1 sector
// BiosParamter: 0 sector
// Fs info: 1 sector
// FAT1: 2-401 sector
// FAT2: 402-803 sector
// 804-805 unused sector
// RootDir: 806 sector


pub fn init_boot(block_device: Arc<BlockFile>) {
    let fat_bs = FatBS {
        unused: [0u8; 11],
            bytes_per_sector: BLOCK_SZ as u16,
            sectors_per_cluster: 1,
            reserved_sector_count: 2,
            table_count: 2,
            root_entry_count: 0,
            total_sectors_16: 0,
            media_type: 0, 
            table_size_16: 0,
            sectors_per_track: 0, 
            head_side_count: 0, 
            hidden_sector_count: 0, 
            total_sectors_32: SECTOR_SIZE as u32
    };
    let fat_ext_bs = FatExtBS {
        table_size_32: FAT_SIZE as u32,
        extended_flags: 0,
        fat_version: 0,
        root_clusters: 2,
        fat_info: 1,
        backup_bs_sector: 0,
        reserved_0: [0u8; 12],
        drive_number: 0x80,
        reserved_1: 0,
        boot_signature: 0
    };
    let mut buf = [0u8; BLOCK_SZ];
    unsafe{
        ptr::write(buf.as_mut_ptr() as *mut FatBS, fat_bs);
        ptr::write(buf.as_mut_ptr().offset(36) as *mut FatExtBS, fat_ext_bs);
    }
    block_device.write_block(0, &buf);
}

pub fn init_fsinfo(block_device: Arc<BlockFile>) {
    let mut buf = [0u8; 512];
    unsafe{
        ptr::write(buf.as_mut_ptr() as *mut u32, LEAD_SIGNATURE);
        ptr::write(buf.as_mut_ptr().offset(484) as *mut u32, SECOND_SIGNATURE);
        // 空闲块
        ptr::write(buf.as_mut_ptr().offset(488) as *mut u32, DATA_SIZE as u32);
    }
    block_device.write_block(1, &buf);
}

pub fn init_fat(block_device: Arc<BlockFile>) {
    // FAT 表项的0号表项和1号表项无用
    // 2号表项应当记录根目录
    let mut buf = [0u8; 512];
    unsafe{
        ptr::write(buf.as_mut_ptr() as *mut u64, 0xFFFFFFFFFFFFFFFF);
        // 根目录此时size大小为0，因此没有后续的簇号记录
        ptr::write(buf.as_mut_ptr().offset(8) as *mut u32, 0x0FFFFFFF);
    }
    block_device.write_block(2, &buf);
}

/// 这里需要初始化root directory
pub fn init_root(block_device: Arc<BlockFile>) {
    let mut buf = [0u8; 512];
    let mut root_dir = ShortDirEntry::new(
        &[0x2F,0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20], 
        &[0x20, 0x20, 0x20], 
        ATTRIBUTE_DIRECTORY
    );
    root_dir.set_first_cluster(2);
    unsafe{
        ptr::write(buf.as_mut_ptr() as *mut ShortDirEntry, root_dir);
    }

    block_device.write_block(10, &buf);
    // 之后需要初始化根目录中的文件或者目录
}