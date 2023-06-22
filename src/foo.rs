
// use crate::{HashT, HeaplessTreeT, HeaplessTree, HeaplessBinaryTree, ProofItem};

// macro_rules! total_size {
//     ($branch_factor:expr, $layers:expr) => {
//         ((1 << ($branch_factor.trailing_zeros() as usize * $layers)) - 1) / ($branch_factor - 1)
//     };
// }

// pub struct Proof<const BRANCH_FACTOR: usize, H: HashT> {
//     items: Vec<ProofItem<BRANCH_FACTOR, H>>,
// }
// impl<const BRANCH_FACTOR: usize, H: HashT> Proof<BRANCH_FACTOR, H> {
//     fn from(items: &[ProofItem<BRANCH_FACTOR, H>]) -> Self {
//         Self {
//             items: Vec::from(items),
//         }
//     }
//     pub fn validate(self, root: &H::Output, input: &[u8]) -> bool {
//         let mut curr_hash = Some(H::hash(&input));
//         // start from the base layer, 
//         // and for every item in the proof
//         // put the hash derived from input into the proof item
//         // at index stored in the proof item
//         // and hash it with the siblings
//         for item in self.items {
//             curr_hash = curr_hash.and_then(|h| item.hash_with_siblings(h));
//         }
//         // validated iff the resulting hash is identical to the root
//         curr_hash.as_ref() == Some(root)
//     }      
// }

// pub struct Foo<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> 
// where
// [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized
// {
// //        subtrees: [HeaplessTree<BRANCH_FACTOR, 3, StdHash>; BRANCH_FACTOR],
//     subtrees: Vec<HeaplessTree<BRANCH_FACTOR, HEIGHT, H>>,
// }

// impl<const BRANCH_FACTOR: usize, const HEIGHT: usize, H: HashT> Foo<BRANCH_FACTOR, HEIGHT, H> 
// where
//     [(); total_size!(BRANCH_FACTOR, HEIGHT)]: Sized
// {
//     pub fn from_base_trees(subtrees: [HeaplessTree<BRANCH_FACTOR, HEIGHT, H>; BRANCH_FACTOR]) -> Self {
//         Self {
//             subtrees: Vec::from(subtrees)
//         }
//     }

//     pub fn try_from(input: &[&[u8]]) -> Result<Self, ()> {
//         let base_layer_size = HeaplessTree::<BRANCH_FACTOR, HEIGHT, H>::base_layer_size();

//         let mut this = Self {
//             subtrees: Vec::new(),
//         };
        
//         for i in (0..input.len()).step_by(base_layer_size) {
//             // check if input is too long
//             if this.subtrees.len() > BRANCH_FACTOR - 1 {
//                 return Err(());
//             }
//             this.subtrees.push(
//                 HeaplessTree::<BRANCH_FACTOR, HEIGHT, H>::try_from(
//                     if i + base_layer_size <= input.len() {
//                         &input[i..i + base_layer_size]
//                     } else {
//                         &input[i..]
//                     }
//                 )?
//             );
//         }
//         Ok(this)
//     }

    
        
//     fn base_layer_size(&self) -> usize {
//         self.subtrees.len() * HeaplessTree::<BRANCH_FACTOR, HEIGHT, H>::base_layer_size()
//     }

//     fn hash_merged_slice(contiguous_array: &[H::Output], len: usize) -> H::Output {
//         H::hash(
//             unsafe { core::slice::from_raw_parts(contiguous_array[0].as_ref().as_ptr(), len) }
//         )
//     }
    
//     // fn generate_proof(&mut self, index: usize) -> (H::Output, Proof<BRANCH_FACTOR, {HEIGHT+1}, H>) 
//     pub fn generate_proof(&mut self, index: usize) -> (H::Output, Proof<BRANCH_FACTOR, H>) 
//     where [(); HEIGHT - 1]: Sized
//     {
//         let base_subtree_index = index / HeaplessTree::<BRANCH_FACTOR, HEIGHT, H>::base_layer_size();
//         let index_in_subtree = index % HeaplessTree::<BRANCH_FACTOR, HEIGHT, H>::base_layer_size();

//         let (subtree_root, subtree_proof) = self.subtrees[base_subtree_index].generate_proof(index_in_subtree);
//         let mut hashes = [H::Output::default(); BRANCH_FACTOR];
//         for (i, subtree) in self.subtrees.iter().enumerate() {
//             hashes[i] = subtree.root();
//         }
//         hashes[base_subtree_index] = subtree_root;

//         let root = Self::hash_merged_slice(&hashes,
//                         self.subtrees.len() * core::mem::size_of::<H::Output>()
//         );
//         let offset = base_subtree_index;
// //            let mut proof = [ProofItem::<BRANCH_FACTOR, H>::default(); {HEIGHT+1}-1];
//         let mut proof = Proof::from(&subtree_proof.items);

//         // for (i, item) in subtree_proof.items.iter().enumerate() {
//         //     proof[i] = *item;
//         // }
//         proof.items.push(ProofItem {
//             hashes: hashes.try_into().ok(),
//             offset,
//         });
//         // proof[HEIGHT - 1] = ProofItem {
//         //     hashes: hashes.try_into().ok(),
//         //     offset,
//         // };
//         (root, proof)
// //            (root, Proof::<BRANCH_FACTOR, {HEIGHT + 1}, H> {items: proof})
//     }
// }
