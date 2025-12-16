use rust_project::device::block_device::MemoryBlockDevice;
use rust_project::fs::boot_sector::BootSector;
use rust_project::fs::fat::Fat;
use rust_project::fs::clusters::ClusterReader;
use rust_project::fs::directory::{DirectoryReader, EntryType};

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

fn make_dir_entry(
    name: &str,
    ext: &str,
    attr: u8,
    start_cluster: u32,
    size: u32,
) -> [u8; 32] {
    let mut e = [0u8; 32];

    let mut n = [b' '; 8];
    n[..name.len()].copy_from_slice(name.as_bytes());
    e[0..8].copy_from_slice(&n);

    let mut x = [b' '; 3];
    x[..ext.len()].copy_from_slice(ext.as_bytes());
    e[8..11].copy_from_slice(&x);

    e[11] = attr;

    e[20..22].copy_from_slice(&((start_cluster >> 16) as u16).to_le_bytes());
    e[26..28].copy_from_slice(&(start_cluster as u16).to_le_bytes());
    e[28..32].copy_from_slice(&size.to_le_bytes());

    e
}

fn make_disk_image() -> Vec<u8> {
    let mut img = vec![0u8; 512]; // reserved
    img.extend_from_slice(&vec![0u8; 512]); // FAT
    let mut cluster = vec![0u8; 512];

    let f = make_dir_entry("FILE", "TXT", 0x20, 5, 123);
    let d = make_dir_entry("DIR", "", 0x10, 6, 0);

    cluster[0..32].copy_from_slice(&f);
    cluster[32..64].copy_from_slice(&d);
    cluster[64] = 0x00; // end

    img.extend_from_slice(&cluster);
    img
}

#[test]
fn read_directory_entries() {
    let img = make_disk_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let cluster_reader = ClusterReader::new(&device, &boot, &fat);
    let dir_reader = DirectoryReader::new(&cluster_reader);

    let entries = dir_reader.read_dir(2).unwrap();

    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0].name, "FILE.TXT");
    assert_eq!(entries[0].entry_type, EntryType::File);
    assert_eq!(entries[0].size, 123);

    assert_eq!(entries[1].name, "DIR");
    assert_eq!(entries[1].entry_type, EntryType::Directory);
}
