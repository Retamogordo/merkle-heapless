use crate::HashT;

#[macro_export]
macro_rules! total_size {
    ($branch_factor:expr, $height:expr) => {
        ((1 << ($branch_factor.trailing_zeros() as usize * $height)) - 1) / ($branch_factor - 1)
    };
}

#[macro_export]
macro_rules! layer_size {
    ($branch_factor:expr, $height:expr, $layer_index:expr) => {
        1 << ($branch_factor.trailing_zeros() as usize * ($height - $layer_index - 1))
    };
}

// hash combined bytes from a contiguous memory chank
pub(crate) fn hash_merged_slice<H: HashT>(contiguous_array: &[H::Output], len: usize) -> H::Output {
    H::hash(
        unsafe { core::slice::from_raw_parts(contiguous_array[0].as_ref().as_ptr(), len) }
    )
}
