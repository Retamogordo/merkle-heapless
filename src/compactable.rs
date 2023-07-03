
//! ```rust
//! use merkle_heapless::compactable::{DefaultCompactable};
//!
//! const BRANCH_FACTOR: usize = 4;
//! const HEIGHT: usize = 3;
//!
//! let mut cmt = DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
//!     "apple", "apricot", "banana", "cherry",
//! ]).unwrap();
//!
//! cmt.try_remove(0).unwrap();
//! cmt.compact();
//! // will try to create a smaller tree from the compacted tree
//! let mut reduced = cmt.try_reduce().unwrap();
//! ```
//!

// pub type DefaultCompactable<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> 
// = CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, Proof<BRANCH_FACTOR, {HEIGHT+1}, H>>;

/// Compactable tree with [Proof] of tree's height
pub type DefaultCompactable<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> 
    = CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, Proof<BRANCH_FACTOR, HEIGHT, H>>;

use core::fmt::Debug;
use crate::{HashT, StaticTreeTrait,  StaticTree, Proof, ProofBuilder, total_size, layer_size, Assert, IsTrue, is_pow2};
use crate::traits::{CanRemove};
/// Tree that can be compacted after leaf removal and reduced to a smaller tree
pub struct CompactableHeaplessTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB = Proof<BRANCH_FACTOR, HEIGHT, H>>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    tree: StaticTree<BRANCH_FACTOR, HEIGHT, H, PB>,
    num_of_leaves: usize,
    leaves_present: [bool; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    /// creates a tree from an input if possible
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
        let mut this = Self {
            tree: StaticTree::try_from(input)?,
            num_of_leaves: input.len(),
            leaves_present: [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
        };   
        for i in 0..input.len() {
            this.leaves_present[i] = true;
        }
        Ok(this)
    }
    /// creates a tree from hashed leaves (of another tree)
    pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
        let mut leaves_present = [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)];
        let mut num_of_leaves = 0;
        for i in 0..leaves.len() {
            if leaves[i] != Default::default() {
                leaves_present[i] = true;
                num_of_leaves += 1;
            }
        }
        Ok(Self {
            tree: StaticTree::try_from_leaves(leaves)?,
            num_of_leaves,
            leaves_present,
        })
    }

    fn try_from_compacted(leaves: &[H::Output; layer_size!(BRANCH_FACTOR, HEIGHT, 0)], num_of_leaves: usize) -> Result<Self, ()> {
        (num_of_leaves <= layer_size!(BRANCH_FACTOR, HEIGHT, 0))
            .then(|| ()).ok_or(())
            .and_then(|()| {
                let mut leaves_present = [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)];
                for i in 0..num_of_leaves {
                    leaves_present[i] = true;
                }
                Ok(Self {
                    tree: StaticTree::try_from_leaves(leaves)?,
                    num_of_leaves,
                    leaves_present,
                })
            })
    }

    fn compacted_leaves<const EXPECTED_HEIGHT: usize>(&self) -> Result<[H::Output; layer_size!(BRANCH_FACTOR, EXPECTED_HEIGHT, 0)], ()> {
        if self.num_of_leaves > layer_size!(BRANCH_FACTOR, EXPECTED_HEIGHT, 0) {
            return Err(());
        }

        let mut leaves = [H::Output::default(); layer_size!(BRANCH_FACTOR, EXPECTED_HEIGHT, 0)];
        let mut j = 0;
        for (i, leaf) in self.tree.leaves().iter().enumerate() {
            if self.leaves_present[i] {
                leaves[j] = *leaf;
                j += 1;
            } 
        }
        Ok(leaves)
    }
    /// move all existing leaves leftwards
    pub fn compact(&mut self) 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
        H: HashT, 
        PB: ProofBuilder<H>,
    {
        *self = self.compacted_leaves::<HEIGHT>()
                    .and_then(|leaves| Self::try_from_leaves(&leaves))
                    .expect("can create from compacted leaves. qed");
    }
    /// tries to compact this tree to a size of a tree with height-1 and create an instance of the new tree
    /// Note: takes ownership, but as it implements Copy trait may need explicit dropping
    /// to prevent being any longer available
    pub fn try_reduce(self) -> Result<CompactableHeaplessTree<BRANCH_FACTOR, {HEIGHT - 1}, H, PB>, Self> 
    where
        [(); total_size!(BRANCH_FACTOR, {HEIGHT - 1})]: Sized,
        [(); layer_size!(BRANCH_FACTOR, {HEIGHT - 1}, 0)]: Sized,
        H: HashT, 
        PB: ProofBuilder<H>,
    {
        self.compacted_leaves::<{HEIGHT - 1}>()
            .and_then(|leaves| 
                CompactableHeaplessTree::<BRANCH_FACTOR, {HEIGHT - 1}, H, PB>::try_from_compacted(&leaves, self.num_of_leaves()))
            .map_err(|_| self)
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> StaticTreeTrait<H, PB> for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,     
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn generate_proof(&mut self, index: usize) -> PB {
        self.tree.generate_proof(index)
    }
    fn replace(&mut self, index: usize, input: &[u8]) {
        self.tree.replace(index, input);

        if !self.leaves_present[index] {
            self.num_of_leaves += 1;
        }
        self.leaves_present[index] = true;
    }
    fn replace_leaf(&mut self, index: usize, leaf: H::Output) {
        self.tree.replace_leaf(index, leaf);

        if !self.leaves_present[index] {
            self.num_of_leaves += 1;
        }
        self.leaves_present[index] = true;
    }
    fn root(&self) -> H::Output {
        *self.tree.hashes.iter().last().expect("hashes are not empty. qed")
    }
    fn leaves(&self) -> &[H::Output] {
        &self.tree.hashes[..self.num_of_leaves]
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

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> CanRemove for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,     
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    // remove element by replacing with nothing
    fn try_remove(&mut self, index: usize) -> Result<(), ()> {
        if self.num_of_leaves == 0 || index > self.num_of_leaves {
            return Err(());
        }
        self.tree.replace(index, &[]);

        if self.leaves_present[index] {
            self.num_of_leaves -= 1;
        }
        self.leaves_present[index] = false;
        Ok(())
    }

    fn num_of_leaves(&self) -> usize {
        self.num_of_leaves
    }
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Clone for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,    
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn clone(&self) -> Self { 
        Self {
            tree: self.tree.clone(),
            num_of_leaves: self.num_of_leaves.clone(),
            leaves_present: self.leaves_present.clone(),
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Default for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,     
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn default() -> Self {
        Self {
            tree: Default::default(),
            num_of_leaves: 0,
            leaves_present: [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
        }
    }
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Debug for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,   
    Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        write!(f, "{:?}", self.tree)
    }
}
