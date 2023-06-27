// // use std::mem::size_of;
// // fn main() {
// //     // let base_layer_size = 8;
// //     // let branch_factor = 2;
// //     // let input_len = 19;
// //     // let x = base_layer_size * branch_factor;

// //     // let pad_len = input_len / x + if input_len % x == 0 {0} else {1};
// //     // println!("pad_len: {pad_len}");
// //     let branch_factor: u32 = 2;
// //     let n_subtrees: usize = 5;
// //     let n_subtrees_padded = branch_factor.trailing_zeros() << (8*size_of::<usize>() as u32 - n_subtrees.leading_zeros());
// //     println!("branch_factor: {branch_factor}, n_subtrees: {n_subtrees}, n_subtrees_padded: {n_subtrees_padded}");

// //     let branch_factor: u32 = 4;
// //     let n_subtrees: usize = 5;
// //     let n_subtrees_padded = branch_factor.trailing_zeros() << (8*size_of::<usize>() as u32 - n_subtrees.leading_zeros());
// //     println!("branch_factor: {branch_factor}, n_subtrees: {n_subtrees}, n_subtrees_padded: {n_subtrees_padded}");
// // }

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

// impl<'a> Fractal<'a> 
// {
//     fn leaf(data: &[u8]) -> Self {
//         Self {
//             root: myhash(data),
//             leaves: None,
//             height: 0,
//         }
//     }
//     fn from(children: [&'a Option<Self>; 2]) -> Self {
//         Self {
// //            root: myhash([children[0].root.as_slice(), children[1].root.as_slice()].as_slice()[0]),
//             root: myhash(&[children[0].root as u8, children[1].root as u8]),
//             leaves: Some(children),
//             height: children[0].height + 1,
//         }
//     }

//     fn bfs<F: Fn(&Self)>(&self, f: F) {
//         let mut stack = VecDeque::new();
//         stack.push_back(self);

//         while let Some(curr) = stack.pop_front() {
//             (f)(curr);
//             if let Some(leaves) = curr.leaves {
//                 stack.push_back(leaves[0]);
//                 stack.push_back(leaves[1]);
//             }
//         }
//     }

//     fn printme(&self) {
//         self.bfs(|t| { println!("{:?}", t.root); });
//     }
// }

// impl<'a> Debug for Fractal<'a>
// {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> { 

//         let mut stack = VecDeque::new();
//         stack.push_back(self);

//         while let Some(curr) = stack.pop_front() {
//             writeln!(f, "{:?}", curr.root)?;
//             if let Some(leaves) = curr.leaves {
//                 stack.push_back(leaves[0]);
//                 stack.push_back(leaves[1]);
//             }
//         }
//         Ok(())        
//     }
// }
// use core::fmt::Debug;

//use merkle_heapless::{total_size, layer_size, HashT, ProofBuilder, ProofItem, BasicTreeTrait, HeaplessTree, HeaplessBinaryTree};
//use merkle_heapless::compactable::{CompactableHeaplessTree};

// fn foo() {
//     x = Vec::<Box<dyn BasicTreeTrait<H, ProofItem<2, H>>>>::new();
// }
//use merkle_heapless::dynamic::{DynamicTree};

// struct MerkleCons<H: HashT, T: BasicTreeTrait<H>>
// {
//     left: T,
//     right: Option<Box<MerkleCons<H, T>>>,
//     h: Option<H>,
// //right: Option<Box<dyn TryInto<MerkleCons<'a, BRANCH_FACTOR, HEIGHT, H>,  Error=()>>>
// //    right: Option<&'a MerkleCons<'a, BRANCH_FACTOR, OTHER_HEIGHT, OTHER_HEIGHT, H>>,
// }

// impl<H: HashT, T: BasicTreeTrait<H>> MerkleCons<H, T> {
//     fn new(left: T) -> Self {
//         Self {
//             left,
//             right: None,
//             h: None,
//         }
//     }
//     // fn append<U: BasicTreeTrait<H>>(&mut self, right: MerkleCons<H, U>) {
//     //     self.right = Some(Box::new(right));
//     // }
// }


// struct MyTree {
// }

// struct Foo<H: HashT> {
//     x: Vec<DynamicTree<H>>,
// }

// #![feature(generic_const_exprs)]
// #![feature(const_evaluatable_checked)]
// struct ConsPeaks<const N: usize>
// where
// [(); N]: Sized, 
// [(); {N-1}]: Sized, 
//     {
//     data: [u8; N],
//     next: Option<Box<ConsPeaks<{N-1}>>>,
// }

// // trait UpgradableTo<const N: usize>: Sized
// // where 
// //     Assert::<{N > 0}>: IsTrue {
// // }

// struct Arr<const N: usize>
// where 
//     Assert::<{N > 0}>: IsTrue,
// {
//     data: [u8; N],
//     next: Option<Box<Arr<{N-1}>>>,
// }

// enum Assert<const COND: bool> {}

// trait IsTrue {}

// impl IsTrue for Assert<true> {}


use merkle_heapless::{HashT, ProofValidator, BasicTreeTrait};
//use crate::compactable::compactable::{MergeableHeaplessTree};
//use merkle_heapless::mergeable::mergeable::{MergeableHeaplessTree};
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

fn main() {
//    type PeakProof<H> = Proof<2, 5, H>;

    mmr_macro::mmr!(BranchFactor = 2, Peaks = 5);
    mmr_macro::mmr!(Type = Foo, BranchFactor = 2, Peaks = 6);

    // let mut cmt = MergeableHeaplessTree::<2, 5, StdHash, PeakProof<StdHash>>::try_from(
    //     &[]
    // ).unwrap();

    // let peak1 = MerklePeak::PeakHeight0(cmt);

    let mmr2 = Foo::<StdHash>::default();
    let mut mmr = MerkleMountainRange::<StdHash>::default();
    // peak leaf numbers: [0, 0, 0, 0, 0]
    mmr.try_append(b"apple").unwrap();
    // peak leaf numbers: [1, 0, 0, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
    let proof = mmr.generate_proof(0);
    let res = proof.validate(b"apple");
    assert!(res);
    
    mmr.try_append(b"banana").unwrap();
    // peak leaf numbers: [2, 0, 0, 0, 0] because 1, 1 is merged -> 2, 0
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
    let proof = mmr.generate_proof(1);
    let res = proof.validate(b"banana");
    assert!(res);

    mmr.try_append(b"cherry").unwrap();
    // peak leaf numbers: [2, 1, 0, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
    let proof = mmr.generate_proof(2);
    let res = proof.validate(b"cherry");
    assert!(res);

    mmr.try_append(b"kiwi").unwrap();
    // peak leaf numbers: [4, 0, 0, 0, 0] because 2, 1, 1 is merged -> 2, 2, 0 -> 4, 0, 0
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
    let proof = mmr.generate_proof(3);
    let res = proof.validate(b"kiwi");
    assert!(res);

    mmr.try_append(b"lemon").unwrap();
    // peak leaf numbers: [4, 1, 0, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
    let proof = mmr.generate_proof(4);
    let res = proof.validate(b"lemon");
    assert!(res);

    mmr.try_append(b"lime").unwrap();
    // peak leaf numbers: [4, 2, 0, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
    let proof = mmr.generate_proof(5);
    let res = proof.validate(b"lime");
    assert!(res);

    mmr.try_append(b"mango").unwrap();
    // peak leaf numbers: [4, 2, 1, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
    assert_eq!(mmr.peaks()[2].num_of_leaves(), 1);

    mmr.try_append(b"carrot").unwrap();
    // peak leaf numbers: [8, 0, 0, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
    
    mmr.try_append(b"potato").unwrap();
    // peak leaf numbers: [8, 1, 0, 0, 0]
    assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
    assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);

}
