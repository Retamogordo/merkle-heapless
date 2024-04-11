use crate::traits::{HashT, ProofBuilder, ProofItemT, ProofValidator};
use crate::Prefixed;
use core::fmt::Debug;

/// Basic implementation of an item making up a proof.
/// Supports a power-of-2 number of siblings
pub struct ProofItem<const ARITY: usize, H: HashT> {
    prefixed: Prefixed<ARITY, H>,
    offset: usize,
}

impl<const ARITY: usize, H: HashT> ProofItem<ARITY, H> {
    /// returns item's hashes
    pub fn hashes(&self) -> &[H::Output; ARITY] {
        &self.prefixed.hashes
    }

    /// returns item's offset
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<const ARITY: usize, H: HashT> ProofItemT<ARITY, H> for ProofItem<ARITY, H> {
    /// constructor
    fn create(offset: usize, prefixed: Prefixed<ARITY, H>) -> Self {
        Self { offset, prefixed }
    }
    /// hashes a provided hashed data at offset with its siblings
    fn hash_with_siblings(mut self, word_hash: H::Output) -> Option<H::Output> {
        self.prefixed.hashes[self.offset] = word_hash;
        Some(self.prefixed.hash_all())
    }
}

impl<const ARITY: usize, H: HashT> Copy for ProofItem<ARITY, H> {}

impl<const ARITY: usize, H: HashT> Clone for ProofItem<ARITY, H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<const ARITY: usize, H: HashT> Default for ProofItem<ARITY, H> {
    fn default() -> Self {
        Self {
            prefixed: Default::default(),
            offset: Default::default(),
        }
    }
}

impl<const ARITY: usize, H: HashT> Debug for ProofItem<ARITY, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "{:?}", self.prefixed.hashes)
    }
}

/// Proof implementation the StaticTree generates
pub struct Proof<const ARITY: usize, const HEIGHT: usize, H: HashT, const MAX_INPUT_LEN: usize>
where
    [(); HEIGHT]: Sized,
{
    root: H::Output,
    height: usize,
    items: [<Self as ProofBuilder<ARITY, H>>::Item; HEIGHT],
}

impl<const ARITY: usize, const HEIGHT: usize, H: HashT, const MAX_INPUT_LEN: usize>
    ProofBuilder<ARITY, H> for Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>
where
    [(); HEIGHT]: Sized,
{
    type Item = ProofItem<ARITY, H>;

    fn from_root(root: H::Output) -> Self {
        Self {
            root,
            items: [ProofItem::default(); HEIGHT],
            height: 0,
        }
    }

    fn push(&mut self, offset: usize, prefixed: Prefixed<ARITY, H>) {
        self.items[self.height] = Self::Item::create(offset, prefixed);
        self.height += 1;
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H: HashT, const MAX_INPUT_LEN: usize> ProofValidator
    for Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>
where
    [(); HEIGHT]: Sized,
{
    /// verifies that the input was contained in the Merkle tree that generated this proof
    fn validate(self, input: &[u8]) -> bool {
        let mut curr_hash = Some(Self::hash_as_leaf(input));

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

impl<const ARITY: usize, const HEIGHT: usize, H: HashT, const MAX_INPUT_LEN: usize>
    Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>
where
    [(); HEIGHT]: Sized,
{
    /// returns the proof's length
    pub fn height(&self) -> usize {
        self.height
    }

    /// returns the proof's root
    pub fn root(&self) -> H::Output {
        self.root
    }

    /// returns the proof's path
    pub fn path(&self) -> &[<Self as ProofBuilder<ARITY, H>>::Item] {
        &self.items
    }

    /// returns the proof's arity
    pub fn arity() -> usize {
        ARITY
    }

    /// prepends input with leaf prefix and hashes it
    pub fn hash_as_leaf(input: &[u8]) -> H::Output {
        let start_index = if input.len() < MAX_INPUT_LEN { 1 } else { 0 };

        let n = input.len() + start_index;
        let mut prefixed = [crate::LEAF_HASH_PREPEND_VALUE; MAX_INPUT_LEN];
        prefixed[start_index..n].copy_from_slice(input);

        H::hash(&prefixed[0..n])
    }
    /// returns the index of claim as tree's leaf
    pub fn claim_index(&self) -> usize {
        let mut a = 1usize;
        let mut index = 0;
        for item in self.items.iter() {
            index += a * item.offset();
            a *= ARITY;
        }
        index
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H: HashT, const MAX_INPUT_LEN: usize> Default
    for Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>
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

impl<const ARITY: usize, const HEIGHT: usize, H: HashT, const MAX_INPUT_LEN: usize> Debug
    for Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>
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
    const ARITY: usize,
    const HEIGHT1: usize,
    const HEIGHT2: usize,
    H: HashT,
    const MAX_INPUT_LEN: usize,
>(
    proof1: Proof<ARITY, HEIGHT1, H, MAX_INPUT_LEN>,
    proof2: Proof<ARITY, HEIGHT2, H, MAX_INPUT_LEN>,
) -> Proof<ARITY, { HEIGHT1 + HEIGHT2 }, H, MAX_INPUT_LEN>
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
