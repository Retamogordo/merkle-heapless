#![cfg_attr(not(test), no_std)] 

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

mod tests;

use core::hash::Hash;

pub trait ConcatHashes<const OUTPUT_SIZE: usize>
where
    [(); 2 * OUTPUT_SIZE]: Sized,
{
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq;

    fn hash(input: &[u8]) -> Self::Output;

    fn concat_hashes(left: &[u8], right: &[u8]) -> Self::Output {
        let mut h = [u8::default(); 2 * OUTPUT_SIZE];

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
pub struct MerkleTree<const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H>
where
    [(); (1 << LAYERS) - 1]: Sized,
    [(); 2 * HASH_OUTPUT_SIZE]: Sized,
    H: ConcatHashes<HASH_OUTPUT_SIZE>,
{
    hashes: [H::Output; (1 << LAYERS) - 1],
    _marker: core::marker::PhantomData<H>,
}

impl<const LAYERS: usize, const HASH_OUTPUT_SIZE: usize, H> MerkleTree<LAYERS, HASH_OUTPUT_SIZE, H>
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

    pub fn generate_proof(
        &mut self,
        index: usize,
    ) -> (H::Output, [Sibling<H::Output>; LAYERS - 1]) {
        let mut proof = [Sibling::None; LAYERS - 1];
        let root = self.build_path(index, &mut proof);
        (root, proof)
    }

    pub fn total_size(&self) -> usize {
        self.hashes.len()
    }
    pub fn total_layers(&self) -> usize {
        LAYERS
    }

    fn fill_layers(&mut self) {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BOTTOM_LAYER_SIZE;

        while next_layer_ind < Self::TOTAL_SIZE {
            let mut j = next_layer_ind;

            for i in (start_ind..next_layer_ind).step_by(2) {
                let joined =
                    Self::merge_adjacent(self.hashes[i].as_ref(), self.hashes[i + 1].as_ref());

                self.hashes[j] = H::hash(joined);
                j += 1;
            }
            start_ind = next_layer_ind;
            next_layer_ind = j;
        }
    }

    fn merge_adjacent<'a>(left: &'a [u8], right: &'a [u8]) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(left.as_ptr(), left.len() + right.len()) }
    }

    // panics on index out of bounds ( >= leaf number )
    fn build_path(
        &mut self,
        index: usize,
        proof: &mut [Sibling<H::Output>; LAYERS - 1],
    ) -> H::Output {
        let mut proof_ind = 0;
        let mut layer_base = 0;
        let mut index = index;
        let mut layer_len = 1 << (LAYERS - 1);

        for _ in 0..LAYERS - 1 {
            proof[proof_ind] = match index & 1 {
                0 => Sibling::Right(self.hashes[index + 1]),
                _ => Sibling::Left(self.hashes[index - 1]),
            };

            proof_ind += 1;

            index = layer_len + (index + layer_base) / 2;
            layer_base += layer_len;
            layer_len >>= 1;
        }
        self.hashes[index]
    }
}

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
            Sibling::Left(h) => H::concat_hashes(h.as_ref(), curr_hash.as_ref()),
            Sibling::Right(h) => H::concat_hashes(curr_hash.as_ref(), h.as_ref()),
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