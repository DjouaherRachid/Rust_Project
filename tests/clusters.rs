use rust_project::device::block_device::MemoryBlockDevice;
use rust_project::fs::boot_sector::BootSector;
use rust_project::fs::fat::Fat;
use rust_project::fs::clusters::{ClusterReader, ClusterError};

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

fn make_disk_image() -> Vec<u8> {
    let mut img = vec![0u8; 512]; // reserved sector

    // FAT sector
    // cluster 0,1 unused
    img.extend_from_slice(&0u32.to_le_bytes());
    img.extend_from_slice(&0u32.to_le_bytes());
    // cluster 2 -> 3
    img.extend_from_slice(&3u32.to_le_bytes());
    // cluster 3 -> EOC
    img.extend_from_slice(&0x0FFF_FFFFu32.to_le_bytes());
    img.resize(1024, 0); // pad FAT sector to 512 bytes

    // cluster 2 data
    let mut c2 = vec![0u8; 512];
    c2[..4].copy_from_slice(b"ABCD");
    img.extend_from_slice(&c2);

    // cluster 3 data
    let mut c3 = vec![0u8; 512];
    c3[..4].copy_from_slice(b"EFGH");
    img.extend_from_slice(&c3);

    img
}

#[test]
fn read_single_cluster() {
    let img = make_disk_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let reader = ClusterReader::new(&device, &boot, &fat);

    let mut buf = vec![0u8; 512];
    reader.read_cluster(2, &mut buf).unwrap();

    assert_eq!(&buf[..4], b"ABCD");
}

#[test]
fn read_cluster_chain() {
    let img = make_disk_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let reader = ClusterReader::new(&device, &boot, &fat);

    let mut out = Vec::new();
    reader.read_cluster_chain(2, &mut out).unwrap();

    assert_eq!(&out[..4], b"ABCD");
    assert_eq!(&out[512..516], b"EFGH");
}

#[test]
fn invalid_cluster_number() {
    let img = make_disk_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let reader = ClusterReader::new(&device, &boot, &fat);

    let mut buf = vec![0u8; 512];
    let err = reader.read_cluster(1, &mut buf).unwrap_err();
    assert_eq!(err, ClusterError::InvalidCluster);
}
