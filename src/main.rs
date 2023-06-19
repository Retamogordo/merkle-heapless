//! This is minimal Merkle tree implementation with proof checking
#![feature(generic_const_exprs)]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

// #[derive(Default)]
// struct Fractal<'a> {
//     root: u64,
//     leaves: Option<[&'a Fractal<'a>; 2]>,
// }

// impl<'a> Fractal<'a> {
//     fn from(h: [u64; 2]) -> Self {
//         Self {
//             root: hash(h[0], h[1]),
//             leaves: None,
//         }
//     }
// } 

// struct FractalTree<'a, const LEAVES: usize> 
// where 
//     [u64; 2*LEAVES]: Sized {
//     hashes: [Fractal<'a>; 2*LEAVES],
// }

// impl<const LEAVES: usize> FractalTree<LEAVES>  
// where 
//     [u64; 2*LEAVES]: Sized {
//     pub fn from(input: &[&str; LEAVES]) -> Self {
//         Self {
//             hashes: [::default(); 2*LEAVES],
//         }
//     }
// }

#[derive(Debug)]
struct MerkleTree<H, const LAYERS: usize, F: Fn(&[u8]) -> H> 
where 
    [(); (1 << LAYERS) - 1]: Sized,
{
    hashes: [H; (1 << LAYERS) - 1],
    hf: F,
}

impl<H: Hash + Default + Copy + AsRef<[u8]>, const LAYERS: usize, F: Fn(&[u8]) -> H> MerkleTree<H, LAYERS, F> 
where 
    [(); (1 << LAYERS) - 1]: Sized, 
{
    const TOTAL_SIZE: usize = (1 << LAYERS) - 1;
    const BOTTOM_LAYER_SIZE: usize = 1 << (LAYERS - 1);
    // panics if LAYERS == 0
    pub fn try_from(input: &[&[u8]], hf: F) -> Result<Self, ()> {
        if input.len() > Self::BOTTOM_LAYER_SIZE {
            return Err(());
        }
        let mut this = Self {
            hashes: [H::default(); (1 << LAYERS) - 1],
            hf,
        };
        let mut i = 0;
        for d in input {
            this.hashes[i] = (this.hf)(d);
            i += 1;
        }
        // pad the rest of hashes
        while i < Self::BOTTOM_LAYER_SIZE {
            this.hashes[i] = (this.hf)(&[]);
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
                self.hashes[j] = concat_hashes(&self.hf, self.hashes[i], self.hashes[i + 1]);
                j += 1;
            }
            start_ind = next_layer_ind;
            next_layer_ind = j;
        }
    }

    pub fn generate_proof(&mut self, index: usize) -> (H, [Sibling<H>; LAYERS - 1]) {    
        let mut proof = [Sibling::None; LAYERS - 1]; 
        let root = self.build_path(index, &mut proof);
        (root, proof)
    }

    fn build_path(&mut self, index: usize, proof: &mut [Sibling<H>; LAYERS - 1]) -> H {
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

pub fn validate_proof<H: Hash + AsRef<[u8]> + PartialEq, const LAYERS: usize, F: Fn(&[u8]) -> H>(
    root: &H, 
    word: &[u8], 
    proof: [Sibling<H>; LAYERS - 1],
    hf: F, 
) -> bool {
//    let mut curr_hash = hash(&word);
    let mut curr_hash = (hf)(&word);

    for sibling in proof {
        curr_hash = match sibling {
            Sibling::Left(h) => concat_hashes(&hf, h, curr_hash),
            Sibling::Right(h) => concat_hashes(&hf, curr_hash, h),
            Sibling::None => unreachable!("sibling is None"), 
        };
    }
    &curr_hash == root
}    

fn concat_hashes<T: Hash + AsRef<[u8]>, H, F: Fn(&[u8]) -> H>(hf: &F, left: T, right: T) -> H {
    let mut h = [u8::default(); 64];
    for i in 0..left.as_ref().len() {
        h[i] = left.as_ref()[i];
    }

    let mut j = left.as_ref().len();
    for i in 0..right.as_ref().len() {
        h[j] = right.as_ref()[i];
        j += 1;
    }
    (hf)(&h)
}        

/// We'll use Rust's built-in hashing which returns a u64 type.
/// This alias just helps us understand when we're treating the number as a hash
//pub type HashValue = [u8; 32];
//pub type HashValue = u64;

/// Helper function that makes the hashing interface easier to understand.
// pub fn hash<T: Hash + AsRef<[u8]>, H>(t: &T, f: fn(&T) -> H) -> H {
//     f(t.as_ref())
// //    let mut s = DefaultHasher::new();
// //    sp_core::blake2_256(t.as_ref())
// //    t.hash(&mut s);
// //    s.finish()
// }


/// A representation of a sibling node along the Merkle path from the data
/// to the root. It is necessary to specify which side the sibling is on
/// so that the hash values can be combined in the same order.
#[derive(Clone, Copy, Debug)]
pub enum Sibling<H> {
    Left(H),
    Right(H),
    None,
}


fn main() {

    // let mt = MerkleTree::<2>::try_from(&["apple", "banana"]);
    // println!("MT: {:?}", mt);

    // let mt = MerkleTree::<2>::try_from(&["apple", "banana", "kiwi"]);
    // println!("MT: {:?}", mt);

    let mut mt = MerkleTree::<[u8; 32], 3, _>::try_from(&[b"apple", b"banana", b"kiwi", b"kotleta"], sp_core::blake2_256);
//    println!("MT: {:?}", mt);

//    let mut path: [Sibling; 3 - 1] = [Sibling::None; 3 - 1];
    let (root, proof) = mt.as_mut().unwrap().generate_proof(2);
    println!("root: {:?}; proof: {:?}", root, proof);
    let res = validate_proof(&root, &b"kiwi"[..], proof, sp_core::blake2_256);
    println!("root: {:?}; res: {:?}", root, res);
    
    let (root, proof) = mt.as_mut().unwrap().generate_proof(1);
    println!("root: {:?}; proof: {:?}", root, proof);
    let res = validate_proof(&root, &b"banana"[..], proof, sp_core::blake2_256);
    println!("root: {:?}; res: {:?}", root, res);

    let (root, proof) = mt.as_mut().unwrap().generate_proof(3);
    println!("root: {:?}; proof: {:?}", root, proof);
    let res = validate_proof(&root, &b"banana"[..], proof, sp_core::blake2_256);
    println!("root: {:?}; res: {:?}", root, res);

    let (root, proof) = mt.as_mut().unwrap().generate_proof(3);
    println!("root: {:?}; proof: {:?}", root, proof);
    let res = validate_proof(&root, &b"kotleta"[..], proof, sp_core::blake2_256);

    println!("root: {:?}; res: {:?}", root, res);
}
