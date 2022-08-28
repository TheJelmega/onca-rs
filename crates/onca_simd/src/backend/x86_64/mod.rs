use core::marker::PhantomData;
use crate::*;

use super::*;

use core::arch::x86_64::*;

mod sse;
pub use sse::*;

mod avx;
pub use avx::*;

mod avx2;
pub use avx2::*;

mod avx512;
pub use avx512::*;


from_transmute!{ unsafe u8x16 =>  __m128i }
from_transmute!{ unsafe u8x32 =>  __m256i }
from_transmute!{ unsafe u8x32 => [__m128i; 2] }
from_transmute!{ unsafe u8x64 =>  __m512i }
from_transmute!{ unsafe u8x64 => [__m256i; 2] }
from_transmute!{ unsafe u8x64 => [__m128i; 4] }
from_transmute!{ unsafe i8x16 =>  __m128i }
from_transmute!{ unsafe i8x32 =>  __m256i }
from_transmute!{ unsafe i8x32 => [__m128i; 2] }
from_transmute!{ unsafe i8x64 =>  __m512i }
from_transmute!{ unsafe i8x64 => [__m256i; 2] }
from_transmute!{ unsafe i8x64 => [__m128i; 4] }

from_transmute!{ unsafe u16x8  =>  __m128i }
from_transmute!{ unsafe u16x16 =>  __m256i }
from_transmute!{ unsafe u16x16 => [__m128i; 2] }
from_transmute!{ unsafe u16x32 =>  __m512i }
from_transmute!{ unsafe u16x32 => [__m256i; 2] }
from_transmute!{ unsafe u16x32 => [__m128i; 4] }
from_transmute!{ unsafe i16x8  =>  __m128i }
from_transmute!{ unsafe i16x16 =>  __m256i }
from_transmute!{ unsafe i16x16 => [__m128i; 2] }
from_transmute!{ unsafe i16x32 =>  __m512i }
from_transmute!{ unsafe i16x32 => [__m256i; 2] }
from_transmute!{ unsafe i16x32 => [__m128i; 4] }

from_transmute!{ unsafe u32x4  =>  __m128i }
from_transmute!{ unsafe u32x8  =>  __m256i }
from_transmute!{ unsafe u32x8  => [__m128i; 2] }
from_transmute!{ unsafe u32x16 =>  __m512i }
from_transmute!{ unsafe u32x16 => [__m256i; 2] }
from_transmute!{ unsafe u32x16 => [__m128i; 4] }
from_transmute!{ unsafe i32x4  =>  __m128i }
from_transmute!{ unsafe i32x8  =>  __m256i }
from_transmute!{ unsafe i32x8  => [__m128i; 2] }
from_transmute!{ unsafe i32x16 =>  __m512i }
from_transmute!{ unsafe i32x16 => [__m256i; 2] }
from_transmute!{ unsafe i32x16 => [__m128i; 4] }

from_transmute!{ unsafe u64x2  =>  __m128i }
from_transmute!{ unsafe u64x4  =>  __m256i }
from_transmute!{ unsafe u64x4  => [__m128i; 2] }
from_transmute!{ unsafe u64x8  =>  __m512i }
from_transmute!{ unsafe u64x8  => [__m256i; 2] }
from_transmute!{ unsafe u64x8  => [__m128i; 4] }
from_transmute!{ unsafe i64x2  =>  __m128i }
from_transmute!{ unsafe i64x4  =>  __m256i }
from_transmute!{ unsafe i64x4  => [__m128i; 2] }
from_transmute!{ unsafe i64x8  =>  __m512i }
from_transmute!{ unsafe i64x8  => [__m256i; 2] }
from_transmute!{ unsafe i64x8  => [__m128i; 4] }

from_transmute!{ unsafe f32x4  =>  __m128 }
from_transmute!{ unsafe f32x8  =>  __m256 }
from_transmute!{ unsafe f32x8  => [__m128; 2] }
from_transmute!{ unsafe f32x16 =>  __m512 }
from_transmute!{ unsafe f32x16 => [__m256; 2] }
from_transmute!{ unsafe f32x16 => [__m128; 4] }

from_transmute!{ unsafe f64x2  =>  __m128d }
from_transmute!{ unsafe f64x4  =>  __m256d }
from_transmute!{ unsafe f64x4  => [__m128d; 2] }
from_transmute!{ unsafe f64x8  =>  __m512d }
from_transmute!{ unsafe f64x8  => [__m256d; 2] }
from_transmute!{ unsafe f64x8  => [__m128d; 4] }