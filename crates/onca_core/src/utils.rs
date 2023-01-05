use core::ops;

/// Convert a slice containing a null terminated string into an array
pub fn null_terminated_arr_to_str_unchecked(arr: &[u8], max_len: usize) -> &str {
    let len = arr.iter().position(|&b| b == 0).unwrap_or(max_len);
    let slice = unsafe { core::slice::from_raw_parts(arr.as_ptr(), len) };
    unsafe { core::str::from_utf8_unchecked(slice) }
}

/// Convert a slice containing a null terminated string into an array
pub fn null_terminated_arr_to_str(arr: &[u8], max_len: usize) -> Result<&str, core::str::Utf8Error> {
    let len = arr.iter().position(|&b| b == 0).unwrap_or(max_len);
    let slice = unsafe { core::slice::from_raw_parts(arr.as_ptr(), len) };
    unsafe { core::str::from_utf8(slice) }
}

/// Check if a flag is set
pub fn is_flag_set<T>(val: T, flag: T) -> bool
where
    T : Copy + PartialEq + ops::BitAnd<Output = T>
{
    val & flag == flag
}