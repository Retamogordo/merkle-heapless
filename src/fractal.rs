use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};



fn myhash(input: &[u8]) -> [u8; 8] {
    let mut s = DefaultHasher::new();
    input.hash(&mut s);
    s.finish().to_ne_bytes()
}

#[derive(Default)]
struct Fractal<'a> {
    root: [u8; 8],
    leaves: Option<[&'a Self; 2]>,
}

impl<'a> Fractal<'a> {
    fn leaf(data: &str) -> Self {
        Self {
            root: myhash(data.as_bytes()),
            leaves: None,
        }
    }
    fn from(children: [&'a Self; 2]) -> Self {
        Self {
            root: myhash([children[0].root.as_slice(), children[1].root.as_slice()].as_slice()[0]),
            leaves: Some(children),
        }
    }
}



fn main() {
    let t1 = Fractal::leaf("apple");
    let t2 = Fractal::leaf("banana");
    let t3 = Fractal::from([&t1, &t2]);
}
