use crate::{HashT, HeaplessTreeT,  HeaplessTree, ProofBuilder, ProofItem, ProofItemT, Proof, total_size, layer_size};
//use crate::compactable::compactable::{MergeableHeaplessTree};
use crate::mergeable::mergeable::{MergeableHeaplessTree};

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

type PeakProof<H> = Proof<2, 8, H>;

pub enum MerklePeak<H: HashT, PB: ProofBuilder<H>> {
    NonMergeable(MergeableHeaplessTree<2, 5, H, PB>),
    First(MergeableHeaplessTree<2, 4, H, PB>),
    Second(MergeableHeaplessTree<2, 3, H, PB>),
    Third(MergeableHeaplessTree<2, 2, H, PB>),
    Forth(MergeableHeaplessTree<2, 1, H, PB>),
}
// pub enum MerklePeak<H: HashT> {
//     NonMergeable(MergeableHeaplessTree<2, 5, H, PeakProof<H>>),
//     First(MergeableHeaplessTree<2, 4, H, PeakProof<H>>),
//     Second(MergeableHeaplessTree<2, 3, H, PeakProof<H>>),
//     Third(MergeableHeaplessTree<2, 2, H, PeakProof<H>>),
//     Forth(MergeableHeaplessTree<2, 1, H, PeakProof<H>>),
// }

impl<H: HashT, PB: ProofBuilder<H>> Clone for MerklePeak<H, PB> {
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

impl<H: HashT, PB: ProofBuilder<H>> Default for MerklePeak<H, PB> {
    fn default() -> Self {
        Self::Forth(MergeableHeaplessTree::try_from(&[]).unwrap())
    }
}

impl<H: HashT, PB: ProofBuilder<H>> Copy for MerklePeak<H, PB> {}

impl<H: HashT, PB: ProofBuilder<H>> HeaplessTreeT<H, PB> for MerklePeak<H, PB> {
    fn generate_proof(&mut self, index: usize) -> PB {
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

impl<H: HashT, PB: ProofBuilder<H>> MerklePeak<H, PB> {
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
macro_rules! height_from_num_of_peaks {
    ($branch_factor:expr, $num_of_peaks:expr) => {
        (8 * core::mem::size_of::<usize>() as u32 - $num_of_peaks.leading_zeros()) as usize + 1
    };
}

pub struct MerkleMR<const PEAKS: usize, H: HashT, PB: ProofBuilder<H>> 
where 
    [(); {height_from_num_of_peaks!(2_usize, PEAKS) - 1}]: Sized,
    [(); total_size!(2_usize, height_from_num_of_peaks!(2_usize, PEAKS))]: Sized,

{
    summit_tree: HeaplessTree<2, {height_from_num_of_peaks!(2_usize, PEAKS)}, H>,
    peaks: [MerklePeak<H, PB>; PEAKS],
    peak_roots: [H::Output; PEAKS],
    curr_peak_index: usize,
}

impl<const PEAKS: usize, H: HashT, PB: ProofBuilder<H>> MerkleMR<PEAKS, H, PB> 
where 
    [(); {height_from_num_of_peaks!(2_usize, PEAKS) - 1}]: Sized,
    [(); total_size!(2_usize, height_from_num_of_peaks!(2_usize, PEAKS))]: Sized,
{    
    pub fn from(peak: MerklePeak<H, PB>) -> Self {
        let mut this = Self {
            summit_tree: HeaplessTree::<2, {height_from_num_of_peaks!(2_usize, PEAKS)}, H>::try_from(&[]).unwrap(),
            peaks: [MerklePeak::<H, PB>::default(); PEAKS],
            peak_roots: [H::Output::default(); PEAKS],
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
                        self.peak_roots[i - 1] = merged.root();
                        self.peaks[i] = Default::default();
                        self.peak_roots[i] = self.peaks[i].root();
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
        // try to append item to the current peak
        let could_append_to_current = self.peaks[self.curr_peak_index].try_append(input);
        // if couldn't append, it's because the underlying tree is full
        if could_append_to_current.is_err() {
            // so if the current peak is not last...
            if self.curr_peak_index < PEAKS {
                // move to the next peak and set it the new current one
                self.curr_peak_index += 1;
                // try append the item now to the new peak
                self.peaks[self.curr_peak_index].try_append(input)?;
                self.peak_roots[self.curr_peak_index] = self.peaks[self.curr_peak_index].root();
            } else { 
                return Err(()); 
            }
        }
        let need_to_rebuild_summit_tree = prev_peak_index != self.curr_peak_index;      
        // now back propagate the peaks and merge them if necessary
        self.merge_collapse()
            .and_then(|_| {
                if need_to_rebuild_summit_tree {
                    self.summit_tree = HeaplessTree::<2, {height_from_num_of_peaks!(2_usize, PEAKS)}, H>::try_from_leaves(&self.peak_roots[..])?;
                } else {
                    self.summit_tree.replace_leaf(self.curr_peak_index, self.peaks[self.curr_peak_index].root());
                }
                Ok(())
            })
    }

    pub fn generate_proof(&mut self, index: usize) -> PB {
        let mut accum_len = 0;
        for (peak_ind, peak) in self.peaks.iter_mut().enumerate() {
            if accum_len + peak.num_of_leaves() > index {

                let mut proof = peak.generate_proof(index - accum_len);

                let summit_proof = self.summit_tree.generate_proof(peak_ind);

                let (summit_root, summit_items) = summit_proof.as_raw();
                
                proof.set_root(summit_root);
                
                for item in summit_items {
                    let (offset, hashes) = item.as_raw();
                    if let Some(hashes) = hashes {
                        proof.push(offset, hashes);
                    } else {
                        break;
                    }
                }
                return proof;
            }
            accum_len += peak.num_of_leaves();
        }
        unreachable!();
    }

    pub fn curr_peak_index(&self) -> usize {
        self.curr_peak_index
    }

    pub fn peaks(&self) -> &[MerklePeak<H, PB>; PEAKS] {
        &self.peaks
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::{HashT, HeaplessTreeT,  HeaplessTree, ProofBuilder, ProofValidator, Proof, total_size, layer_size};
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
        let mut mmr = MerkleMR::<PEAKS, StdHash, PeakProof<StdHash>>::from(first_peak);

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

        let mut first_peak = MerklePeak::Forth(cmt);
        let mut mmr = MerkleMR::<PEAKS, StdHash, PeakProof<StdHash>>::from(first_peak);
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