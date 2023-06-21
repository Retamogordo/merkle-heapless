#![cfg_attr(not(test), no_std)] 

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod multi_branch;
mod tests;

use core::hash::Hash;

pub trait ConcatHashes<const OUTPUT_SIZE: usize>
where
    [(); 2 * OUTPUT_SIZE]: Sized,
{
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq;

    fn hash(input: &[u8]) -> Self::Output;

//    fn concat_and_hash(hashes: &[&[u8]; 2]) -> Self::Output {
    fn concat_and_hash(left: &[u8], right: &[u8]) -> Self::Output {
        let mut h = [u8::default(); 2 * OUTPUT_SIZE];
        // let left = hashes[0];
        // let right = hashes[1];

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

#[derive(Debug)]
pub struct MerkleTreeBinary<const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H>
where
    [(); (1 << LAYERS) - 1]: Sized,
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashes<HASH_OUTPUT_SIZE>,
{
    hashes: [H::Output; (1 << LAYERS) - 1],
    _marker: core::marker::PhantomData<H>,
}

impl<const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> MerkleTreeBinary<LAYERS, HASH_OUTPUT_SIZE, H>
where
    [(); (1 << LAYERS) - 1]: Sized,
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashes<HASH_OUTPUT_SIZE>,
{
    const TOTAL_SIZE: usize = (1 << LAYERS) - 1;
    const BOTTOM_LAYER_SIZE: usize = 1 << (LAYERS - 1);
    // panics if LAYERS == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
        if input.len() > Self::BOTTOM_LAYER_SIZE {
            return Err(());
        }
        let mut this = Self {
            hashes: [H::Output::default(); (1 << LAYERS) - 1],
            _marker: Default::default(),
        };
        let mut i = 0;
        for d in input {
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

    pub fn generate_proof(&mut self, index: usize) -> (H::Output, [Sibling<H::Output>; LAYERS - 1]) {
        let mut proof = [Sibling::None; LAYERS - 1];
        let root = self.build_path(index, &mut proof);

        (root, proof)
    }

    // panics if index is out of leaf layer bound
    pub fn insert(&mut self, index: usize, input: &[u8]) {
        let mut layer_base = 0;
        let mut index = index;

        self.hashes[index] = H::hash(input);

        for layer in 0..LAYERS - 1 {
//            let parent_merged = self.merged_slice(index - (index % 2), 2 * HASH_OUTPUT_SIZE);
            let parent_hashed = self.hash_merged_slice(index - (index % 2), 2 * HASH_OUTPUT_SIZE);
            // let parent_merged = match index & 1 {
            //     0 => self.merged_slice(index, 2 * HASH_OUTPUT_SIZE),
            //     _ => self.merged_slice(index - 1, 2 * HASH_OUTPUT_SIZE)
            // };

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);

//            self.hashes[index] = H::hash(&parent_merged);
            self.hashes[index] = parent_hashed;
        }
    }

    // panics if index is out of leaf layer bound
    pub fn remove(&mut self, index: usize) {
        self.insert(index, &[]);
    }

    pub fn total_size(&self) -> usize {
        self.hashes.len()
    }
    pub fn total_layers(&self) -> usize {
        LAYERS
    }
    pub fn root(&self) -> H::Output {
        self.hashes[Self::TOTAL_SIZE - 1]
    }

    fn fill_layers(&mut self) {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BOTTOM_LAYER_SIZE;

        while next_layer_ind < Self::TOTAL_SIZE {
            let mut j = next_layer_ind;

            for i in (start_ind..next_layer_ind).step_by(2) {
//                let merged = self.merged_slice(i, 2 * HASH_OUTPUT_SIZE);

//                self.hashes[j] = H::hash(merged);
                self.hashes[j] = self.hash_merged_slice(i, 2 * HASH_OUTPUT_SIZE);
                j += 1;
            }
            start_ind = next_layer_ind;
            next_layer_ind = j;
        }
    }

    fn hash_merged_slice(&self, start_index: usize, len: usize) -> H::Output {
//        fn hash_merged_slice(&self, start_index: usize, len: usize) -> &[u8] {
        H::hash(
            unsafe { core::slice::from_raw_parts(self.hashes[start_index].as_ref().as_ptr(), len) }
        )
    }

    // panics on index out of bounds ( >= leaf number )
    fn build_path(&mut self, index: usize, proof: &mut [Sibling<H::Output>; LAYERS - 1]) -> H::Output {
        let mut layer_base = 0;
        let mut index = index;

        for layer in 0..LAYERS - 1 {
            proof[layer] = match index % 2 {
                0 => Sibling::Right(self.hashes[index + 1]),
                _ => Sibling::Left(self.hashes[index - 1]),
            };

            (index, layer_base) = self.parent_index_and_base(index, layer, layer_base);
        }
        self.hashes[index]
    }

    fn parent_index_and_base(&self, curr_index: usize, layer: usize, layer_base: usize) -> (usize, usize) {
        let curr_layer_len = 1 << (LAYERS - layer - 1);
        let parent_index = curr_layer_len + (curr_index + layer_base) / 2;
        let parent_layer_base = layer_base + curr_layer_len;

        (parent_index, parent_layer_base)
    }
}

// pub fn validate_proof<const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> (
//     root: &H::Output,
//     word: &[u8],
//     proof: [Sibling<H::Output>; LAYERS - 1],
// ) -> bool
// where
//     [(); 2 * HASH_OUTPUT_SIZE]: Sized,
//     H: ConcatHashes<HASH_OUTPUT_SIZE>,
// {
//     let mut curr_hash = H::hash(&word);

//     for sibling in proof {
//         let mut hashes_to_concat = [Default::default(); 2];
//         curr_hash = match sibling {
//             Sibling::Left(h) => { hashes_to_concat[0] = h.as_ref(); hashes_to_concat[1] = curr_hash.as_ref(); H::concat_and_hash(&hashes_to_concat)},
//             Sibling::Right(h) => { hashes_to_concat[0] = curr_hash.as_ref(); hashes_to_concat[1] = h.as_ref(); H::concat_and_hash(&hashes_to_concat)},
//             Sibling::None => unreachable!("sibling is None"),
//         };
//     }
//     &curr_hash == root
// }
pub fn validate_proof<const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> (
    root: &H::Output,
    word: &[u8],
    proof: [Sibling<H::Output>; LAYERS - 1],
) -> bool
where
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashes<HASH_OUTPUT_SIZE>,
{
    let mut curr_hash = H::hash(&word);

    for sibling in proof {
        curr_hash = match sibling {
            Sibling::Left(h) => H::concat_and_hash(h.as_ref(), curr_hash.as_ref()),
            Sibling::Right(h) => H::concat_and_hash(curr_hash.as_ref(), h.as_ref()),
            Sibling::None => unreachable!("sibling is None"),
        };
    }
    &curr_hash == root
}

#[derive(Clone, Copy, Debug)]
pub enum Sibling<H> {
    Left(H),
    Right(H),
    None,
}

// impl<H> Sibling<H> {
//     fn concat_and_hash(&self) {
//     }
// }
