# Static Merkle Tree and Mountain Range
This Merkle tree is implemented as a contiguous memory array and does not betake to dynamic allocations.
As such it allows for certain optimizations and compile-time imposed constraints on arity and size boundaries.

## Features
- no std dependencies (actually no dependencies)
- 2, 4, 8,... power of 2 general branching arity
- any hash function that takes ```&[u8]``` and returns something that implements ```AsRef<[u8]>```
- 99% safe Rust 
- optionally augmentable or reducible 
- optional Mountain Range proc macro (when compiled with a feature)

## Basic functionality
The most basic tree is the StaticTree.
This kind of tree is instantiated to its final size.

### Basic operations
```rust
use merkle_heapless::{StaticBinaryTree};
use merkle_heapless::traits::{StaticTreeTrait, ProofValidator};
// tree height 3, 8 leaves
const MAX_HEIGHT: usize = 3;
const MAX_WORD_LEN: usize = 10;
// supposing the YourHash struct exists
let mut tree = StaticBinaryTree::<MAX_HEIGHT, YourHash, MAX_WORD_LEN>::try_from::<&[u8]>(
    &[b"apple", b"banana"]
).unwrap();

let proof = tree.generate_proof(0);
assert!(proof.validate(b"apple"));
```
### Replace and remove leaf
You can replace a leaf with another value
```rust
// snip
// replace
tree.replace(5, b"cherry");
let proof = tree.generate_proof(5);
assert!(proof.validate(b"cherry"));
// remove
tree.replace(1, &[]);
let proof = tree.generate_proof(1);
assert!(!proof.validate(b"banana"));
let proof = tree.generate_proof(1);
assert!(proof.validate(&[]));
```
### Arity other than 2
It's a generalized form of the above tree.
```rust
use merkle_heapless::{StaticTree};

const ARITY: usize = 4;
let mut tree = StaticTree::<ARITY, MAX_HEIGHT, YourHash, MAX_WORD_LEN>::try_from::<&[u8]>(
    &[b"apple", b"banana"]
).unwrap();
// same operations can be applied
```

### Custom Hash implementation
Examples: blake256 and standard Rust's hash used for HashMaps

```rust
use merkle_heapless::traits::HashT;

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

    fn hash(input: &[u8]) -> Self::Output {
        Wrapped32(sp_core::blake2_256(input))
    }
}
```

```rust
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use merkle_heapless::traits::HashT;
#[derive(Debug)]
pub struct StdHash;
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

    fn hash(input: &[u8]) -> Self::Output {
        let mut s = DefaultHasher::new();
        input.hash(&mut s);
        Wrapped8(s.finish().to_ne_bytes())
    }
}
```

## Augmentation and Reduction
These extentions provide limited dynamic behaviour as for tree size handling.

### Augmentation
A tree is augmented by creating a new tree with a height bigger by one, so the new tree contains as twice as nodes the former tree had.
Then the contents of the former tree are copied and hashes recalculated.
#### Augment
```rust
use merkle_heapless::augmentable::{DefaultAugmentable};

const ARITY: usize = 4;
const HEIGHT: usize = 3;
const MAX_WORD_LEN: usize = 10;

let mt1 = DefaultAugmentable::<ARITY, HEIGHT, StdHash, MAX_WORD_LEN>::try_from::<&[u8]>(&[
    "apple", "apricot", "banana", "cherry",
]).unwrap();

let mut mt = mt1.augment();
assert_eq!(mt.height(), HEIGHT + 1);
```
#### Merge
You can ```try_merge``` a smaller (or equally-sized) tree into the original tree. 
This operation does not imply augmentation, rather it fails if merge is not possible.
```rust
// snip
let mt2 = DefaultAugmentable::<ARITY, HEIGHT_2, StdHash, MAX_WORD_LEN>::try_from::<&[u8]>(&[
    "kiwi", "lemon",
]).unwrap();

mt1.try_merge(mt2).unwrap();

### Reduction
Similarly, if remove, compact and reduce semantics is needed it is achievable through a Compactable tree variation:
```rust
use merkle_heapless::compactable::{DefaultCompactable};

const ARITY: usize = 4;
const HEIGHT: usize = 3;
const MAX_WORD_LEN: usize = 10;

let mut cmt = DefaultCompactable::<ARITY, HEIGHT, StdHash, MAX_WORD_LEN>::try_from::<&[u8]>(&[
    "apple", "apricot", "banana", "cherry",
]).unwrap();

cmt.try_remove(0).unwrap();
cmt.compact();
// will try to create a smaller tree from the compacted tree
let mut reduced = cmt.try_reduce().unwrap();
```

## Mountain Range
Merkle Mountain Range offers append-only growable Merkle Tree semantics optimized for space.
The rules for this implementation of Mountain Range are:
- space limitations are defined at compile-time (no dynamic allocations) by number of peaks only
- an element is inserted by appending to the right-most peak having a capacity to append a new item
- the left-most peak is the highest peak at any moment
- when two adjacent peaks have the same height they are recursively merged into the left sibling 
- roots of the peaks form leaves for the "summit Merkle tree"
- the Mountain Range proof is generated by chaining the proof of the corresponding peak with the proof generated by the relevant path in the summit tree 
- for MMR declared with N peaks, it will handle peaks with heights [0..N] thus simulating a tree with number of leaves in range [0..N*2^N] in case of a binary MMR 

Include ```features = ["mmr_macro"]``` in the ```merkle-heapless``` dependency in ```Cargo.toml```.

### Declaration and instantiation
```rust
// compulsory at the beginning of the .rs file in order the macro to compile
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
// snip
use merkle_heapless::{mmr_macro};
// declaration with expicit type name for your MMR
mmr_macro::mmr!(Type = FooMMR, BranchFactor = 2, Peaks = 3, Hash = StdHash, MaxInputWordLength = 10);
let mmr = FooMMR::default();
// implicitly creates MerkleMountainRange type
mmr_macro::mmr!(BranchFactor = 2, Peaks = 5, Hash = StdHash, MaxInputWordLength = 10);
// create with default current peak of height 0
let mmr = MerkleMountainRange::default();
// or create with current peak of height 2
let mut mmr = MerkleMountainRange::from_peak(MerkleMountainRangePeak::Peak3(Default::default()));
assert_eq!(mmr.peaks()[0].height(), 5 - 3);
```
### Functionality
The functionality of Mountain Range is similar to that of the Merkle tree.   
```rust
mmr.try_append(b"apple").unwrap();
// peak leaf numbers: [1, 0, 0, 0, 0]
assert_eq!(mmr.peaks()[0].height(), 0);
assert_eq!(mmr.peaks()[0].num_of_leaves(), 1);
assert_eq!(mmr.peaks()[1].num_of_leaves(), 0);
let proof = mmr.generate_proof(0);
assert!(proof.validate(b"apple"));
```
