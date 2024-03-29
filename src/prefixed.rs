use core::fmt::Debug;
use core::slice::from_raw_parts;

use crate::traits::HashT;

/// structure containing a prefix (aligned to 4 bytes) and hashes as a contiguous memory block
/// prefix is used to prevent a proof length extension attack
#[repr(C)]
pub struct Prefixed<const BRANCH_FACTOR: usize, H: HashT> {
//    prefix: [u8; 4],
    prefix: H::Output,
    pub(crate) hashes: [H::Output; BRANCH_FACTOR],
}

impl<const BRANCH_FACTOR: usize, H: HashT> Prefixed<BRANCH_FACTOR, H> {
    /// hash of &[] prefixed with LEAF_HASH_PREPEND_VALUE
    #[inline]
    pub fn default_hash() -> H::Output {
        H::hash(&[crate::LEAF_HASH_PREPEND_VALUE; 1])
//        H::Output::default()
    }
    /// hash the prefix together with inner hashes
    #[inline]
    pub fn hash_all(&self) -> H::Output {      
        unsafe {
            H::concat_then_hash(from_raw_parts(
                &self.prefix as *const <H as HashT>::Output,
                BRANCH_FACTOR + 1,
            ))
        }

    }

//     pub fn hash_all(&self) -> H::Output {
//         unsafe {
//             H::hash(from_raw_parts(
//                 self.hashes.as_ref().as_ptr() as *const u8,
// //                self.prefix.as_ref().as_ptr() as *const u8,
//                 size_of::<Self>(),
//             ))
//         }
//     }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Clone for Prefixed<BRANCH_FACTOR, H> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<const BRANCH_FACTOR: usize, H: HashT> Copy for Prefixed<BRANCH_FACTOR, H> {}
impl<const BRANCH_FACTOR: usize, H: HashT> Default for Prefixed<BRANCH_FACTOR, H> {
    fn default() -> Self {
        Self {
            prefix: H::Output::default(),
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
