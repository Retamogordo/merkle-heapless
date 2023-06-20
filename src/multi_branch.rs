#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use core::fmt::Debug;
use core::hash::Hash;

#[derive(Debug)]
pub struct ProofItem1<const HASH_OUTPUT_SIZE: usize, const BRANCH_FACTOR: usize, H> 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    hashes: [H::Output; BRANCH_FACTOR],
    word_index: usize,
}

impl<const HASH_OUTPUT_SIZE: usize, 
    const BRANCH_FACTOR: usize, 
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>
> Copy for ProofItem1<HASH_OUTPUT_SIZE, BRANCH_FACTOR, H> 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized
{}

impl<const HASH_OUTPUT_SIZE: usize, 
    const BRANCH_FACTOR: usize, 
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>
> Clone for ProofItem1<HASH_OUTPUT_SIZE, BRANCH_FACTOR, H> 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
{
    fn clone(&self) -> Self { 
        Self {
            hashes: self.hashes.clone(),
            word_index: self.word_index.clone(),
        }
    }
}

impl<const HASH_OUTPUT_SIZE: usize, const BRANCH_FACTOR: usize, H> ProofItem1<HASH_OUTPUT_SIZE, BRANCH_FACTOR, H>
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized, 
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    fn hash(mut self, word_hash: H::Output) -> H::Output {
        self.hashes[self.word_index] = word_hash;
        hash_merged_slice::<HASH_OUTPUT_SIZE, H>(self.hashes.as_ref(), 0, BRANCH_FACTOR * HASH_OUTPUT_SIZE)
    }
}


#[derive(Debug)]
pub struct ProofItem<const HASH_OUTPUT_SIZE: usize, H: ConcatHashesMulti<HASH_OUTPUT_SIZE>> 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
{
    left_sibling: Option<H::Output>,
    word_index: usize,
    right_sibling: Option<H::Output>,
}

impl<const HASH_OUTPUT_SIZE: usize, H: ConcatHashesMulti<HASH_OUTPUT_SIZE>> Copy for ProofItem<HASH_OUTPUT_SIZE, H> 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
{
}
impl<const HASH_OUTPUT_SIZE: usize, H: ConcatHashesMulti<HASH_OUTPUT_SIZE>> Clone for ProofItem<HASH_OUTPUT_SIZE, H> 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
{
    fn clone(&self) -> Self { 
        Self {
            left_sibling: self.left_sibling.clone(),
            word_index: self.word_index.clone(),
            right_sibling: self.right_sibling.clone(),
        }
    }
}

impl<const HASH_OUTPUT_SIZE: usize, H> ProofItem<HASH_OUTPUT_SIZE, H>
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized, 
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    fn hash(self, word_hash: H::Output) -> H::Output 
    {
        let first_chunk = match self.left_sibling {
            Some(left) => H::concat_and_hash(&left, &word_hash),
            None => word_hash,
        };
        match self.right_sibling {
            Some(right) => H::concat_and_hash(&first_chunk, &right),
            None => first_chunk
        } 
    }
}

pub trait ConcatHashesMulti<const HASH_OUTPUT_SIZE: usize>
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
{
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq + Debug;

    fn hash(input: &[u8]) -> Self::Output;

    fn concat_and_hash(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let mut h = [u8::default(); 2 * HASH_OUTPUT_SIZE];

        let left = left.as_ref();
        let right = right.as_ref();
        for i in 0..left.len() {
            h[i] = left[i];
        }

        let mut j = left.len();
        for i in 0..right.len() {
            h[j] = right[i];
            j += 1;
        }
        Self::hash(&h)
    }
}

macro_rules! total_size {
    ($branch_factor:expr, $layers:expr) => {
//        ((($branch_factor << $layers) - 1) / ($branch_factor - 1))
        ((1 << ($branch_factor.trailing_zeros() as usize * $layers)) - 1) / ($branch_factor - 1)
    };
}

macro_rules! layer_size {
    ($branch_factor:expr, $layers:expr, $layer_index:expr) => {
        1 << ($branch_factor.trailing_zeros() as usize * ($layers - $layer_index - 1))
    };
}

//#[derive(Debug)]
pub struct MerkleTreeMulti<const BRANCH_FACTOR: usize, const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H>
where
//    [(); ((BRANCH_FACTOR << LAYERS) - 1 / (BRANCH_FACTOR - 1))]: Sized,
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    hashes: [H::Output; total_size!(BRANCH_FACTOR, LAYERS)],
    _marker: core::marker::PhantomData<H>,
}

impl<const BRANCH_FACTOR: usize, const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> 
    MerkleTreeMulti<BRANCH_FACTOR, LAYERS, HASH_OUTPUT_SIZE, H>
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    const TOTAL_SIZE: usize = total_size!(BRANCH_FACTOR, LAYERS);
    const BOTTOM_LAYER_SIZE: usize = layer_size!(BRANCH_FACTOR, LAYERS, 0);
    // panics if LAYERS == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
        println!("total size: {}", Self::TOTAL_SIZE);
        if input.len() > Self::BOTTOM_LAYER_SIZE {
            return Err(());
        }
        let mut this = Self {
            hashes: [H::Output::default(); total_size!(BRANCH_FACTOR, LAYERS)],
            _marker: Default::default(),
        };
        let mut i = 0;
        for d in input {
            println!("d: {:?}", d);
            this.hashes[i] = H::hash(d);
            i += 1;
        }
        // pad the rest of hashes
        while i < Self::BOTTOM_LAYER_SIZE {
            this.hashes[i] = H::hash(&[]);
            //            this.hashes[i] = H::Output::default();
            i += 1;
        }

        this.fill_layers();

        Ok(this)
    }

    fn fill_layers(&mut self) {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BOTTOM_LAYER_SIZE;

        while next_layer_ind < Self::TOTAL_SIZE {
            let mut j = next_layer_ind;

            for i in (start_ind..next_layer_ind).step_by(BRANCH_FACTOR) {
                self.hashes[j] = hash_merged_slice::<HASH_OUTPUT_SIZE, H>(self.hashes.as_ref(), i, BRANCH_FACTOR * HASH_OUTPUT_SIZE);

                j += 1;
            }
            start_ind = next_layer_ind;
            next_layer_ind = j;
        }
    }

    pub fn generate_proof(&mut self, index: usize) -> (H::Output, [ProofItem1<HASH_OUTPUT_SIZE, BRANCH_FACTOR, H>; LAYERS - 1]) {
        let mut proof = [
            ProofItem1 {
                hashes: [Default::default(); BRANCH_FACTOR],
                word_index: 0,
            }; 
            LAYERS - 1
        ];
        let root = self.build_path(index, &mut proof);

        (root, proof)
    }
    
    // panics on index out of bounds ( >= leaf number )
    fn build_path(&mut self, index: usize, proof: &mut [ProofItem1<HASH_OUTPUT_SIZE, BRANCH_FACTOR, H>; LAYERS - 1]) -> H::Output {
        let mut layer_base = 0;
        let mut index = index;

        println!("build_path -> index: {}", index);
        for layer in 0..LAYERS - 1 {
            let index_in_proof_item = index % BRANCH_FACTOR;
//            let index_aligned = (index / BRANCH_FACTOR) * BRANCH_FACTOR;
            let index_aligned = index - index_in_proof_item;

            proof[layer] = ProofItem1 {
                hashes: self.hashes[index_aligned..index_aligned + BRANCH_FACTOR].try_into().unwrap(),
                word_index: index_in_proof_item,
            };
            
            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);
            println!("gen -> index: {}", index);
        }
        self.hashes[index]
    }

    fn parent_index_and_base(&self, curr_index: usize, layer: usize, layer_base: usize) -> (usize, usize) {
//        let curr_layer_len = 1 << (LAYERS - layer - 1);
        let curr_layer_len = layer_size!(BRANCH_FACTOR, LAYERS, layer);
        println!("gen -> curr_layer_len: {}", curr_layer_len);
        let parent_layer_base = layer_base + curr_layer_len;
        let parent_index = parent_layer_base + (curr_index - layer_base) / BRANCH_FACTOR;

        (parent_index, parent_layer_base)
    }
}

impl <const BRANCH_FACTOR: usize, const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> core::fmt::Debug 
    for MerkleTreeMulti<BRANCH_FACTOR, LAYERS, HASH_OUTPUT_SIZE, H> 
where
    [(); total_size!(BRANCH_FACTOR, LAYERS)]: Sized,
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        writeln!(f, "[branch factor]:   {BRANCH_FACTOR}");
        writeln!(f, "[layers]:          {LAYERS}");
        writeln!(f, "[total size]:      {}", Self::TOTAL_SIZE);
        writeln!(f, "[hash output len]: {HASH_OUTPUT_SIZE} bytes");
        write!(f, "{:?}", self.hashes)
    }
}

pub fn validate_proof_multi<const BRANCH_FACTOR: usize, const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> (
    root: &H::Output,
    word: &[u8],
    proof: [ProofItem1<HASH_OUTPUT_SIZE, BRANCH_FACTOR, H>; LAYERS - 1]
) -> bool
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    let mut curr_hash = H::hash(&word);

    for item in proof {
        curr_hash = item.hash(curr_hash);
        println!("curr_hash: {:?}", curr_hash);
    }
    println!("curr_hash: {:?}, root: {:?}", curr_hash, root);
    &curr_hash == root
}  

fn hash_merged_slice<const HASH_OUTPUT_SIZE: usize, H>(contiguous_array: &[H::Output], start_index: usize, len: usize) -> H::Output 
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashesMulti<HASH_OUTPUT_SIZE>,
{
    H::hash(
        unsafe { core::slice::from_raw_parts(contiguous_array[start_index].as_ref().as_ptr(), len) }
    )
}
