//! Lecture et validation du Boot Sector FAT32.

use crate::device::block_device::{BlockDevice, BlockDeviceError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_count: u8,
    pub sectors_per_fat: u32,
    pub root_cluster: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootSectorError {
    Io(BlockDeviceError),
    InvalidSignature,
    InvalidBytesPerSector,
    InvalidSectorsPerCluster,
    NotFAT32,
}

impl From<BlockDeviceError> for BootSectorError {
    fn from(e: BlockDeviceError) -> Self {
        BootSectorError::Io(e)
    }
}

impl BootSector {
    /// Lit et valide le Boot Sector FAT32 depuis le périphérique.
    pub fn read<D: BlockDevice>(device: &D) -> Result<Self, BootSectorError> {
        let mut sector = [0u8; 512];
        device.read_at(0, &mut sector)?;

        // Signature de fin (0x55AA)
        if sector[510] != 0x55 || sector[511] != 0xAA {
            return Err(BootSectorError::InvalidSignature);
        }

        let bytes_per_sector = u16::from_le_bytes([sector[11], sector[12]]);
        if !matches!(bytes_per_sector, 512 | 1024 | 2048 | 4096) {
            return Err(BootSectorError::InvalidBytesPerSector);
        }

        let sectors_per_cluster = sector[13];
        if !sectors_per_cluster.is_power_of_two() || sectors_per_cluster == 0 {
            return Err(BootSectorError::InvalidSectorsPerCluster);
        }

        let reserved_sectors = u16::from_le_bytes([sector[14], sector[15]]);
        let fat_count = sector[16];

        let sectors_per_fat_16 = u16::from_le_bytes([sector[22], sector[23]]);
        if sectors_per_fat_16 != 0 {
            return Err(BootSectorError::NotFAT32);
        }

        let sectors_per_fat =
            u32::from_le_bytes([sector[36], sector[37], sector[38], sector[39]]);
        let root_cluster =
            u32::from_le_bytes([sector[44], sector[45], sector[46], sector[47]]);

        Ok(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            fat_count,
            sectors_per_fat,
            root_cluster,
        })
    }
}
