use core::fmt::Debug;
use crate::{HashT, StaticTreeTrait,  StaticTree, Proof, ProofBuilder, total_size, layer_size, Assert, IsTrue, is_pow2};
use crate::traits::{AppendOnly};

pub type DefaultAugmentable<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> 
    = AugmentableTree<BRANCH_FACTOR, HEIGHT, H, Proof<BRANCH_FACTOR, {HEIGHT+1}, H>>;

//pub mod augmentable {

    pub struct AugmentableTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB = Proof<BRANCH_FACTOR, HEIGHT, H>>
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
        Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
        H: HashT,
        PB: ProofBuilder<H>,
    {
        tree: StaticTree<BRANCH_FACTOR, HEIGHT, H, PB>,
        num_of_leaves: usize,
    }

    impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB>
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
        Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
        H: HashT,
        PB: ProofBuilder<H>,
    {
        pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
            Ok(Self {
                tree: StaticTree::try_from(input)?,
                num_of_leaves: input.len(),
            })
        }

        pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
            Ok(Self {
                tree: StaticTree::try_from_leaves(leaves)?, 
                num_of_leaves: leaves.len(),
            })
        }

        pub fn augment(self) -> AugmentableTree<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>
        where
            [(); total_size!(BRANCH_FACTOR, {HEIGHT + 1})]: Sized,
            [(); layer_size!(BRANCH_FACTOR, {HEIGHT + 1}, 0)]: Sized,         
            H: HashT, 
            PB: ProofBuilder<H>,
        {
            AugmentableTree::<BRANCH_FACTOR, {HEIGHT + 1}, H, PB> {
                tree: StaticTree::try_from_leaves(self.leaves()).expect("can create from smaller tree. qed"),
                num_of_leaves: self.num_of_leaves,
            }
        }

        pub fn augment_and_merge<const OTHER_HEIGHT: usize, OTHERPB: ProofBuilder<H>>(
            self, 
            other: AugmentableTree<BRANCH_FACTOR, OTHER_HEIGHT, H, OTHERPB>
        ) -> AugmentableTree<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>
        where
            [(); total_size!(BRANCH_FACTOR, {HEIGHT + 1})]: Sized,
            [(); layer_size!(BRANCH_FACTOR, {HEIGHT + 1}, 0)]: Sized,         
            [(); total_size!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
            [(); layer_size!(BRANCH_FACTOR, OTHER_HEIGHT, 0)]: Sized,
            Assert::<{OTHER_HEIGHT <= HEIGHT}>: IsTrue,
            H: HashT, 
            PB: ProofBuilder<H>,
        {
            let mut this = AugmentableTree::<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>::default();

            let len1 = self.num_of_leaves();
            let len2 = other.num_of_leaves();
            let len = len1 + len2;

            this.tree.hashes[0..len1].copy_from_slice(&self.tree.hashes[0..len1]);
            this.tree.hashes[len1..len].copy_from_slice(&other.tree.hashes[0..len2]);
            // pad the rest of hashes in the base layer
            for i in len..layer_size!(BRANCH_FACTOR, HEIGHT + 1, 0) {
                this.tree.hashes[i] = H::hash(&[]);
            }

            this.num_of_leaves = len;
            
            this.tree.fill_layers();
            this
        }

        pub fn try_merge<const OTHER_HEIGHT: usize, OTHERPB: ProofBuilder<H>>(
            &mut self, 
            other: AugmentableTree<BRANCH_FACTOR, OTHER_HEIGHT, H, OTHERPB>
        ) -> Result<(), ()> 
        where
            [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
            [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,    
            [(); total_size!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
            [(); layer_size!(BRANCH_FACTOR, OTHER_HEIGHT, 0)]: Sized,
            Assert::<{OTHER_HEIGHT <= HEIGHT}>: IsTrue,
            H: HashT, 
            PB: ProofBuilder<H>,
        {
            let len1 = self.num_of_leaves();
            let len2 = other.num_of_leaves();
            let len = len1 + len2;
            if len > layer_size!(BRANCH_FACTOR, HEIGHT, 0) {
                return Err(());
            }
            self.tree.hashes[len1..len].copy_from_slice(&other.tree.hashes[0..len2]);
            self.tree.fill_layers();
            self.num_of_leaves += len2;
            Ok(())
        }    
    }

    impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> StaticTreeTrait<H, PB> for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB> 
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
        }
        fn replace_leaf(&mut self, index: usize, leaf: H::Output) {
            self.tree.replace_leaf(index, leaf);
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

    impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> AppendOnly for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,     
        Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
        H: HashT,
        PB: ProofBuilder<H>,
    {
        fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
            if self.num_of_leaves >= self.base_layer_size() {
                return Err(());
            }        
            self.replace(self.num_of_leaves, input);
            self.num_of_leaves += 1;
            Ok(())
        }
        fn num_of_leaves(&self) -> usize {
            self.num_of_leaves
        }
    }

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Clone for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB> 
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
            }
        }
    }

    impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Default for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB> 
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
            }
        }
    }

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Debug for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB> 
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

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Copy for AugmentableTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,      
        Assert::<{is_pow2!(BRANCH_FACTOR)}>: IsTrue,
        H: HashT,
        PB: ProofBuilder<H>,
    {}

//}
