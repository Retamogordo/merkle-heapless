#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };
    
    use merkle_heapless::{mmr_macro, HashT, HeaplessTree, BasicTreeTrait, ProofValidator};

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
    fn fail_4layers_std_hash_bad_word() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let mut mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
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
    fn mmr() {
        mmr_macro::mmr!(BranchFactor = 2, Peaks = 7);
    }
}

fn main() {
//    mmr_macro::mmr!(BranchFactor = 2, Peaks = 7);

//    let tree = HeaplessTree<2, 3, StdHash>::try_from
}