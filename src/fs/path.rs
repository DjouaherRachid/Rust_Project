//! Résolution des chemins FAT32 (absolus et relatifs)
use crate::fs::boot_sector::BootSector;
use crate::fs::directory::{DirectoryReader, DirectoryEntry, EntryType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    NotFound,
    NotADirectory,
    EmptyPath,
}

pub struct PathResolver<'a, D: crate::device::block_device::BlockDevice> {
    boot: &'a BootSector,
    dir_reader: &'a DirectoryReader<'a, D>,
}

impl<'a, D: crate::device::block_device::BlockDevice> PathResolver<'a, D> {
    pub fn new(
        boot: &'a BootSector,
        dir_reader: &'a DirectoryReader<'a, D>,
    ) -> Self {
        Self { boot, dir_reader }
    }

    /// Résout un chemin et retourne l’entrée correspondante
    pub fn resolve(
        &self,
        path: &str,
        cwd: u32,
    ) -> Result<(u32, Option<DirectoryEntry>), PathError> {
        if path.is_empty() {
            return Err(PathError::EmptyPath);
        }

        let mut current_cluster = if path.starts_with('/') {
            self.boot.root_cluster
        } else {
            cwd
        };

        let parts = path.split('/').filter(|p| !p.is_empty());

        let mut last_entry = None;

        for part in parts {
            if part == "." {
                continue;
            }

            let entries = self
                .dir_reader
                .read_dir(current_cluster)
                .map_err(|_| PathError::NotFound)?;

            let entry = entries
                .into_iter()
                .find(|e| e.name == part)
                .ok_or(PathError::NotFound)?;

            match entry.entry_type {
                EntryType::Directory => {
                    current_cluster = entry.start_cluster;
                }
                EntryType::File => {
                    last_entry = Some(entry.clone());
                    break;
                }
            }

            last_entry = Some(entry);
        }

        Ok((current_cluster, last_entry))
    }
}
