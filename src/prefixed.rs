use core::fmt::Debug;
use core::slice::from_raw_parts;

use crate::traits::HashT;

/// structure containing a prefix (aligned to 4 bytes) and hashes as a contiguous memory block
/// prefix is used to prevent a proof length extension attack
#[repr(C)]
pub struct Prefixed<const ARITY: usize, H: HashT> {
//    prefix: [u8; 4],
    prefix: H::Output,
    pub(crate) hashes: [H::Output; ARITY],
}

impl<const ARITY: usize, H: HashT> Prefixed<ARITY, H> {
    /// hash of &[] prefixed with LEAF_HASH_PREPEND_VALUE
    #[inline]
    pub fn default_hash() -> H::Output {
        //        H::hash(&[crate::LEAF_HASH_PREPEND_VALUE; 1])
        H::Output::default()
    }
    /// hash the prefix together with inner hashes
    #[inline]
    pub fn hash_all(&self) -> H::Output {
        unsafe {
            H::concat_then_hash(from_raw_parts(
                &self.prefix as *const <H as HashT>::Output,
                ARITY + 1,
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

impl<const ARITY: usize, H: HashT> Clone for Prefixed<ARITY, H> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<const ARITY: usize, H: HashT> Copy for Prefixed<ARITY, H> {}
impl<const ARITY: usize, H: HashT> Default for Prefixed<ARITY, H> {
    fn default() -> Self {
        Self {
//            prefix: H::Output::default(),
            prefix: crate::INNER_HASH_PREPEND_VALUE.into(),
            hashes: [Self::default_hash(); ARITY],
        }
    }
}

impl<const ARITY: usize, H: HashT> Debug for Prefixed<ARITY, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "prefix: {:?}", self.prefix)?;
        for (i, h) in self.hashes.iter().enumerate() {
            writeln!(f, "h[{i}]: {h:?}")?;
        }
        Ok(())
    }
}
