#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

//mod basic;

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    
    use merkle_heapless::{mmr_macro, StaticTree};
    use merkle_heapless::traits::{HashT, ProofValidator, StaticTreeTrait, AppendOnly};

    #[derive(Debug)]
    pub struct StdHash;
    
    impl HashT for StdHash {
        type Output = [u8; 8];
    
        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }

    #[test]
    fn fail_4layers_std_hash_bad_word() {
        const HEIGHT: usize = 3;
        const BRANCH_FACTOR: usize = 4;

        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "kiwi";
        let res = proof.validate(word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );

        assert!(!res);
    }

    #[test]
    fn mmr_binary() {
        mmr_macro::mmr!(Type = FooMMR, BranchFactor = 2, Peaks = 7, Hash = StdHash);
//        let mut mmr = FooMMR::from(FooMMRPeak::Peak0(Default::default())).unwrap();
        let mut mmr = FooMMR::default();
        // peak leaf numbers: [0, 0, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].height(), 0);

        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1, 0, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);
        
        mmr.try_append(b"banana").unwrap();
        assert_eq!(mmr.peaks()[0].height(), 1);
        // peak leaf numbers: [2, 0, 0, 0, 0] because 1, 1 is merged -> 2, 0
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].height(), 0);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"banana");
        assert!(res);
    
        mmr.try_append(b"cherry").unwrap();
        // peak leaf numbers: [2, 1, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(2);
        let res = proof.validate(b"cherry");
        assert!(res);
    
        mmr.try_append(b"kiwi").unwrap();
        // peak leaf numbers: [4, 0, 0, 0, 0] because 2, 1, 1 is merged -> 2, 2, 0 -> 4, 0, 0
        assert_eq!(mmr.peaks()[0].height(), 2);
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
        
        mmr.try_append(b"peach").unwrap();
        // peak leaf numbers: [8, 1, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);

        mmr.try_append(b"pear").unwrap();
        // peak leaf numbers: [8, 2, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);

        mmr.try_append(b"potato").unwrap();
        // peak leaf numbers: [8, 2, 1, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[2].num_of_leaves(), 1);
    
        mmr.try_append(b"strawberry").unwrap();
        // peak leaf numbers: [8, 4, 0, 0, 0]
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 8);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 4);
    
    }

    #[test]
    fn mmr_binary_1_peak() {
        mmr_macro::mmr!(BranchFactor = 2, Peaks = 1, Hash = StdHash);

        let mut mmr = MerkleMountainRange::default();

        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1]
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);

        mmr.try_append(b"banana").unwrap();
        // peak leaf numbers: [1]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"banana");
        assert!(res);

        assert!(mmr.try_append(b"cherry").is_err());
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
    }

    #[test]
    fn mmr_binary_2_peaks() {
        mmr_macro::mmr!(BranchFactor = 2, Peaks = 2, Hash = StdHash);

        let mut mmr = MerkleMountainRange::default();

        mmr.try_append(b"apple").unwrap();
        // peak leaf numbers: [1, 0]
        assert_eq!(mmr.peaks()[0].height(), 0);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(0);
        let res = proof.validate(b"apple");
        assert!(res);

        mmr.try_append(b"ananas").unwrap();
        // peak leaf numbers: [2, 0]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(1);
        let res = proof.validate(b"ananas");
        assert!(res);

        mmr.try_append(b"banana").unwrap();
        // peak leaf numbers: [2, 1]
        assert_eq!(mmr.peaks()[0].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 2);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(2);
        let res = proof.validate(b"banana");
        assert!(res);

        mmr.try_append(b"berry").unwrap();
        // peak leaf numbers: [4, 0]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
        let proof = mmr.generate_proof(3);
        let res = proof.validate(b"berry");
        assert!(res);

        mmr.try_append(b"cherry").unwrap();
        // peak leaf numbers: [4, 1]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 1);
        let proof = mmr.generate_proof(4);
        let res = proof.validate(b"cherry");
        assert!(res);

        mmr.try_append(b"kiwi").unwrap();
        // peak leaf numbers: [4, 2]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[1].height(), 1);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 2);
        let proof = mmr.generate_proof(5);
        let res = proof.validate(b"kiwi");
        assert!(res);

        mmr.try_append(b"lemon").unwrap();
        // peak leaf numbers: [4, 3]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[1].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 3);
        let proof = mmr.generate_proof(6);
        let res = proof.validate(b"lemon");
        assert!(res);

        mmr.try_append(b"lime").unwrap();
        // peak leaf numbers: [4, 3]
        assert_eq!(mmr.peaks()[0].height(), 2);
        assert_eq!(mmr.peaks()[1].height(), 2);
        assert_eq!(mmr.peaks()[0].num_of_leaves(), 4);
        assert_eq!(mmr.peaks()[1].num_of_leaves(), 4);
        let proof = mmr.generate_proof(7);
        let res = proof.validate(b"lime");
        assert!(res);

        assert!(mmr.try_append(b"mango").is_err());

    }
}

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use merkle_heapless::traits::HashT;

#[derive(Debug)]
pub struct StdHash;

impl HashT for StdHash {
    type Output = [u8; 8];

    fn hash(input: &[u8]) -> Self::Output {
        let mut s = DefaultHasher::new();
        input.hash(&mut s);
        s.finish().to_ne_bytes()
    }
}

#[derive(Debug)]
struct Blake2_256Hash;

impl HashT for Blake2_256Hash {
    type Output = [u8; 32];

    fn hash(input: &[u8]) -> Self::Output {
        sp_core::blake2_256(input)
    }
}

fn main() {
    use merkle_heapless::{StaticBinaryTree, StaticTree};
    use merkle_heapless::traits::{StaticTreeTrait, ProofValidator};

    const MAX_HEIGHT: usize = 3;

    let mut tree = StaticBinaryTree::<MAX_HEIGHT, StdHash>::try_from(
        &[b"apple", b"banana"]
    ).unwrap();

    let proof = tree.generate_proof(0);
    assert!(proof.validate(b"apple"));

    tree.replace(5, b"cherry");
    let proof = tree.generate_proof(5);
    assert!(proof.validate(b"cherry"));

    tree.replace(1, &[]);
    let proof = tree.generate_proof(1);
    assert!(!proof.validate(b"banana"));
    let proof = tree.generate_proof(1);
    assert!(proof.validate(&[]));

    const BRANCH_FACTOR: usize = 4;
    let mut tree = StaticTree::<BRANCH_FACTOR, MAX_HEIGHT, StdHash>::try_from(
        &[b"apple", b"banana"]
    ).unwrap();


    use merkle_heapless::{mmr_macro};

    mmr_macro::mmr!(BranchFactor = 2, Peaks = 7, Hash = StdHash);

    let mut mmr = MerkleMountainRange::default();
}