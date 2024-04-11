//use crate::HashT;

#[inline]
pub fn location_in_prefixed<const ARITY: usize>(index: usize) -> (usize, usize) {
    let offset = index & (ARITY - 1); // index modulo ARITY
    let index = index >> ARITY.trailing_zeros();
    (index, offset)
}

#[macro_export]
///
macro_rules! num_of_prefixed {
    ($arity:expr, $height:expr) => {
        ((1 << ($arity.trailing_zeros() as usize * ($height))) - 1) / ($arity - 1)
    };
}

#[macro_export]
/// total size of elements in a tree with given arity and height
macro_rules! total_size {
    ($arity:expr, $height:expr) => {
        ((1 << ($arity.trailing_zeros() as usize * ($height + 1))) - 1) / ($arity - 1)
    };
}

#[macro_export]
/// size of a layer at index in a tree with given arity and height
macro_rules! layer_size {
    ($arity:expr, $height:expr, $layer_index:expr) => {
        //1 << ($arity.trailing_zeros() as usize * ($height - $layer_index))
        1 << ($arity.trailing_zeros() as usize * ($height - $layer_index - 1))
    };
}

#[macro_export]
/// size of a layer at index in a tree with given arity and height
macro_rules! max_leaves {
    ($arity:expr, $height:expr) => {
        $arity * layer_size!($arity, $height, 0)
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
