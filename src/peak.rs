use crate::{HashT, HeaplessTreeT,  HeaplessTree, ProofT, ProofItem, total_size, layer_size};
use crate::compactable::{CompactableHeaplessTree};

pub struct PeakProof<H: HashT> {
    items: Vec<ProofItem<2, H>>,
//    items: Vec<<Self as ProofT<H>>::Item>,
//    items: [ProofItem<BRANCH_FACTOR, H>; HEIGHT - 1]
}

pub enum MerklePeak<H: HashT> {
    NonMergeable(CompactableHeaplessTree<2, 5, H>),
    First(CompactableHeaplessTree<2, 4, H>),
    Second(CompactableHeaplessTree<2, 3, H>),
    Third(CompactableHeaplessTree<2, 2, H>),
    Forth(CompactableHeaplessTree<2, 1, H>),
}

impl<H: HashT> Clone for MerklePeak<H> {
    fn clone(&self) -> Self { 
        use MerklePeak::*;
        match self {
            NonMergeable(tree) => NonMergeable(tree.clone()),
            First(tree) => First(tree.clone()),
            Second(tree) => Second(tree.clone()),
            Third(tree) => Third(tree.clone()),
            Forth(tree) => Forth(tree.clone()),
        }
    }
}

impl<H: HashT> Default for MerklePeak<H> {
    fn default() -> Self {
        Self::Forth(CompactableHeaplessTree::try_from(&[]).unwrap())
    }
}

impl<H: HashT> Copy for MerklePeak<H> {}

// impl<H: HashT> PeakProof<H> {
//     pub fn next(&self) -> Self 
// }

impl<H: HashT> ProofT<H, ProofItem<2, H>> for PeakProof<H> {

    fn push_item(&mut self, item: ProofItem<2, H>) {
//        fn push_item(&mut self, item: Self::Item) {
        self.items.push(item);
    }
    // SHOULD BE UNIMPLEMENTED
    fn validate(self, root: &H::Output, input: &[u8]) -> bool {
        let mut curr_hash = Some(H::hash(&input));
        for item in self.items {
            curr_hash = curr_hash.and_then(|h| item.hash_with_siblings(h));
        }
        curr_hash.as_ref() == Some(root)
    }
}

// impl <H: HashT> IntoIterator for PeakProof<H> {
//     type Item = ProofItem<2, H>;
//     type IntoIter = <Vec<ProofItem<2, H>> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
//         self.items.into_iter()
//     }
// }


impl<H: HashT> Default for PeakProof<H>
{
    fn default() -> Self {
        Self { 
            items: Default::default(), 
        }
    }
}

#[macro_export]
macro_rules! apply {
    ($peak: expr, $func:ident, $($args:expr),*) => {
        match $peak {
            Self::Forth(this) => this.$func($($args),*),
            Self::Third(this) => this.$func($($args),*),
            Self::Second(this) => this.$func($($args),*),
            Self::First(this) => this.$func($($args),*),
            Self::NonMergeable(this) => this.$func($($args),*),
        }
    };
}

impl<H: HashT> HeaplessTreeT<H, ProofItem<2, H>> for MerklePeak<H> 
{
    type Proof = PeakProof<H>;

    fn generate_proof(&mut self, index: usize) -> (H::Output, Self::Proof) {
        let (root, items) = match self {
            Self::Forth(this) => {
                let (root, proof) = this.generate_proof(index); 
                let items = proof.items[..proof.curr_index].into_iter().map(|item| item.clone()).collect();
                (root, items)
            },
            Self::Third(this) => {
                let (root, proof) = this.generate_proof(index); 
                let items = proof.items[..proof.curr_index].into_iter().map(|item| item.clone()).collect();
                (root, items)
            },
            Self::Second(this) => {
                let (root, proof) = this.generate_proof(index); 
                let items = proof.items[..proof.curr_index].into_iter().map(|item| item.clone()).collect();
                (root, items)
            },
            Self::First(this) => {
                let (root, proof) = this.generate_proof(index); 
                let items = proof.items[..proof.curr_index].into_iter().map(|item| item.clone()).collect();
                (root, items)
            },
            Self::NonMergeable(this) => {
                let (root, proof) = this.generate_proof(index); 
                let items = proof.items[..proof.curr_index].into_iter().map(|item| item.clone()).collect();
//                let items = proof.into_iter().collect();
                (root, items)
            },
            _ => unimplemented!(),
            // Self::Second(this) => this.$func($($args),*),
            // Self::First(this) => this.$func($($args),*),
            // Self::NonMergeable(this) => this.$func($($args),*),
        };

//        let (root, proof) = self.tree().generate_proof(index);
        (
            root,
            Self::Proof { 
                items,
            }
        )
    }

    fn replace(&mut self, index: usize, input: &[u8]) {
//        self.tree().replace(index, input);
        apply!(self, replace, index, input)
    }

    fn remove(&mut self, index: usize) {
//        self.tree().remove(index);
        apply!(self, remove, index)
    }
    fn root(&self) -> H::Output {
        apply!(self, root,)
//        self.tree().root()
    }

    fn leaves(&self) -> &[H::Output] {
        apply!(self, leaves,)
//        &self.tree().leaves()
    }

    fn base_layer_size(&self) -> usize {
        apply!(self, base_layer_size,)
//        self.tree().base_layer_size()
    }
    
    fn branch_factor(&self) -> usize {
        apply!(self, branch_factor,)
//        self.tree().branch_factor()
    }

    fn height(&self) -> usize {
        apply!(self, height,)
//        self.tree().height()
    }
}

impl<H: HashT> MerklePeak<H> {
    pub fn foo(&self) -> usize {
        apply!(self, height,)
    }
    pub fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
        apply!(self, try_append, input)
    }
    // pub fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
    //     use MerklePeak::{*};
    //     match self {
    //         Forth(this) => this.try_append(input),
    //         Third(this) => this.try_append(input),
    //         Second(this) => this.try_append(input),
    //         First(this) => this.try_append(input),
    //         NonMergeable(this) => this.try_append(input),
    //     }
    // }

    pub fn try_merge(self, other: Self) -> Result<Self, Self> {
        use MerklePeak::{*};
        match (self, other) {
            (Forth(this), Forth(other)) => this.try_merge(other).map_err(Forth).map(Third),
            (Third(this), Third(other)) => this.try_merge(other).map_err(Third).map(Second),
            (Second(this), Second(other)) => this.try_merge(other).map_err(Second).map(First),
            (First(this), First(other)) => this.try_merge(other).map_err(First).map(NonMergeable),
            _ => unreachable!(),
        }
    }
    pub fn num_of_leaves(&self) -> usize {
        apply!(self, num_of_leaves,)
    }

}

#[macro_export]
macro_rules! height_from_num_of_leaves {
    ($branch_factor:expr, $num_of_leaves:expr) => {
        ($num_of_leaves >> $branch_factor.trailing_zeros() as usize) + 1
    };
}

pub struct MerkleMR<const PEAKS: usize, H: HashT> 
where 
    [(); height_from_num_of_leaves!(2_usize, PEAKS)]: Sized,
    [(); total_size!(2_usize, height_from_num_of_leaves!(2_usize, PEAKS))]: Sized,

{
    main_tree: HeaplessTree<2, {height_from_num_of_leaves!(2_usize, PEAKS)}, H>,
    peaks: [MerklePeak<H>; PEAKS],
}

impl<const PEAKS: usize, H: HashT> MerkleMR<PEAKS, H> 
where 
    [(); height_from_num_of_leaves!(2_usize, PEAKS)]: Sized, 
    [(); total_size!(2_usize, height_from_num_of_leaves!(2_usize, PEAKS))]: Sized,
{
    pub fn from(peak: MerklePeak<H>) -> Self {
        let mut this = Self {
            main_tree: HeaplessTree::<2, {height_from_num_of_leaves!(2_usize, PEAKS)}, H>::try_from(&[]).unwrap(),
            peaks: [MerklePeak::<H>::default(); PEAKS]
        }; 
        this.peaks[0] = peak;
        this
    } 
}