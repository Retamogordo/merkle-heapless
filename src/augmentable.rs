//!
//! ## Augment
//! ```rust
//! use merkle_heapless::augmentable::{DefaultAugmentable};
//!
//! const BRANCH_FACTOR: usize = 4;
//! const HEIGHT_1: usize = 3;
//! const HEIGHT_2: usize = 2;
//! const MAX_WORD_LEN: usize = 10;
//!
//! let mt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT, StdHash, MAX_WORD_LEN>::try_from::<&[u8]>(&[
//!     "apple", "apricot", "banana", "cherry",
//! ]).unwrap();
//!
//! let mut mt = mt1.augment();
//! assert_eq!(mt.height(), HEIGHT_1 + 1);
//! ```
//!
//! ## Merge
//!
//! You can ```try_merge``` a smaller (or equally-sized) tree into the original tree.
//! This operation does not imply augmentation, rather it fails if merge is not possible.
//! ```rust
//! // snip
//! let mt2 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_2, StdHash, MAX_WORD_LEN>::try_from::<&[u8]>(&[
//!     "kiwi", "lemon",
//! ]).unwrap();
//!
//! mt.try_merge(mt2).unwrap();
//! ```

use core::ops::Deref;
use crate::traits::AppendOnly;
use crate::{
    is_pow2, layer_size, num_of_prefixed, Assert, HashT, IsTrue, Prefixed, Proof, ProofBuilder,
    StaticTree, StaticTreeTrait, Error
};
use core::fmt::Debug;
/// Augmentable tree with default Proof size of (tree.height + 1)
pub type DefaultAugmentable<
    const BRANCH_FACTOR: usize,
    const HEIGHT: usize,
    H,
    const MAX_INPUT_LEN: usize,
> = AugmentableTree<
    BRANCH_FACTOR,
    HEIGHT,
    H,
    MAX_INPUT_LEN,
    Proof<BRANCH_FACTOR, { HEIGHT + 1 }, H, MAX_INPUT_LEN>,
>;

/// Augmentable Tree can be converted into a bigger tree with height+1
pub struct AugmentableTree<
    const BRANCH_FACTOR: usize,
    const HEIGHT: usize,
    H,
    const MAX_INPUT_LEN: usize,
    PB = Proof<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN>,
> where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    tree: StaticTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>,
    num_of_leaves: usize,
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB>
    AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    /// creates a tree from an input if possible
    pub fn try_from<T: AsRef<[u8]> + Deref<Target = [u8]>>(input: &[T]) -> Result<Self, Error> {
        Ok(Self {
            tree: StaticTree::try_from(input)?,
            num_of_leaves: input.len(),
        })
    }
    /// creates a tree from hashed leaves (of another tree)
    pub fn try_from_leaves(prefixed_leaves: &[Prefixed<BRANCH_FACTOR, H>]) -> Result<Self, Error> {
        let mut num_of_leaves = 0;
        let default_hash = Prefixed::<BRANCH_FACTOR, H>::default_hash();

        for leaf in prefixed_leaves {
            num_of_leaves += leaf
                .hashes
                .iter()
                .filter_map(|h| (h != &default_hash).then_some(()))
                .count();
        }

        Ok(Self {
            tree: StaticTree::try_from_leaves(prefixed_leaves)?,
            num_of_leaves,
        })
    }
    /// creates a tree with height+1 and copies the contents of this tree to the new one
    /// Note: takes ownership, but as it implements Copy trait may need explicit dropping
    /// to prevent being any longer available
    pub fn augment(self) -> AugmentableTree<BRANCH_FACTOR, { HEIGHT + 1 }, H, MAX_INPUT_LEN, PB>
    where
        [(); num_of_prefixed!(BRANCH_FACTOR, { HEIGHT + 1 })]: Sized,
        H: HashT,
        PB: ProofBuilder<BRANCH_FACTOR, H>,
    {
        AugmentableTree::<BRANCH_FACTOR, { HEIGHT + 1 }, H, MAX_INPUT_LEN, PB> {
            tree: StaticTree::try_from_leaves(self.leaves())
                .expect("can create from smaller tree. qed"),
            num_of_leaves: self.num_of_leaves,
        }
    }
    /// create a tree with height+1 and copies the contents of this and another tree to the new one
    /// Note: takes ownership, but as it implements Copy trait may need explicit dropping
    /// to prevent being any longer available
    pub fn augment_and_merge<const OTHER_HEIGHT: usize, OTHERPB: ProofBuilder<BRANCH_FACTOR, H>>(
        self,
        other: AugmentableTree<BRANCH_FACTOR, OTHER_HEIGHT, H, MAX_INPUT_LEN, OTHERPB>,
    ) -> AugmentableTree<BRANCH_FACTOR, { HEIGHT + 1 }, H, MAX_INPUT_LEN, PB>
    where
        [(); num_of_prefixed!(BRANCH_FACTOR, { HEIGHT + 1 })]: Sized,
        [(); num_of_prefixed!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
        Assert<{ OTHER_HEIGHT <= { HEIGHT + 1 } }>: IsTrue,
        H: HashT,
        PB: ProofBuilder<BRANCH_FACTOR, H>,
    {
        let mut this = self.augment();
        this.try_merge(other)
            .expect("can merge into augmented tree. qed");
        this
    }
    /// tries to merge a tree to this one if there is enough room in it
    pub fn try_merge<const OTHER_HEIGHT: usize, OTHERPB: ProofBuilder<BRANCH_FACTOR, H>>(
        &mut self,
        other: AugmentableTree<BRANCH_FACTOR, OTHER_HEIGHT, H, MAX_INPUT_LEN, OTHERPB>,
    ) -> Result<(), Error>
    where
        [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); num_of_prefixed!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
        Assert<{ OTHER_HEIGHT <= HEIGHT }>: IsTrue,
        H: HashT,
        PB: ProofBuilder<BRANCH_FACTOR, H>,
    {
        let with_offset = self.num_of_leaves();
        let total_len = with_offset + other.num_of_leaves();
        if total_len > BRANCH_FACTOR * layer_size!(BRANCH_FACTOR, HEIGHT, 0) {
            return Err(Error::Merge);
        }
        self.tree = self.tree.with_leaves_inner(other.leaves(), with_offset);
        self.num_of_leaves += other.num_of_leaves();

        Ok(())
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB>
    StaticTreeTrait<BRANCH_FACTOR, H, PB>
    for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn generate_proof(&mut self, index: usize) -> PB {
        self.tree.generate_proof(index)
    }
    fn replace(&mut self, index: usize, input: &[u8]) {
        self.tree.replace(index, input);
    }
    fn replace_leaf(&mut self, index: usize, leaf: H::Output) {
        self.tree.replace_leaf(index, leaf);
    }
    fn root(&self) -> H::Output {
        self.tree.root()
    }
    fn leaves(&self) -> &[Prefixed<BRANCH_FACTOR, H>] {
        &self.tree.prefixed[..layer_size!(BRANCH_FACTOR, HEIGHT, 0)]
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

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> AppendOnly
    for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,

    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn try_append(&mut self, input: &[u8]) -> Result<(), Error> {
        if self.num_of_leaves >= (self.base_layer_size() << BRANCH_FACTOR.trailing_zeros()) {
            return Err(Error::Append);
        }
        self.replace(self.num_of_leaves, input);
        self.num_of_leaves += 1;
        Ok(())
    }
    fn num_of_leaves(&self) -> usize {
        self.num_of_leaves
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Clone
    for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            num_of_leaves: self.num_of_leaves,
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Default
    for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn default() -> Self {
        Self {
            tree: Default::default(),
            num_of_leaves: 0,
        }
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Debug
    for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:?}", self.tree)
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, const MAX_INPUT_LEN: usize, PB> Copy
    for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, MAX_INPUT_LEN, PB>
where
    [(); num_of_prefixed!(BRANCH_FACTOR, HEIGHT)]: Sized,
    Assert<{ is_pow2!(BRANCH_FACTOR) }>: IsTrue,
    H: HashT,
    PB: ProofBuilder<BRANCH_FACTOR, H>,
{
}
