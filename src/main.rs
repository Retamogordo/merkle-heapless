//! This is minimal Merkle tree implementation with proof checking
#![feature(generic_const_exprs)]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Debug)]
struct MerkleTree<'a, const LAYERS: usize> 
where 
    [u64; (1 << LAYERS) - 1]: Sized,
{
    hashes: [u64; (1 << LAYERS) - 1],
}

impl<'a, const LAYERS: usize> MerkleTree<'a, LAYERS> 
where 
    [u64; (1 << LAYERS) - 1]: Sized, 
{
    const TOTAL_SIZE: usize = (1 << LAYERS) - 1;
    const BOTTOM_LAYER_SIZE: usize = 1 << (LAYERS - 1);
    // panics if LAYERS == 0
    pub fn try_from(input: &[&'a str]) -> Result<Self, ()> {
        if input.len() > Self::BOTTOM_LAYER_SIZE {
            return Err(());
        }
        let mut this = Self {
            hashes: [Default::default(); (1 << LAYERS) - 1],
        };
        let mut i = 0;
        for d in input {
            this.hashes[i] = hash(d);
            i += 1;
        }
        // pad the rest of hashes
        while i < Self::BOTTOM_LAYER_SIZE {
            this.hashes[i] = hash::<&str>(&Default::default());
            i += 1;
        }

        this.fill_layers();
        
        Ok(this)
    }

    fn fill_layers(&mut self) {
        let mut start_ind = 0;
        let mut next_layer_ind = Self::BOTTOM_LAYER_SIZE;
        while next_layer_ind < Self::TOTAL_SIZE {
            let mut j = next_layer_ind;
            for i in (start_ind..next_layer_ind).step_by(2) {
                self.hashes[j] = concat_hashes(self.hashes[i], self.hashes[i + 1]);
                j += 1;
            }
            start_ind = next_layer_ind;
            next_layer_ind = j;
        }
    }

    pub fn generate_proof(&mut self, index: usize) -> (u64, [Sibling; LAYERS - 1]) {    
        let mut proof = [Sibling::None; LAYERS - 1]; 
        let root = self.build_path(index, &mut proof);
        (root, proof)
    }

    fn build_path(&mut self, index: usize, proof: &mut [Sibling; LAYERS - 1]) -> HashValue {
        let mut proof_ind = 0;
        let mut layer_base = 0;
        let mut index = index;
        let mut layer_len = 1 << (LAYERS - 1);

        for _ in 0..LAYERS - 1 {

            proof[proof_ind] = match index & 1 {
                0 => Sibling::Right(self.hashes[index + 1]),
                _ => Sibling::Left(self.hashes[index - 1]),
            };

            proof_ind += 1;

            index = layer_len + (index + layer_base) / 2;
            layer_base += layer_len;
            layer_len >>= 1;
        }
        self.hashes[index]
    }  

}

pub fn validate_proof<const LAYERS: usize>(root: &HashValue, word: &str, proof: [Sibling; LAYERS - 1]) -> bool {
    let mut curr_hash = hash(&word);

    for sibling in proof {
        curr_hash = match sibling {
            Sibling::Left(h) => concat_hashes(h, curr_hash),
            Sibling::Right(h) => concat_hashes(curr_hash, h),
            Sibling::None => unreachable!("sibling is None"), 
        };
    }
    &curr_hash == root
}    

fn concat_hashes(left: HashValue, right: HashValue) -> HashValue {
    hash(&[left, right])
}        

/// We'll use Rust's built-in hashing which returns a u64 type.
/// This alias just helps us understand when we're treating the number as a hash
pub type HashValue = u64;

/// Helper function that makes the hashing interface easier to understand.
pub fn hash<T: Hash>(t: &T) -> HashValue {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


/// A representation of a sibling node along the Merkle path from the data
/// to the root. It is necessary to specify which side the sibling is on
/// so that the hash values can be combined in the same order.
#[derive(Clone, Copy, Debug)]
pub enum Sibling {
    Left(HashValue),
    Right(HashValue),
    None,
}


fn main() {
    // let mt = MerkleTree::<2>::try_from(&["apple", "banana"]);
    // println!("MT: {:?}", mt);

    // let mt = MerkleTree::<2>::try_from(&["apple", "banana", "kiwi"]);
    // println!("MT: {:?}", mt);

    let mut mt = MerkleTree::<3>::try_from(&["apple", "banana", "kiwi", "kotleta", "kaka"]);
    println!("MT: {:?}", mt);

//    let mut path: [Sibling; 3 - 1] = [Sibling::None; 3 - 1];
    let (root, proof) = mt.as_mut().unwrap().generate_proof(2);
    println!("root: {}; proof: {:?}", root, proof);
    let res = validate_proof(&root, "kiwi", proof);
    println!("root: {}; res: {:?}", root, res);
    
    let (root, proof) = mt.as_mut().unwrap().generate_proof(1);
    println!("root: {}; proof: {:?}", root, proof);
    let res = validate_proof(&root, "banana", proof);
    println!("root: {}; res: {:?}", root, res);

    let (root, proof) = mt.as_mut().unwrap().generate_proof(3);
    println!("root: {}; proof: {:?}", root, proof);
    let res = validate_proof(&root, "banana", proof);
    println!("root: {}; res: {:?}", root, res);

    let (root, proof) = mt.as_mut().unwrap().generate_proof(3);
    println!("root: {}; proof: {:?}", root, proof);
    let res = validate_proof(&root, "kotleta", proof);

    println!("root: {}; res: {:?}", root, res);
}
