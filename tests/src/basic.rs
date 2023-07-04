#[cfg(test)]
mod basic {
    use merkle_heapless::augmentable::DefaultAugmentable;
    use merkle_heapless::compactable::DefaultCompactable;
    use merkle_heapless::traits::{AppendOnly, CanRemove, HashT, ProofValidator, StaticTreeTrait};
    use merkle_heapless::{StaticBinaryTree, StaticTree};

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
            [if input.len() > 0 { input[0] } else { 0 }; 1]
        }
    }

    #[test]
    fn validate_default_padding_word_4branches_std_hash() {
        let mut mt = StaticTree::<4, 6, Blake2_256Hash>::try_from(&[
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
    fn fail_4layers_std_hash_bad_word() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, Blake2_256Hash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "kiwi";
        let res = proof.validate(word.as_bytes());
        assert!(!res);
    }

    #[test]
    fn fail_creating_merkle_tree_too_few_layers_for_input() {
        let mt = StaticBinaryTree::<2, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta", b"apple", b"banana", b"kiwi", b"kotleta",
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn minimal_tree_size() {
        let mut mt = StaticBinaryTree::<0, StdHash>::try_from(&[b"apple"]);
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
    fn insert_replace_binary() {
        const HEIGHT: usize = 4;
        let mut mt = StaticBinaryTree::<HEIGHT, StdHash>::try_from(&[
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
        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
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
        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
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
        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
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
        let mut mt =
            StaticTree::<4, 1, StdHash>::try_from(&[b"apple", b"banana", b"kiwi", b"kotleta"]);
        let word_index = 8;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");
    }

    #[test]
    fn validate_binary_5layers_default() {
        const HEIGHT: usize = 5;
        const BRANCH_FACTOR: usize = 2;
        let words: &[&str] = &[
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
            "cherry",
        ];
        let test_words: &[&str] = &[
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
            "cherry",
        ];
        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        );

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.as_mut().unwrap().generate_proof(i);
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
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
            "cherry",
        ];
        let test_words: &[&str] = &[
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
            "cherry",
        ];
        let mut mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        );

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.as_mut().unwrap().generate_proof(i);
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
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
            "cherry",
        ];
        let mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        );

        let cloned = mt.as_ref().unwrap().clone();
        assert_eq!(mt.unwrap(), cloned);
    }

    #[test]
    fn cloned_modified() {
        const HEIGHT: usize = 3;
        const BRANCH_FACTOR: usize = 4;
        let words: &[&str] = &[
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
            "cherry",
        ];
        let mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        );

        let mut cloned = mt.as_ref().unwrap().clone();

        cloned.replace(2, b"ciruela");

        assert_ne!(mt.unwrap(), cloned);
    }

    #[test]
    fn try_reduce() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;
        let cmt = DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ])
        .unwrap();

        cmt.try_reduce().unwrap();
    }

    #[test]
    fn too_big_to_reduce() {
        const HEIGHT: usize = 3;
        const BRANCH_FACTOR: usize = 2;
        let words: &[&str] = &[
            "apple",
            "apricot",
            "asai",
            "avocado",
            "banana",
            "blueberry",
            "blackberry",
            "blackcurrant",
        ];
        let cmt = DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        assert!(cmt.try_reduce().is_err());
    }

    #[test]
    fn try_reduce_small() {
        const HEIGHT: usize = 1;
        const BRANCH_FACTOR: usize = 2;
        let cmt =
            DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[b"apple"]).unwrap();

        cmt.try_reduce().unwrap();
    }

    #[test]
    fn compact_small() {
        const HEIGHT: usize = 1;
        const BRANCH_FACTOR: usize = 2;
        let mut cmt =
            DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(&[b"apple"]).unwrap();

        cmt.try_remove(0).unwrap();
        cmt.compact();
        let mut cmt = cmt.try_reduce().unwrap();

        cmt.replace(0, &[]);
        let proof = cmt.generate_proof(0);
        let res = proof.validate(&[]);

        assert!(res);
    }

    #[test]
    fn remove_and_compact() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];
        let test_words: &[&str] = &["apple", "apricot", "kiwi", "kotleta"];

        let mut cmt = DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt.try_remove(2).unwrap();
        cmt.compact();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            //            println!("testing -> {w}, proof: {:?}", proof);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn minimal_compactable() {
        const HEIGHT: usize = 0;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &["apple"];

        let mut cmt = DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt.try_remove(0).unwrap();
        assert_eq!(cmt.num_of_leaves(), 0);
        cmt.compact();
        // cmt.try_reduce() will not compile
    }

    #[test]
    fn remove_replace_and_compact() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];
        let test_words: &[&str] = &["cherry", "kiwi", "kotleta", "ciruela"];

        let mut cmt = DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt.try_remove(2).unwrap(); // remove banana
        cmt.replace(2, b"cherry");
        cmt.try_remove(0).unwrap(); // remove apple
        cmt.try_remove(1).unwrap(); // remove apricot
        cmt.replace(7, b"ciruela");

        cmt.compact();
        let mut reduced = cmt.try_reduce().unwrap().try_reduce().unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = reduced.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn augment_tree_from_leaves() {
        const HEIGHT: usize = 4;
        const BRANCH_FACTOR: usize = 2;

        let words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];
        let test_words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];

        let mt = StaticTree::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        const NEW_HEIGHT: usize = 5;
        let mut mt =
            StaticTree::<BRANCH_FACTOR, NEW_HEIGHT, StdHash>::try_from_leaves(&mt.leaves())
                .unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn augment() {
        const BRANCH_FACTOR: usize = 4;
        const HEIGHT: usize = 3;
        let words1: &[&str] = &["apple", "apricot", "banana", "cherry"];
        let cmt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &["apple", "apricot", "banana", "cherry"];

        let mut cmt = cmt1.augment();

        assert_eq!(cmt.num_of_leaves(), words1.len());
        assert_eq!(cmt.height(), HEIGHT + 1);

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn merge_2trees_same_heights() {
        const BRANCH_FACTOR: usize = 4;
        const HEIGHT_1: usize = 3;
        const HEIGHT_2: usize = 3;
        let words1: &[&str] = &["apple", "apricot", "banana", "cherry"];
        let cmt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["kiwi", "kotleta"];
        let cmt2 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &["apple", "apricot", "banana", "cherry", "kiwi", "kotleta"];

        let mut cmt = cmt1.augment_and_merge(cmt2);
        assert_eq!(cmt.height(), HEIGHT_1 + 1);

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn merge_2trees_different_heights() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["kiwi", "kotleta"];
        let cmt2 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple",
            "apricot",
            "banana",
            "cherry",
            "blueberry",
            "kiwi",
            "kotleta",
        ];

        cmt1.try_merge(cmt2).unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt1.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn merge_2trees_different_heights_empty() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &[];
        let cmt2 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];

        cmt1.try_merge(cmt2).unwrap();

        assert_eq!(cmt1.num_of_leaves(), words1.len());

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt1.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn fail_merge_2trees_different_heights() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 3;
        const HEIGHT_2: usize = 2;

        let words1: &[&str] = &[
            "apple",
            "apricot",
            "banana",
            "cherry",
            "blueberry",
            "kiwi",
            "kotleta",
        ];
        let mut cmt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["lemon", "lime"];
        let cmt2 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        assert!(cmt1.try_merge(cmt2).is_err());
    }

    #[test]
    fn merge_2trees_different_heights_after_removal() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["kiwi", "kompot", "kotleta", "sardina"];
        let mut cmt2 = DefaultCompactable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple",
            "apricot",
            "cherry",
            "blueberry",
            "kiwi",
            "kotleta",
            "sardina",
        ];

        cmt1.try_remove(2).unwrap();
        cmt1.compact();
        cmt2.try_remove(1).unwrap();
        cmt2.compact();

        let mut cmt =
            DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_1, StdHash>::try_from_leaves(&cmt1.leaves())
                .unwrap();
        cmt.try_merge(
            DefaultAugmentable::<BRANCH_FACTOR, HEIGHT_2, StdHash>::try_from_leaves(&cmt2.leaves())
                .unwrap(),
        )
        .unwrap();

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

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &[
            "apple",
            "apricot",
            "banana",
            "cherry",
            "blueberry",
            "kiwi",
            "kotleta",
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
        const HEIGHT: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt.try_append(b"kiwi").unwrap();
        cmt.try_append(b"kotleta").unwrap();
        cmt.try_append(b"blueberry").unwrap();
        assert!(cmt.try_append(b"blackberry").is_err());
    }

    #[test]
    fn create_compactable_from_augmentable() {
        const BRANCH_FACTOR: usize = 2;
        const HEIGHT: usize = 3;

        let words1: &[&str] = &[
            "apple",
            "apricot",
            "banana",
            "cherry",
            "blueberry",
            "kiwi",
            "kotleta",
        ];
        let cmt1 = DefaultAugmentable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let cmt2 =
            DefaultCompactable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from_leaves(&cmt1.leaves())
                .unwrap();
        assert_eq!(cmt1.num_of_leaves(), cmt2.num_of_leaves());

        let mut cmt3 =
            DefaultAugmentable::<BRANCH_FACTOR, HEIGHT, StdHash>::try_from_leaves(&cmt2.leaves())
                .unwrap();

        assert_eq!(cmt2.num_of_leaves(), cmt3.num_of_leaves());

        for (i, w) in words1.iter().enumerate() {
            let proof = cmt3.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }
}
