//! # Static Tree
//! A Merkle Tree implementation that requires no dynamic memory allocations.
//! This Merkle tree is implemented as a contiguous memory array and does not betake to dynamic allocations.
//! As such it allows for certain optimizations and compile-time imposed constraints on arity and size boundaries.
//! - no std dependencies (actually no dependencies)
//! - 2, 4, 8,... power of 2 general branching arity
//! - any hash function that takes ```&[u8]``` and returns something that implements ```AsRef<[u8]>```
//! - 99% safe Rust
//! - optionally augmentable or reducible
//! - optional Mountain Range proc macro (when compiled with a mmr-macro feature)
//!
//! ## Hashing
//! Leaves are prefixed with ```LEAF_HASH_PREPEND_VALUE``` prior to being hashed, while the intermediate nodes are prefixed with ```[1u8; 4]```. 
//!
//! # Mountain Range
//!
//! Merkle Mountain Range offers append-only growable Merkle Tree semantics optimized for space.
//!
//! The rules for this implementation of Mountain Range are:
//! - space limitations are defined at compile-time (no dynamic allocations) by number of peaks only
//! - an element is inserted by appending to the right-most peak having a capacity to append a new item
//! - the left-most peak is the highest peak at any moment
//! - when two adjacent peaks have the same height they are recursively merged into the left sibling
//! - roots of the peaks form leaves for the "summit Merkle tree"
//! - the Mountain Range proof is generated by chaining the proof of the corresponding peak with the proof generated by the relevant path in the summit tree
//! - for MMR declared with N peaks, it will handle peaks with heights [0..N] thus simulating a tree with number of leaves in range [0..N*2^N] in case of a binary MMR
//!
//! # Examples
//!
//! ### Hash implementation examples
//! ```rust
//! use std::{
//!     collections::hash_map::DefaultHasher,
//!     hash::{Hash, Hasher},
//! };
//! use merkle_heapless::traits::HashT;
//!
//! #[derive(Debug)]
//! struct Blake2_256Hash;
//! impl HashT for Blake2_256Hash {
//!     type Output = [u8; 32];
//!
//!     fn hash(input: &[u8]) -> Self::Output {
//!         // from Parity's sp_core crate
//!         sp_core::blake2_256(input)
//!     }
//! }
//!
//! #[derive(Debug)]
//! pub struct StdHash;
//! impl HashT for StdHash {
//!     type Output = [u8; 8];
//!
//!     fn hash(input: &[u8]) -> Self::Output {
//!         let mut s = DefaultHasher::new();
//!         input.hash(&mut s);
//!         s.finish().to_ne_bytes()
//!     }
//! }
//! ```
//!
//! ### Proof generation and verification
//! ```rust
//! use merkle_heapless::{StaticBinaryTree};
//! use merkle_heapless::traits::{StaticTreeTrait, ProofValidator};
//! // tree height 3, 8 leaves, 15 total nodes
//! const MAX_HEIGHT: usize = 3;
//! // stands for the maximum possible length of the longest input word
//! const MAX_INPUT_WORD_LEN: usize = 10;
//! // supposing the YourHash struct exists
//! let mut tree = StaticBinaryTree::<MAX_HEIGHT, YourHash, MAX_INPUT_WORD_LEN>::try_from::<&[u8]>(
//!     &[b"apple", b"banana"]
//! ).unwrap();
//!
//! let proof = tree.generate_proof(0);
//! assert!(proof.validate(b"apple"));
//! ```
//!
//! ### Replace and remove leaf
//! ```rust
//! // snip
//! // replace
//! tree.replace(5, b"cherry");
//! let proof = tree.generate_proof(5);
//! assert!(proof.validate(b"cherry"));
//! // remove
//! tree.replace(1, &[]);
//! let proof = tree.generate_proof(1);
//! assert!(!proof.validate(b"banana"));
//! let proof = tree.generate_proof(1);
//! assert!(proof.validate(&[]));
//! ```
//!
//! ### Arity other than 2
//! ```rust
//! use merkle_heapless::{StaticTree};
//!
//! const BRANCH_FACTOR: usize = 4;
//! const MAX_INPUT_WORD_LEN: usize = 10;
//! let mut tree = StaticTree::<BRANCH_FACTOR, MAX_HEIGHT, YourHash, MAX_INPUT_WORD_LEN>::try_from::<&[u8]>(
//!     &[b"apple", b"banana"]
//! ).unwrap();
//! // same operations can be applied
//! ```
//!
//! ## Mountain Range
//! Include ["mmr_macro"] feature in merkle-heapless dependency
//! ### Declaration and instantiation
//! ```rust
//! // compulsory at the beginning of the .rs file in order the macro to compile
//! #![allow(incomplete_features)]
//! #![feature(generic_const_exprs)]
//! // snip
//! use merkle_heapless::{mmr_macro};
//! // declaration with expicit type name for your MMR
//! mmr_macro::mmr!(Type = FooMMR, BranchFactor = 2, Peaks = 3, Hash = StdHash, MaxInputWordLength = 10);
//! let mmr = FooMMR::default();
//! // implicitly creates MerkleMountainRange type
//! mmr_macro::mmr!(BranchFactor = 2, Peaks = 5, Hash = StdHash, MaxInputWordLength = 10);
//! // create with default current peak of height 0
//! let mmr = MerkleMountainRange::default();
//! // or create with current peak of height 2
//! let mut mmr = MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak3(Default::default()));
//! assert_eq!(mmr.peaks()[0].height(), 5 - 3);
//! ```
//! ### Functionality
//! The functionality of Mountain Range is similar to that of the Merkle tree.   
//! ```rust
//! mmr.try_append(b"apple").unwrap();
//! // peak leaf numbers: [1, 0, 0, 0, 0]
//! assert_eq!(mmr.peaks()[0].height(), 0);
//! assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
//! assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
//! let proof = mmr.generate_proof(0);
//! assert!(proof.validate(b"apple"));
//! ```

//#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(trivial_bounds)]
#![warn(missing_docs)]
/// contains implementation of an extention for a Merkle Tree that can be augmented into a bigger tree
// and merge a smaller tree into the tree
pub mod augmentable;
/// contains implementation of an extention for a Merkle Tree that can remove a leaf, compact and reduce
/// the tree to a smaller tree.
pub mod compactable;
/// prefixed hashes
pub mod prefixed;
/// module containing [Proof] implementation the [StaticTree] generates
pub mod proof;
/// module declaring basic traits for tree and proof
pub mod traits;
mod utils;

#[cfg(feature = "mmr_macro")]
pub use mmr_macro;

use core::fmt::Debug;
use core::mem::size_of;
use core::ops::Deref;

use crate::prefixed::Prefixed;
use crate::proof::Proof;
use crate::traits::{HashT, ProofBuilder, StaticTreeTrait};
use crate::utils::{location_in_prefixed, Assert, IsTrue};

/// leaves will be pretended with this value prior to hashing
pub const LEAF_HASH_PREPEND_VALUE: u8 = 0;

/// Merkle Tree Errors
#[derive(Debug)]
pub enum Error {
    /// Error on tree creation
    Create,
    /// Error on trying appending an element
    Append,
    /// Error on merging
    Merge,
}

/// type alias for [StaticTree] with arity of 2
pub type StaticBinaryTree<
    const HEIGHT: usize,
    H,
    const MAX_INPUT_LEN: usize,
    PB = Proof<2, HEIGHT, H, MAX_INPUT_LEN>,
> = StaticTree<2, HEIGHT, H, MAX_INPUT_LEN, PB>;
/// Basic statically-allocated Merkle Tree
pub struct StaticTree<
    const BRANCH_FACTOR: usize,
    const HEIGHT: usize,
    H,
    const MAX_INPUT_LEN: usize,
    PB = Proof<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN>,
> where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    root: H::Output,
    prefixed: [Prefixed<BRANCH_FACTOR, H>; num_of_prefixed!(BRANCH_FACTOR, HEIGHT)],
}

// impl<'a, T, const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> TryFrom<&'a [T]> for
//     StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
// where
//     T: AsRef<[u8]> + Deref<Target = [u8]>,
//     [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
//     Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
//     H: HashT,
//     PB: ProofBuilder<BRANCH_FACTOR, H>,
// {
//     type Error = Error;
//     fn try_from(input: &'a [T]) -> Result<Self, Error> {
//         Self::create(input.len()).map(|this| this.create_inner(input, 0))
//     }
// }

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB>
    StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    const BASE_LAYER_SIZE: usize = layer_size!(BRANCH_FACTOR, HEIGHT, 0);

    fn create(input_len: usize) -> Result<Self, Error> {
        (input_len <= Self::BASE_LAYER_SIZE * BRANCH_FACTOR)
            .then_some(Self {
                root: Default::default(),
                prefixed: [Default::default(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)],
            })
            .ok_or(Error::Create)
    }

    /// creates a tree from an input if possible
    pub fn try_from<T: AsRef<[u8]> + Deref<Target = [u8]>>(input: &[T]) -> Result<Self, Error> {
        Self::create(input.len()).map(|this| this.create_inner(input, 0))
    }

    /// creates a tree from an input if possible
    pub fn from<T: AsRef<[u8]> + Deref<Target = [u8]>>(input: &[T]) -> Self {
        Self {
            root: Default::default(),
            prefixed: [Default::default(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)],
        }
        .create_inner(input, 0)
    }

    #[inline]
    pub(crate) fn pad_leaves(&mut self, from_index: usize) {
        let default_hash = Prefixed::<BRANCH_FACTOR, H>::default_hash();
        let default_hashes = [default_hash; BRANCH_FACTOR];
        let to_index = core::cmp::min(
            (from_index / BRANCH_FACTOR + 1) * BRANCH_FACTOR,
            max_leaves!(BRANCH_FACTOR, HEIGHT),
        );
        // pad first partial prefixed hashes in the base layer
        for i in from_index..to_index {
            let (index, offset) = location_in_prefixed::<BRANCH_FACTOR>(i);
            self.prefixed[index].hashes[offset] = default_hash;
        }
        // pad the rest of hashes in the base layer
        let start_prefixed_index = to_index / BRANCH_FACTOR;
        for i in start_prefixed_index..layer_size!(BRANCH_FACTOR, HEIGHT, 0) {
            self.prefixed[i].hashes = default_hashes;
        }
    }

    pub(crate) fn create_inner<T: AsRef<[u8]> + Deref<Target = [u8]>>(mut self, input: &[T], with_offset: usize) -> Self {
        let mut prefixed = [LEAF_HASH_PREPEND_VALUE; MAX_INPUT_LEN];
        
        let start_index = if input.iter().map(|d| d.len()).max() < Some(MAX_INPUT_LEN) {1} else {0};
        // fill the base layer
        for (i, d) in input.iter().enumerate() {
            prefixed[start_index..d.len() + start_index].copy_from_slice(d.as_ref());

            let (index, offset) = location_in_prefixed::<BRANCH_FACTOR>(i + with_offset);
            self.prefixed[index].hashes[offset] = H::hash(&prefixed[0..d.len() + start_index]);
        }

        self.pad_leaves(input.len());
        // fill the rest of layers
        self.fill_layers();
        self
    }

    /// creates a tree from hashed leaves (of another tree)
    pub fn try_from_leaves(leaves: &[Prefixed<BRANCH_FACTOR, H>]) -> Result<Self, Error> {
        Self::create(leaves.len()).map(|this| this.with_leaves_inner(leaves, 0))
    }

    pub(crate) fn with_leaves_inner(
        mut self,
        leaves: &[Prefixed<BRANCH_FACTOR, H>],
        with_offset: usize,
    ) -> Self {
        let mut i = with_offset;

        for leaf in leaves {
            for h in leaf.hashes {
                let (index, offset) = location_in_prefixed::<BRANCH_FACTOR>(i);

                self.prefixed[index].hashes[offset] = h;
                i += 1;
            }
        }
        self.pad_leaves(i);
        // fill the rest of layers
        self.fill_layers();
        self
    }

    fn fill_layers(&mut self) {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BASE_LAYER_SIZE;

        for h in 0..HEIGHT - 1 {
            // hash packed siblings of the current layer and fill the upper layer
            for i in start_ind..next_layer_ind {
                let offset = i & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
                let (j, _) = self.parent_index_and_base(i, h, start_ind);

                // hash concatenated siblings from the contiguous memory
                // each element has (BRANCH_FACTOR-1) siblings
                // store it as a parent hash
                self.prefixed[j].hashes[offset] = self.prefixed[i].hash_all();
            }
            let d = next_layer_ind - start_ind;
            // move on to the upper layer
            start_ind = next_layer_ind;
            next_layer_ind += d >> BRANCH_FACTOR.trailing_zeros();
        }

        self.root = self
            .prefixed
            .iter()
            .last()
            .expect("prefixed buffer is not empty. qed")
            .hash_all();
    }

    #[inline]
    fn parent_index_and_base(
        &self,
        index: usize,
        layer: usize,
        layer_base: usize,
    ) -> (usize, usize) {
        let curr_layer_len = layer_size!(BRANCH_FACTOR, HEIGHT, layer);
        let parent_layer_base = layer_base + curr_layer_len;
        let parent_index =
            parent_layer_base + ((index - layer_base) >> BRANCH_FACTOR.trailing_zeros());

        (parent_index, parent_layer_base)
    }

    fn replace_inner(&mut self, index: usize) {
        let mut layer_base = 0;
        let mut j = index / BRANCH_FACTOR;

        // start from the base layer and propagate the new hashes upwords
        for layer in 0..HEIGHT - 1 {
            let parent_hashed = self.prefixed[j].hash_all();

            let offset = j & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
            (j, layer_base) = self.parent_index_and_base(j, layer, layer_base);

            self.prefixed[j].hashes[offset] = parent_hashed;
        }
        self.root = self
            .prefixed
            .iter()
            .last()
            .expect("prefixed buffer is not empty. qed")
            .hash_all();
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB>
    StaticTreeTrait<BRANCH_FACTOR, H, PB>
    for StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    /// generate proof at given index on base layer
    fn generate_proof(&self, index: usize) -> PB {
        let mut proof = PB::from_root(self.root());
        let mut layer_base = 0;
        let mut j = index / BRANCH_FACTOR;
        let mut offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR (power of 2)

        for layer in 0..HEIGHT {
            proof.push(offset, self.prefixed[j]);

            offset = j & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
            (j, layer_base) = self.parent_index_and_base(j, layer, layer_base);
        }
        proof
    }
    /// replace an element at index with input
    /// panics if index is out of leaf layer bound
    fn replace(&mut self, index: usize, input: &[u8]) {
        let mut prefixed = [LEAF_HASH_PREPEND_VALUE; MAX_INPUT_LEN];
        let prefixed_len = input.len() + 1;
        prefixed[1..prefixed_len].copy_from_slice(input);

        let (prefixed_index, offset) = location_in_prefixed::<BRANCH_FACTOR>(index);

        self.prefixed[prefixed_index].hashes[offset] = H::hash(&prefixed[0..prefixed_len]);
        self.replace_inner(index);
    }

    fn replace_leaf(&mut self, index: usize, leaf: H::Output) {
        let (prefixed_index, offset) = location_in_prefixed::<BRANCH_FACTOR>(index);
        self.prefixed[prefixed_index].hashes[offset] = leaf;

        self.replace_inner(index);
    }
    fn root(&self) -> H::Output {
        self.root
    }
    fn leaves(&self) -> &[Prefixed<BRANCH_FACTOR, H>] {
        &self.prefixed[..layer_size!(BRANCH_FACTOR, HEIGHT, 0)]
    }
    fn base_layer_size(&self) -> usize {
        layer_size!(BRANCH_FACTOR, HEIGHT, 0)
    }
    fn branch_factor(&self) -> usize {
        BRANCH_FACTOR
    }
    fn height(&self) -> usize {
        HEIGHT
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Clone
    for StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Copy
    for StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Default
    for StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn default() -> Self {
        Self {
            root: Default::default(),
            prefixed: [Default::default(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)],
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> PartialEq
    for StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn eq(&self, other: &Self) -> bool {
        self.root() == other.root() && self.height() == other.height()
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Debug
    for StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "[branch factor]:   {BRANCH_FACTOR}")?;
        writeln!(f, "[height]:          {HEIGHT}")?;
        writeln!(
            f,
            "[num of prefixed]: {}",
            num_of_prefixed!(BRANCH_FACTOR, HEIGHT)
        )?;
        writeln!(
            f,
            "[total bytes]:     {}",
            size_of::<H::Output>()
                + size_of::<Prefixed<BRANCH_FACTOR, H>>() * num_of_prefixed!(BRANCH_FACTOR, HEIGHT)
        )?;
        writeln!(f, "[hash output len]: {} bytes", size_of::<H::Output>())?;
        write!(f, "{:?}", self.prefixed)
    }
}
