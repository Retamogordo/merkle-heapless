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
    #[derive(Hash, Clone, Copy, Default, PartialEq, Debug)]
    pub struct Wrapped32([u8; 32]);
    impl From<u8> for Wrapped32 {
        fn from(n: u8) -> Self {
            let mut arr = [0u8; 32];
            arr[0] = n;
            Self(arr)
        }
    }

    impl HashT for Blake2_256Hash {
        type Output = Wrapped32;
//        type Output = [u8; 32];

        fn hash(input: &[u8]) -> Self::Output {
            Wrapped32(sp_core::blake2_256(input))
        }
    }

    #[derive(Debug)]
    struct StdHash;
    #[derive(Hash, Clone, Copy, Default, PartialEq, Debug)]
    pub struct Wrapped8([u8; 8]);
    impl From<u8> for Wrapped8 {
        fn from(n: u8) -> Self {
            let mut arr = [0u8; 8];
            arr[0] = n;
            Self(arr)
        }
    }

    impl HashT for StdHash {
        type Output = Wrapped8;
//        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            Wrapped8(s.finish().to_ne_bytes())
        }
    }

    #[derive(Debug)]
    struct IdentityHash;
    impl HashT for IdentityHash {
 //       type Output = [u8; 1];
        type Output = u8;

        fn hash(input: &[u8]) -> Self::Output {
//            [if input.len() > 0 { input[0] } else { 0 }; 1]
            if input.len() > 0 { input[0] } else { 0 }
        }
    }

    #[test]
    fn basic_1() {
        let mut mt = StaticTree::<2, 1, Blake2_256Hash, 100>::try_from::<&[u8]>(&[b"apple"]);
        let word_index = 0;
        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word: &str = "apple";
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
    fn fail_validate_default_padding_word_4branches() {
        let mut mt = StaticTree::<4, 6, Blake2_256Hash, 15>::try_from::<&[u8]>(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = &[0u8];
        let res = proof.validate(word);
        assert!(!res);
        
        let word_index = 7;
        let proof = mt.as_mut().unwrap().generate_proof(word_index);
        let word = &[];
        let res = proof.validate(word);
        assert!(!res);
    }

    #[test]
    fn fail_4layers_std_hash_bad_word() {
        const HEIGHT: usize = 4;
        const ARITY: usize = 2;

        let mut mt = StaticTree::<ARITY, HEIGHT, Blake2_256Hash, 100>::try_from::<&[u8]>(&[
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
        let mt = StaticBinaryTree::<2, StdHash, 100>::try_from::<&[u8]>(&[
            b"apple", b"banana", b"kiwi", b"kotleta", b"apple", b"banana", b"kiwi", b"kotleta",
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn insert_replace_binary() {
        const HEIGHT: usize = 4;
        const ARITY: usize = 2;
        let mut mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(&[
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
        const ARITY: usize = 8;
        let mut mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(&[
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
        let mut mt = StaticTree::<4, 1, StdHash, 100>::try_from::<&[u8]>(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 8;

        mt.as_mut().unwrap().replace(word_index, b"ciruela");
    }

    #[test]
    fn validate_binary_5layers_default() {
        const HEIGHT: usize = 5;
        const ARITY: usize = 2;
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
        let mut mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
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
        const ARITY: usize = 4;
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
        let mut mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
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
        const ARITY: usize = 4;
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
        let mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        );

        let cloned = mt.as_ref().unwrap().clone();
        assert_eq!(mt.unwrap(), cloned);
    }

    #[test]
    fn cloned_modified() {
        const HEIGHT: usize = 3;
        const ARITY: usize = 4;
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
        let mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        );

        let mut cloned = mt.as_ref().unwrap().clone();

        cloned.replace(2, b"ciruela");

        assert_ne!(mt.unwrap(), cloned);
    }

    #[test]
    fn try_reduce() {
        const HEIGHT: usize = 4;
        const ARITY: usize = 2;
        let cmt = DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ])
        .unwrap();

        cmt.try_reduce().unwrap();
    }

    #[test]
    fn too_big_to_reduce() {
        const HEIGHT: usize = 3;
        const ARITY: usize = 2;
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
        let cmt = DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        assert!(cmt.try_reduce().is_err());
    }

    #[test]
    fn compact_small() {
        const HEIGHT: usize = 1;
        const ARITY: usize = 2;
        let mut cmt =
            DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(&[b"apple"])
                .unwrap();

        println!("before removal: {:?}", cmt);
        cmt.remove(0);
        println!("before: {:?}", cmt);
        cmt.compact();
        println!("after: {:?}", cmt);

        // let proof = cmt.generate_proof(0);
        // let res = proof.validate(&[0u8]);

        //assert!(res);
    }

    #[test]
    fn remove_and_compact() {
        const HEIGHT: usize = 4;
        const ARITY: usize = 2;

        let words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];
        let test_words: &[&str] = &["apple", "apricot", "kiwi", "kotleta"];

        let mut cmt = DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt.remove(2);
        cmt.compact();

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn remove_replace_and_compact() {
        const HEIGHT: usize = 4;
        const ARITY: usize = 2;

        let words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];
        let test_words: &[&str] = &["cherry", "kiwi", "kotleta", "ciruela"];

        let mut cmt = DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt.remove(2); // remove banana
        cmt.replace(2, b"cherry");
        cmt.remove(0); // remove apple
        cmt.remove(1); // remove apricot
        cmt.replace(7, b"ciruela");

        cmt.compact();
        let reduced = cmt.try_reduce().unwrap().try_reduce().unwrap();

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
        const ARITY: usize = 2;

        let words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];
        let test_words: &[&str] = &["apple", "apricot", "banana", "kiwi", "kotleta"];

        let mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        const NEW_HEIGHT: usize = 5;
        let mt =
            StaticTree::<ARITY, NEW_HEIGHT, StdHash, 100>::try_from_leaves(&mt.leaves()).unwrap();

        for (i, w) in test_words.iter().enumerate() {
            let proof = mt.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn augment() {
        const ARITY: usize = 2;
        const HEIGHT: usize = 4;
        let words1: &[&str] = &["apple", "apricot", "banana", "cherry"];
        let cmt1 = DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &["apple", "apricot", "banana", "cherry"];

        let cmt = cmt1.augment();

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
        const ARITY: usize = 4;
        const HEIGHT_1: usize = 3;
        const HEIGHT_2: usize = 3;
        let words1: &[&str] = &["apple", "apricot", "banana", "cherry"];
        let cmt1 = DefaultAugmentable::<ARITY, HEIGHT_1, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        for (i, w) in words1.iter().enumerate() {
            let proof = cmt1.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }

        let words2: &[&str] = &["kiwi", "kotleta"];
        let cmt2 = DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, 100>::try_from::<&[u8]>(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let test_words: &[&str] = &["apple", "apricot", "banana", "cherry", "kiwi", "kotleta"];

        let cmt = cmt1.augment_and_merge(cmt2);
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
        const ARITY: usize = 4;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultAugmentable::<ARITY, HEIGHT_1, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["kiwi", "kotleta"];
        let cmt2 = DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, 100>::try_from::<&[u8]>(
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
    fn merge_2trees_different_heights_1() {
        const ARITY: usize = 2;
        const HEIGHT: usize = 3;

        let input = (0u8..8).map(|i| vec![i]).collect::<Vec<_>>();

        let amt1 = DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &input.iter().map(|d| d.as_ref()).collect::<Vec<_>>(),
        )
        .unwrap();

        for (i, d) in input.iter().enumerate() {
            let proof = amt1.generate_proof(i);
            let res = proof.validate(d);
            assert!(res);
        }

        let input_2 = (100u8..108).map(|i| vec![i]).collect::<Vec<_>>();
        let amt2 = DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &input_2.iter().map(|d| d.as_ref()).collect::<Vec<_>>(),
        )
        .unwrap();

        for (i, d) in input_2.iter().enumerate() {
            let proof = amt2.generate_proof(i);
            let res = proof.validate(d);
            assert!(res);
        }

        let num_of_leaves_before_merge_1 = amt1.num_of_leaves();
        let amt1 = amt1.augment_and_merge(amt2);

        for (i, d) in input.iter().enumerate() {
            let proof = amt1.generate_proof(i);
            let res = proof.validate(d);
            assert!(res);
        }

        for (i, d) in input_2.iter().enumerate() {
            let proof = amt1.generate_proof(i + num_of_leaves_before_merge_1);
            println!("testing -> {:?}", &d);
            let res = proof.validate(d);
            assert!(res);
        }
    }

    #[test]
    fn merge_2trees_different_heights_branch_factor_4() {
        const ARITY: usize = 4;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultAugmentable::<ARITY, HEIGHT_1, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["kiwi", "kotleta"];
        let cmt2 = DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, 100>::try_from::<&[u8]>(
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
        const ARITY: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultAugmentable::<ARITY, HEIGHT_1, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &[];
        let cmt2 = DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, 100>::try_from::<&[u8]>(
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
        const ARITY: usize = 2;
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
        let mut cmt1 = DefaultAugmentable::<ARITY, HEIGHT_1, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["lemon", "lime"];
        let cmt2 = DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, 100>::try_from::<&[u8]>(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        assert!(cmt1.try_merge(cmt2).is_err());
    }

    #[test]
    fn merge_2trees_different_heights_after_removal() {
        const ARITY: usize = 2;
        const HEIGHT_1: usize = 4;
        const HEIGHT_2: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt1 = DefaultCompactable::<ARITY, HEIGHT_1, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let words2: &[&str] = &["kiwi", "kompot", "kotleta", "sardina"];
        let mut cmt2 = DefaultCompactable::<ARITY, HEIGHT_2, StdHash, 100>::try_from::<&[u8]>(
            &words2.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        cmt1.remove(2);
        cmt1.compact();

        let test_words: &[&str] = &["apple", "apricot", "cherry", "blueberry"];

        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt1.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }

        cmt2.remove(1);
        cmt2.compact();
        assert_eq!(cmt2.num_of_leaves(), 3);

        let test_words: &[&str] = &["kiwi", "kotleta", "sardina"];
        for (i, w) in test_words.iter().enumerate() {
            let proof = cmt2.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }

        let mut cmt =
            DefaultAugmentable::<ARITY, HEIGHT_1, StdHash, 100>::try_from_leaves(&cmt1.leaves())
                .unwrap();
        assert_eq!(cmt1.num_of_leaves(), 4);
        assert_eq!(cmt.num_of_leaves(), 4);

        cmt.try_merge(
            DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, 100>::try_from_leaves(&cmt2.leaves())
                .unwrap(),
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
        assert_eq!(cmt.num_of_leaves(), test_words.len());

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
        const ARITY: usize = 2;
        const HEIGHT: usize = 4;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut cmt = DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
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
        const ARITY: usize = 2;
        const HEIGHT: usize = 3;

        let words1: &[&str] = &["apple", "apricot", "banana", "cherry", "blueberry"];
        let mut amt = DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        amt.try_append(b"kiwi").unwrap();
        amt.try_append(b"kotleta").unwrap();
        amt.try_append(b"blueberry").unwrap();
        assert!(amt.try_append(b"blackberry").is_err());
    }

    #[test]
    fn create_compactable_from_augmentable() {
        const ARITY: usize = 2;
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
        let cmt1 = DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &words1.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
        )
        .unwrap();

        let cmt2 =
            DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from_leaves(&cmt1.leaves())
                .unwrap();
        assert_eq!(cmt1.num_of_leaves(), cmt2.num_of_leaves());

        let cmt3 =
            DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from_leaves(&cmt2.leaves())
                .unwrap();

        assert_eq!(cmt2.num_of_leaves(), cmt3.num_of_leaves());

        for (i, w) in words1.iter().enumerate() {
            let proof = cmt3.generate_proof(i);
            println!("testing -> {w}");
            let res = proof.validate(w.as_bytes());
            assert!(res);
        }
    }

    #[test]
    fn overall_test() {
        const ARITY: usize = 4;
        const HEIGHT: usize = 3;

        let input = (0u8..20).map(|i| vec![i]).collect::<Vec<_>>();

        let mut cmt1 = DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &input.iter().map(|d| d.as_ref()).collect::<Vec<_>>(),
        )
        .unwrap();

        for (i, _) in input.iter().enumerate() {
            if i % 2 == 0 {
                cmt1.remove(i);
            }
        }

        for (i, d) in input.iter().enumerate() {
            if i % 2 == 1 {
                let proof = cmt1.generate_proof(i);
                println!("testing -> {:?}", &d);
                let res = proof.validate(d);
                assert!(res);
            }
        }

        let amt1 =
            DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from_leaves(&cmt1.leaves())
                .unwrap();

        for (i, d) in input.iter().enumerate() {
            if i % 2 == 1 {
                let proof = amt1.generate_proof(i);
                println!("testing -> {:?}", &d);
                let res = proof.validate(d);
                assert!(res);
            }
        }

        cmt1.compact();

        let mut amt1 =
            DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from_leaves(&cmt1.leaves())
                .unwrap();

        for j in 100u8..100 + 64 - 10 {
            if let Err(_) = amt1.try_append(&[j]) {
                panic!("Error on appending: {}", j);
            }
        }

        for (i, d) in input.iter().enumerate() {
            if i % 2 == 1 {
                let proof = amt1.generate_proof(i / 2);
                println!("testing -> {:?}", &d);
                let res = proof.validate(d);
                assert!(res);
            }
        }
        for j in input.len() / 2..input.len() / 2 + 20 {
            let d = j as u8 - input.len() as u8 / 2 + 100;
            let proof = amt1.generate_proof(j);
            println!("testing -> {:?}", &[d]);
            let res = proof.validate(&[d]);
            assert!(res);
        }

        let input1 = (200u8..220).map(|i| vec![i]).collect::<Vec<_>>();

        let cmt2 = DefaultCompactable::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &input1.iter().map(|d| d.as_ref()).collect::<Vec<_>>(),
        )
        .unwrap();

        let amt2 =
            DefaultAugmentable::<ARITY, HEIGHT, StdHash, 100>::try_from_leaves(&cmt2.leaves())
                .unwrap();

        for (i, d) in input1.iter().enumerate() {
            if i % 2 == 1 {
                let proof = amt2.generate_proof(i);
                println!("testing -> {:?}", &d);
                let res = proof.validate(d);
                assert!(res);
            }
        }

        let mut amt1 = amt1.augment();
        let num_of_leaves_before_merge = amt1.num_of_leaves();

        assert_eq!(amt1.height(), HEIGHT + 1);

        amt1.try_merge(amt2).unwrap();

        for (i, d) in input.iter().enumerate() {
            if i % 2 == 1 {
                let proof = amt1.generate_proof(i / 2);
                println!("testing -> {:?}", &d);
                let res = proof.validate(d);
                assert!(res);
            }
        }

        for j in input.len() / 2..input.len() / 2 + 20 {
            let d = j as u8 - input.len() as u8 / 2 + 100;
            let proof = amt1.generate_proof(j);
            println!("testing -> {:?}", &[d]);
            let res = proof.validate(&[d]);
            assert!(res);
        }
        println!("num_of_leaves_before_merge: {num_of_leaves_before_merge}");
        for (i, d) in input1.iter().enumerate() {
            let proof = amt1.generate_proof(i + num_of_leaves_before_merge);
            println!("testing -> {:?}", &d);
            let res = proof.validate(d);
            assert!(res);
        }
    }

    #[test]
    fn claim_index_from_proof() {
        const ARITY: usize = 4;
        const HEIGHT: usize = 3;

        let input = (0u8..20).map(|i| vec![i]).collect::<Vec<_>>();

        let mt = StaticTree::<ARITY, HEIGHT, StdHash, 100>::try_from::<&[u8]>(
            &input.iter().map(|d| d.as_ref()).collect::<Vec<_>>(),
        )
        .unwrap();

        for (i, d) in input.iter().enumerate() {
            let proof = mt.generate_proof(i);

            let ind = proof.claim_index();
            assert_eq!(i, ind);
            //            println!("ind: {ind}");

            let res = proof.validate(d);
            assert!(res);
        }
    }

    // #[test]
    // fn break_it() {
    //     use merkle_heapless::proof::Proof;
    //     use merkle_heapless::traits::ProofBuilder;
    //     use merkle_heapless::prefixed::Prefixed;

    //     const MAX_HEIGHT: usize = 1;
    //     const FAKE_MAX_HEIGHT: usize = 2;

    //     let fake_0 = Blake2_256Hash::hash(b"1234567890123456789012345678901234567890123456789012345678901234");
    //     let fake_1 = Blake2_256Hash::hash(b"1234567890123456789012345678901234567890123456789012345678901234");
    //     let mut fake_concat = [0u8; 64];
    //     fake_concat[..32].copy_from_slice(&fake_0);
    //     fake_concat[32..].copy_from_slice(&fake_1);
    //     let fc_hash = Blake2_256Hash::concat_then_hash(&[fake_0, fake_1]);
    //     let fc_hash1 = Blake2_256Hash::hash(&fake_concat);

    //     assert_eq!(fc_hash, fc_hash1);
    //     // Merkle tree as the creator of the tree sees it
    //     //
    //     //             root
    //     //         apple    (some value)
    //     //
    //     //
    //     //             As the attacker sees it
    //     //
    //     //              root
    //     //          apple    (some value)
    //     //                   hi0      hi1
    //     //
    //     //
    //     let mut tree =
    //         StaticBinaryTree::<MAX_HEIGHT, Blake2_256Hash, 64>::try_from::<&[u8]>(&[b"kiwi", &fake_concat])
    //             .unwrap();

    //     let proof = tree.generate_proof(1);
    //     let apple_hash = Blake2_256Hash::hash(b"kiwi");

    //     println!("proof root: {:?}", proof.root());

    //     let mut alt_proof: Proof<2, FAKE_MAX_HEIGHT, Blake2_256Hash, 64> = Proof::from_root(proof.root());
    //     alt_proof.push(0, Prefixed {
    //                             prefix: <Blake2_256Hash as HashT>::Output::default(),
    //                             hashes: [fake_0, fake_1]
    //                     });

    //     alt_proof.push(1, Prefixed {
    //                             prefix: <Blake2_256Hash as HashT>::Output::default(),
    //                             hashes: [apple_hash, fc_hash]
    //                     });

    //     assert!(proof.validate(&fake_concat));
    //     assert!(alt_proof.validate(b"1234567890123456789012345678901234567890123456789012345678901234"));
    // }
}
