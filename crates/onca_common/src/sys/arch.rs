use core::fmt;

use onca_common_macros::flags;

pub mod x86_64;


pub enum CpuFeatures {
    X86_64(x86_64::CpuidInfo),
}

impl CpuFeatures {
    /// Get the CPU features for the current CPU
    pub fn get() -> CpuFeatures {
        if cfg!(target_arch = "x86_64") {
            CpuFeatures::X86_64(x86_64::CpuidInfo::get())
        } else {
            panic!("unsupported CPU architecture")
        }
    }
}

impl fmt::Display for CpuFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuFeatures::X86_64(cpuid) => write!(f, "{}", cpuid),
        }
    }
}