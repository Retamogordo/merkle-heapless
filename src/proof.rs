use crate::traits::{HashT, ProofBuilder, ProofItemT, ProofValidator};
use crate::utils::hash_merged_slice;
use core::fmt::Debug;
use core::mem::size_of;

/// Basic implementation of an item making up a proof.
/// Supports a power-of-2 number of siblings
pub struct ProofItem<const BRANCH_FACTOR: usize, H: HashT> {
    hashes: Option<[H::Output; BRANCH_FACTOR]>,
    offset: usize,
}

impl<const BRANCH_FACTOR: usize, H: HashT> ProofItemT<H> for ProofItem<BRANCH_FACTOR, H> {
    /// constructor
    fn create(offset: usize, hashes: &[H::Output]) -> Self {
        Self {
            offset,
            hashes: hashes[..BRANCH_FACTOR].try_into().ok(),
        }
    }
    /// hashes a provided hashed data at offset with its siblings
    fn hash_with_siblings(mut self, word_hash: H::Output) -> Option<H::Output> {
        let bytes_in_chunk: usize = BRANCH_FACTOR * size_of::<H::Output>();

        self.hashes.as_mut().map(|hashes| {
            hashes[self.offset] = word_hash;
            hash_merged_slice::<H>(&hashes[0..], bytes_in_chunk)
        })
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Copy for ProofItem<BRANCH_FACTOR, H> {}

impl<const BRANCH_FACTOR: usize, H: HashT> Clone for ProofItem<BRANCH_FACTOR, H> {
    fn clone(&self) -> Self {
        Self {
            hashes: self.hashes.clone(),
            offset: self.offset.clone(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Default for ProofItem<BRANCH_FACTOR, H> {
    fn default() -> Self {
        Self {
            hashes: Default::default(),
            offset: Default::default(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, H: HashT> Debug for ProofItem<BRANCH_FACTOR, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "{:?}", self.hashes)
    }
}

/// Proof implementation the StaticTree generates
pub struct Proof<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT>
where
    [(); HEIGHT]: Sized,
{
    root: H::Output,
    height: usize,
    items: [<Self as ProofBuilder<H>>::Item; HEIGHT],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> ProofBuilder<H>
    for Proof<BRANCH_FACTOR, HEIGHT, H>
where
    [(); HEIGHT]: Sized,
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
{
    /// verifies that the input was contained in the Merkle tree that generated this proof
    fn validate(self, input: &[u8]) -> bool {
        let mut curr_hash = Some(H::hash(&input));
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
