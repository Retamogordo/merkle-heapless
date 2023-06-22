use std::mem::size_of;
fn main() {
    // let base_layer_size = 8;
    // let branch_factor = 2;
    // let input_len = 19;
    // let x = base_layer_size * branch_factor;

    // let pad_len = input_len / x + if input_len % x == 0 {0} else {1};
    // println!("pad_len: {pad_len}");
    let branch_factor: u32 = 2;
    let n_subtrees: usize = 5;
    let n_subtrees_padded = branch_factor.trailing_zeros() << (8*size_of::<usize>() as u32 - n_subtrees.leading_zeros());
    println!("branch_factor: {branch_factor}, n_subtrees: {n_subtrees}, n_subtrees_padded: {n_subtrees_padded}");

    let branch_factor: u32 = 4;
    let n_subtrees: usize = 5;
    let n_subtrees_padded = branch_factor.trailing_zeros() << (8*size_of::<usize>() as u32 - n_subtrees.leading_zeros());
    println!("branch_factor: {branch_factor}, n_subtrees: {n_subtrees}, n_subtrees_padded: {n_subtrees_padded}");
}