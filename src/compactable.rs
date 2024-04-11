//! ```rust
//! use merkle_heapless::compactable::{DefaultCompactable};
//!
//! const ARITY: usize = 4;
//! const HEIGHT: usize = 3;
//!
//! let mut cmt = DefaultCompactable::<ARITY, HEIGHT, StdHash>::try_from::<&[u8]>(&[
//!     "apple", "apricot", "banana", "cherry",
//! ]).unwrap();
//!
//! cmt.try_remove(0).unwrap();
//! cmt.compact();
//! // will try to create a smaller tree from the compacted tree
//! let mut reduced = cmt.try_reduce().unwrap();
//! ```
//!

/// Compactable tree with [Proof] of tree's height
pub type DefaultCompactable<
    const ARITY: usize,
    const HEIGHT: usize,
    H,
    const MAX_INPUT_LEN: usize,
> = CompactableHeaplessTree<
    ARITY,
    HEIGHT,
    H,
    MAX_INPUT_LEN,
    Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>,
>;

use crate::traits::CanRemove;
use crate::{
    is_pow2, layer_size, location_in_prefixed, max_leaves, num_of_prefixed, Assert, Error, HashT,
    IsTrue, Prefixed, Proof, ProofBuilder, StaticTree, StaticTreeTrait,
};
use core::fmt::Debug;
use core::ops::Deref;
/// Tree that can be compacted after leaf removal and reduced to a smaller tree
pub struct CompactableHeaplessTree<
    const ARITY: usize,
    const HEIGHT: usize,
    H,
    const MAX_INPUT_LEN: usize,
    PB = Proof<ARITY, HEIGHT, H, MAX_INPUT_LEN>,
> where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    tree: StaticTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>,
    num_of_leaves: usize,
    leaves_present: [bool; max_leaves!(ARITY, HEIGHT)],
}

impl<const ARITY: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB>
    CompactableHeaplessTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    /// creates a tree from an input if possible
    pub fn try_from<T: AsRef<[u8]> + Deref<Target = [u8]>>(input: &[T]) -> Result<Self, Error> {
        let mut this = Self {
            tree: StaticTree::try_from(input)?,
            num_of_leaves: input.len(),
            leaves_present: [false; max_leaves!(ARITY, HEIGHT)],
        };
        for i in 0..input.len() {
            this.leaves_present[i] = true;
        }
        Ok(this)
    }

    /// creates a tree from an input if possible
    pub fn from<T: AsRef<[u8]> + Deref<Target = [u8]>>(input: &[T]) -> Self {
        let mut this = Self {
            tree: StaticTree::from(input),
            num_of_leaves: input.len(),
            leaves_present: [false; max_leaves!(ARITY, HEIGHT)],
        };
        for i in 0..input.len() {
            this.leaves_present[i] = true;
        }
        this
    }

    /// creates a tree from hashed leaves (of another tree)
    pub fn try_from_leaves(prefixed: &[Prefixed<ARITY, H>]) -> Result<Self, Error> {
        let mut leaves_present = [false; max_leaves!(ARITY, HEIGHT)];
        let mut num_of_leaves = 0;
        let default_hash = Prefixed::<ARITY, H>::default_hash();
        let mut j = 0;
        for leaf in prefixed {
            num_of_leaves += leaf
                .hashes
                .iter()
                .enumerate()
                .filter(|&(_, h)| (h != &default_hash))
                .map(|(i, _)| {
                    leaves_present[i + j] = true;
                })
                .count();
            j += ARITY;
        }
        Ok(Self {
            tree: StaticTree::try_from_leaves(prefixed)?,
            num_of_leaves,
            leaves_present,
        })
    }

    fn try_from_compacted(
        leaves: &[Prefixed<ARITY, H>; layer_size!(ARITY, HEIGHT, 0)],
        num_of_leaves: usize,
    ) -> Result<Self, Error> {
        (num_of_leaves <= max_leaves!(ARITY, HEIGHT))
            .then_some(())
            .ok_or(Error::Create)
            .and_then(|()| {
                let mut leaves_present = [false; max_leaves!(ARITY, HEIGHT)];
                for present in leaves_present.iter_mut().take(num_of_leaves) {
                    *present = true;
                }
                Ok(Self {
                    tree: StaticTree::try_from_leaves(leaves)?,
                    num_of_leaves,
                    leaves_present,
                })
            })
    }

    fn compacted_leaves<const EXPECTED_HEIGHT: usize>(
        &self,
    ) -> Result<[Prefixed<ARITY, H>; layer_size!(ARITY, EXPECTED_HEIGHT, 0)], Error> {
        if self.num_of_leaves > max_leaves!(ARITY, EXPECTED_HEIGHT) {
            return Err(Error::Create);
        }

        let mut prefixed =
            [Prefixed::<ARITY, H>::default(); layer_size!(ARITY, EXPECTED_HEIGHT, 0)];

        let mut j = 0;
        for i in 0..self.leaves_present.len() {
            if self.leaves_present[i] {
                let (old_index, old_offset) = location_in_prefixed::<ARITY>(i);
                let (new_index, new_offset) = location_in_prefixed::<ARITY>(j);

                prefixed[new_index].hashes[new_offset] =
                    self.tree.leaves()[old_index].hashes[old_offset];
                j += 1;
            }
        }
        assert_eq!(self.num_of_leaves(), j);
        Ok(prefixed)
    }
    /// move all existing leaves leftwards
    pub fn compact(&mut self)
    where
        [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
        [(); max_leaves!(ARITY, HEIGHT)]: Sized,
        H: HashT,
        PB: ProofBuilder<ARITY, H>,
    {
        *self = self
            .compacted_leaves::<HEIGHT>()
            .and_then(|leaves| Self::try_from_leaves(&leaves))
            .expect("can create from compacted leaves. qed");
    }
    /// tries to compact this tree to a size of a tree with height-1 and create an instance of the new tree
    /// Note: takes ownership, but as it implements Copy trait may need explicit dropping
    /// to prevent being any longer available
    pub fn try_reduce(
        self,
    ) -> Result<CompactableHeaplessTree<ARITY, { HEIGHT - 1 }, H, MAX_INPUT_LEN, PB>, Self>
    where
        [(); num_of_prefixed!(ARITY, { HEIGHT - 1 })]: Sized,
        [(); max_leaves!(ARITY, { HEIGHT - 1 })]: Sized,
        H: HashT,
        PB: ProofBuilder<ARITY, H>,
    {
        self.compacted_leaves::<{ HEIGHT - 1 }>()
            .and_then(|leaves| {
                CompactableHeaplessTree::<ARITY, { HEIGHT - 1 }, H, MAX_INPUT_LEN, PB>::try_from_compacted(
                    &leaves,
                    self.num_of_leaves(),
                )
            })
            .map_err(|_| self)
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB>
    StaticTreeTrait<ARITY, H, PB> for CompactableHeaplessTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    fn generate_proof(&self, index: usize) -> PB {
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
        self.tree.root()
    }
    fn leaves(&self) -> &[Prefixed<ARITY, H>] {
        &self.tree.prefixed[..layer_size!(ARITY, HEIGHT, 0)]
    }
    fn base_layer_size(&self) -> usize {
        layer_size!(ARITY, HEIGHT, 0)
    }
    fn arity(&self) -> usize {
        ARITY
    }
    fn height(&self) -> usize {
        HEIGHT
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> CanRemove
    for CompactableHeaplessTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    // remove element by replacing with nothing
    // panics if index is out of leaf layer bound
    fn remove(&mut self, index: usize) {
        self.tree.replace(index, &[]);

        if self.leaves_present[index] {
            self.num_of_leaves -= 1;
        }
        self.leaves_present[index] = false;
    }

    fn num_of_leaves(&self) -> usize {
        self.num_of_leaves
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Clone
    for CompactableHeaplessTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            num_of_leaves: self.num_of_leaves,
            leaves_present: self.leaves_present,
        }
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Default
    for CompactableHeaplessTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    fn default() -> Self {
        Self {
            tree: Default::default(),
            num_of_leaves: 0,
            leaves_present: [false; max_leaves!(ARITY, HEIGHT)],
        }
    }
}

impl<const ARITY: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Debug
    for CompactableHeaplessTree<ARITY, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(ARITY, HEIGHT)]: Sized,
    [(); max_leaves!(ARITY, HEIGHT)]: Sized,
    Assert<{ is_pow2!(ARITY) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<ARITY, H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        writeln!(f, "{:?}", self.tree)?;
        writeln!(f, "[num of leaves]: {:?}", self.num_of_leaves())
    }
}
