use rust_project::device::block_device::MemoryBlockDevice;
use rust_project::fs::boot_sector::BootSector;
use rust_project::fs::fat::{Fat, FatError};

fn make_boot_sector() -> BootSector {
    BootSector {
        bytes_per_sector: 512,
        sectors_per_cluster: 1,
        reserved_sectors: 1,
        fat_count: 1,
        sectors_per_fat: 1,
        root_cluster: 2,
    }
}

fn make_fat_image(entries: &[u32]) -> Vec<u8> {
    let mut img = vec![0u8; 512]; // reserved sector

    for &e in entries {
        img.extend_from_slice(&e.to_le_bytes());
    }

    img
}

#[test]
fn follow_simple_chain() {
    // cluster 2 -> 3 -> EOC
    let fat_entries = [
        0x0000_0000, // cluster 0 (unused)
        0x0000_0000, // cluster 1 (unused)
        3,           // cluster 2
        0x0FFF_FFFF, // cluster 3 (EOC)
    ];

    let image = make_fat_image(&fat_entries);
    let device = MemoryBlockDevice::new(&image);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);

    assert_eq!(fat.next_cluster(2).unwrap(), Some(3));
    assert_eq!(fat.next_cluster(3).unwrap(), None);
}

#[test]
fn invalid_cluster_number() {
    let image = make_fat_image(&[]);
    let device = MemoryBlockDevice::new(&image);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);

    assert_eq!(fat.next_cluster(1), Err(FatError::InvalidCluster));
}

#[test]
fn invalid_fat_entry() {
    let fat_entries = [
        0x0000_0000,
        0x0000_0000,
        0x0000_0000, // cluster 2 → invalid
    ];

    let image = make_fat_image(&fat_entries);
    let device = MemoryBlockDevice::new(&image);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);

    assert_eq!(fat.next_cluster(2), Err(FatError::InvalidCluster));
}
