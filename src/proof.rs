use crate::traits::{HashT, ProofBuilder, ProofItemT, ProofValidator};
//use crate::utils::{hash_merged_slice};
use crate::{prefixed_size};
use core::fmt::Debug;
use core::mem::size_of;

/// Basic implementation of an item making up a proof.
/// Supports a power-of-2 number of siblings
pub struct ProofItem<const BRANCH_FACTOR: usize, H: HashT> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    hashes: Option<[H::Output; BRANCH_FACTOR]>,
    offset: usize,
    prefixed_buffer: [u8; prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]
}

impl<const BRANCH_FACTOR: usize, H: HashT> ProofItem<BRANCH_FACTOR, H> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{    fn hash_merged_slice(&mut self, index: usize) -> H::Output
    {
        let chunk = unsafe { core::slice::from_raw_parts(self.hashes.unwrap()[index].as_ref().as_ptr(), Self::BYTES_IN_CHUNK) };
        let len = self.prefixed_buffer.len();

        self.prefixed_buffer[1..len].copy_from_slice(chunk);
        
        H::hash(&self.prefixed_buffer)
    }

    const BYTES_IN_CHUNK: usize = BRANCH_FACTOR * size_of::<H::Output>(); 

}

impl<const BRANCH_FACTOR: usize, H: HashT> ProofItemT<H> for ProofItem<BRANCH_FACTOR, H> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    /// constructor
    fn create(offset: usize, hashes: &[H::Output]) -> Self {
        Self {
            offset,
            hashes: hashes[..BRANCH_FACTOR].try_into().ok(),
            prefixed_buffer: [1u8; prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]
        }
    }
    /// hashes a provided hashed data at offset with its siblings
    fn hash_with_siblings(mut self, word_hash: H::Output) -> Option<H::Output> {
        self.hashes.as_mut().map(|hashes| {
            hashes[self.offset] = word_hash;
//            Self::hash_merged_slice(&hashes[0..], prefix)
//            hash_merged_slice::<H, {Self::BYTES_IN_CHUNK}>(&hashes[0..], 1)
        })
        .map(|_| self.hash_merged_slice(0))
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Copy for ProofItem<BRANCH_FACTOR, H> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{}

impl<const BRANCH_FACTOR: usize, H: HashT> Clone for ProofItem<BRANCH_FACTOR, H> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    fn clone(&self) -> Self {
        Self {
            hashes: self.hashes.clone(),
            offset: self.offset.clone(),
            prefixed_buffer: [1u8; prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]
        }
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Default for ProofItem<BRANCH_FACTOR, H> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    fn default() -> Self {
        Self {
            hashes: Default::default(),
            offset: Default::default(),
            prefixed_buffer: [1u8; prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]
        }
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Debug for ProofItem<BRANCH_FACTOR, H> 
where
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "{:?}", self.hashes)
    }
}

/// Proof implementation the StaticTree generates
pub struct Proof<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT>
where
    [(); HEIGHT]: Sized,
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    root: H::Output,
    height: usize,
    items: [<Self as ProofBuilder<H>>::Item; HEIGHT],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> ProofBuilder<H>
    for Proof<BRANCH_FACTOR, HEIGHT, H>
where
    [(); HEIGHT]: Sized,
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    type Item = ProofItem<BRANCH_FACTOR, H>;

    fn from_root(root: H::Output) -> Self {
        Self {
            root,
            items: [ProofItem::default(); HEIGHT],
            height: 0,
        }
    }
    fn root(&self) -> H::Output {
        self.root
    }
    fn push(&mut self, offset: usize, hashes: &[H::Output]) {
        self.items[self.height] = Self::Item::create(offset, hashes);
        self.height += 1;
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> ProofValidator
    for Proof<BRANCH_FACTOR, HEIGHT, H>
where
    [(); HEIGHT]: Sized,
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    /// verifies that the input was contained in the Merkle tree that generated this proof
    fn validate(self, input: &[u8]) -> bool {
        const MAX_INPUT_LEN: usize = 1000;
        let mut prefixed = [0u8; MAX_INPUT_LEN];
        prefixed[1..input.len() + 1].copy_from_slice(input);

        let mut curr_hash = Some(H::hash(&prefixed[0..input.len() + 1]));
        // start from the base layer,
        // and for every item in the proof
        // put the hash derived from input into the proof item
        // at index stored in the proof item
        // and hash it with the siblings
        for item in &self.items[..self.height] {
            curr_hash = curr_hash.and_then(|h| item.hash_with_siblings(h));
        }
        // validated iff the resulting hash is identical to the root
        curr_hash.as_ref() == Some(&self.root)
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> Default
    for Proof<BRANCH_FACTOR, HEIGHT, H>
where
    [(); HEIGHT]: Sized,
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    fn default() -> Self {
        Self {
            root: Default::default(),
            items: [Default::default(); HEIGHT],
            height: 0,
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> Debug
    for Proof<BRANCH_FACTOR, HEIGHT, H>
where
    [(); HEIGHT]: Sized,
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "[proof height]:   {:?}", self.height)?;
        writeln!(f, "[proof root]:   {:?}", self.root)?;
        write!(f, "{:?}", self.items)
    }
}
/// Chains two proofs into one
/// The second root becomes the root of the target proof
pub fn chain_proofs<
    const BRANCH_FACTOR: usize,
    const HEIGHT1: usize,
    const HEIGHT2: usize,
    H: HashT,
>(
    proof1: Proof<BRANCH_FACTOR, HEIGHT1, H>,
    proof2: Proof<BRANCH_FACTOR, HEIGHT2, H>,
) -> Proof<BRANCH_FACTOR, { HEIGHT1 + HEIGHT2 }, H>
where
    [(); HEIGHT1]: Sized,
    [(); HEIGHT2]: Sized,
    [(); HEIGHT1 + HEIGHT2]: Sized,
    [(); prefixed_size!(BRANCH_FACTOR, size_of::<H::Output>())]: Sized,
{
    let mut proof = Proof::from_root(proof2.root());
    proof.height = proof1.height + proof2.height;
    for i in 0..proof1.height {
        proof.items[i] = proof1.items[i];
    }
    for i in 0..proof2.height {
        proof.items[i + proof1.height] = proof2.items[i];
    }
    proof
}
