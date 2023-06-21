
#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::{MerkleTreeBinary};
    use crate::multi_branch::{HashT, MerkleTree};

        //use crate::merkle::{MerkleTree, validate_proof, ConcatHashes};


    #[derive(Debug)]
    struct Blake2_256Hash;

    impl HashT for Blake2_256Hash {
        type Output = [u8; 32];

        fn hash(input: &[u8]) -> Self::Output {
            sp_core::blake2_256(input)
        }
    }


    #[derive(Debug)]
    struct StdHash;

    impl HashT for StdHash {
//        impl ConcatHashesMulti<8> for StdHash {
        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }

    #[derive(Debug)]
    struct IdentityHash;
    impl HashT for IdentityHash {
        type Output = [u8; 1];

        fn hash(input: &[u8]) -> Self::Output {
            [if input.len() > 0 {input[0]} else {0}; 1]
        }
    }

    #[test]
    fn fail_4layers_std_hash_bad_word() {
        const LAYERS: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "kiwi";
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );

        assert!(!res);
    }

    #[test]
    #[should_panic]
    fn fail_binary_4layers_std_hash_bad_index() {
        const LAYERS: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 8;
        let (_root, _proof) = mt.as_mut().unwrap().generate_proof(word_index);
    }

    #[test]
    fn validate_default_padding_word_4layers_std_hash() {
        let mut mt = MerkleTree::<4, 8, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word: &str = Default::default();
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );

        assert!(res);
    }

    #[test]
    fn fail_creating_merkle_tree_too_few_layers_for_input() {
        let mt = MerkleTree::<2, 3, Blake2_256Hash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta", b"apple", b"banana", b"kiwi", b"kotleta",
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn minimal_tree_size() {
        let mut mt = MerkleTree::<2, 1, Blake2_256Hash>::try_from(&[
            b"apple",
        ]);

        let word_index = 0;

        mt.as_mut().unwrap().insert(word_index, b"ciruela");

        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    #[test]
    fn illegal_branch_factor() {
        let mut mt = MerkleTree::<3, 1, Blake2_256Hash>::try_from(&[
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn insert_replace_binary() {
        const LAYERS: usize = 4;
        const BRANCH_FACTOR: usize = 2;
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 2;

        mt.as_mut().unwrap().insert(word_index, b"ciruela");

        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    #[test]
    fn insert_append_binary() {
        const LAYERS: usize = 4;
        const BRANCH_FACTOR: usize = 2;
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 6;

        mt.as_mut().unwrap().insert(word_index, b"ciruela");

        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    #[test]
    fn insert_replace_branch_factor_8() {
        const LAYERS: usize = 4;
        const BRANCH_FACTOR: usize = 8;
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 2;

        mt.as_mut().unwrap().insert(word_index, b"ciruela");

        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    #[test]
    fn insert_append_branch_factor_8() {
        const LAYERS: usize = 4;
        const BRANCH_FACTOR: usize = 8;
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 32;

        mt.as_mut().unwrap().insert(word_index, b"ciruela");

        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(&root, word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    //     #[test]
//     #[should_panic]
//     fn fail_insertion_out_of_bound() {
//         let mut mt = MerkleTree::<4, 8, StdHash>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 8;

//         mt.as_mut().unwrap().insert(word_index, b"ciruela");
//     }

    #[test]
    fn validate_binary_5layers_default() {
        const LAYERS: usize = 5;
        const BRANCH_FACTOR: usize = 2;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let test_words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        for (i, w) in test_words.iter().enumerate() {
            let (root, proof) = mt.as_mut().unwrap().generate_proof(i);
//            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(&root, w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn validate_branch_factor4_3layers_default() {
        const LAYERS: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let test_words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

//        println!("{:?}", mt.as_ref().unwrap());

        for (i, w) in test_words.iter().enumerate() {
            let (root, proof) = mt.as_mut().unwrap().generate_proof(i);
//            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(&root, w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn validate_branch_factor4_3layers_default_with_scrambling() {
        const LAYERS: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple",
        ];
        let test_words: &[&str] = &[
            "apple",
        ];
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from_with_scrambling(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
            &[
                &[0], &[1], &[1, 1], &[2], &[1, 2], &[2, 1], &[2, 2], &[0, 1, 1],
                &[1, 0, 1], &[1, 1, 1], &[1, 0, 2], &[2, 0, 1], &[2, 0, 2], &[2, 1, 2], &[2, 2, 1], &[2, 2, 2],
            ]
        );

        for (i, w) in test_words.iter().enumerate() {
            let (root, proof) = mt.as_mut().unwrap().generate_proof(i);
            let res = proof.validate(&root, w.as_bytes());
            assert!(res);
        }
        println!("{:?}", mt.unwrap());
    }

    #[test]
    fn clone_tree() {
        const LAYERS: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        let cloned = mt.as_ref().unwrap().clone();
        assert_eq!(mt.unwrap(), cloned);
    }

    #[test]
    fn cloned_modified() {
        const LAYERS: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        let mut cloned = mt.as_ref().unwrap().clone();

        cloned.insert(2, b"ciruela");

        assert_ne!(mt.unwrap(), cloned);
    }

    #[test]
    fn print_tree() {
        const LAYERS: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mut mt = MerkleTree::<BRANCH_FACTOR, LAYERS, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );
        println!("{:?}", mt.unwrap());
    }
    
}
