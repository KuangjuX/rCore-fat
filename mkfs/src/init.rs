use std::sync::Arc;
use std::ptr;
use FAT32::{
    BlockDevice,
    FatBS,
    FatExtBS,
    FSInfo,
    SECTOR_SIZE,
    BLOCK_SZ,
    FAT_SIZE,
    LEAD_SIGNATURE,
    SECOND_SIGNATURE
};
use super::BlockFile;


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
    }
    block_device.write_block(1, &buf);
}