use core::hash::Hash;
use core::fmt::Debug;

pub trait HashT {
    type Output: Hash + Default + Copy + AsRef<[u8]> + PartialEq + Debug;

    fn hash(input: &[u8]) -> Self::Output;
}

pub trait ProofItemT<H: HashT>: Clone + Default + Debug {
    fn create(offset: usize, hashes: &[H::Output]) -> Self;
    fn hash_with_siblings(self, word_hash: H::Output) -> Option<H::Output>;
}

pub trait ProofBuilder<H: HashT>: Default {
    type Item: ProofItemT<H>;

    fn from_root(root: H::Output) -> Self;
    fn root(&self) -> H::Output;
    fn push(&mut self, offset: usize, hashes: &[H::Output]);
}

pub trait ProofValidator {
    fn validate(self, input: &[u8]) -> bool;
}

pub trait StaticTreeTrait<H: HashT, PB: ProofBuilder<H>> {
//    pub trait StaticTreeTrait<H: HashT, PB: ProofBuilder<H>>: Clone + Default {
//    pub trait StaticTreeTrait<H: HashT, PB: ProofBuilder<H>>: Clone + Copy + Default {
    fn generate_proof(&mut self, index: usize) -> PB;
    
    fn replace(&mut self, index: usize, input: &[u8]);
    fn replace_leaf(&mut self, index: usize, leaf: H::Output);

    fn root(&self) -> H::Output;
    fn leaves(&self) -> &[H::Output];
    fn base_layer_size(&self) -> usize;
    fn branch_factor(&self) -> usize;
    fn height(&self) -> usize;
} 

pub trait AppendOnly {
    fn try_append(&mut self, input: &[u8]) -> Result<(), ()>;
    fn num_of_leaves(&self) -> usize;
}

pub trait CanRemove {
    fn try_remove(&mut self, index: usize) -> Result<(), ()>;
    fn num_of_leaves(&self) -> usize;
}
