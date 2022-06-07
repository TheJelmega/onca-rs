pub mod sync;

use windows::core::PCWSTR;
use crate::sync::MAX_NAME_LENGTH;


fn name_to_wstr(name: &str, buf: &mut [u16; (MAX_NAME_LENGTH + 1) * 2]) -> PCWSTR
{
    if name.len() == 0
        { PCWSTR::default() }
    else
    {
        name.encode_utf16().zip(buf.iter_mut()).for_each(|(src, dst)| *dst = src);
        PCWSTR( buf.as_ptr() )
    }
}