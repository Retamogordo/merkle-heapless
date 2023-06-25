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
#![feature(generic_const_exprs)]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
// use std::collections::VecDeque;


// // fn myhash(input: &[u8]) -> [u8; 8] {
// //     let mut s = DefaultHasher::new();
// //     input.hash(&mut s);
// //     s.finish().to_ne_bytes()
// // }
// fn myhash(input: &[u8]) -> u16 {
//     match input.len() {
//         1 => input[0] as u16,
//         2 => core::cmp::max(input[0], input[1]) as u16 + 1,
// //        2 => ((input[0] as u16) << 8) + input[1] as u16,
//         _ => unimplemented!(),
//     }
// }

// #[derive(Default)]
// struct Fractal<'a> {
// //    root: [u8; 8],
//     root: u16,
//     leaves: Option<[&'a Option<Self>; 2]>,
//     height: usize,
// }

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

use merkle_heapless::{total_size, layer_size, HashT, ProofBuilder, ProofItem, HeaplessTreeT, HeaplessTree, HeaplessBinaryTree};
//use merkle_heapless::compactable::{CompactableHeaplessTree};

// fn foo() {
//     x = Vec::<Box<dyn HeaplessTreeT<H, ProofItem<2, H>>>>::new();
// }
//use merkle_heapless::dynamic::{DynamicTree};

// struct MerkleCons<H: HashT, T: HeaplessTreeT<H>>
// {
//     left: T,
//     right: Option<Box<MerkleCons<H, T>>>,
//     h: Option<H>,
// //right: Option<Box<dyn TryInto<MerkleCons<'a, BRANCH_FACTOR, HEIGHT, H>,  Error=()>>>
// //    right: Option<&'a MerkleCons<'a, BRANCH_FACTOR, OTHER_HEIGHT, OTHER_HEIGHT, H>>,
// }

// impl<H: HashT, T: HeaplessTreeT<H>> MerkleCons<H, T> {
//     fn new(left: T) -> Self {
//         Self {
//             left,
//             right: None,
//             h: None,
//         }
//     }
//     // fn append<U: HeaplessTreeT<H>>(&mut self, right: MerkleCons<H, U>) {
//     //     self.right = Some(Box::new(right));
//     // }
// }

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

struct MyTree {
}

// struct Foo<H: HashT> {
//     x: Vec<DynamicTree<H>>,
// }

// #![feature(generic_const_exprs)]
// #![feature(const_evaluatable_checked)]
// struct ConsPeaks<const N: usize>
// where
//     [(); N]: Sized, {
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

fn main() {
//    let left = CompactableHeaplessTree::<2, 3, 5, StdHash>::try_from(&[b"apple"]).unwrap();
    // let mut cons = MerkleCons::new(left);

    // let left_next = CompactableHeaplessTree::<2, 2, StdHash>::try_from(&[b"banana"]).unwrap();
    // let right = MerkleCons::new(left_next);

//    cons.append(right);

//     let foo = Foo::<3_usize, StdHash>{
//         left: CompactableHeaplessTree<2_usize, {3_usize - 1}, StdHash>::try_from(&[b"apple"]).unwrap(),
//         right: None,
// _    };
//     let t1 = Fractal::leaf(&[1]);
//     let t2 = Fractal::leaf(&[2]);
//     let t3 = Fractal::from([&t1, &t2]);

// //    println!("{:?}", t3);
//     t3.printme();
}
