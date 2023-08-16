use core::fmt::Debug;
use core::mem::size_of;
use core::slice::from_raw_parts;

use crate::traits::HashT;

/// structure containing a prefix (aligned to 4 bytes) and hashes as a contiguous memory block
/// prefix is used to prevent a proof length extension attack
#[repr(C)]
pub struct Prefixed<const BRANCH_FACTOR: usize, H: HashT> {
    prefix: [u8; 4],
    pub(crate) hashes: [H::Output; BRANCH_FACTOR],
}

impl<const BRANCH_FACTOR: usize, H: HashT> Prefixed<BRANCH_FACTOR, H> {
    /// hash of &[] prefixed with 0u8
    #[inline]
    pub fn default_hash() -> H::Output {
        H::hash(&[0u8; 1])
    }
    /// hash the prefix together with inner hashes
    #[inline]
    pub fn hash_all(&self) -> H::Output {
        unsafe {
            H::hash(from_raw_parts(
                self.prefix.as_ref().as_ptr() as *const u8,
                size_of::<Self>(),
            ))
        }
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Clone for Prefixed<BRANCH_FACTOR, H> {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix,
            hashes: self.hashes,
        }
    }
}
impl<const BRANCH_FACTOR: usize, H: HashT> Copy for Prefixed<BRANCH_FACTOR, H> {}
impl<const BRANCH_FACTOR: usize, H: HashT> Default for Prefixed<BRANCH_FACTOR, H> {
    fn default() -> Self {
        Self {
            prefix: [1u8; 4],
            hashes: [Self::default_hash(); BRANCH_FACTOR],
        }
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Debug for Prefixed<BRANCH_FACTOR, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "prefix: {:?}", self.prefix)?;
        for (i, h) in self.hashes.iter().enumerate() {
            writeln!(f, "h[{i}]: {h:?}")?;
        }
        Ok(())
    }
}
