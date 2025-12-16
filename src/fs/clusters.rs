//! Lecture des clusters FAT32 (cluster -> données)

use crate::device::block_device::{BlockDevice, BlockDeviceError};
use crate::fs::boot_sector::BootSector;
use crate::fs::fat::{Fat, FatError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClusterError {
    Io(BlockDeviceError),
    Fat(FatError),
    InvalidCluster,
}

impl From<BlockDeviceError> for ClusterError {
    fn from(e: BlockDeviceError) -> Self {
        ClusterError::Io(e)
    }
}

impl From<FatError> for ClusterError {
    fn from(e: FatError) -> Self {
        ClusterError::Fat(e)
    }
}

/// Fournit des opérations de lecture de clusters.
pub struct ClusterReader<'a, D: BlockDevice> {
    device: &'a D,
    boot: &'a BootSector,
    fat: &'a Fat<'a, D>,
}

impl<'a, D: BlockDevice> ClusterReader<'a, D> {
    pub fn new(device: &'a D, boot: &'a BootSector, fat: &'a Fat<'a, D>) -> Self {
        Self { device, boot, fat }
    }

    /// Calcule l’offset disque du début d’un cluster.
    pub fn cluster_offset(&self, cluster: u32) -> Result<u64, ClusterError> {
        if cluster < 2 {
            return Err(ClusterError::InvalidCluster);
        }

        let bytes_per_sector = self.boot.bytes_per_sector as u64;
        let sectors_per_cluster = self.boot.sectors_per_cluster as u64;

        let fat_region_size =
            self.boot.fat_count as u64 * self.boot.sectors_per_fat as u64;

        let data_region_sector =
            self.boot.reserved_sectors as u64 + fat_region_size;

        let cluster_index = cluster as u64 - 2;

        Ok((data_region_sector + cluster_index * sectors_per_cluster)
            * bytes_per_sector)
    }

    /// Lit un cluster complet dans `buf`.
    pub fn read_cluster(&self, cluster: u32, buf: &mut [u8]) -> Result<(), ClusterError> {
        let cluster_size =
            self.boot.bytes_per_sector as usize * self.boot.sectors_per_cluster as usize;

        if buf.len() != cluster_size {
            return Err(ClusterError::InvalidCluster);
        }

        let offset = self.cluster_offset(cluster)?;
        self.device.read_at(offset, buf)?;
        Ok(())
    }

    /// Lit une chaîne de clusters complète et concatène les données.
    ///
    /// S’arrête automatiquement à la fin de chaîne (EOC).
    pub fn read_cluster_chain(
        &self,
        start_cluster: u32,
        out: &mut alloc::vec::Vec<u8>,
    ) -> Result<(), ClusterError> {
        let cluster_size =
            self.boot.bytes_per_sector as usize * self.boot.sectors_per_cluster as usize;

        let mut current = start_cluster;

        loop {
            let mut buf = alloc::vec![0u8; cluster_size];
            self.read_cluster(current, &mut buf)?;
            out.extend_from_slice(&buf);

            match self.fat.next_cluster(current)? {
                Some(next) => current = next,
                None => break,
            }
        }

        Ok(())
    }
}
