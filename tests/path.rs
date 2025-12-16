use rust_project::device::block_device::MemoryBlockDevice;
use rust_project::fs::boot_sector::BootSector;
use rust_project::fs::fat::Fat;
use rust_project::fs::clusters::ClusterReader;
use rust_project::fs::directory::{DirectoryReader, EntryType};
use rust_project::fs::path::{PathResolver, PathError};

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

// Réutilise l’image de répertoire simple
fn make_image() -> Vec<u8> {
    let mut img = vec![0u8; 512];
    img.extend_from_slice(&vec![0u8; 512]);

    let mut root = vec![0u8; 512];

    // DIR -> cluster 3
    root[0..32].copy_from_slice(&super_dir("DIR", 3));
    root[32] = 0x00;

    let mut dir = vec![0u8; 512];
    dir[0..32].copy_from_slice(&super_file("FILE", "TXT", 5));
    dir[32] = 0x00;

    img.extend_from_slice(&root);
    img.extend_from_slice(&dir);

    img
}

fn super_dir(name: &str, cluster: u32) -> [u8; 32] {
    let mut e = [0u8; 32];
    e[0..name.len()].copy_from_slice(name.as_bytes());
    e[11] = 0x10;
    e[26..28].copy_from_slice(&(cluster as u16).to_le_bytes());
    e
}

fn super_file(name: &str, ext: &str, cluster: u32) -> [u8; 32] {
    let mut e = [0u8; 32];
    e[0..name.len()].copy_from_slice(name.as_bytes());
    e[8..8 + ext.len()].copy_from_slice(ext.as_bytes());
    e[11] = 0x20;
    e[26..28].copy_from_slice(&(cluster as u16).to_le_bytes());
    e
}

#[test]
fn resolve_absolute_path() {
    let img = make_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let cluster = ClusterReader::new(&device, &boot, &fat);
    let dir = DirectoryReader::new(&cluster);
    let resolver = PathResolver::new(&boot, &dir);

    let (cluster, entry) = resolver.resolve("/DIR", 2).unwrap();

    assert_eq!(cluster, 3);
    assert_eq!(entry.unwrap().entry_type, EntryType::Directory);
}

#[test]
fn resolve_file_path() {
    let img = make_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let cluster = ClusterReader::new(&device, &boot, &fat);
    let dir = DirectoryReader::new(&cluster);
    let resolver = PathResolver::new(&boot, &dir);

    let (_, entry) = resolver.resolve("/DIR/FILE.TXT", 2).unwrap();

    assert_eq!(entry.unwrap().entry_type, EntryType::File);
}

#[test]
fn path_not_found() {
    let img = make_image();
    let device = MemoryBlockDevice::new(&img);
    let boot = make_boot_sector();
    let fat = Fat::new(&device, &boot);
    let cluster = ClusterReader::new(&device, &boot, &fat);
    let dir = DirectoryReader::new(&cluster);
    let resolver = PathResolver::new(&boot, &dir);

    assert_eq!(
        resolver.resolve("/NOPE", 2),
        Err(PathError::NotFound)
    );
}
