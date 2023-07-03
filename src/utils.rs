use crate::HashT;

#[macro_export]
/// total size of elements in a tree with given arity and height 
macro_rules! total_size {
    ($branch_factor:expr, $height:expr) => {
        ((1 << ($branch_factor.trailing_zeros() as usize * ($height + 1))) - 1) / ($branch_factor - 1)
    };
}

#[macro_export]
/// size of a layer at index in a tree with given arity and height 
macro_rules! layer_size {
    ($branch_factor:expr, $height:expr, $layer_index:expr) => {
        1 << ($branch_factor.trailing_zeros() as usize * ($height - $layer_index))
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

// hash combined bytes from a contiguous memory chank
pub(crate) fn hash_merged_slice<H: HashT>(contiguous_array: &[H::Output], len: usize) -> H::Output {
    H::hash(
        unsafe { core::slice::from_raw_parts(contiguous_array[0].as_ref().as_ptr(), len) }
    )
}