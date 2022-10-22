// Portions of the project have been copied from parking_lot and is copyrighted by Amanieu d'Antra under the MIT license (located in: '3rd-party-licenses/parking_lot')

use core::sync::atomic::AtomicUsize;

pub trait AtomicElisionExt {
    type IntType;

    // Perform a compare_exchange and start a transition.
    fn elision_compare_exchange_acquire(&self, current: Self::IntType, new: Self::IntType) -> Result<Self::IntType, Self::IntType>;

    // Perform a fetch_sub and end a transaction
    fn elision_fetch_sub_release(&self, val: Self::IntType) -> Self::IntType;
}

// Indicated whether the target architecture supports lock elision
#[inline]
pub fn have_elision() -> bool {
    // Onca only supports 64-bit
    // TODO(jel): Hardware elision support could be disabled on intel processor, completely disregarding the possible performance improvement it could bring, check and cache availability with cpuid:
    //            https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=c2955f270a84762343000f103e0640d29c7a96f3
    cfg!(target_arch = "x86_64")
}

// This implementation is never acutally called, because it is guareded by have_elision().
#[cfg(not(target_arch = "x86_64"))]
impl AtomicElisionExt for AtomicUsize {
    type IntType = usize;

    #[inline]
    fn elision_compare_exchange_acquire(&self, current: usize, new: usize) -> Result<usize, usize> {
        unreachable!()
    }

    #[inline]
    fn elision_fetch_sub_release(&self, val: usize) -> usize {
        unreachable!()
    }
}

#[cfg(target_arch = "x86_64")]
impl AtomicElisionExt for AtomicUsize {
    type IntType = usize;

    #[inline]
    fn elision_compare_exchange_acquire(&self, current: usize, new: usize) -> Result<usize, usize> {
        unsafe {
            use core::arch::asm;
            let prev: usize;
            // Onca only supports 64-bit
            asm!(
                "xacquire",
                "lock",
                "cmpxchg [{}], {}",
                in(reg) self,
                in(reg) new,
                inout("rax") current => prev,
            );
            if prev == current {
                Ok(prev)
            } else {
                Err(prev)
            }
        }
    }

    #[inline]
    fn elision_fetch_sub_release(&self, val: usize) -> usize {
        unsafe {
            use core::arch::asm;
            let prev: usize;
            // Onca only supports 64-bit
            asm!(
                "xrelease",
                "lock",
                "xadd [{}], {}",
                in(reg) self,
                inout(reg) val.wrapping_neg() => prev
            );
            prev
        }
    }
}