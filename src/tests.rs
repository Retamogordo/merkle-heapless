
#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::{validate_proof, ConcatHashes, MerkleTree};

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

    struct Blake2_256ConcatHashes;

    impl ConcatHashes<32> for Blake2_256ConcatHashes {
        type Output = [u8; 32];

        fn hash(input: &[u8]) -> Self::Output {
            sp_core::blake2_256(input)
        }
    }

    struct DefaultConcatHashes;

    impl ConcatHashes<8> for DefaultConcatHashes {
        type Output = [u8; 8];

        fn hash(input: &[u8]) -> Self::Output {
            let mut s = DefaultHasher::new();
            input.hash(&mut s);
            s.finish().to_ne_bytes()
        }
    }

    #[test]
    fn validate_3layers_blake2_256() {
        const LAYERS: usize = 3;
        let mut mt = MerkleTree::<LAYERS, 32, Blake2_256ConcatHashes>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);

        let word_index = 2;
        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "kiwi";
        let res =
            validate_proof::<LAYERS, 32, Blake2_256ConcatHashes>(&root, word.as_bytes(), proof);
        println!(
            "word: {:?} {} validated, proof was generated for word at index {}",
            word,
            if res { "" } else { "NOT" },
            word_index
        );
        assert!(res);
    }

    #[test]
    fn validate_4layers_std_hash() {
        let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 2;
        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "kiwi";
        let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
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
        let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word = "kiwi";
        let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
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
    fn fail_4layers_std_hash_bad_index() {
        let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 8;
        let (_root, _proof) = mt.as_mut().unwrap().generate_proof(word_index);
    }

    #[test]
    fn validate_default_padding_word_4layers_std_hash() {
        let mut mt = MerkleTree::<4, 8, DefaultConcatHashes>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta",
        ]);
        let word_index = 7;
        let (root, proof) = mt.as_mut().unwrap().generate_proof(word_index);
        let word: &str = Default::default();
        let res = validate_proof::<4, 8, DefaultConcatHashes>(&root, word.as_bytes(), proof);
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
        let mt = MerkleTree::<4, 32, Blake2_256ConcatHashes>::try_from(&[
            b"apple", b"banana", b"kiwi", b"kotleta", b"apple", b"banana", b"kiwi", b"kotleta",
            b"apple",
        ]);

        assert!(mt.is_err());
    }

    #[test]
    fn total_size_and_layers() {
        const LAYERS: usize = 7;
        let mt = MerkleTree::<LAYERS, 32, Blake2_256ConcatHashes>::try_from(&[b"apple"]);

        assert_eq!(mt.as_ref().unwrap().total_size(), (1 << LAYERS) - 1);
        assert_eq!(mt.unwrap().total_layers(), LAYERS);
    }
}
