
#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::{validate_proof, ConcatHashes, MerkleTree};
    use crate::multi_branch::{ConcatHashesMulti, MerkleTreeMulti, validate_proof_multi};

    macro_rules! layer_size {
        ($branch_factor:expr, $layers:expr, $layer_index:expr) => {
            1 << ($branch_factor.trailing_zeros() * ($layers - $layer_index - 1))
        };
    }
        //use crate::merkle::{MerkleTree, validate_proof, ConcatHashes};

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
    struct Blake2_256ConcatHashes;

    impl ConcatHashes<32> for Blake2_256ConcatHashes {
        type Output = [u8; 32];

        fn hash(input: &[u8]) -> Self::Output {
            sp_core::blake2_256(input)
        }
    }
    impl ConcatHashesMulti<32> for Blake2_256ConcatHashes {
        type Output = [u8; 32];

        fn hash(input: &[u8]) -> Self::Output {
            sp_core::blake2_256(input)
        }
    }

    #[derive(Debug)]
    struct DefaultConcatHashes;

    impl ConcatHashes<8> for DefaultConcatHashes {
        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }
    impl ConcatHashesMulti<8> for DefaultConcatHashes {
        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }

    #[derive(Debug)]
    struct IdentityHash;
    impl ConcatHashesMulti<1> for IdentityHash {
        type Output = [u8; 1];

        fn hash(input: &[u8]) -> Self::Output {
            [if input.len() > 0 {input[0]} else {0}; 1]
        }
    }

//     #[test]
//     fn validate_3layers_blake2_256() {
//         const LAYERS: usize = 3;
//         let mut mt = MerkleTree::<LAYERS, 32, Blake2_256ConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);

//         let word_index = 2;
//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word = "kiwi";
//         let res =
//             validate_proof::<LAYERS, 32, Blake2_256ConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );
//         assert!(res);
//     }

//     #[test]
//     fn validate_4layers_std_hash() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 2;
//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word = "kiwi";
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );

//         assert!(res);
//     }

//     #[test]
//     fn fail_4layers_std_hash_bad_word() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 7;
//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word = "kiwi";
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );

//         assert!(!res);
//     }

//     #[test]
//     #[should_panic]
//     fn fail_4layers_std_hash_bad_index() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 8;
//         let (_root, _proof) = mt.as_mut().unwrap().generate_proof(word_index);
//     }

//     #[test]
//     fn validate_default_padding_word_4layers_std_hash() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 7;
//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word: &str = Default::default();
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );

//         assert!(res);
//     }

//     #[test]
//     fn fail_creating_merkle_tree_too_few_layers_for_input() {
//         let mt = MerkleTree::<4, 32, Blake2_256ConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta", b"apple", b"banana", b"kiwi", b"kotleta",
//             b"apple",
//         ]);

//         assert!(mt.is_err());
//     }

//     #[test]
//     fn total_size_and_layers() {
//         const LAYERS: usize = 7;
//         let mt = MerkleTree::<LAYERS, 32, Blake2_256ConcatHashes>::try_from(&[b"apple"]);

//         assert_eq!(mt.as_ref().unwrap().total_size(), (1 << LAYERS) - 1);
//         assert_eq!(mt.unwrap().total_layers(), LAYERS);
//     }

//     #[test]
//     fn insert_replace() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 2;

//         mt.as_mut().unwrap().insert(word_index, b"ciruela");

//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word = "ciruela";
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );
//         assert!(res);
//     }

//     #[test]
//     fn insert_append() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 6;

//         mt.as_mut().unwrap().insert(word_index, b"ciruela");

//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word = "ciruela";
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );
//         assert!(res);
//     }

//     #[test]
//     #[should_panic]
//     fn fail_insertion_out_of_bound() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 8;

//         mt.as_mut().unwrap().insert(word_index, b"ciruela");
//     }

//     #[test]
//     fn remove() {
//         let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);
//         let word_index = 0;

//         mt.as_mut().unwrap().remove(word_index);

//         let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
//         let word = "apple";
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );
//         assert!(!res);

//         let word = ""; // empty string corresponds to empty element in tree
//         let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
//         println!(
//             "word: {:?} {} validated, proof was generated for word at index {}",
//             word,
//             if res { "" } else { "NOT" },
//             word_index
//         );
//         assert!(res);
//     }

//     #[test]
//     fn basic_multi_branch() {
//         let mut mt = MerkleTreeMulti::<2, 4, 8, DefaultConcatHashes>::try_from(&[
//             b"apple", b"banana", b"kiwi", b"kotleta",
//         ]);

//         println!("{:?}", mt);
//     }

//    #[test]
//     fn layer_size() {
//         // layer_size!(branch_factor, layers, layer_index)
//         assert_eq!(layer_size!(4_u8, 3, 0), 16);
//         assert_eq!(layer_size!(4_u8, 3, 2), 1);
//         assert_eq!(layer_size!(4_u8, 4, 0), 64);
//         assert_eq!(layer_size!(4_u8, 4, 1), 16);
//         assert_eq!(layer_size!(4_u8, 4, 3), 1);

//         assert_eq!(layer_size!(2_u8, 4, 0), 8);
//         assert_eq!(layer_size!(2_u8, 4, 3), 1);
//         assert_eq!(layer_size!(2_u8, 4, 2), 2);
//     }

    // #[test]
    // fn validate_branch_factor2_3layers_identity() {
    //     const LAYERS: usize = 3;
    //     const BRANCH_FACTOR: usize = 2;
    //     let mut mt = MerkleTreeMulti::<BRANCH_FACTOR, LAYERS, 1, IdentityHash>::try_from(&[
    //         b"a", b"b", b"c",
    //     ]);

    //     println!("{:?}", mt.as_ref().unwrap());


    //     let word_index = 2;
    //     let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
    //     let word = "c";
    //     let res =
    //         validate_proof_multi::<BRANCH_FACTOR, LAYERS, 1, IdentityHash>(&root, word.as_bytes(), proof);
    //     println!(
    //         "word: {:?} {} validated, proof was generated for word at index {}",
    //         word,
    //         if res { "" } else { "NOT" },
    //         word_index
    //     );
    //     println!(
    //         "proof: {:?}",proof);
    //     println!(
    //         "root: {:?}",root);
    //         assert!(res);
    // }

    // #[test]
    // fn validate_binary_3layers_default() {
    //     const LAYERS: usize = 3;
    //     const BRANCH_FACTOR: usize = 2;

    //     let words: &[&str] = &["apple", "banana", "kiwi", "kotleta"];
    //     let mut mt = MerkleTreeMulti::<BRANCH_FACTOR, LAYERS, 8, DefaultConcatHashes>::try_from(
    //         &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
    //     );

    //     println!("{:?}", mt.as_ref().unwrap());

    //     for (i, w) in words.iter().enumerate() {
    //         let (root, proof) = mt.as_mut().unwrap().generate_proof(i);
    //         println!("testing -> {w}");
    //         let res =
    //             validate_proof_multi::<BRANCH_FACTOR, LAYERS, 8, DefaultConcatHashes>(&root, w.as_bytes(), proof); 
    //         assert!(res);
    //     }
    // }

    #[test]
    fn validate_branch_factor4_3layers_default() {
        const LAYERS: usize = 3;
        const BRANCH_FACTOR: usize = 4;

        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
        ];
        let mut mt = MerkleTreeMulti::<BRANCH_FACTOR, LAYERS, 8, DefaultConcatHashes>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        println!("{:?}", mt.as_ref().unwrap());
        let test_words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
        ];

        for (i, w) in test_words.iter().enumerate() {
            let (root, proof) = mt.as_mut().unwrap().generate_proof(i);
            println!("testing -> {w}, proof: {:?}", proof);
            let res =
                validate_proof_multi::<BRANCH_FACTOR, LAYERS, 8, DefaultConcatHashes>(&root, w.as_bytes(), proof); 
            assert!(res);
        }
    }
}
