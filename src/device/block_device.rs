//! Abstraction d’un périphérique de stockage lisible par offset.
//!
//! Ce module ne dépend pas de FAT32.
//! Il définit un contrat minimal pour lire des octets depuis une source quelconque.

use core::fmt;

/// Erreurs possibles lors d’une lecture sur un périphérique de stockage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDeviceError {
    /// Lecture demandée en dehors des limites du stockage.
    OutOfBounds,
    /// Erreur générique d’entrée/sortie.
    IoError,
}

impl fmt::Display for BlockDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockDeviceError::OutOfBounds => write!(f, "read out of bounds"),
            BlockDeviceError::IoError => write!(f, "I/O error"),
        }
    }
}

/// Résultat d’une opération sur un périphérique de stockage.
pub type BlockDeviceResult<T> = Result<T, BlockDeviceError>;

/// Trait représentant un périphérique de stockage lisible.
///
/// # Contrat
/// - `offset` est exprimé en octets
/// - la fonction doit remplir entièrement `buf`
/// - aucune allocation ne doit être faite
/// - aucune panique ne doit se produire
pub trait BlockDevice {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> BlockDeviceResult<()>;
}

/// Implémentation mémoire d’un périphérique de stockage.
///
/// Utilisée principalement pour les tests.
pub struct MemoryBlockDevice<'a> {
    data: &'a [u8],
}

impl<'a> MemoryBlockDevice<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> BlockDevice for MemoryBlockDevice<'a> {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> BlockDeviceResult<()> {
        let offset = offset as usize;
        let end = offset.checked_add(buf.len())
            .ok_or(BlockDeviceError::OutOfBounds)?;

        if end > self.data.len() {
            return Err(BlockDeviceError::OutOfBounds);
        }

        buf.copy_from_slice(&self.data[offset..end]);
        Ok(())
    }
}
