
#[cfg(test)]
mod tests {
    use crate::{HashT, HeaplessTreeT, HeaplessTree, HeaplessBinaryTree, ProofValidator, Proof};
    use crate::compactable::{DefaultCompactable};
    use crate::peak::{MerklePeak, MerkleMR};
//    use crate::foo::{Foo};

    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };


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
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let mut mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
//            let mut mt = HeaplessTree::<BRANCH_FACTOR,HEIGHT, StdHash, Proof<BRANCH_FACTOR, HEIGHT, StdHash>>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let mut proof = mt.as_mut().unwrap().generate_proof(word_index);
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
    #[should_panic]
    fn fail_binary_4layers_std_hash_bad_index() {
        const HEIGHT: usize = 4;

        let mut mt = HeaplessBinaryTree::<HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 8;
        let _proof = mt.as_mut().unwrap().generate_proof(word_index);
    }

    #[test]
    fn validate_default_padding_word_4layers_std_hash() {
        let mut mt = HeaplessTree::<4, 8, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word: &str = Default::default();
        let res = proof.validate(word.as_bytes());
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
        let mt = HeaplessBinaryTree::<3, Blake2_256Hash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta", b"apple", b"banana", b"kiwi", b"kotleta",
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn minimal_tree_size() {
        let mut mt = HeaplessBinaryTree::<1, Blake2_256Hash>::try_from(&[
            b"apple",
        ]);

        let word_index = 0;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");

        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(word.as_bytes());
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
        let mt = HeaplessTree::<3, 1, Blake2_256Hash>::try_from(&[
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn insert_replace_binary() {
        const HEIGHT: usize = 4;
        let mut mt = HeaplessBinaryTree::<HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 2;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");

        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(word.as_bytes());
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
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;
        let mut mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 6;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");

        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(word.as_bytes());
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
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 8;
        let mut mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 2;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");

        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(word.as_bytes());
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
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 8;
        let mut mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 32;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");

        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "ciruela";
        let res = proof.validate(word.as_bytes());
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    #[test]
    #[should_panic]
    fn fail_insertion_out_of_bound() {
        let mut mt = HeaplessTree::<4, 8, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 8;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");
    }

    #[test]
    fn validate_binary_5layers_default() {
        const HEIGHT: usize = 5;
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
        let mut mt = HeaplessTree::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.as_mut().unwrap().generate_proof(i);
//            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn validate_branch_factor4_3layers_default() {
        const HEIGHT: usize = 3;
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
        let mut mt = HeaplessTree::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

//        println!("{:?}", mt.as_ref().unwrap());

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.as_mut().unwrap().generate_proof(i);
//            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn clone_tree() {
        const HEIGHT: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        let cloned = mt.as_ref().unwrap().clone();
        assert_eq!(mt.unwrap(), cloned);
    }

    #[test]
    fn cloned_modified() {
        const HEIGHT: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mt = HeaplessTree::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );

        let mut cloned = mt.as_ref().unwrap().clone();

        cloned.replace(2, b"ciruela");

        assert_ne!(mt.unwrap(), cloned);
    }

    #[test]
    fn print_tree() {
        const HEIGHT: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
            "cherry",
        ];
        let mt = HeaplessTree::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        );
        println!("{:?}", mt.unwrap());
    }
    
    // #[test]
    // fn foo() {
    //     const HEIGHT: usize = 3;
    //     let mt1 = HeaplessBinaryTree::<HEIGHT, StdHash>::try_from(
    //         &[b"apple", b"apricot", b"asai", b"avocado"]
    //     ).unwrap();

    //     let mt2 = HeaplessBinaryTree::<HEIGHT, StdHash>::try_from(
    //         &[b"banana", b"blueberry"]
    //     ).unwrap();
    //     let mut foo = Foo::from_base_trees([mt1, mt2].try_into().unwrap());
        
    //     let proof = foo.generate_proof(0);
    //     let res = proof.validate(b"apple");        
    //     assert!(res);

    //     let proof = foo.generate_proof(1);
    //     let res = proof.validate(b"apricot");        
    //     assert!(res);

    //     let proof = foo.generate_proof(2);
    //     let res = proof.validate(b"asai");        
    //     assert!(res);

    //     let proof = foo.generate_proof(3);
    //     let res = proof.validate(b"avocado");        
    //     assert!(res);

    //     let proof = foo.generate_proof(4);
    //     let res = proof.validate(b"banana");        
    //     assert!(res);

    //     let proof = foo.generate_proof(5);
    //     let res = proof.validate(b"blueberry");        
    //     assert!(res);
    // }

//     // #[test]
//     // fn foo1() {
//     //     const HEIGHT: usize = 3;

//     //     let mut foo = Foo::<2, HEIGHT, StdHash>::try_from(
//     //         &[b"apple", b"apricot", b"asai", b"avocado", b"banana", b"blueberry"]
//     //     ).unwrap();
        
//     //     let proof = foo.generate_proof(0);
//     //     let res = proof.validate(b"apple");        
//     //     assert!(res);

//     //     let proof = foo.generate_proof(1);
//     //     let res = proof.validate(b"apricot");        
//     //     assert!(res);

//     //     let proof = foo.generate_proof(2);
//     //     let res = proof.validate(b"asai");        
//     //     assert!(res);

//     //     let proof = foo.generate_proof(3);
//     //     let res = proof.validate(b"avocado");        
//     //     assert!(res);

//     //     let proof = foo.generate_proof(4);
//     //     let res = proof.validate(b"banana");        
//     //     assert!(res);

//     //     let proof = foo.generate_proof(5);
//     //     let res = proof.validate(b"blueberry");        
//     //     assert!(res);
//     // }

    #[test]
    fn try_compact() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;
        let cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ])
        .unwrap();

        cmt.try_compact().unwrap();
    }

    #[test]
    fn too_big_to_compact() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;
        let words: &[&str] = &[
            "apple", "apricot", "asai", "avocado",
            "banana", "blueberry", "blackberry", "blackcurrant",
        ];
        let cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        assert!(cmt.try_compact().is_err());
    }

    #[test]
    fn compact_and_proof() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &[
            "apple", "apricot", "kiwi", "kotleta",
        ];

        let cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let mut cmt = cmt.try_compact();

        for (i, w) in words.iter().enumerate() {
            let proof = cmt.as_mut().unwrap().generate_proof(i);
//            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn remove_and_compact() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &[
            "apple", "apricot", "banana", "kiwi", "kotleta",
        ];
        let test_words: &[&str] = &[
            "apple", "apricot", "kiwi", "kotleta",
        ];

        let mut cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        cmt.remove(2);

        let mut cmt = cmt.try_compact();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.as_mut().unwrap().generate_proof(i);
//            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn remove_insert_and_compact() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &[
            "apple", "apricot", "banana", "kiwi", "kotleta",
        ];
        let test_words: &[&str] = &[
            "cherry", "kiwi", "kotleta", "ciruela",
        ];

        let mut cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        cmt.remove(2); // remove banana
        cmt.replace(2, b"cherry");
        cmt.remove(0); // remove apple
        cmt.remove(1); // remove apricot
        cmt.replace(7, b"ciruela");

        let mut cmt = cmt.try_compact();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.as_mut().unwrap().generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn augment_tree_from_leaves() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &[
            "apple", "apricot", "banana", "kiwi", "kotleta",
        ];
        let test_words: &[&str] = &[
            "apple", "apricot", "banana", "kiwi", "kotleta",
        ];

        let mt = HeaplessTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        const NEW_HEIGHT: usize = 5;
        let mut mt = HeaplessTree::<BRANCH_FACTOR, NEW_HEIGHT, StdHash>::try_from_leaves(
            &mt.leaves()
        )
        .unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn merge_2trees_same_heights() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 3;
        const HEIGHT_2: usize = 3;
        const MAX_PROOF_HEIGHT: usize = 7;
        let words1: &[&str] = &[
            "apple", "apricot", "banana", "cherry",
        ];
        let cmt1 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let words2: &[&str] = &[
            "kiwi", "kotleta",
        ];
        let cmt2 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "kiwi", "kotleta",
        ];

        let mut cmt = cmt1.try_merge(cmt2).unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
        assert_eq!(cmt.height(), HEIGHT_1 + 1);
    }

    #[test]
    fn merge_2trees_different_heights() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;
//        const MAX_PROOF_HEIGHT: usize = 5;

        let words1: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry"
        ];
        let cmt1 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let words2: &[&str] = &[
            "kiwi", "kotleta",
        ];
        let cmt2 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry", "kiwi", "kotleta",
        ];

        let mut cmt = cmt1.try_merge(cmt2).unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
        assert_eq!(cmt.height(), HEIGHT_1 + 1);
    }

    #[test]
    fn merge_2trees_different_heights_after_removal() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;
        const MAX_PROOF_HEIGHT: usize = 5;

        let words1: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry"
        ];
        let mut cmt1 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let words2: &[&str] = &[
            "kiwi", "kompot", "kotleta", "sardina"
        ];
        let mut cmt2 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple", "apricot", "cherry", "blueberry", "kiwi", "kotleta", "sardina"
        ];

        cmt1.remove(2);
        cmt2.remove(1);

        let mut cmt = cmt1.try_merge(cmt2).unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
        assert_eq!(cmt.height(), HEIGHT_1 + 1);
    }


    #[test]
    fn compact_and_append() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry"
        ];
        let cmt1 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let words2: &[&str] = &[
            "kiwi", "kotleta",
        ];
        let cmt2 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry", "kiwi", "kotleta",
        ];

        let mut cmt = cmt1.try_compact_and_append(cmt2).unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
        assert_eq!(cmt.height(), HEIGHT_1);
    }

    #[test]
    fn try_append() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT: usize = 4;

        let words1: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry"
        ];
        let mut cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry", "kiwi", "kotleta",
        ];

        cmt.try_append(b"kiwi").unwrap();
        cmt.try_append(b"kotleta").unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn fail_try_append_size_exceeded() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT: usize = 4;

        let words1: &[&str] = &[
            "apple", "apricot", "banana", "cherry", "blueberry"
        ];
        let mut cmt = DefaultCompactable::<BRANCH_FACTOR,HEIGHT, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
        )
        .unwrap();

        cmt.try_append(b"kiwi").unwrap();
        cmt.try_append(b"kotleta").unwrap();
        cmt.try_append(b"blueberry").unwrap();
        assert!(cmt.try_append(b"blackberry").is_err());
    }


//     #[test]
//     fn montain_try_merge() {
//         const BRANCH_FACTOR: usize = 2;
//         const PEAK_HEIGHT_1: usize = 3;
//         const PEAK_HEIGHT_2: usize = 3;

//         let words1: &[&str] = &[
//             "apple", "apricot", "banana",
//         ];

//         let cmt1 = DefaultCompactable::<BRANCH_FACTOR, PEAK_HEIGHT_1, 5, StdHash>::try_from(
//             &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
//         )
//         .unwrap();

//         let words2: &[&str] = &[
//             "cherry", "kiwi", 
//         ];

//         let cmt2 = DefaultCompactable::<BRANCH_FACTOR, PEAK_HEIGHT_2, 5, StdHash>::try_from(
//             &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>()
//         )
//         .unwrap();

//         let mut peak1 = MerklePeak::Second(cmt1);
//         let mut peak2 = MerklePeak::Second(cmt2);

//         if let Ok(mut new_peak) = peak1.try_merge(peak2) {
//             assert_eq!(new_peak.num_of_leaves(), 5);

// //            let proof = new_peak.generate_proof(3);
// //            let res = proof.validate(b"cherry");
// //            assert!(res);
//         } else {
//             panic!("could not merge");
//         }


//         // cmt.try_append(b"kiwi").unwrap();
//         // cmt.try_append(b"kotleta").unwrap();
//         // cmt.try_append(b"blueberry").unwrap();
//         // assert!(cmt.try_append(b"blackberry").is_err());
//     }

}
