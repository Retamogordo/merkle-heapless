//#![cfg_attr(not(test), no_std)] 

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod tests;
mod foo;
mod utils;
pub mod compactable;
pub mod peak;

use core::fmt::Debug;
use core::hash::Hash;
use core::mem::size_of;
use crate::utils::hash_merged_slice;

pub trait HashT {
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq + Debug;

    fn hash(input: &[u8]) -> Self::Output;
}

pub trait ProofItemT<H: HashT>: Clone + Default + Debug {
    fn create(offset: usize, hashes: &[H::Output]) -> Self;
    fn hash_with_siblings(self, word_hash: H::Output) -> Option<H::Output>;
}

//#[derive(Debug)]
pub struct ProofItem<const BRANCH_FACTOR: usize, H: HashT> {
    hashes: Option<[H::Output; BRANCH_FACTOR]>,
    offset: usize,
}

impl<const BRANCH_FACTOR: usize, H: HashT> ProofItemT<H> for ProofItem<BRANCH_FACTOR, H> {

    fn create(offset: usize, hashes: &[H::Output]) -> Self {
        Self {
            offset,
            hashes: hashes[..BRANCH_FACTOR].try_into().ok()
        }
    }

    fn hash_with_siblings(mut self, word_hash: H::Output) -> Option<H::Output> {
        let bytes_in_chunk: usize = BRANCH_FACTOR * size_of::<H::Output>();

        self.hashes
            .as_mut()
            .map(|hashes| {
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

impl<const BRANCH_FACTOR: usize, H: HashT>  Debug for ProofItem<BRANCH_FACTOR, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        unimplemented!();
        // writeln!(f, "[branch factor]:   {BRANCH_FACTOR}")?;
    }
}


pub trait ProofBuilder<H: HashT>: Default {
    type Item: ProofItemT<H>;

    fn from_root(root: H::Output) -> Self;
    fn root(&self) -> H::Output;
    fn push_item(&mut self, item: Self::Item);
    fn as_raw(&self) -> (H::Output, &[Self::Item]);
}

pub trait ProofValidator {
    fn validate(self, input: &[u8]) -> bool;
}

pub struct Proof<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT>
where [(); HEIGHT - 1]: Sized {
    root: H::Output,
    height: usize,
    items: [<Self as ProofBuilder<H>>::Item; HEIGHT - 1],
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> ProofBuilder<H> for Proof<BRANCH_FACTOR, HEIGHT, H>
where [(); HEIGHT - 1]: Sized {
    type Item = ProofItem<BRANCH_FACTOR, H>;

    fn from_root(root: H::Output) -> Self {
        Self {
            root,
            items: [ProofItem::default(); HEIGHT - 1],
            height: 0,
        }
    }
    fn root(&self) -> H::Output {
        self.root
    } 

    fn push_item(&mut self, item: Self::Item) {
        self.items[self.height] = item;
        self.height += 1;
    }

    fn as_raw(&self) -> (H::Output, &[Self::Item]) {
        (self.root, &self.items[..])
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> ProofValidator for Proof<BRANCH_FACTOR, HEIGHT, H>
where [(); HEIGHT - 1]: Sized {
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

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> Default for Proof<BRANCH_FACTOR, HEIGHT, H>
where [(); HEIGHT - 1]: Sized {
    fn default() -> Self {
        Self {
            root: Default::default(),
            items: [Default::default(); HEIGHT - 1],
            height: 0,
        }
    }
}

pub trait HeaplessTreeT<H: HashT, PB: ProofBuilder<H>> {
//    type Proof: ProofBuilder<H>;

    fn generate_proof(&mut self, index: usize) -> PB;
//    fn generate_proof(&mut self, index: usize, proof: &mut PB );

    fn replace(&mut self, index: usize, input: &[u8]);
    fn remove(&mut self, index: usize);
    fn root(&self) -> H::Output;
    fn leaves(&self) -> &[H::Output];
    fn base_layer_size(&self) -> usize;
    fn branch_factor(&self) -> usize;
    fn height(&self) -> usize;
} 

pub type HeaplessBinaryTree<const HEIGHT: usize, H, PB = Proof<2, HEIGHT, H>> = HeaplessTree<2, HEIGHT, H, PB>;

pub struct HeaplessTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB = Proof<BRANCH_FACTOR, HEIGHT, H>>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
    PB: ProofBuilder<H>,
{
    hashes: [H::Output; total_size!(BRANCH_FACTOR, HEIGHT)],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> 
    HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
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

//     fn generate_proof_new(&mut self, index: usize) -> PB {
//         let mut proof = PB::from_root(Default::default());
// //        self.generate_proof(index, &mut proof);
//         proof
//     }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> HeaplessTreeT<H, PB> 
    for HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
//impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> HeaplessTreeT<H, ProofItem<BRANCH_FACTOR, H>> for HeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    H: HashT,
    PB: ProofBuilder<H>,
{
//    type Proof = Proof<BRANCH_FACTOR, HEIGHT, H> where [(); HEIGHT - 1]: Sized;
//    type Proof = Proof<BRANCH_FACTOR, H> where [(); MAX_PROOF_HEIGHT]: Sized;

    /// generate proof at given index on base layer
    /// panics on index out of bounds ( >= leaf number )
//    fn generate_proof(&mut self, index: usize, proof: &mut PB) {
    fn generate_proof(&mut self, index: usize) -> PB {
        let mut proof = PB::from_root(self.root());
        //    type Proof = Proof<BRANCH_FACTOR, H> where [(); MAX_PROOF_HEIGHT]: Sized;

        let mut layer_base = 0;
        let mut index = index;

        for layer in 0..HEIGHT - 1{
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR (power of 2)
            let aligned = index - offset;

            proof.push_item(PB::Item::create(offset, &self.hashes[aligned..]));
            //     ProofItem {
            //         hashes: self.hashes[aligned..aligned + BRANCH_FACTOR].try_into().ok(),
            //         offset,
            // });

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);
        }
        proof
    }

    /// replace an element at index with input
    /// panics if index is out of leaf layer bound
    fn replace(&mut self, index: usize, input: &[u8]) {
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
        self.replace(index, &[]);
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

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Clone 
    for HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn clone(&self) -> Self { 
        Self {
            hashes: self.hashes.clone(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Copy 
    for HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
    PB: ProofBuilder<H>,
{}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> PartialEq 
    for HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn eq(&self, other: &Self) -> bool { 
        self.hashes == other.hashes    
    }
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Debug 
    for HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        writeln!(f, "[branch factor]:   {BRANCH_FACTOR}")?;
        writeln!(f, "[layers]:          {HEIGHT}")?;
        writeln!(f, "[total size]:      {}", Self::TOTAL_SIZE)?;
        writeln!(f, "[hash output len]: {} bytes", size_of::<H::Output>())?;
        write!(f, "{:?}", self.hashes)
    }
}

