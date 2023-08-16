#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use wasm_bindgen::prelude::*;

use merkle_heapless::StaticBinaryTree;
use merkle_heapless::traits::{HashT, StaticTreeTrait, ProofValidator};

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

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn test_1() {
    const HEIGHT: usize = 5;

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
    let mut mt = StaticBinaryTree::<HEIGHT, StdHash, 100>::try_from(
        &words.iter().map(|w| w.as_bytes()).collect::<Vec<_>>(),
    );

    let mut results = Vec::new();
    for (i, w) in test_words.iter().enumerate() {
        let proof = mt.as_mut().unwrap().generate_proof(i);
        let res = proof.validate(w.as_bytes());
        results.push(res);
    }

    let bad_words: &[&str] = &["kiwi"];
    for (i, w) in bad_words.iter().enumerate() {
        let proof = mt.as_mut().unwrap().generate_proof(i);
        let res = proof.validate(w.as_bytes());
        results.push(res);
    }

    alert(&format!("words: {:?}\nbad_words: {:?}\nresults: {:?}", words, bad_words, results));
}
