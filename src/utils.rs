//use crate::HashT;

#[inline]
pub fn location_in_prefixed<const BRANCH_FACTOR: usize>(index: usize) -> (usize, usize) {     
    let offset = index & (BRANCH_FACTOR - 1); // index modulo BRANCH_FACTOR
    let index = index >> BRANCH_FACTOR.trailing_zeros();
    (index, offset)
}

#[macro_export]
/// total size of elements in a tree with given arity and height
macro_rules! num_of_prefixed {
    ($branch_factor:expr, $height:expr) => {
        ((1 << ($branch_factor.trailing_zeros() as usize * ($height))) - 1)
            / ($branch_factor - 1)
    };
}


#[macro_export]
/// total size of elements in a tree with given arity and height
macro_rules! total_size {
    ($branch_factor:expr, $height:expr) => {
        ((1 << ($branch_factor.trailing_zeros() as usize * ($height + 1))) - 1)
            / ($branch_factor - 1)
    };
}

#[macro_export]
/// size of a layer at index in a tree with given arity and height
macro_rules! layer_size {
    ($branch_factor:expr, $height:expr, $layer_index:expr) => {
        1 << ($branch_factor.trailing_zeros() as usize * ($height - $layer_index - 1))
    };
}

#[macro_export]
/// size of a layer at index in a tree with given arity and height
macro_rules! max_leaves {
    ($branch_factor:expr, $height:expr) => {
        $branch_factor * layer_size!($branch_factor, $height, 0)
    };
}

#[macro_export]
/// determines if a number is power of 2
macro_rules! is_pow2 {
    ($x:expr) => {
        ($x.leading_zeros() + $x.trailing_zeros()) as usize == 8 * core::mem::size_of::<usize>() - 1
    };
}
/// auxiliary struct to impose boolean constraints at compile-time
pub struct Assert<const COND: bool>;
/// companion for boolean [Assert]
pub trait IsTrue {}
impl IsTrue for Assert<true> {}
