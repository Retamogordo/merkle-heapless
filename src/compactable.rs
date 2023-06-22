use core::fmt::Debug;

use crate::{HashT, MerkleTree,  HeaplessTree, HeaplessBinaryTree, Proof, ProofItem, total_size, layer_size};

pub struct CompactableHeaplessTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
    H: HashT,
{
    tree: HeaplessTree<BRANCH_FACTOR, HEIGHT, H>,
    leaf_number: usize,
    leaves_present: [bool; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT,H>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
    H: HashT,
{
    // panics if HEIGHT == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
//        println!("total size: {}", Self::TOTAL_SIZE);
        let mut this = Self {
            tree: HeaplessTree::try_from(input)?,
            leaf_number: input.len(),
            leaves_present: [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
        };   
        for i in 0..input.len() {
            this.leaves_present[i] = true;
        }
        Ok(this)
    }

    pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
        let mut this = Self {
            tree: HeaplessTree::try_from_leaves(leaves)?,
            leaf_number: leaves.len(),
            leaves_present: [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
        };   
        for i in 0..leaves.len() {
            this.leaves_present[i] = true;
        }
        Ok(this)
    }

    pub fn try_compact(self) -> Result<CompactableHeaplessTree<BRANCH_FACTOR, {HEIGHT - 1}, H>, Self> 
    where
        [(); total_size!(BRANCH_FACTOR, {HEIGHT - 1})]: Sized,
        [(); layer_size!(BRANCH_FACTOR, {HEIGHT - 1}, 0)]: Sized,
        H: HashT, 
    {
        if self.leaf_number > layer_size!(BRANCH_FACTOR, {HEIGHT - 1}, 0) {
            return Err(self);
        }

        let mut leaves = [H::Output::default(); layer_size!(BRANCH_FACTOR, {HEIGHT - 1}, 0)];
        let mut j = 0;
        for (i, leaf) in self.tree.leaves().iter().enumerate() {
            if self.leaves_present[i] {
                leaves[j] = *leaf;
                j += 1;
            } 
        }

        CompactableHeaplessTree::<BRANCH_FACTOR, {HEIGHT - 1}, H>
            ::try_from_leaves(&leaves)
            .map_err(|_| self)
    }

    pub fn insert(&mut self, index: usize, input: &[u8]) {
        self.tree.insert(index, input);

        if !self.leaves_present[index] {
            self.leaf_number += 1;
        }
        self.leaves_present[index] = true;
    }

    pub fn remove(&mut self, index: usize) {
        self.tree.remove(index);

        if self.leaves_present[index] {
            self.leaf_number -= 1;
        }
        self.leaves_present[index] = false;
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> MerkleTree<BRANCH_FACTOR, HEIGHT, H> for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
    H: HashT,
{
    fn generate_proof(&mut self, index: usize) -> (H::Output, Proof<BRANCH_FACTOR, HEIGHT, H>) 
    where [(); HEIGHT - 1]: Sized {
        self.tree.generate_proof(index)
    }
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H> Debug for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
    H: HashT,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        write!(f, "{:?}", self.tree)
    }
}
