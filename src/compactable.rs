
use crate::{Proof};

pub type DefaultCompactable<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> 
    = compactable::CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, Proof<BRANCH_FACTOR, {HEIGHT+1}, H>>;

pub(crate) mod compactable {
    use core::fmt::Debug;
    use crate::{HashT, HeaplessTreeT,  HeaplessTree, Proof, ProofItem, ProofBuilder, total_size, layer_size};
    
    pub struct CompactableHeaplessTree<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB = Proof<BRANCH_FACTOR, HEIGHT, H>>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    
    H: HashT,
    PB: ProofBuilder<H>,
{
    tree: HeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB>,
    num_of_leaves: usize,
    leaves_present: [bool; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> 
    CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB>
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    
    H: HashT,
    PB: ProofBuilder<H>,
{
    // panics if HEIGHT == 0
    pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
        let mut this = Self {
            tree: HeaplessTree::try_from(input)?,
            num_of_leaves: input.len(),
            leaves_present: [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
        };   
        for i in 0..input.len() {
            this.leaves_present[i] = true;
        }
        Ok(this)
    }

    pub fn try_from_leaves(leaves: &[H::Output]) -> Result<Self, ()> {
        Self::try_from_compacted(
            &leaves[0..layer_size!(BRANCH_FACTOR, HEIGHT, 0)].try_into().unwrap(), 
            leaves.len()
        )
    }

    fn try_from_compacted(leaves: &[H::Output; layer_size!(BRANCH_FACTOR, HEIGHT, 0)], num_of_leaves: usize) -> Result<Self, ()> {
        let mut this = Self {
            tree: HeaplessTree::try_from_leaves(leaves)?,
            num_of_leaves,
            leaves_present: [false; layer_size!(BRANCH_FACTOR, HEIGHT, 0)],
        };
        for i in 0..num_of_leaves {
            this.leaves_present[i] = true;
        }
        Ok(this)
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

    pub fn try_compact(self) -> Result<CompactableHeaplessTree<BRANCH_FACTOR, {HEIGHT - 1}, H, PB>, Self> 
    where
        [(); total_size!(BRANCH_FACTOR, {HEIGHT - 1})]: Sized,
        [(); layer_size!(BRANCH_FACTOR, {HEIGHT - 1}, 0)]: Sized,
        [(); {HEIGHT - 1} - 1]: Sized,
        H: HashT, 
        PB: ProofBuilder<H>,
    {
        self.compacted_leaves::<{HEIGHT - 1}>()
            .and_then(|leaves| 
                CompactableHeaplessTree::<BRANCH_FACTOR, {HEIGHT - 1}, H, PB>::try_from_compacted(&leaves, self.num_of_leaves()))
            .map_err(|_| self)
    }

    pub fn try_merge<const OTHER_HEIGHT: usize, OTHER_PB: ProofBuilder<H>>(
        self, 
        other: CompactableHeaplessTree<BRANCH_FACTOR, OTHER_HEIGHT, H, OTHER_PB>
    ) -> Result<CompactableHeaplessTree<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>, Self> 
    where
        [(); total_size!(BRANCH_FACTOR, {HEIGHT + 1})]: Sized,
        [(); layer_size!(BRANCH_FACTOR, {HEIGHT + 1}, 0)]: Sized,
//        [(); {HEIGHT + 1} - 1]: Sized,
        
        [(); total_size!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, OTHER_HEIGHT, 0)]: Sized,
//        [(); OTHER_HEIGHT - 1]: Sized,
        
        H: HashT, 
        PB: ProofBuilder<H>,
    {
        // height of other must be no greater than this tree height
        // so the resulting tree height is safely HEIGHT + 1s
        if OTHER_HEIGHT > HEIGHT {
            return Err(self);
        }

        self.compacted_leaves::<{HEIGHT + 1}>()
            .and_then(|mut first_leaves| {
                other.compacted_leaves::<OTHER_HEIGHT>()
                    .map(|second_leaves| {
                        for i in 0..other.num_of_leaves {
                            first_leaves[self.num_of_leaves + i] = second_leaves[i];
                        }
                        first_leaves
                    })
            })
            .and_then(|leaves| 
                CompactableHeaplessTree::<BRANCH_FACTOR, {HEIGHT + 1}, H, PB>::try_from_compacted(
                    &leaves, 
                    self.num_of_leaves + other.num_of_leaves
                )
            )
            .map_err(|_| self)
    }

    pub fn try_compact_and_append<const OTHER_HEIGHT: usize, OTHER_PB: ProofBuilder<H>>(
        self, 
        other: CompactableHeaplessTree<BRANCH_FACTOR, OTHER_HEIGHT, H, OTHER_PB>) 
        -> Result<Self, Self> 
    where
        [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//        [(); HEIGHT - 1]: Sized,

        [(); total_size!(BRANCH_FACTOR, OTHER_HEIGHT)]: Sized,
        [(); layer_size!(BRANCH_FACTOR, OTHER_HEIGHT, 0)]: Sized,
//        [(); OTHER_HEIGHT - 1]: Sized,
        
        H: HashT, 
        PB: ProofBuilder<H>,
        {
        if other.num_of_leaves > self.base_layer_size() - self.num_of_leaves {
            return Err(self);
        }

        self.compacted_leaves::<HEIGHT>()
            .and_then(|mut first_leaves| {
                other.compacted_leaves::<OTHER_HEIGHT>()
                    .map(|second_leaves| {
                        for i in 0..other.num_of_leaves {
                            first_leaves[self.num_of_leaves + i] = second_leaves[i];
                        }
                        first_leaves
                    })
            })
            .and_then(|leaves| Self::try_from_compacted(&leaves, self.num_of_leaves))
            .map_err(|_| self)
    }

    pub fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
        if self.num_of_leaves >= self.base_layer_size() {
            return Err(());
        }
        
        self.replace(self.num_of_leaves, input);
        
        Ok(())
    }

    pub fn num_of_leaves(&self) -> usize {
        self.num_of_leaves
    }
}

impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> HeaplessTreeT<H, PB> 
    for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
//impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H> HeaplessTreeT<H, ProofItem<BRANCH_FACTOR, H>> for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    
    H: HashT,
    PB: ProofBuilder<H>,
{
//    type Proof = Proof<BRANCH_FACTOR, HEIGHT, H> where [(); HEIGHT - 1]: Sized;
//    type Proof = Proof<BRANCH_FACTOR, H> where [(); HEIGHT - 1]: Sized;

    fn generate_proof(&mut self, index: usize) -> PB {
        self.tree.generate_proof(index)
    }

    // fn generate_proof(&mut self, index: usize, proof: &mut PB) {
    //     self.tree.generate_proof(index, proof)
    // }


    fn replace(&mut self, index: usize, input: &[u8]) {
        self.tree.replace(index, input);

        if !self.leaves_present[index] {
            self.num_of_leaves += 1;
        }
        self.leaves_present[index] = true;
    }

    fn remove(&mut self, index: usize) {
        self.tree.remove(index);

        if self.leaves_present[index] {
            self.num_of_leaves -= 1;
        }
        self.leaves_present[index] = false;
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
}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Clone 
    for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    
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

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Copy 
    for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    
    H: HashT,
    PB: ProofBuilder<H>,
{}

impl <const BRANCH_FACTOR: usize, const HEIGHT: usize, H, PB> Debug 
    for CompactableHeaplessTree<BRANCH_FACTOR, HEIGHT, H, PB> 
where
    [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized,
    [(); layer_size!(BRANCH_FACTOR, HEIGHT, 0)]: Sized,
//    [(); HEIGHT - 1]: Sized,
    
    H: HashT,
    PB: ProofBuilder<H>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 
        write!(f, "{:?}", self.tree)
    }
}
}
