use rust_project::device::block_device::MemoryBlockDevice;
use rust_project::fs::boot_sector::{BootSector, BootSectorError};

fn make_valid_boot_sector() -> [u8; 512] {
    let mut bs = [0u8; 512];

    // bytes per sector = 512
    bs[11..13].copy_from_slice(&512u16.to_le_bytes());

    // sectors per cluster = 8
    bs[13] = 8;

    // reserved sectors = 32
    bs[14..16].copy_from_slice(&32u16.to_le_bytes());

    // FAT count = 2
    bs[16] = 2;

    // sectors per FAT (16 bits) = 0 => FAT32
    bs[22..24].copy_from_slice(&0u16.to_le_bytes());

    // sectors per FAT (32 bits)
    bs[36..40].copy_from_slice(&1234u32.to_le_bytes());

    // root cluster
    bs[44..48].copy_from_slice(&2u32.to_le_bytes());

    // signature
    bs[510] = 0x55;
    bs[511] = 0xAA;

    bs
}

#[test]
fn read_valid_boot_sector() {
    let bs = make_valid_boot_sector();
    let device = MemoryBlockDevice::new(&bs);

    let boot = BootSector::read(&device).unwrap();

    assert_eq!(boot.bytes_per_sector, 512);
    assert_eq!(boot.sectors_per_cluster, 8);
    assert_eq!(boot.fat_count, 2);
    assert_eq!(boot.sectors_per_fat, 1234);
    assert_eq!(boot.root_cluster, 2);
}

#[test]
fn invalid_signature() {
    let mut bs = make_valid_boot_sector();
    bs[510] = 0x00;

    let device = MemoryBlockDevice::new(&bs);
    let result = BootSector::read(&device);

    assert_eq!(result, Err(BootSectorError::InvalidSignature));
}
