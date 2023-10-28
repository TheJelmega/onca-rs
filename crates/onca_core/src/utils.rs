use core::ops;

/// If the array contains `0`, return the sub-slice until that point, otherwise return the full slice.
pub fn null_terminate_slice(slice: &[u8]) -> &[u8] {
    let len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    &slice[..len]
}

// TODO: Should be unsafe, as this is currently unsound
/// Convert a slice containing a null terminated string into an array
pub fn null_terminated_arr_to_str_unchecked(arr: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(null_terminate_slice(arr)) }
}

/// Convert a slice containing a null terminated string into an array
pub fn null_terminated_arr_to_str(arr: &[u8]) -> Result<&str, core::str::Utf8Error> {
    unsafe { core::str::from_utf8(null_terminate_slice(arr)) }
}

/// Check if a flag is set
pub fn is_flag_set<T>(val: T, flag: T) -> bool
where
    T : Copy + PartialEq + ops::BitAnd<Output = T>
{
    val & flag == flag
}

/// Check if aany flag is set
pub fn is_any_flag_set<T>(val: T, flag: T, zero: T) -> bool
where
    T : Copy + PartialEq + ops::BitAnd<Output = T>
{
    val & flag != zero
}

/// Trait to get the number of elements in an enum
pub trait EnumCount {
    /// Count or number of element in an enum
    const COUNT : usize;
}

/// Trait to get an enum from a given index
pub trait EnumFromIndex : Sized {
    /// Try to convert an index to an enum
    fn from_idx(idx: usize) -> Option<Self>;
    /// Convert an index to an enum, without checking bounds
    /// 
    /// # SAFETY
    /// 
    /// The user is required to make sure that the index is an index of a valid enum variant
    unsafe fn from_idx_unchecked(idx: usize) -> Self;
}

/// Count the number of token trees
// mutliple version to limit recusion
#[macro_export]
macro_rules! count_tt {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($rest:tt)*) => {
        20usize + onca_core::count_tt!($($rest)*)
    };
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($rest:tt)*) => {
        10usize + onca_core::count_tt!($($rest)*)
    };
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($rest:tt)*) => {
        5usize + onca_core::count_tt!($($rest)*)
    };
    ($_first:tt $($rest:tt)*) => {
        1usize + onca_core::count_tt!($($rest)*)
    };
    () => {
        0usize
    };
}

/// Count the number of comma separated expressions
// mutliple version to limit recusion
#[macro_export]
macro_rules! count_exprs {
    ($_a:expr, $_b:expr, $_c:expr, $_d:expr, $_e:expr,
     $_f:expr, $_g:expr, $_h:expr, $_i:expr, $_j:expr,
     $_k:expr, $_l:expr, $_m:expr, $_n:expr, $_o:expr,
     $_p:expr, $_q:expr, $_r:expr, $_s:expr, $_t:expr,
     $($rest:expr),* $(,)?) => {
        20usize + onca_core::count_exprs!($($rest),*)
    };
    ($_a:expr, $_b:expr, $_c:expr, $_d:expr, $_e:expr,
     $_f:expr, $_g:expr, $_h:expr, $_i:expr, $_j:expr,
     $($rest:expr),* $(,)?) => {
        10usize + onca_core::count_exprs!($($rest),*)
    };
    ($_a:expr, $_b:expr, $_c:expr, $_d:expr, $_e:expr,
     $($rest:expr),* $(,)?) => {
        5usize + onca_core::count_exprs!($($rest),*)
    };
    ($_first:expr, $($rest:expr),* $(,)?) => {
        1usize + onca_core::count_exprs!($($rest),*)
    };
    ($_first:expr $(,)?) => {
        1usize
    };
    () => {
        0usize
    };
}

/// Macro to help improve errors, specifically improve diagnostics in pattern positions
/// 
/// see liballoc `__rust_force_expr`
#[macro_export]
macro_rules! __rust_force_expr {
    ($e:expr) => {
        $e
    };
}