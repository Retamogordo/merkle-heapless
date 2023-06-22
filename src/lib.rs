//#![cfg_attr(not(test), no_std)] 

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod tests;
mod foo;
mod utils;
pub mod compactable;

use core::fmt::Debug;
use core::hash::Hash;
use core::mem::size_of;
use crate::utils::hash_merged_slice;

pub trait HashT {
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq + Debug;

    fn hash(input: &[u8]) -> Self::Output;
}

#[derive(Debug)]
pub struct ProofItem<const BRANCH_FACTOR: usize, H: HashT> {
    hashes: Option<[H::Output; BRANCH_FACTOR]>,
    offset: usize,
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

impl<const BRANCH_FACTOR: usize, H: HashT> ProofItem<BRANCH_FACTOR, H> {
    const BYTES_IN_CHUNK: usize = BRANCH_FACTOR * size_of::<H::Output>();

    fn hash_with_siblings(mut self, word_hash: H::Output) -> Option<H::Output> {
        self.hashes
            .as_mut()
            .map(|hashes| {
                hashes[self.offset] = word_hash;
                hash_merged_slice::<H>(&hashes[0..], Self::BYTES_IN_CHUNK)
            })
    }
}

pub trait ProofT<H: HashT> {
    fn validate(self, root: &H::Output, input: &[u8]) -> bool;
}


pub struct Proof<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT>
where [(); HEIGHT - 1]: Sized {
    items: [ProofItem<BRANCH_FACTOR, H>; HEIGHT - 1]
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> ProofT<H> for Proof<BRANCH_FACTOR, HEIGHT, H>
where [(); HEIGHT - 1]: Sized {
    /// verifies that the input was contained in the Merkle tree that generated this proof
    fn validate(self, root: &H::Output, input: &[u8]) -> bool {
        let mut curr_hash = Some(H::hash(&input));
        // start from the base layer, 
        // and for every item in the proof
        // put the hash derived from input into the proof item
        // at index stored in the proof item
        // and hash it with the siblings
        for item in self.items {
            curr_hash = curr_hash.and_then(|h| item.hash_with_siblings(h));
        }
        // validated iff the resulting hash is identical to the root
        curr_hash.as_ref() == Some(root)
    }  
}

pub trait HeaplessTreeT<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); HEIGHT - 1]: Sized,
    H: HashT,
{
    type Proof: ProofT<H>;

    fn generate_proof(&mut self, index: usize) -> (H::Output, Self::Proof) where [(); HEIGHT - 1]: Sized;

    fn insert(&mut self, index: usize, input: &[u8]);
    fn remove(&mut self, index: usize);
    fn root(&self) -> H::Output;
    fn leaves(&self) -> &[H::Output];
    fn base_layer_size(&self) -> usize;
    fn branch_factor(&self) -> usize;
    fn height(&self) -> usize;
} 

pub type HeaplessBinaryTree<const HEIGHT: usize, H> = HeaplessTree<2, HEIGHT, H>;

pub struct HeaplessTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
{
    hashes: [H::Output; total_size!(BRANCH_FACTOR, HEIGHT)],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> HeaplessTree<BRANCH_FACTOR, HEIGHT,H>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
{
    const TOTAL_SIZE: usize = total_size!(BRANCH_FACTOR, HEIGHT);
    const BASE_LAYER_SIZE: usize = layer_size!(BRANCH_FACTOR, HEIGHT, 0);
    const BYTES_IN_CHUNK: usize = BRANCH_FACTOR * size_of::<H::Output>();

    fn create_this(input_len: usize) -> Result<Self, ()> {
        if input_len > Self::BASE_LAYER_SIZE || BRANCH_FACTOR >> BRANCH_FACTOR.trailing_zeros() != 1 {
            return Err(());
        }

        Ok(Self {
            hashes: [H::Output::default(); total_size!(BRANCH_FACTOR, HEIGHT)],
        })
    }
    // panics if HEIGHT == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
        let this = Self::create_this(input.len())?;
        Self::try_from_inner(input, this)
    }

    fn try_from_inner(input: &[&[u8]], mut this: Self) -> Result<Self, ()> {
        // check input can be hold in base layer and branch factor is of power of 2
        // fill the base layer
        for (i, d) in input.iter().enumerate() {
            this.hashes[i] = H::hash(d);
        }
        // pad the rest of hashes in the base layer
        for i in input.len()..Self::BASE_LAYER_SIZE {
            this.hashes[i] = H::hash(&[]);
        }
        // fill the rest of layers
        this.fill_layers();

        Ok(this)
    }
    
    // panics if HEIGHT == 0
    pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
        let this = Self::create_this(leaves.len())?;
        Self::try_from_leaves_inner(leaves, this)
    }
    
    fn try_from_leaves_inner(leaves: &[H::Output], mut this: Self) -> Result<Self, ()> {
        for (i, leaf) in leaves.iter().enumerate() {
            this.hashes[i] = *leaf;
        }
        // pad the rest of hashes in the base layer
        for i in leaves.len()..Self::BASE_LAYER_SIZE {
            this.hashes[i] = H::hash(&[]);
        }
        // fill the rest of layers
        this.fill_layers();

        Ok(this)
    }
    
    fn fill_layers(&mut self) {
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
    }
    
    fn parent_index_and_base(&self, curr_index: usize, layer: usize, layer_base: usize) -> (usize, usize) {
        let curr_layer_len = layer_size!(BRANCH_FACTOR, HEIGHT, layer);
        let parent_layer_base = layer_base + curr_layer_len;
        let parent_index = parent_layer_base + (curr_index - layer_base) / BRANCH_FACTOR;

        (parent_index, parent_layer_base)
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> HeaplessTreeT<BRANCH_FACTOR, HEIGHT, H> for HeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); HEIGHT - 1]: Sized,
    H: HashT,
{
    type Proof = Proof<BRANCH_FACTOR, HEIGHT, H> where [(); HEIGHT - 1]: Sized;
    /// generate proof at given index on base layer
    /// panics on index out of bounds ( >= leaf number )
    fn generate_proof(&mut self, index: usize) -> (H::Output, Self::Proof) 
        where [(); HEIGHT - 1]: Sized {

        let mut proof = [ProofItem::default(); HEIGHT - 1];
        let mut layer_base = 0;
        let mut index = index;

        for layer in 0..HEIGHT - 1 {
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR (power of 2)
            let aligned = index - offset;

            proof[layer] = ProofItem {
                hashes: self.hashes[aligned..aligned + BRANCH_FACTOR].try_into().ok(),
                offset,
            };
            
            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);
        }

        (self.hashes[index], Self::Proof { items: proof })
    }

    /// replace an element at index with input
    /// panics if index is out of leaf layer bound
    fn insert(&mut self, index: usize, input: &[u8]) {
        let mut layer_base = 0;
        let mut index = index;

        self.hashes[index] = H::hash(input);

        // start from the base layer and propagate the new hashes upwords
        for layer in 0..HEIGHT - 1 {
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
            let aligned = index - offset;

            let parent_hashed = hash_merged_slice::<H>(&self.hashes[aligned..], Self::BYTES_IN_CHUNK);

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);

            self.hashes[index] = parent_hashed;
        }
    }

    // remove element by inserting nothing
    // panics if index is out of leaf layer bound
    fn remove(&mut self, index: usize) {
        self.insert(index, &[]);
    }

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

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> Clone for HeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
{
    fn clone(&self) -> Self { 
        Self {
            hashes: self.hashes.clone(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> PartialEq for HeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
{
    fn eq(&self, other: &Self) -> bool { 
        self.hashes == other.hashes    
    }
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H> Debug for HeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        writeln!(f, "[branch factor]:   {BRANCH_FACTOR}")?;
        writeln!(f, "[layers]:          {HEIGHT}")?;
        writeln!(f, "[total size]:      {}", Self::TOTAL_SIZE)?;
        writeln!(f, "[hash output len]: {} bytes", size_of::<H::Output>())?;
        write!(f, "{:?}", self.hashes)
    }
}

