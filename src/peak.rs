use crate::{HashT, BasicTreeTrait,  HeaplessTree, ProofBuilder, ProofDetails, ProofItemT, Proof, merge_proofs, total_size};
//use crate::compactable::compactable::{MergeableHeaplessTree};
use crate::mergeable::mergeable::{MergeableHeaplessTree};

#[macro_export]
macro_rules! height_from_num_of_peaks {
    ($branch_factor:expr, $num_of_peaks:expr) => {
        (8 * core::mem::size_of::<usize>() as u32 - ($num_of_peaks as usize).leading_zeros()) as usize + 1
    };
}

#[macro_export]
macro_rules! total_mmr_height {
    ($branch_factor:expr, $num_of_peaks:expr) => {
        height_from_num_of_peaks!($branch_factor, $num_of_peaks) + $num_of_peaks as usize
    };
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

macro_rules! decl_peak_proof {
    ($max_height:expr) => {
        type PeakProof<H> = Proof<2, $max_height, H>;
        
    };
}

// macro_rules! decl_peak {
//     ($max_height:expr, $($args:ident),*) => {
//         decl_peak_proof!(8);

//         pub enum Foo {
//             $($args(u32)),*
//         }
//     };
// }

//decl_peak!(8, First, Second);

type PeakProof<H> = Proof<2, 5, H>;
//type MMRProof<H> = Proof<2, {total_mmr_height!(2_usize, 5)}, H>;
type MMRProof<H> = Proof<2, 9, H>;
//static foo: Foo = Foo::First(8);

pub enum MerklePeak<H: HashT> {
    NonMergeable(MergeableHeaplessTree<2, 5, H, PeakProof<H>>),
    First(MergeableHeaplessTree<2, 4, H, PeakProof<H>>),
    Second(MergeableHeaplessTree<2, 3, H, PeakProof<H>>),
    Third(MergeableHeaplessTree<2, 2, H, PeakProof<H>>),
    Forth(MergeableHeaplessTree<2, 1, H, PeakProof<H>>),
}

impl<H: HashT> MerklePeak<H> {
    fn branch_factor(&self) -> usize {
//        use MerklePeak::*;
        match self {
            MerklePeak::NonMergeable(tree) => tree as &dyn BasicTreeTrait<H, PeakProof<H>>,
            MerklePeak::First(tree) => tree as &dyn BasicTreeTrait<H, PeakProof<H>>,
            MerklePeak::Second(tree) => tree as &dyn BasicTreeTrait<H, PeakProof<H>>,
            MerklePeak::Third(tree) => tree as &dyn BasicTreeTrait<H, PeakProof<H>>,
            MerklePeak::Forth(tree) => tree as &dyn BasicTreeTrait<H, PeakProof<H>>,
        }
        .branch_factor()
    }
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
        Self::Forth(MergeableHeaplessTree::try_from(&[]).unwrap())
    }
}

impl<H: HashT> Copy for MerklePeak<H> {}

impl<H: HashT> BasicTreeTrait<H, PeakProof<H>> for MerklePeak<H> {
    fn generate_proof(&mut self, index: usize) -> PeakProof<H> {
        apply!(self, generate_proof, index)
    }
    fn replace(&mut self, index: usize, input: &[u8]) {
        apply!(self, replace, index, input)
    }
    fn replace_leaf(&mut self, index: usize, leaf: H::Output) {
        apply!(self, replace_leaf, index, leaf)
    }
    fn remove(&mut self, index: usize) {
        apply!(self, remove, index)
    }
    fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
        apply!(self, try_append, input)
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
    fn num_of_leaves(&self) -> usize {
        apply!(self, num_of_leaves,)
    }
}

impl<H: HashT> MerklePeak<H> {

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
}

pub struct MerkleMR<const PEAKS: usize, H: HashT> 
where 
    [(); PEAKS - 1]: Sized,
    [(); {PEAKS + height_from_num_of_peaks!(2_usize, PEAKS)} - 1]: Sized,
    [(); height_from_num_of_peaks!(2_usize, PEAKS) - 1]: Sized,
    [(); total_size!(2_usize, height_from_num_of_peaks!(2_usize, PEAKS))]: Sized,
{
//    summit_tree: HeaplessTree<2, {height_from_num_of_peaks!(2_usize, PEAKS)}, H>,
    summit_tree: HeaplessTree<2, 4, H>,
    peaks: [MerklePeak<H>; PEAKS],
    curr_peak_index: usize,
}

impl<const PEAKS: usize, H: HashT> MerkleMR<PEAKS, H> 
where 
    [(); PEAKS - 1]: Sized,
    [(); {PEAKS + height_from_num_of_peaks!(2_usize, PEAKS)} - 1]: Sized,
    [(); height_from_num_of_peaks!(2_usize, PEAKS) - 1]: Sized,
    [(); total_size!(2_usize, height_from_num_of_peaks!(2_usize, PEAKS))]: Sized,
{    
    pub fn from(peak: MerklePeak<H>) -> Self {
        let mut this = Self {
            summit_tree: HeaplessTree::<2, 4, H>::try_from(&[]).unwrap(),
//            summit_tree: HeaplessTree::<2, {height_from_num_of_peaks!(2_usize, PEAKS)}, H>::try_from(&[]).unwrap(),
            peaks: [MerklePeak::<H>::default(); PEAKS],
            curr_peak_index: 0,
        }; 
        this.peaks[0] = peak;
        this
    } 

    fn merge_collapse(&mut self) -> Result<(), ()> {
        let mut i = self.curr_peak_index;
        // back propagate and merge peaks while possible    
        // the indicator that two peaks can merge is that they have the same rank (can check height or num_of_leaves)
        while i > 0 && self.peaks[i].height() == self.peaks[i - 1].height() {
            if self.peaks[i - 1]
                    .try_merge(self.peaks[i])
                    .map(|merged| {
                        self.peaks[i - 1] = merged;
                        self.peaks[i] = Default::default();
                    }).is_err() {
                return Err(());
            }  
            i -= 1;
            self.curr_peak_index = i;
        }
        Ok(())
    }

    pub fn try_append(&mut self, input: &[u8]) -> Result<(), ()> {
        let prev_peak_index = self.curr_peak_index;
        self.peaks[self.curr_peak_index]
            // try to append item to the current peak
            .try_append(input)
            // if couldn't append, it's because the underlying tree is full
            .or_else(|_| {
                // so if the current peak is not last...
                if self.curr_peak_index < PEAKS {
                    // move to the next peak and set it the new current one
                    self.curr_peak_index += 1;
                    // try append the item now to the new peak
                    self.peaks[self.curr_peak_index].try_append(input)
                } else { 
                    Err(())
                }
            })
            .and_then(|_| {
                let need_to_rebuild_summit_tree = prev_peak_index != self.curr_peak_index;      
                // now back propagate the peaks and merge them if necessary
                self.merge_collapse()
                    .map(|_| {
                        if need_to_rebuild_summit_tree {
                            self.peaks.iter().enumerate().for_each(|(i, peak)| 
                                self.summit_tree.replace_leaf(i, peak.root())
                            )
                        } else {
                            self.summit_tree.replace_leaf(self.curr_peak_index, self.peaks[self.curr_peak_index].root());
                        }
                    })
            })
    }

    pub fn generate_proof(&mut self, index: usize) -> MMRProof<H> {
        let mut accum_len = 0;
        let mut peak_ind = 0;

        for peak in self.peaks.iter() {
            if accum_len + peak.num_of_leaves() > index {
                break;
            }
            peak_ind += 1;
            accum_len += peak.num_of_leaves();
        }
        merge_proofs(
            self.peaks[peak_ind].generate_proof(index - accum_len),
            self.summit_tree.generate_proof(peak_ind)
        )
    }

    pub fn curr_peak_index(&self) -> usize {
        self.curr_peak_index
    }

    pub fn peaks(&self) -> &[MerklePeak<H>; PEAKS] {
        &self.peaks
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::{HashT, BasicTreeTrait, ProofValidator};
    use crate::mergeable::mergeable::{MergeableHeaplessTree};
    use crate::peak::{MerklePeak, PeakProof, MerkleMR};

    #[derive(Debug)]
    struct StdHash;

    impl HashT for StdHash {
        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }

    #[test]
    fn montain_peak_append() {
        const BRANCH_FACTOR: usize = 2;
        const FIRST_PEAK_HEIGHT: usize = 3;
        const PEAKS: usize = 5;

        let words1: &[&str] = &[
            "apple", "apricot", "banana",
        ];

        let cmt = MergeableHeaplessTree::<BRANCH_FACTOR, FIRST_PEAK_HEIGHT, StdHash, PeakProof<StdHash>>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let mut first_peak = MerklePeak::Second(cmt);

        first_peak.try_append(b"kiwi").unwrap();
        assert_eq!(first_peak.num_of_leaves(), 4);

        let proof = first_peak.generate_proof(3);
        let res = proof.validate(b"kiwi");

        assert!(res);
        let mut mmr = MerkleMR::<PEAKS, StdHash>::from(first_peak);

        // cmt.try_append(b"kiwi").unwrap();
        // cmt.try_append(b"kotleta").unwrap();
        // cmt.try_append(b"blueberry").unwrap();
        // assert!(cmt.try_append(b"blackberry").is_err());
    }

    #[test]
    fn mmr_merge_peaks_on_item_append() {
        const BRANCH_FACTOR: usize = 2;
        const FIRST_PEAK_HEIGHT: usize = 1;
        const PEAKS: usize = 5;

        let cmt = MergeableHeaplessTree::<BRANCH_FACTOR, FIRST_PEAK_HEIGHT, StdHash, PeakProof<StdHash>>::try_from(
            &[]
        ).unwrap();

        let first_peak = MerklePeak::Forth(cmt);
        let mut mmr = MerkleMR::<PEAKS, StdHash>::from(first_peak);
        // peak leaf numbers: [0, 0, 0, 0, 0]
        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1, 0, 0, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 1);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);
        
        mmr.try_append(b"banana").unwrap();
        // peak leaf numbers: [2, 0, 0, 0, 0] because 1, 1 is merged -> 2, 0
        assert_eq!(mmr.peaks[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"banana");
        assert!(res);

        mmr.try_append(b"cherry").unwrap();
        // peak leaf numbers: [2, 1, 0, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(2);
        let res = proof.validate(b"cherry");
        assert!(res);

        mmr.try_append(b"kiwi").unwrap();
        // peak leaf numbers: [4, 0, 0, 0, 0] because 2, 1, 1 is merged -> 2, 2, 0 -> 4, 0, 0
        assert_eq!(mmr.peaks[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(3);
        let res = proof.validate(b"kiwi");
        assert!(res);

        mmr.try_append(b"lemon").unwrap();
        // peak leaf numbers: [4, 1, 0, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(4);
        let res = proof.validate(b"lemon");
        assert!(res);

        mmr.try_append(b"lime").unwrap();
        // peak leaf numbers: [4, 2, 0, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 2);
        let proof = mmr.generate_proof(5);
        let res = proof.validate(b"lime");
        assert!(res);

        mmr.try_append(b"mango").unwrap();
        // peak leaf numbers: [4, 2, 1, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 2);
        assert_eq!(mmr.peaks[2].num_of_leaves(), 1);

        mmr.try_append(b"carrot").unwrap();
        // peak leaf numbers: [8, 0, 0, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 0);
        
        mmr.try_append(b"potato").unwrap();
        // peak leaf numbers: [8, 1, 0, 0, 0]
        assert_eq!(mmr.peaks[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks[1].num_of_leaves(), 1);
        // assert_eq!(first_peak.num_of_leaves(), 1);

        // let proof = first_peak.generate_proof(0);
        // let res = proof.validate(b"kiwi");

        // assert!(res);

        // cmt.try_append(b"kiwi").unwrap();
        // cmt.try_append(b"kotleta").unwrap();
        // cmt.try_append(b"blueberry").unwrap();
        // assert!(cmt.try_append(b"blackberry").is_err());
    }

}