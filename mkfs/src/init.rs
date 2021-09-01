use std::sync::Arc;
use FAT32::{
    BlockDevice,
    FatBS,
    FatExtBS,
    FSInfo,
    SECTOR_SIZE,
    BLOCK_SZ,
    FAT_SIZE
};

pub fn init_boot(block_device: Arc<dyn BlockDevice>) {
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
}