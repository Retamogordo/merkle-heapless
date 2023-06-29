#![cfg_attr(not(test), no_std)] 

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(trivial_bounds)]

pub mod traits;
pub mod proof;
pub mod mergeable;
pub mod compactable;
mod utils;
mod tests;

#[cfg(feature = "mmr_macro")]
pub use mmr_macro;

use core::fmt::Debug;
use core::mem::size_of;

use crate::utils::{Assert, IsTrue, hash_merged_slice};
use crate::traits::{HashT, ProofItemT, ProofBuilder, ProofValidator, StaticTreeTrait};
use crate::proof::{ProofItem, Proof};

pub type HeaplessBinaryTree<const HEIGHT: usize, H, PB = Proof<2, HEIGHT, H>> = StaticTree<2, HEIGHT, H, PB>;

pub struct StaticTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB = Proof<BRANCH_FACTOR, HEIGHT, H>>
where 
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    hashes: [H::Output; total_size!(BRANCH_FACTOR, HEIGHT)],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> StaticTree<BRANCH_FACTOR, HEIGHT, H, PB>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    const TOTAL_SIZE: usize = total_size!(BRANCH_FACTOR, HEIGHT);
    const BASE_LAYER_SIZE: usize = layer_size!(BRANCH_FACTOR, HEIGHT, 0);
    const BYTES_IN_CHUNK: usize = BRANCH_FACTOR * size_of::<H::Output>();

    fn create(input_len: usize) -> Result<Self, ()> {
        if input_len > Self::BASE_LAYER_SIZE || BRANCH_FACTOR >> BRANCH_FACTOR.trailing_zeros() != 1 {
            return Err(());
        }

        Ok(Self {
            hashes: [H::Output::default(); total_size!(BRANCH_FACTOR, HEIGHT)],
        })
    }
    // panics if HEIGHT == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
        Self::create(input.len()).map(|this| this.from_inner(input))
    }

    fn from_inner(mut self, input: &[&[u8]]) -> Self {
        // check input can be hold in base layer and branch factor is of power of 2
        // fill the base layer
        for (i, d) in input.iter().enumerate() {
            self.hashes[i] = H::hash(d);
        }
        // pad the rest of hashes in the base layer
        for i in input.len()..Self::BASE_LAYER_SIZE {
            self.hashes[i] = H::hash(&[]);
        }
        // fill the rest of layers
        self.fill_layers()
    }
    
    // panics if HEIGHT == 0
    pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
        Self::create(leaves.len()).map(|this| this.from_leaves_inner(leaves))
    }
    
    fn from_leaves_inner(mut self, leaves: &[H::Output]) -> Self {
        for (i, leaf) in leaves.iter().enumerate() {
            self.hashes[i] = *leaf;
        }
        // pad the rest of hashes in the base layer
        for i in leaves.len()..Self::BASE_LAYER_SIZE {
            self.hashes[i] = H::hash(&[]);
        }
        // fill the rest of layers
        self.fill_layers()
    }
    
    fn fill_layers(mut self) -> Self {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BASE_LAYER_SIZE;

        let mut j = next_layer_ind;
        while next_layer_ind < Self::TOTAL_SIZE {
            // hash packed siblings of the current layer and fill the upper layer
            for i in (start_ind..next_layer_ind).step_by(BRANCH_FACTOR) {
                // hash concatenated siblings from the contiguous memory
                // each element has (BRANCH_FACTOR-1) siblings
                // store it as a parent hash
                self.hashes[j] = hash_merged_slice::<H>(&self.hashes[i..], Self::BYTES_IN_CHUNK);

                j += 1;
            }
            // move on to the upper layer
            start_ind = next_layer_ind;
            next_layer_ind = j;
        }
        self
    }
    
    fn parent_index_and_base(&self, height: usize, layer: usize, layer_base: usize) -> (usize, usize) {
        let curr_layer_len = layer_size!(BRANCH_FACTOR, HEIGHT, layer);
        let parent_layer_base = layer_base + curr_layer_len;
        let parent_index = parent_layer_base + (height - layer_base) / BRANCH_FACTOR;

        (parent_index, parent_layer_base)
    }

    fn replace_inner(&mut self, index: usize, ) {
        let mut layer_base = 0;
        let mut index = index;

        // start from the base layer and propagate the new hashes upwords
        for layer in 0..HEIGHT - 1 {
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
            let aligned = index - offset;

            let parent_hashed = hash_merged_slice::<H>(&self.hashes[aligned..], Self::BYTES_IN_CHUNK);

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);

            self.hashes[index] = parent_hashed;
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> StaticTreeTrait<H, PB> for StaticTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    /// generate proof at given index on base layer
    /// panics on index out of bounds ( >= leaf number )
    fn generate_proof(&mut self, index: usize) -> PB {
        let mut proof = PB::from_root(self.root());
        let mut layer_base = 0;
        let mut index = index;

        for layer in 0..HEIGHT - 1{
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR (power of 2)
            let aligned = index - offset;

            proof.push(offset, &self.hashes[aligned..]);

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);
        }
        proof
    }

    /// replace an element at index with input
    /// panics if index is out of leaf layer bound
    fn replace(&mut self, index: usize, input: &[u8]) {
        self.hashes[index] = H::hash(input);
        self.replace_inner(index);
    }

    fn replace_leaf(&mut self, index: usize, leaf: H::Output) {
        self.hashes[index] = leaf;
        self.replace_inner(index);
    }
    // remove element by inserting nothing
    // panics if index is out of leaf layer bound
    
    fn root(&self) -> H::Output {
        self.hashes[Self::TOTAL_SIZE - 1]
    }

    fn leaves(&self) -> &[H::Output] {
        &self.hashes[..layer_size!(BRANCH_FACTOR, HEIGHT, 0)]
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

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Clone for StaticTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn clone(&self) -> Self { 
        Self {
            hashes: self.hashes.clone(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Copy for StaticTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> PartialEq for StaticTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn eq(&self, other: &Self) -> bool { 
        self.hashes == other.hashes    
    }
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Debug for StaticTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        writeln!(f, "[branch factor]:   {BRANCH_FACTOR}")?;
        writeln!(f, "[height]:          {HEIGHT}")?;
        writeln!(f, "[total size]:      {}", Self::TOTAL_SIZE)?;
        writeln!(f, "[hash output len]: {} bytes", size_of::<H::Output>())?;
        write!(f, "{:?}", self.hashes)
    }
}

