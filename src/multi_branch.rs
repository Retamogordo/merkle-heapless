#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use core::fmt::Debug;
use core::hash::Hash;
use core::mem::size_of;

pub trait HashT {
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq + Debug;

    fn hash(input: &[u8]) -> Self::Output;
}

#[derive(Debug)]
struct ProofItem<const BRANCH_FACTOR: usize, H: HashT> {
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

impl<const BRANCH_FACTOR: usize, H: HashT> ProofItem<BRANCH_FACTOR, H>
{
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

pub struct Proof<const BRANCH_FACTOR: usize, const LAYERS: usize, H: HashT>
where [(); LAYERS - 1]: Sized {
    items: [ProofItem<BRANCH_FACTOR, H>; LAYERS - 1]
}

impl <const BRANCH_FACTOR: usize, const LAYERS: usize, H: HashT> Proof<BRANCH_FACTOR, LAYERS, H>
where [(); LAYERS - 1]: Sized {
    /// verifies that the input was contained in the Merkle tree that generated this proof
    pub fn validate(self, root: &H::Output, input: &[u8]) -> bool {
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

macro_rules! total_size {
    ($branch_factor:expr, $layers:expr) => {
        ((1 << ($branch_factor.trailing_zeros() as usize * $layers)) - 1) / ($branch_factor - 1)
    };
}

macro_rules! layer_size {
    ($branch_factor:expr, $layers:expr, $layer_index:expr) => {
        1 << ($branch_factor.trailing_zeros() as usize * ($layers - $layer_index - 1))
    };
}

pub struct MerkleTree<const BRANCH_FACTOR: usize, const LAYERS: usize, H>
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    H: HashT,
{
    hashes: [H::Output; total_size!(BRANCH_FACTOR, LAYERS)],
}

impl<const BRANCH_FACTOR: usize, const LAYERS: usize, H> 
    MerkleTree<BRANCH_FACTOR, LAYERS,H>
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    H: HashT,
{
    const TOTAL_SIZE: usize = total_size!(BRANCH_FACTOR, LAYERS);
    const BASE_LAYER_SIZE: usize = layer_size!(BRANCH_FACTOR, LAYERS, 0);
    const BYTES_IN_CHUNK: usize = BRANCH_FACTOR * size_of::<H::Output>();

    // panics if LAYERS == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
//        println!("total size: {}", Self::TOTAL_SIZE);
        Self::try_from_with_scrambling(input, &[&[]])
    }

    pub fn try_from_with_scrambling(input: &[&[u8]], scrambling_alphabet: &[&[u8]]) -> Result<Self, ()> {
        // check input can be hold in base layer and branch factor is of power of 2
        if input.len() > Self::BASE_LAYER_SIZE 
            || BRANCH_FACTOR >> BRANCH_FACTOR.trailing_zeros() != 1 {
//            || (scrambling_alphabet.len() != 1 && scrambling_alphabet.len() < Self::BASE_LAYER_SIZE){
            return Err(());
        }

        let mut this = Self {
            hashes: [H::Output::default(); total_size!(BRANCH_FACTOR, LAYERS)],
        };
        // fill the base layer
        let mut i = 0;
        for d in input {
//            println!("d: {:?}", d);
            this.hashes[i] = H::hash(d);
            i += 1;
        }
        // pad the rest of hashes in the base layer
        while i < Self::BASE_LAYER_SIZE {
            this.hashes[i] = H::hash(scrambling_alphabet[i % scrambling_alphabet.len()]);
            i += 1;
        }
        // fill the rest of layers
        this.fill_layers();

        Ok(this)
    }

    fn fill_layers(&mut self) {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BASE_LAYER_SIZE;

        while next_layer_ind < Self::TOTAL_SIZE {
            let mut j = next_layer_ind;
            // hash packed siblings of the current layer and fill the upper layer
            for i in (start_ind..j).step_by(BRANCH_FACTOR) {
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

    /// generate proof at given index on base layer
    /// panics on index out of bounds ( >= leaf number )
    pub fn generate_proof(&mut self, index: usize) -> (H::Output, Proof<BRANCH_FACTOR, LAYERS, H>) 
    where [(); LAYERS - 1]: Sized {
//        pub fn generate_proof(&mut self, index: usize) -> Result<(H::Output, [ProofItem<BRANCH_FACTOR, H>; LAYERS - 1]), ()> {

        let mut proof = [ProofItem::default(); LAYERS - 1];
        let mut layer_base = 0;
        let mut index = index;

//        println!("build_path -> index: {}", index);
        for layer in 0..LAYERS - 1 {
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR (power of 2)
            let aligned = index - offset;

            proof[layer] = ProofItem {
                hashes: self.hashes[aligned..aligned + BRANCH_FACTOR].try_into().ok(),
                offset,
            };
            
            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);
//            println!("gen -> index: {}", index);
        }

        (self.hashes[index], Proof{ items: proof })
    }

    /// replace an element at index with input
    /// panics if index is out of leaf layer bound
    pub fn insert(&mut self, index: usize, input: &[u8]) {
        let mut layer_base = 0;
        let mut index = index;

        self.hashes[index] = H::hash(input);

        // start from the base layer and propagate the new hashes upwords
        for layer in 0..LAYERS - 1 {
            let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
            let aligned = index - offset;

            let parent_hashed = hash_merged_slice::<H>(&self.hashes[aligned..], Self::BYTES_IN_CHUNK);

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);

            self.hashes[index] = parent_hashed;
        }
    }

    // remove element by inserting nothing
    // panics if index is out of leaf layer bound
    pub fn remove(&mut self, index: usize) {
        self.insert(index, &[]);
    }

    pub fn root(&self) -> H::Output {
        self.hashes[Self::TOTAL_SIZE - 1]
    }
    
    fn parent_index_and_base(&self, curr_index: usize, layer: usize, layer_base: usize) -> (usize, usize) {
        let curr_layer_len = layer_size!(BRANCH_FACTOR, LAYERS, layer);
//        println!("gen -> curr_layer_len: {}", curr_layer_len);
        let parent_layer_base = layer_base + curr_layer_len;
        let parent_index = parent_layer_base + (curr_index - layer_base) / BRANCH_FACTOR;

        (parent_index, parent_layer_base)
    }
}

impl<const BRANCH_FACTOR: usize, const LAYERS: usize, H> Clone for MerkleTree<BRANCH_FACTOR, LAYERS, H> 
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    H: HashT,
{
    fn clone(&self) -> Self { 
        Self {
            hashes: self.hashes.clone(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, const LAYERS: usize, H> PartialEq for MerkleTree<BRANCH_FACTOR, LAYERS, H> 
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    H: HashT,
{
    fn eq(&self, other: &Self) -> bool { 
        self.hashes == other.hashes    
    }
}

impl <const BRANCH_FACTOR: usize, const LAYERS: usize, H> Debug for MerkleTree<BRANCH_FACTOR, LAYERS, H> 
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    H: HashT,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        writeln!(f, "[branch factor]:   {BRANCH_FACTOR}")?;
        writeln!(f, "[layers]:          {LAYERS}")?;
        writeln!(f, "[total size]:      {}", Self::TOTAL_SIZE)?;
        writeln!(f, "[hash output len]: {} bytes", size_of::<H::Output>())?;
        write!(f, "{:?}", self.hashes)
    }
}

// hash combined bytes from a contiguous memory chank
fn hash_merged_slice<H: HashT>(contiguous_array: &[H::Output], len: usize) -> H::Output {
    H::hash(
        unsafe { core::slice::from_raw_parts(contiguous_array[0].as_ref().as_ptr(), len) }
    )
}
