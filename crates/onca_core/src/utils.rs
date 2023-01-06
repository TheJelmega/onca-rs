use core::ops;

/// If the array contains `0`, return the sub-slice until that point, otherwise return the full slice.
pub fn null_terminate_slice(slice: &[u8]) -> &[u8] {
    let len = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    &slice[..len]
}

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