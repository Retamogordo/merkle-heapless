use crate::{HashT, HeaplessTreeT,  HeaplessTree, ProofT, ProofItem, total_size, layer_size};
use crate::compactable::{CompactableHeaplessTree};

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

pub struct PeakProof<H: HashT> {
    root: H::Output,
//    items: Vec<ProofItem<2, H>>,
    items: Vec<<Self as ProofT<H>>::Item>,
}

impl<H: HashT> PeakProof<H> {
    fn from_tree_proof<T: ProofT<H, Item = <Self as ProofT<H>>::Item>>(proof: T) -> Self {
//        fn from_tree_proof<T: ProofT<H, ProofItem<2, H>>>(proof: T) -> Self {
        let (root, items) = proof.as_raw();
        Self {
            root,
            items: items.to_vec(),
        }
    }
} 

// impl<H: HashT> From<( &[Self::Item], H::Output )> for PeakProof<H> {
//     fn from(items_and_root: ( &[Self::Item], H::Output )) -> Self {
//         Self {
//             root: items_and_root.1,
//             items: items_and_root.0.into_iter().map(Self::Item::clone).collect(),
//         }
//     }
// }
// impl<H: HashT> From<( &[ProofItem<2, H>], H::Output )> for PeakProof<H> {
//     fn from(items_and_root: ( &[ProofItem<2, H>], H::Output )) -> Self {
//         Self {
//             root: items_and_root.1,
//             items: items_and_root.0.into_iter().map(ProofItem::<2, H>::clone).collect(),
//         }
//     }
// }

impl<H: HashT> ProofT<H> for PeakProof<H> {
    type Item = ProofItem<2, H>;

    fn root(&self) -> H::Output {
        self.root
    } 
    fn set_root(&mut self, root: H::Output) {
        self.root = root;
    }


//    fn push_item(&mut self, item: ProofItem<2, H>) {
    fn push_item(&mut self, item: Self::Item) {
        self.items.push(item);
    }
    // SHOULD BE UNIMPLEMENTED
    fn validate(self, input: &[u8]) -> bool {
        let mut curr_hash = Some(H::hash(&input));
        for item in self.items {
            curr_hash = curr_hash.and_then(|h| item.hash_with_siblings(h));
        }
        curr_hash.as_ref() == Some(&self.root)
    }
    fn as_raw(&self) -> (H::Output, &[Self::Item]) {
        (self.root, &self.items[..])
    }

}

impl<H: HashT> Default for PeakProof<H>
{
    fn default() -> Self {
        Self { 
            root: Default::default(),
            items: Default::default(), 
        }
    }
}

pub enum MerklePeak<H: HashT> {
    NonMergeable(CompactableHeaplessTree<2, 5, H>),
    First(CompactableHeaplessTree<2, 4, H>),
    Second(CompactableHeaplessTree<2, 3, H>),
    Third(CompactableHeaplessTree<2, 2, H>),
    Forth(CompactableHeaplessTree<2, 1, H>),
}

// impl<H: HashT> MerklePeak<H> {
//     fn convert_proof()
// }

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

impl<H: HashT> HeaplessTreeT<H> for MerklePeak<H> 
{
    type Proof = PeakProof<H>;

    fn generate_proof(&mut self, index: usize) -> Self::Proof {
        use MerklePeak::*;
        match self {
            NonMergeable(this) => Self::Proof::from_tree_proof(this.generate_proof(index)),
            Forth(this) => Self::Proof::from_tree_proof(this.generate_proof(index)),
            Third(this) => Self::Proof::from_tree_proof(this.generate_proof(index)),
            Second(this) => Self::Proof::from_tree_proof(this.generate_proof(index)),
            First(this) => Self::Proof::from_tree_proof(this.generate_proof(index)),
        }
    }

    fn replace(&mut self, index: usize, input: &[u8]) {
        apply!(self, replace, index, input)
    }
    fn remove(&mut self, index: usize) {
        apply!(self, remove, index)
    }
    fn root(&self) -> H::Output {
        apply!(self, root,)
    }
    fn leaves(&self) -> &[H::Output] {
        apply!(self, leaves,)
    }
    fn base_layer_size(&self) -> usize {
        apply!(self, base_layer_size,)
    }    
    fn branch_factor(&self) -> usize {
        apply!(self, branch_factor,)
    }
    fn height(&self) -> usize {
        apply!(self, height,)
    }
}

impl<H: HashT> MerklePeak<H> {
    pub fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
        apply!(self, try_append, input)
    }

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