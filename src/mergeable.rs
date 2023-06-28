
use crate::{Proof};

pub type DefaultMergeable<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> 
    = mergeable::MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, Proof<BRANCH_FACTOR, {HEIGHT+1}, H>>;

pub mod mergeable {
    use core::fmt::Debug;
    use crate::{HashT, BasicTreeTrait, HeaplessTree, Proof, ProofBuilder, total_size, layer_size};

    pub struct MergeableHeaplessTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB = Proof<BRANCH_FACTOR, HEIGHT, H>>
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
        H: HashT,
        PB: ProofBuilder<H>,
    {
        tree: HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB>,
        num_of_leaves: usize,
    }

    impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB>
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
        H: HashT,
        PB: ProofBuilder<H>,
    {
        const BASE_LAYER_SIZE: usize = layer_size!(BRANCH_FACTOR, HEIGHT, 0);
        
        // panics if HEIGHT == 0
        pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
            HeaplessTree::try_from(input).map(|tree|
                Self {
                    tree,
                    num_of_leaves: input.len(),
                }
            )   
        }

        pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
            HeaplessTree::try_from_leaves(leaves).map(|tree|
                Self {
                    tree,
                    num_of_leaves: leaves.len(),
                }
            )  
        }

        pub fn try_merge<const OTHER_HEIGHT: usize, OTHERPB: ProofBuilder<H>>(
            self, 
            other: MergeableHeaplessTree<BRANCH_FACTOR, OTHER_HEIGHT, H, OTHERPB>
        ) -> Result<MergeableHeaplessTree<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>, Self> 
        where
            [(); total_size!(BRANCH_FACTOR, {HEIGHT + 1})]: Sized,
            [(); layer_size!(BRANCH_FACTOR, {HEIGHT + 1}, 0)]: Sized,
            
            [(); total_size!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
            [(); layer_size!(BRANCH_FACTOR, OTHER_HEIGHT, 0)]: Sized,
            
            H: HashT, 
            PB: ProofBuilder<H>,
        {
            // height of other must be no greater than this tree height
            // so the resulting tree height is safely HEIGHT + 1s
            if OTHER_HEIGHT > HEIGHT {
                return Err(self);
            }

            Ok(MergeableHeaplessTree::<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>::from_leaves2(
                self.tree.leaves(),
                other.tree.leaves(),
            ))
        }

        fn from_leaves2(leaves1: &[H::Output], leaves2: &[H::Output]) -> Self {
            let mut tree = HeaplessTree::try_from(&[]).expect("can create tree from empty input. qed");
            let mut i = 0;
            for leaf in leaves1 {
                tree.hashes[i] = *leaf;
                i += 1;
            }
            for leaf in leaves2 {
                tree.hashes[i] = *leaf;
                i += 1;
            }
            let len = i;
            // pad the rest of hashes in the base layer
            for i in len..Self::BASE_LAYER_SIZE {
                tree.hashes[i] = H::hash(&[]);
            }
            // fill the rest of layers
            Self {
                tree: tree.fill_layers(),
                num_of_leaves: len,
            }
        }

        pub fn num_of_leaves(&self) -> usize {
            self.num_of_leaves
        }
    }

    impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> BasicTreeTrait<H, PB> for MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,     
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

        fn remove(&mut self, index: usize) {
            unimplemented!("Remove is not implemented for mergeable")
        }

        fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
            if self.num_of_leaves >= self.base_layer_size() {
                return Err(());
            }        
            self.replace(self.num_of_leaves, input);
            self.num_of_leaves += 1;
            Ok(())
        }
        fn root(&self) -> H::Output {
            *self.tree.hashes.iter().last().expect("hashes are not empty. qed")
        }
        fn leaves(&self) -> &[H::Output] {
            &self.tree.hashes[..layer_size!(BRANCH_FACTOR, HEIGHT, 0)]
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
        fn num_of_leaves(&self) -> usize {
            self.num_of_leaves
        }
    }

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Clone for MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,    
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

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Copy for MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,      
        H: HashT,
        PB: ProofBuilder<H>,
    {}

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Default for MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,      
        H: HashT,
        PB: ProofBuilder<H>,
    {
        fn default() -> Self {
            Self::try_from(&[]).expect("can create from empty input. qed")
        }
    }

    impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Debug for MergeableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,   
        H: HashT,
        PB: ProofBuilder<H>,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
            write!(f, "{:?}", self.tree)
        }
    }
}
