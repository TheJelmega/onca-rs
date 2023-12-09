//! Partially implements returning info from cpuid
//! 
//! https://en.wikipedia.org/wiki/CPUID

use core::{
    fmt::{self, Write},
    ptr::copy_nonoverlapping,
    mem::size_of_val
};

use onca_common_macros::{flags, EnumDisplay, EnumFromIndex};

use crate::fmt::Indenter;

/// CPU manufacturer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Manufacturer {
    //#[display("AMD (early engineering sampled of K5): \"AMDisbetter!\"")]
    /// Early engineering sampled for the AMD K5 processor: "AMDisbetter!"
    EarlyAMD,
    /// AMD: "AuthenticAMD"
    AMD,
    /// IDT WinChip/Centaur (including some VIA and Zhaoxin CPUs): "CentaurHauls"
    Centaur,
    /// Cyris/early STMicroelectronics and IBM: "CyrixInstead"
    Cyrix,
    /// Intel: "GenuineIntel"
    Intel,
    /// Intel (rare): "GenuineIotel"
    Iotel,
    /// Transmeta: "TransmetaCPU"
    Transmeta,
    /// Transmeta: "GenuineTMx86"
    Transmeta2,
    /// National Semiconductor: "Geode by NSC"
    NationalSemiconductor,
    /// NexGen: "NexGenDriven"
    NexGen,
    /// Rise: "RiseRiseRise"
    Rise,
    /// SIS (Silicon Integrated Systems): "SiS SiS SiS "
    SIS,
    /// UMC (United Microelectronics Corporation): "UMC UMC UMC"
    UMC,
    /// VIA (VIA Technologies Inc.): "VIA VIA VIA "
    VIA,
    /// DM&P Vortex86: "Vertex86 SoC"
    DmPVortex86,
    /// Zhaoxin: "  Shanghai  "
    Zhaoxin,
    /// Hygon: "HygonGenuine"
    Hygon,
    /// RDC Semiconductor Co. Ltd.: "Genuine  RDC"
    RDC,
    /// MSCT Eibrus: "E2K MACHINE"
    MCST,
    /// ao486 CPU: "MiSTer AO486 "
    AO486,
    /// Bhyve: "bhyve bhyve "
    Bhyve,
    /// KVM (Kernel-based Virtual Machine): "KVMKVMKVM\0\0\0"
    KVM,
    /// QEMU (Quick EMUlator): "TCGTCGTCGTCG"
    QEMU,
    /// Microsoft Hyper-V or Windows Virtual PC: "Microsoft Hv"
    HyperV,
    /// Microsoft x86-to-ARM: "MicrosoftXTA"
    MsXTM,
    /// Parallels: " lrpepyh  vr"
    Parallels,
    /// VMware: "VMwareVMware"
    VMware,
    /// Xen HVM: "XenVMMXenVMM"
    XenHVM,
    /// Project ACRN: "ACRNACRNACRN"
    ProjectACRN,
    /// QNX hypervisor: " QNXQVMBSQG "
    QNX,
    /// Apple rosetta (versions after 2): "VirtualApple"
    AppleRosetta,
    /// Unknown
    Unknown([u8; 12]),
}

impl Default for Manufacturer {
    fn default() -> Self {
        Manufacturer::Unknown([0; 12])
    }
}

impl fmt::Display for Manufacturer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Manufacturer::EarlyAMD              => f.write_str("AMD, early engineering sampled of the AMD K5 (\"AMDisbetter!\")"),
            Manufacturer::AMD                   => f.write_str("AMD (\"AuthenticAMD\")"),
            Manufacturer::Centaur               => f.write_str("IDT WinChip/Centaur, including some VIA and Zhaoxin CPUs (\"CentaurHauls\")"),
            Manufacturer::Cyrix                 => f.write_str("Cyrix/early STMicroelectronics and IBM (\"CyrixInstead\")"),
            Manufacturer::Intel                 => f.write_str("Intel (\"GenuineIntel\")"),
            Manufacturer::Iotel                 => f.write_str("Intel (\"GenuineIotel\")"),
            Manufacturer::Transmeta             => f.write_str("Transmeta (\"TransmetaCPU\")"),
            Manufacturer::Transmeta2            => f.write_str("Transmeta (\"GenuineTMx86\")"),
            Manufacturer::NationalSemiconductor => f.write_str("National Semiconductor(\"Geode by NSC\")"),
            Manufacturer::NexGen                => f.write_str("NexGen (\"NexGenDriven\")"),
            Manufacturer::Rise                  => f.write_str("Rise (\"RiseRiseRise\")"),
            Manufacturer::SIS                   => f.write_str("Silicon Integrated Systems (\"SiS SiS SiS \")"),
            Manufacturer::UMC                   => f.write_str("United Microelectronics Corporation (\"UMC UMC UMC\")"),
            Manufacturer::VIA                   => f.write_str("VIA Technologies Inc. (\"VIA VIA VIA\")"),
            Manufacturer::DmPVortex86           => f.write_str("DM&P Vertex86 (\"Vertex86 SoC\")"),
            Manufacturer::Zhaoxin               => f.write_str("Zhaoxin (\"  Shangai  \")"),
            Manufacturer::Hygon                 => f.write_str("Hygon (\"HygonGenuine\")"),
            Manufacturer::RDC                   => f.write_str("RDC Semiconductor Co. Ltd. (\"Genuine  RDC\")"),
            Manufacturer::MCST                  => f.write_str("MCST Eibrus (\"E2K MACHINE\")"),
            Manufacturer::AO486                 => f.write_str("ao486 CPU (\"\")"),
            Manufacturer::Bhyve                 => f.write_str("bhyve (\"bhyve bhyve \")"),
            Manufacturer::KVM                   => f.write_str("KVM (Kernel-based Virutal Machine) (\"KVMKVMKVM\\0\\0\\0\")"),
            Manufacturer::QEMU                  => f.write_str("QEMU (Quick Emulator) (\"TCGTCGTCGTCG\")"),
            Manufacturer::HyperV                => f.write_str("Microsoft Hyper-V (\"Microsoft Hv\")"),
            Manufacturer::MsXTM                 => f.write_str("Microsoft x86-to-ARM (\"\")"),
            Manufacturer::Parallels             => f.write_str("Parallels (\" lrpepyh  vr\")"),
            Manufacturer::VMware                => f.write_str("VMware (\"VMwareVMware\")"),
            Manufacturer::XenHVM                => f.write_str("Xen HVM (\"XenVMMXenVMM\")"),
            Manufacturer::ProjectACRN           => f.write_str("Project ACRN (\"ACRNACRNACRN\")"),
            Manufacturer::QNX                   => f.write_str("QNX (\" QNXQVMBSQG \")"),
            Manufacturer::AppleRosetta          => f.write_str("Apple Rosetta (\"GenuineIntel\")"),
            Manufacturer::Unknown(arr)          => write!(f, "Unknown ({:X},{:X},{:X},{:X},{:X},{:X},{:X},{:X},{:X},{:X},{:X},{:X})", 
                                                             arr[0], arr[1], arr[2], arr[3], arr[4], arr[5], arr[6], arr[7], arr[8], arr[9], arr[10], arr[11]),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay, Default, EnumFromIndex)]
pub enum ProcessorType {
    #[default]
    OEM,
    IntelOverdrive,
    DualProcessor,
    Reserved
}

/// CPU family info
#[derive(Clone, Copy, Debug, Default)]
pub struct FamilyInfo {
    pub family_id:      u16,
    pub model_id:       u16,
    pub processor_type: ProcessorType,
    pub stepping_id:    u8,
}

impl fmt::Display for FamilyInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Family info:")?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Family ID:      {}", self.family_id)?;
        writeln!(indenter, "Model ID:       {}", self.model_id)?;
        writeln!(indenter, "Processor type: {}", self.processor_type)?;
        write!  (indenter, "Stepping ID:    {}", self.stepping_id)
    }
}

/// Additional feature info
#[derive(Clone, Copy, Debug, Default)]
pub struct AdditionalFeatureInfo {
    /// Brand index
    pub brand_index:         u8,
    /// `CLFLUSH` line size.
    /// 
    /// # Note
    /// 
    /// Only used when the `CLFSH` flag is set.
    pub clflush_line_size:   u8,
    /// Maximum number of addressable IDs for logical processors in this physical package.
    /// 
    /// The nearest power-of-2 integer that is not smaller than this value is the number of unique initial APIC IDs reserved for addressing different logical processors in a physical package.
    /// 
    /// Former use: Number of logical processors per physical processors. 2 for the Pentium 4 processor with Hyper-Threading.
    pub max_addressable_ids: u8,
    /// Local APID ID: The initial APIC-ID is used to identify the executing logical processor.
    /// 
    /// It can also be identified via the cpuid 0Bh leaf.
    pub local_apic_id:       u8,
}

/// Feature flags
/// 
/// Upper: cpuid(eax=1).ecx << 32 | cpuid(eax=1).edx
#[flags]
pub enum FeatureFlags {
    // cpuid(eax=1).edx
    /// CPU has an onboard x87 FPU.
    FPU,
    /// Virtual 8086 mode extensions (such as VIF, VIP, and PVI).
    VME,
    /// Debugging Extensions (`CR4` bit 3).
    DE,
    /// Page Size Extensions (4MiB pages).
    PSE,
    /// Time Stamp Counter.
    TSC,
    /// Model-specfic registers and RDMSR/WRMSR instuctions.
    MSR,
    /// Physical Address Extension.
    PAE,
    /// Machine Check Exception.
    MCE,
    /// `CMPXCHG8B`` (compare-and-swap) instruction.
    CX8,
    /// Onboard Advanced Programmable Interrupt Controller.
    APIC,
    /// `SYSENTER` and `SYSEXIT` fast system call instructions.
    SEP = 0x800,
    /// Memory Type Range Registers.
    MTRR,
    /// Page Global Enable bit in `CR4`.
    PGE,
    /// Machine Check Architecture.
    MCA,
    /// Conditional move `CMOV`, `FCMOV`, and `FCOMI` instructions.
    CMOV,
    /// Page Attribute Table.
    PAT,
    /// 36-bit Page Size Extension.
    PSE36,
    /// Processor Serial Number supported and enabled.
    PSN,
    /// `CLFLUSH` cache line flush instruction (SSE2).
    CLFSH,
    /// No-execute (NX) bit.
    NX,
    /// Debug store: save trace of executed jump.
    DS,
    /// Onboard thermal control MSRs ofor ACPI.
    ACPI,
    /// MMX instructions (64-bit SIMD).
    MMX,
    /// `FXSAVE` and `FXRSTOR` instructions, `CR4` bit 9.
    FXSR,
    /// Streaming SIMD Extensions (SSE) instructions (aka "Katmai New Instrucitons", 128-bit SIMD).
    SSE,
    /// SSE2 instructions.
    SSE2,
    /// CPU cache implements self-snoop.
    SS,
    /// Max APIC IDs reserved field is valid.
    HTT,
    /// Thermal monitor automatically limits temperature.
    TM,
    /// IA64 processor emulating x86.
    IA64,
    /// Pending Break Enable (PBE# pin) wakeup capability.
    PBE,

    // cpuid(eax=1).ecx
    /// SSE3 (Prescott new instructions - PNI).
    SSE3,
    /// `PCLMULQDQ` (carry-less multiply) instuctions.
    PCLMULQDQ,
    /// 64-bit debug store (`edx` bit 21).
    DTES64,
    /// `MONITOR` and `MWAIT` instructions (PNI).
    Monitor,
    /// CPL qualified debug store.
    DS_CPL,
    /// Virtual Machine eXtensions.
    VMX,
    /// Safer Mode eXtensions (LaGrance) (`GETSEC` instruction).
    SMX,
    /// Enhanced SpeedStep.
    EST,
    /// Thermal Monitor 2.
    TM2,
    /// Supplemental SSE3 instructions.
    SSSE3,
    /// L1 Context ID.
    CnxtId,
    /// Silicon Debug Interface.
    SDBG,
    /// Fused Multiply-Add (FMA3).
    FMA,
    /// `CMPXCHG12B` instruction.
    CX16,
    /// Can disable sending task priority messages.
    XTPR,
    /// Perfmon & debug capability.
    PDCM,
    /// Process context identifiers (`CR4` it 17).
    PCID = 0x2_0000_0000_0000,
    /// Direct Cache Access for DMA writes.
    DCA,
    /// SSE 4.1 instructions.
    SSE41,
    /// SSE 4.2 instructions.
    SSE42,
    /// x2APIC (enhanced APIC).
    X2APIC,
    /// `MOVBE` instruction (big-endian).
    MOVBE,
    /// `POPCNT` instruction.
    POPCNT,
    /// APIC implements one-shot operation usig a TSC deadline value.
    TscDeadline,
    /// AES instruction set.
    AES_NI,
    /// eXtensible processor state save/restore: `XSAVE`, `XRSTOR`, `XSETBV`, and `XGETBV` instructions.
    XSAVE,
    /// `XSAVE` enabled by the OS.
    OSXSAVE,
    /// Advanced Vector Extensions (256-bit SIMD).
    AVX,
    /// Floating point conversion instruction to/from FP16 format.
    F16C,
    /// `RDRND` (on-chip random number generator) feature.
    RDRND,
    /// Hypervisor present (always zero on physical CPUs).
    Hypervisor,
}

/// Extended feature flags
/// 
/// Upper: cpuid(eax=80000001h).ecx << 32 | cpuid(eax=80000001h).edx
/// 
/// # Note
/// 
/// These flags contain duplicate values from [`FeatureFlags`]: bits[0:9], bits[12:17], and bits[23:24].
#[flags]
pub enum ExtendedFeatureFlags {
    // cpuid(eax=80000001h).edx
    /// CPU has an onboard x87 FPU.
    FPU,
    /// Virtual 8086 mode extensions (such as VIF, VIP, and PVI).
    VME,
    /// Debugging Extensions (`CR4` bit 3).
    DE,
    /// Page Size Extensions (4MiB pages).
    PSE,
    /// Time Stamp Counter.
    TSC,
    /// Model-specfic registers and RDMSR/WRMSR instuctions.
    MSR,
    /// Physical Address Extension.
    PAE,
    /// Machine Check Exception.
    MCE,
    /// `CMPXCHG8B`` (compare-and-swap) instruction.
    CX8,
    /// Onboard Advanced Programmable Interrupt Controller.
    APIC,
    /// `SYSCALL/SYSRET` (K6 only).
    SyscallK6,
    /// `SYSCALL/SYSRET`.
    Syscall,
    /// Memory Type Range Registers
    MTRR,
    /// Page Global Enable bit in `CR4`.
    PGE,
    /// Machine Check Architecture.
    MCA,
    /// Conditional move `CMOV`, `FCMOV`, and `FCOMI` instructions.
    CMOV,
    /// Page Attribute Table.
    PAT,
    /// 36-bit Page Size Extension.
    PSE36,
    /// 'Athlon MP`/`Sempron` CPU brand identification.
    ECC,
    /// NX (no-execute) bit.
    NX,
    /// Extended MMX.
    MmxExt,
    /// MMX instructions (64-bit SIMD).
    MMX,
    /// `FXSAVE` and `FXRSTOR` instructions, `CR4` bit 9.
    FXSR,
    /// `FXSAVE` and `FXSTOR` optimizations.
    FxsrOpt,
    /// Gigabyte pages.
    PDPE1GB,
    /// `RDTSCP` instruction.
    RDTSCP,
    /// Long Mode.
    LM = 0x1000_0000,
    /// Extended 3DNow!
    _3DNowExt,
    /// 3DNow!
    _3DNow,

    // cpuid(eax=80000001h).ecx
    /// `LAHF` in Long Mode.
    LAHF_LM,
    /// Hyperthreading not valid.
    CmpLegacy,
    /// Secure Virtual Machine.
    SVM,
    /// Extended APIC space.
    ExtApic,
    /// CR8 in 32-bit mode.
    Cr8Legacy,
    /// Advanced Bit Manipulation (`LZCNT` and `POPCNT`).
    AmbLzcnt,
    /// SSE4a.
    Sse4a,
    /// Misaligned SSE mode.
    MisalignSse,
    /// `PREFETCH` and `PREFETCHW` instructionsk.
    _3DNowPrefetch,
    /// OS Visible Workaround.
    OSVW,
    /// Instruction Based Sampling.
    IBS,
    /// XOP instruction set.
    XOP,
    /// `SKINIT/STGI` instructions.
    SKINIT,
    /// Watchdog timer.
    WDT,
    /// Light Weight Profiling.
    LWP = 0x8000_0000_0000,
    /// 4-operand Fused Multiply-Add.
    FMA4,
    /// Translation Cache Extension.
    TCE,
    /// NodeID MSR (C001_100C).
    NodeIdMsr,
    /// Trailing Bit Manipulation.
    TBM = 0x10_0000_0000,
    /// Topology extensions.
    TopoExt,
    /// Core performance counter extensions.
    PerfCtrCore,
    /// Northbridge performance coutner extensions.
    PerfCtrNb,
    /// Streaming Performnce Monitor Architecture.
    StreamPerfMon,
    /// Data breakpoint extensions.
    DBX,
    /// Performance timestamp counter (PTSC).
    PerfTSC,
    /// L2i Perf Counter eXtensions.
    PcxL2i,
    /// `MONITORX` and `MWAITX` instructions.
    MonitorX,
    /// Address mask extensions to 32 bits for instruction breakpoints.
    AddrMaskExt,
}

/// Extended fature flags
/// 
/// Upper: cpuid(eax=6,ecx=0).ecx << 32 | cpuid(eax=6,ecx=0).ebx
#[flags]
pub enum FeatureFlags2 {
    // cpuid(eax=7,ecx=0).ebx
    /// Access to base %fs and %gs.
    FsGsBase,
    /// `IA32_TSC_ADJUST` MSR.
    Ia32TscAdjust,
    /// Software Guard Extensions.
    SGX,
    /// Bit Manipulation Instructions set 1.
    BMI1,
    /// TSX Hardware Lock Elision.
    HLE,
    /// Advanced Vector Extension 2.
    AVX2,
    /// x87 FPU dat pointer register updated on exceptions only.
    FdpExceptnOnly,
    /// Supervisor Mode Execution Prevention.
    SMEP,
    /// Bit Manipulation Instructions set 2.
    BMI2,
    /// Enhanced `REP MOVSB/STOSB`.
    ERMS,
    /// `INVPCID` instruction.
    INVPCID,
    /// TSX Restricted Transactional Memory.
    RTM,
    /// Intel Resource Director (RDT) Monitoring or AMD Platform QOS Monitoring.
    RdtMPqm,
    /// X86 FPU CD and DS deprecated.
    X86FpuCsAndDsDepricated,
    /// Intel MPX (Memory Protection Extensions).
    MPX,
    /// Intel Resource Director (RDT) Allocation or AMD Platform QOS Enforcement.
    RdtAPqe,
    /// AVX-512 Foundation.
    AVX512F,
    /// AVX-512 Doubleword and Quadword instructions.
    AVX512DQ,
    /// `RDSEED` instruction.
    RDSEED,
    /// Intel ADX (Multi-precision Add-Cary instruction eXtensions).
    ADX,
    /// Supervisor Mode Access Prevention.
    SMAP,
    /// AVX512 Integer Fused Multiply-Add instructions.
    AVX512IFMA,
    /// `PCOMMIT` instruction (deprecated).
    PCOMMIT,
    /// `CLFLUSHOPT` instruction.
    ClFlushOpt,
    /// `CLWB`: Cash-Line WriteBack instruction.
    CLWB,
    /// Intel Processor Trace.
    PT,
    /// AVX-512 PreFetch instructions.
    AVX512PF,
    /// AVX-512 Exponential and Reciprocal instructions.
    AVX512ER,
    /// AVX-512 Coflict Detection instructions.
    AVX512CD,
    /// SHA-1 and SHA-256 extensions.
    SHA,
    /// AVX-512 Byte and Word instructions.
    AVX512BW,
    /// AVX-512 Vector Length instructions.
    AVX512VL,
    
    // cpuid(eax=7,ecx=0).ecx
    /// `PREFETCHWT1` instruction.
    PrefetchWT1,
    /// AVX-512 Vector Bit Manipulation Instructions.
    AVX512BVMI,
    /// User-Mode Instruction Prevention.
    UMIP,
    /// Memory Protection Keys for User-mode pages.
    PKU,
    /// PKU enabled by OS.
    OSPKE,
    /// Times paused and uer-level monitor/wait instructions (`TPAUSE`, `UMONITOR`, `UMWAIT`).
    WaitPkg,
    /// AVX-512 Vector Bit Manipulation Instructions 2.
    AVX512VBMI2,
    /// Control flow enforcement (`CET`): Shadow stack (`SHSTK` alternative name).
    CetSsShstk,
    /// Galois Field instructions.
    GFNI,
    /// Vector AES instruction set (VEX-256/EVEX).
    VAES,
    /// `CLMUL` instruction set (VEX-256/EVEX).
    VPCLMULQDQ,
    /// AVX-512 Vector Neural Networks Instructions.
    AVX512VNNI,
    /// AVX-512 BITALG instructions.
    AVX512BITALG,
    /// Total Memory Encryption MSRs available.
    TME_EN,
    /// AVX-512 Vector Population Count Double and Quad-word.
    AVX512VPOPCNTDQ,
    /// FZM.
    FZM,
    /// 5-level page (57 address bits).
    LA57,
    /// `RDPID` (Read Processor ID) instruction and IA32_TSC_AUX MSR.
    RDPID = 0x40_0000_0000_0000,
    /// AES Key Locker.
    KL,
    /// Bus lock debug exceptions.
    BusLockDetect,
    /// `CLDEMOTE` (Cache Line Demote) instruction.
    CLDEMOTE,
    /// MPRR.
    MPRR,
    /// `MOVEDIRI` instruction.
    MOVDIRI,
    /// `MOVDIR64B` (64-byte direct store) instructions.
    MOVDIR64B,
    /// Enqueue Stores and `EMQCMD/EMQCMDS` instructions.
    ENQCMD,
    /// SGX Launch Configurations.
    SGX_LC,
    /// Protection Keys for supervisor-mode pages.
    PKS,
}

/// Extended fature flags
/// 
/// Upper: cpuid(eax=7,ecx=1).eax << 32 | cpuid(eax=7,ecx=0).edx
#[flags]
pub enum FeatureFlags3 {
    // cpuid(eax=7,ecx=0).edx
    /// SGX_TEM.
    SGX_TEM,
    /// Attestation Services for Intel SGX.
    SGX_KEYS,
    /// AVX-512 4-register Neural Network Instructions.
    AVX512_4VNNIW,
    /// AVX-512 4-register Multiply Accumulation Signlee Precision.
    AVX512_4FMAPS,
    /// Fast Short `REP MOVSB`.
    FSRM,
    /// User Inter-preocess Interrupts.
    UINTR,
    /// AVX-512 Vector Intersection instructions on 32/64-bit integers.
    AVX512_VP2INTERSECT = 0x100,
    /// Special Register Buffer Data Sampling Mitigations.
    SRBDS_CTRL,
    /// `VERW` instruction clears CPU buffers.
    MD_CLEAR,
    /// All TSX transactions are aborted.
    RTM_ALWAYS_ABORT,
    /// `TSX_FORCE_ABORT` MSR is available.
    TSX_FORCE_ABORT = 0x1000,
    /// `SERIALIZE` instruction.
    Serialize,
    /// Mixture of CPU types in rocessor topology (e.g. Alder Lake).
    Hybrid,
    /// TSX load address tracking suspend/resumre instructions (`TSUSLDTRK` and `TRESLDTRK`).
    TSXLDTRK,
    /// Platform Configuration (Memory Encryption Technologies Instructions).
    PCONFIG = 0x4_0000,
    /// Architectural Last Branch Records.
    LBR,
    /// Control flow enforcement (CET): Indirect Branch Tracking.
    CET_IBT,
    /// AMX tile computation on bfloat16 numbers.
    AMX_BF16 = 0x40_0000,
    /// AVX-512 half-precision arithmetic instructons.
    AVX512_FP16,
    /// AMX tile load/store instuctions.
    AMX_TILE,
    /// AMX tile computation on 8-bit integers.
    AMX_INT8,
    /// Speculation Control, part of Indirect Branch Control (IBC): Indirect Branch Restricted Speculation (IBRS) and Indirect Brach Prediciton Barrier (IBPB).
    IbrsSpecCtrl,
    /// Single Thread Indreict Branch Predictor, part of IBC.
    STIBP,
    /// `IA32_FLUSH_CMD` MSR.
    L1dFlush,
    /// `IA32_ARCH_CAPABILITIES` MSR (lists speculative side channel migrations).
    IA32_ARCH_CAPABILITIES,
    /// `IA32_CORE_CAPABILITIES` MSR (lists model-specific core capabilities).
    IA32_CORE_CAPABILITIES,
    /// Speculative Store Bypass Disable, as mitigation for Speculative Store Bypass (IA32_SPEC_CTRL).
    SSBD,

    // cpuid(eax=7,ecx=1).eax
    /// SHA-512 extensions.
    SHA512,
    /// SM3 hash extensions.
    SM3,
    /// SM4 cipher extensions.
    SM4,
    /// Remote Atomic Operations on INTegers: `AADD`, `AAND`, `AOR`, and `AXOR` instructions.
    RAO_INT,
    /// AVX Vector Neural Network Instructions (VNNI) (VEX encoded).
    AVX_VNNI,
    /// AVX512 instructions for bfloat16 numbers.
    AVX512_BF16,
    /// Linear Address Space Separation (CR4 bit 27).
    LASS,
    /// `CMPccXASS` instructions.
    CMPCCXADD,
    /// Architectural Performance Monitoring Extended Lead (EAX=23h).
    ArchPerfMonExt,
    /// DEDUP.
    DEDUP,
    /// Fast zero-length `REP STOSB`.
    FZRM,
    /// Fast short `REP STOSB`.
    FSRS,
    /// Fast short `REP CMPSB` and `REP SCASB`.
    RSRCS,
    /// Flexible Return and Event Delivery.
    FRED = 0x200_0000_0000,
    /// `LKGS` instruction.
    LKGS,
    /// `WRMSRNS` instruction (non-serializing write to MSRs).
    WRMSRNS,
    /// AMX instructions for FP16 numbers.
    AMX_FP16 = 0x20_0000_0000_0000,
    /// `HRESET` instruction, `IA32_HRESET_ENABLE` (17DAh) MSR, and processor history reset leaf (EAX=20h).
    HRESET,
    /// `AVX IFMA instructions.
    AMX_IFMA,
    /// Linear Address Masking.
    LAM = 0x400_0000_0000_0000,
    /// `RDMSRLIST` and `WRMSRLIST` instructions, and the `IA32_BARRIER` (02Fh) MSR.
    MsrLists,
}

/// Extended fature flags
/// 
/// Upper: cpuid(eax=7,ecx=1).edx << 32 | cpuid(eax=7,ecx=0).ebx
#[flags]
pub enum FeatureFlags4 {
    // cpuid(eax=7,ecx=0).ebx
    /// Intel PPIN (Protected processor inventory number): `IA32_PPIN_CTL` (04Eh) and `IA32_PPIN` (04Fh).
    PPIN,
    /// Total storage encryption: `PBNDKB` instruction and `TSE_CAPABILITY` (9F1h) MSR.
    PBNDKB,

    // cpuid(eax=7,ecx=1).edx
    /// AVX VNNI INT8 instructions.
    AvxVnniInt8 = 0x10_0000_0000,
    /// AVX no-exception FP conversion instruction (bfloat16<->fp32 and fp16<->fp32).
    AvxNeConvert,
    /// AMX support for 'complex' tiles (`TCMMIMFP16PS` and `TCMMRLFP16PS`).
    AmxComplex = 0x100_0000_0000,
    /// AVX VNNI INT16 instructions.
    AvxVnniInt16 = 0x400_0000_0000,
    /// Instruction-cache prefetch instructions (`PREFETCHIT0` and `PREFETCHIT1`).
    PrefetchI = 0x4000_0000_0000,
    /// User-mode MSR access instructions (`USDMSR` and `UWRMSR`).
    UserMsr,
    /// `UIRET` (User Interrupt Return) instruction will set UIF (User Interrupt Flag) to the value of bit 1 of the RFLAGS image popped of the stack.
    UiretUifFromRflags,
    /// Control-flow enforcement (CET) supervisor shadow stack (SSS) are guaranteed not to become prematurely busy 
    /// as long as shadow stack switching does not cause page faults on the stack being switched to.
    CetSss,
    /// AVX10 converged vector ISA (see also leaf 24h).
    AVX10,
    /// Advanced Perforamnce Extnesions, Foundation (adds REX2 and extended EVEX prefix encoding to support 32 GPRs, as well as some new instructions).
    APX_F = 0x20_0000_0000_0000,
}

/// Extended fature flags
/// 
/// Upper: 0 << 32 | cpuid(eax=7,ecx=2).edx
#[flags]
pub enum FeatureFlags5 {
    /// Fast Store Forwarding Predictor disable supported (`SPEC_CTRL` (MSR 48h) bit 7).
    PSFD,
    /// IPRED_DIS controls supported (`SPEC_CTRL` bits 3 and 4).
    /// 
    /// IPRED_DIS prevetns instructions at an indirect branch target from speculatively executing until the branch address is resolved.
    IPredCtrl,
    /// RRSBA behavior disable supported (`SPEC_CTRL` bits 5 and 6).
    RrsbaCtrl,
    /// Data Dependent Prefetcher disable support (`SPEC_CTRL` bit 8).
    DdpdU,
    /// BHI_DIS_S behavior enable supported (`SPEC_CTRL` bits 10).
    /// 
    /// BHI_DIS_S prevents predicted targets of indirect branches executed in ring 0/1/2 from being selected based on branch history from branched executed in ring 3.
    BhiCtrl,
    /// The processor does not exhibit MXCSR configuration dependent timing.
    McdtNo,
    /// UC-lock disbale feature supported.
    UcLockDisable,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, EnumDisplay, EnumFromIndex)]
pub enum CacheTlbDescriptor {
    /// Null descriptor.
    #[display("Null descriptor")]
    #[default]
    Null                        = 0x00,
    /// Instruction TLB, 32 entries, 4k pages, 4-way associative.
    #[display("Instruction TLB, 32 entries, 4k pages, 4-way associative")]
    Itlb32e4kp4a                = 0x01,
    /// Instruction TLB, 2 entries, 4M pages, fully-associative.
    #[display("Instruction TLB, 2 entries, 4M pages, fully-associative")]
    Itlb2e4mpFa                 = 0x02,
    /// Data TLB, 64 entries, 4K pages, 4-way associative.
    #[display("Data TLB, 64 entries, 4K pages, 4-way associative")]
    Dtlb64e4kp4a                = 0x03,
    /// Data TLB, 8 entries, 4M pages, 4-way associative.
    #[display("Data TLB, 8 entries, 4M pages, 4-way associative")]
    Dtlb8e4mp4a                 = 0x04,
    /// Data TLB, 32 entries, 4M pages, 4-way associative.
    #[display("Data TLB, 32 entries, 4M pages, 4-way associative")]
    Dtlb32e4mp4a                = 0x05,
    /// L1 i$, 8K, 4-way associative, 32-byte cache lines.
    #[display("L1 i$, 8K, 4-way associative, 32-byte cache lines")]
    L1i8k4a32l                  = 0x06,
    /// L1 i$, 16K, 4-way associative, 32-byte cache lines.
    #[display("L1 i$, 16K, 4-way associative, 32-byte cache lines")]
    L1i16k4a32l                 = 0x08,
    /// L1 i$, 32K, 4-way associative, 64-byte cache lines.
    #[display("L1 i$, 32K, 4-way associative, 64-byte cache lines")]
    L1i32k4a64l                 = 0x09,
    /// L1 d$, 8K, 2-way associative, 32-byte cache lines.
    #[display("L1 d$, 8K, 2-way associative, 32-byte cache lines")]
    L1d8k2a32l                  = 0x0A,
    /// Instruction TLB, 4 entries, 4M pages, fully associative.
    #[display("Instruction TLB, 4 entries, 4M pages, fully associative")]
    Itlb4e4mpFa                 = 0x0B,
    /// L1 d$, 16K, 4-way associative, 32-byte cache lines.
    #[display("L1 d$, 16K, 4-way associative, 32-byte cache lines")]
    L1d16k4a32l                 = 0x0C,
    /// L1 d$, 16K, 4-way associative, 64-byte cache lines.
    #[display("L1 d$, 16K, 4-way associative, 64-byte cache lines")]
    L1d16k4a64l                 = 0x0D,
    /// L1 d$, 24K, 6-way associative, 64-byte cache lines.
    #[display("L1 d$, 24K, 6-way associative, 64-byte cache lines")]
    L1d24k6a64l                 = 0x0E,
    /// L1 d$, 16K, 4-way associative, 32-byte cache lines (documented for IA-32 operation of Itanium only).
    #[display("L1 d$, 16K, 4-way associative, 32-byte cache lines")]
    L1d16k4a32lIA32             = 0x10,
    /// L1 i$, 16K, 4-way associative, 32-byte cache lines (documented for IA-32 operation of Itanium only).
    #[display("L1 i$, 16K, 4-way associative, 32-byte cache lines")]
    L1i16k4a32lIA32             = 0x15,
    /// L2 $, 96K, 6-way associative, 64-byte cache lines (documented for IA-32 operation of Itanium only).
    #[display("L2 $, 96K, 6-way associative, 64-byte cache lines")]
    L2c96k6a64lIA32             = 0x1A,
    /// L2 $, 128K, 2-way associative, 64-byte cache lines.
    #[display("L2 $, 128K, 2-way associative, 64-byte cache lines")]
    L2c128k2a64l                = 0x1D,
    /// L2 $, 256K, 8-way associative, 64-byte cache lines.
    #[display("L2 $, 256K, 8-way associative, 64-byte cache lines")]
    L2c256k8a64l                = 0x21,
    /// L3 $, 512K, 4-way associative, 64-byte cache lines, 2 line cache sectors.
    #[display("L3 $, 512K, 4-way associative, 64-byte cache lines, 2 line cache sectors")]
    L3c512k4a64l2s              = 0x22,
    /// L3 $, 1M, 8-way associative, 64-byte cache lines, 2 line cache sectors.
    #[display("L3 $, 1M, 8-way associative, 64-byte cache lines, 2 line cache sectors")]
    L3c1m8a64l2s                = 0x23,
    /// L2 $, 1M, 16-way associative, 64-byte cache lines.
    #[display("L2 $, 1M, 16-way associative, 64-byte cache lines")]
    L2c1m16a64l                 = 0x24,
    /// L3 $, 2M, 8-way associative, 64-byte cache lines, 2 line cache sectors.
    #[display("L3 $, 2M, 8-way associative, 64-byte cache lines, 2 line cache sectors")]
    L4c2m8a64l2d                = 0x25,
    /// Not listed in intel documentation, but has been reported by windows.
    #[display("Not listed in intel documentation, but has been reported by windows: 26h")]
    Reported26h                 = 0x26,
    /// Not listed in intel documentation, but has been reported by windows.
    #[display("Not listed in intel documentation, but has been reported by windows: 27h")]
    Reported27h                 = 0x27,
    /// Not listed in intel documentation, but has been reported by windows.
    #[display("Not listed in intel documentation, but has been reported by windows: 28h")]
    Reported28h                 = 0x28,
    /// L3 $, 4M, 8-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L3 $, 4M, 8-way associative, 64-byte cache lines, 2 line cache sector")]
    L3c4m8a64l2s                = 0x29,
    /// L1 d$, 32K, 8-way associative, 64-byte cache lines.
    #[display("L1 d$, 32K, 8-way associative, 64-byte cache lines")]
    L1d32k8a64l                 = 0x2C,
    /// L1 i$, 32k, 8-way associative, 64-byte cache lines.
    #[display("L1 i$, 32k, 8-way associative, 64-byte cache lines")]
    L1i32k8a64l                 = 0x30,
    /// L2 $, 128k, 6-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 128k, 6-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c128k6a64l2s              = 0x39,
    /// L2 $, 192k, 6-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 192k, 6-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c192k6a64l2s              = 0x3A,
    /// L2 $, 128k, 2-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 128k, 2-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c128k2a64l2s              = 0x3B,
    /// L2 $, 256k, 4-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 256k, 4-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c256k4a64l2s              = 0x3C,
    /// L2 $, 384k, 6-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 384k, 6-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c384k6a64l2s              = 0x3D,
    /// L2 $, 512k, 4-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 512k, 4-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c512k4a64l2s              = 0x3E,
    /// No L3 $ present.
    #[display("No L3 $ present")]
    NoL3Present                 = 0x40,
    /// L2 $, 128k, 4-way associative, 32-byte cache lines, 2 line cache sector.
    #[display("L2 $, 128k, 4-way associative, 32-byte cache lines, 2 line cache sector")]
    L2c128k4a32l                = 0x41,
    /// L2 $, 256k, 4-way associative, 32-byte cache lines, 2 line cache sector.
    #[display("L2 $, 256k, 4-way associative, 32-byte cache lines, 2 line cache sector")]
    L2c256k4a32l                = 0x42,
    /// L2 $, 512k, 4-way associative, 32-byte cache lines, 2 line cache sector.
    #[display("L2 $, 512k, 4-way associative, 32-byte cache lines, 2 line cache sector")]
    L2c512k4a32l                = 0x43,
    /// L2 $, 1M, 4-way associative, 32-byte cache lines, 2 line cache sector.
    #[display("L2 $, 1M, 4-way associative, 32-byte cache lines, 2 line cache sector")]
    L2c1m4a32l                  = 0x44,
    /// L2 $, 2M, 4-way associative, 32-byte cache lines, 2 line cache sector.
    #[display("L2 $, 2M, 4-way associative, 32-byte cache lines, 2 line cache sector")]
    L2c2m4a32l                  = 0x45,
    /// L3 $, 4M, 4-way associative, 64-byte cache lines.
    #[display("L3 $, 4M, 4-way associative, 64-byte cache lines")]
    L3c4m4a64l                  = 0x46,
    /// L3 $, 8M, 8-way associative, 64-byte cache lines.
    #[display("L3 $, 8M, 8-way associative, 64-byte cache lines")]
    L3c8m8a64l                  = 0x47,
    /// L2 $, 3M, 12-way associative, 64-byte cache lines.
    #[display("L2 $, 3M, 12-way associative, 64-byte cache lines")]
    L2c3m12a64l                 = 0x48,
    /// L2/L3 $, 4M, 16-way associative, 64-byte cache lines.
    #[display("L2/L3 $, 4M, 16-way associative, 64-byte cache lines")]
    L23c4m16a64l                = 0x49,
    /// L3 $, 6M, 12-way associative, 64-byte cache lines.
    #[display("L3 $, 6M, 12-way associative, 64-byte cache lines")]
    L3c6m12a64l                 = 0x4A,
    /// L3 $, 8M, 16-way associative, 64-byte cache lines.
    #[display("L3 $, 8M, 16-way associative, 64-byte cache lines")]
    L3c8m16a64l                 = 0x4B,
    /// L3 $, 12M, 12-way associative, 64-byte cache lines.
    #[display("L3 $, 12M, 12-way associative, 64-byte cache lines")]
    L3c12m12a64l                = 0x4C,
    /// L3 $, 16M, 16-way associative, 64-byte cache lines.
    #[display("L3 $, 16M, 16-way associative, 64-byte cache lines")]
    L3c16m16a64l                = 0x4D,
    /// L3 $, 24M, 24-way associative, 64-byte cache lines.
    #[display("L3 $, 24M, 24-way associative, 64-byte cache lines")]
    L2c6m24a64l                 = 0x4E,
    /// Instruction TLB, 32 entries, 4K pages.
    #[display("Instruction TLB, 32 entries, 4K pages")]
    Itlb32e4kp                  = 0x4F,
    /// Instruction TLB, 64 entries, 4K/2M/4M pages.
    #[display("Instruction TLB, 64 entries, 4K/2M/4M pages")]
    Itlb64efa4k2m4mp            = 0x50,
    /// Instruction TLB, 128 entries, 4K/2M/4M pages.
    #[display("Instruction TLB, 128 entries, 4K/2M/4M pages")]
    Itlb128efa4k2m4mp           = 0x51,
    /// Instruction TLB, 256 entries, 4K/2M/4M pages.
    #[display("Instruction TLB, 256 entries, 4K/2M/4M pages")]
    Itlb256efa4k2m4mp           = 0x52,
    /// Instruction TLB, 7 entries, 2M/4M pages, fully-associative.
    #[display("Instruction TLB, 7 entries, 2M/4M pages, fully-associative")]
    Itlb7e2m4mpfa               = 0x55,
    /// Data TLB, 16 entries, 4M pages, 4-way associative.
    #[display("Data TLB, 16 entries, 4M pages, 4-way associative")]
    Dtlb16e4mp4a                = 0x56,
    /// Data TLB, 16 entries, 4K pages, 4-way associative.
    #[display("Data TLB, 16 entries, 4K pages, 4-way associative")]
    Dtlb16e4kp4a                = 0x57,
    /// Data TLB, 16 entries, 4K pages, fully-associative.
    #[display("Data TLB, 16 entries, 4K pages, fully-associative")]
    Dtlb16e4kpfa                = 0x59,
    /// Data TLB, 32 entries, 2M/4M pages, 4-way associative.
    #[display("Data TLB, 32 entries, 2M/4M pages, 4-way associative")]
    Dtlb32e2m4mp4a              = 0x5A,
    /// Data TLB, 64 entries, 4K/4M pages, fully-associative.
    #[display("Data TLB, 64 entries, 4K/4M pages, fully-associative")]
    Dtlb64e4k4mpfa              = 0x5B,
    /// Data TLB, 128 entries, 4K/4M pages, fully-associative.
    #[display("Data TLB, 128 entries, 4K/4M pages, fully-associative")]
    Dtlb128e4k4mpfa             = 0x5C,
    /// Data TLB, 256 entries, 4K/4M pages, fully-associative.
    #[display("Data TLB, 256 entries, 4K/4M pages, fully-associative")]
    Dtlb256e4k4mpfa             = 0x5D,
    /// L1 d$, 16K, 8-way associative, 64-byte cache lines.
    #[display("L1 d$, 16K, 8-way associative, 64-byte cache lines")]
    L1d16k8a64l                 = 0x60,
    /// Instruction TLB, 48 entries, 4K pages, fully-associative.
    #[display("Instruction TLB, 48 entries, 4K pages, fully-associative")]
    Itlb48e4kpfa                = 0x61,
    /// Two data TLBs, 32 entries, 2M/4M pages, 4-way associative + 4 entries, 1G pages, fully associative.
    #[display("Two data TLBs, 32 entries, 2M/4M pages, 4-way associative + 4 entries, 1G pages, fully associative")]
    TwoDtlbs32e2M4Mp4a4e1GpFa   = 0x63,
    /// Data TLB, 512 entries, 4K pages, 4-way associative.
    #[display("Data TLB, 512 entries, 4K pages, 4-way associative")]
    Dtlb512e4kp4a               = 0x64,
    /// L1 d$, 8K, 4-way associative, 64-byte cache lines.
    #[display("L1 d$, 8K, 4-way associative, 64-byte cache lines")]
    L1d8k4a64l                  = 0x66,
    /// L1 d$, 16K, 4-way associative, 64-byte cache lines.
    #[display("L1 d$, 16K, 4-way associative, 64-byte cache lines")]
    L1d16k4a64l2                = 0x67,
    /// L1 d$, 32K, 4-way associative, 64-byte cache lines.
    #[display("L1 d$, 32K, 4-way associative, 64-byte cache lines")]
    L1d32k4a64l                 = 0x68,
    /// Data TLB, 64 entries, 4K pages, 8-way associative.
    #[display("Data TLB, 64 entries, 4K pages, 8-way associative")]
    Dtlb64e4kp8a                = 0x6A,
    /// Data TLB, 256 entries, 4K pages, 8-way associative.
    #[display("Data TLB, 256 entries, 4K pages, 8-way associative")]
    Dtlb256e4kp8a               = 0x6B,
    /// Data TLB, 128 entries, 2M/4M pages, 8-way associative.
    #[display("Data TLB, 128 entries, 2M/4M pages, 8-way associative")]
    Dtlb128e2m4mp8a             = 0x6C,
    /// Data TLB, 16 entries, 1G pages, fully-associative.
    #[display("Data TLB, 16 entries, 1G pages, fully-associative")]
    Dtlb16e1gpfa                = 0x6D,
    /// Trace cache, 12k-uops, 8-way associative.
    #[display("Trace cache, 12k-uops, 8-way associative")]
    Trace12kuops8a              = 0x70,
    /// Trace cache, 16k-uops, 8-way associative.
    #[display("Trace cache, 16k-uops, 8-way associative")]
    Trace16kuops8a              = 0x71,
    /// Trace cache, 32k-uops, 8-way associative.
    #[display("Trace cache, 32k-uops, 8-way associative")]
    Trace32kuops8a              = 0x72,
    /// Trace cache, 64k-uops, 8-way associative.
    #[display("Trace cache, 64k-uops, 8-way associative")]
    Trace64kuops8a              = 0x73,
    /// Instruction TLB, 8 entries, 2M/4M pages, fully-associative.
    #[display("Instruction TLB, 8 entries, 2M/4M pages, fully-associative")]
    Itlb8e2m4mpfa               = 0x76,
    /// L1 i$, 16K, 4-way associative, 64-byte cache lines (documented for IA-32 operation of Itanium 2 only).
    #[display("L1 i$, 16K, 4-way associative, 64-byte cache lines")]
    L1i16k4a64lIA32             = 0x77,
    /// L2 $, 1M, 4-way associative, 64-byte cache lines.
    #[display("L2 $, 1M, 4-way associative, 64-byte cache lines")]
    L2c1m4a64l                  = 0x78,
    /// L2 $, 128K, 8-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 128K, 8-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c128k8a64l2s              = 0x79,
    /// L2 $, 256K, 8-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 256K, 8-way associative, 64-byte cache lines, 2 line cache sector")]
    L2c256k8a64l2s              = 0x7A,
    /// L2 $, 512K, 8-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 512K, 8-way associative, 64-byte cache lines, 2 line cache sector")]
    L2C512k8a64l2s              = 0x7B,
    /// L2 $, 1M, 8-way associative, 64-byte cache lines, 2 line cache sector.
    #[display("L2 $, 1M, 8-way associative, 64-byte cache lines, 2 line cache sector")]
    L2C1m8a64l2s                = 0x7C,
    /// L2 $, 1M, 8-way associative, 64-byte cache lines.
    #[display("L2 $, 1M, 8-way associative, 64-byte cache lines")]
    L2C2m8a64l                  = 0x7D,
    /// L2 $, 256K, 8-way associative, 128-byte cache lines (documented for IA-32 operation of Itanium 2 only).
    #[display("L2 $, 256K, 8-way associative, 128-byte cache lines")]
    L2c256k8a128lIA32           = 0x7E,
    /// L2 $, 512K, 2-way associative, 64-byte cache lines.
    #[display("L2 $, 512K, 2-way associative, 64-byte cache lines")]
    L2c512k2a64l                = 0x7F,
    /// L2 $, 512K, 8-way associative, 64-byte cache lines.
    #[display("L2 $, 512K, 8-way associative, 64-byte cache lines")]
    L2c512k8a64l                = 0x80,
    /// L2 $, 128K, 8-way associative, 32-byte cache lines (documented for IA-32 operation of Itanium 2 only).
    #[display("L2 $, 128K, 8-way associative, 32-byte cache lines")]
    L2c128k8a32lIA32            = 0x81,
    /// L2 $, 256K, 8-way associative, 32-byte cache lines.
    #[display("L2 $, 256K, 8-way associative, 32-byte cache lines")]
    L2c256k8a32l                = 0x82,
    /// L2 $, 512K, 8-way associative, 32-byte cache lines.
    #[display("L2 $, 512K, 8-way associative, 32-byte cache lines")]
    L2c512k8a32l                = 0x83,
    /// L2 $, 1M, 8-way associative, 32-byte cache lines.
    #[display("L2 $, 1M, 8-way associative, 32-byte cache lines")]
    L2c1m8a32l                  = 0x84,
    /// L2 $, 2M, 8-way associative, 32-byte cache lines.
    #[display("L2 $, 2M, 8-way associative, 32-byte cache lines")]
    L2c2m8a32l                  = 0x85,
    /// L2 $, 512K, 4-way associative, 64-byte cache lines.
    #[display("L2 $, 512K, 4-way associative, 64-byte cache lines")]
    L2c512k4a64l                = 0x86,
    /// L2 $, 1M, 8-way associative, 64-byte cache lines.
    #[display("L2 $, 1M, 8-way associative, 64-byte cache lines")]
    L2c1m8a64l                  = 0x87,
    /// L3 $, 2M, 4-way associative, 64-byte cache lines (documented for IA-32 operation of Itanium only).
    #[display("L3 $, 2M, 4-way associative, 64-byte cache lines")]
    L3c2m4a64lIA32              = 0x88,
    /// L3 $, 4M, 4-way associative, 64-byte cache lines (documented for IA-32 operation of Itanium only).
    #[display("L3 $, 4M, 4-way associative, 64-byte cache lines")]
    L3c4m4a64lIA32              = 0x89,
    /// L3 $, 8M, 4-way associative, 64-byte cache lines.
    #[display("L3 $, 8M, 4-way associative, 64-byte cache lines")]
    L3c8m4a64l                  = 0x8A,
    /// L3 $, 3M, 12-way associative, 128-byte cache lines (documented for IA-32 operation of Itanium 2 only).
    #[display("L3 $, 3M, 12-way associative, 128-byte cache lines")]
    L3c3m12a128l                = 0x8D,
    /// Instruction TLB, 64 entries, fully-associative, 4K/256M pages (documented for IA-32 operation of Itanium 2 only).
    #[display("Instruction TLB, 64 entries, fully-associative, 4K/256M pages")]
    Itlb64efa4k256Mp            = 0x90,
    /// Data TLB, 64 entries, fully-associative, 4K/256M pages (documented for IA-32 operation of Itanium 2 only).
    #[display("Data TLB, 64 entries, fully-associative, 4K/256M pages")]
    Dtlb64efa4k256Mp            = 0x96,
    /// Data TLB, 96 entries, fully-associative, 4K/256M pages (documented for IA-32 operation of Itanium 2 only).
    #[display("Data TLB, 96 entries, fully-associative, 4K/256M pages")]
    Dtlb96efa4k256Mp            = 0x9B,
    /// Data TLB, 32 entries, 4K pages, fully-associative.
    #[display("Data TLB, 32 entries, 4K pages, fully-associative")]
    Dtlb32e4kpfa                = 0xA0,
    /// Instruction TLB, 128 entries, 4K pages, 4-way associative.
    #[display("Instruction TLB, 128 entries, 4K pages, 4-way associative")]
    Itlb128e4kp4a               = 0xB0,
    /// Instruction TLB, 8 entries, 4K pages, 4-way associative.
    #[display("Instruction TLB, 8 entries, 4K pages, 4-way associative")]
    Itlb8e2m4mp4a               = 0xB1,
    /// Instruction TLB, 64 entries, 4K pages, 4-way associative.
    #[display("Instruction TLB, 64 entries, 4K pages, 4-way associative")]
    Itlb64e4kp4a                = 0xB2,
    /// Data TLB, 128 entries, 4K pages, 4-way associative.
    #[display("Data TLB, 128 entries, 4K pages, 4-way associative")]
    Dtlb128e4kp4a               = 0xB3,
    /// Data TLB, 256 entries, 4K pages, 4-way associative.
    #[display("Data TLB, 256 entries, 4K pages, 4-way associative")]
    Dtlb256e4kp4a               = 0xB4,
    /// Instruction TLB, 64 entries, 4K pages, 8-way associative.
    #[display("Instruction TLB, 64 entries, 4K pages, 8-way associative")]
    Itlb64e4kp8a                = 0xB5,
    /// Instruction TLB, 128 entries, 4K pages, 8-way associative.
    #[display("Instruction TLB, 128 entries, 4K pages, 8-way associative")]
    Itlb128e4kp8a               = 0xB6,
    /// Data TLB, 128 entries, 4K pages, 4-way associative.
    #[display("Data TLB, 128 entries, 4K pages, 4-way associative")]
    Dtlb64e4kp4a2               = 0xBA,
    /// Data TLB, 8 entries, 4K/4M pages, 4-way associative.
    #[display("Data TLB, 8 entries, 4K/4M pages, 4-way associative")]
    Dtlb8e4k4mp4a               = 0xC0,
    /// L2 TLB, 1024 entries, 4K/2M pages, 8-way associative.
    #[display("L2 TLB, 1024 entries, 4K/2M pages, 8-way associative")]
    L2tlb1024e4k2mp8a           = 0xC1,
    /// Data TLB, 16 entries, 2M/4M pages, 4-way associative.
    #[display("Data TLB, 16 entries, 2M/4M pages, 4-way associative")]
    Dtlb16e2m4mp4a              = 0xC2,
    /// Two L2 TLBs: 1536 entries, 4K/2M pages, 6-was associative + 16 entries, 1G pages, 4-way associative.
    #[display("Two L2 TLBs: 1536 entries, 4K/2M pages, 6-was associative + 16 entries, 1G pages, 4-way associative")]
    TwoL2tlb1536e4k2m6a16e1gp4a = 0xC3,
    /// Data TLB, 32 entries, 2M/4M pages, 4-way associative.
    #[display("Data TLB, 32 entries, 2M/4M pages, 4-way associative")]
    Dtlb32e2m4mp4a2             = 0xC4,
    /// L2 TLB, 512 entries, 4K pages, 4-way associative.
    #[display("L2 TLB, 512 entries, 4K pages, 4-way associative")]
    L2tlb512e4kp4a              = 0xCA,
    // L3 $, 512K, 4-way associative, 64-byte cache lines.
    #[display("L3 $, 512K, 4-way associative, 64-byte cache lines")]
    L3c512k4a64l                = 0xD0,
    /// L3 $, 1M, 4-way associative, 64-byte cache lines.
    #[display("L3 $, 1M, 4-way associative, 64-byte cache lines")]
    L3c1m4a64l                  = 0xD1,
    /// L3 $, 2M, 4-way associative, 64-byte cache lines.
    #[display("L3 $, 2M, 4-way associative, 64-byte cache lines")]
    L3c2m4a64l                  = 0xD2,
    /// L3 $, 1M, 8-way associative, 64-byte cache lines.
    #[display("L3 $, 1M, 8-way associative, 64-byte cache lines")]
    L3c1m8a64l                  = 0xD6,
    /// L3 $, 2M, 8-way associative, 64-byte cache lines.
    #[display("L3 $, 2M, 8-way associative, 64-byte cache lines")]
    L3c2m8a64l                  = 0xD7,
    /// L3 $, 4M, 8-way associative, 64-byte cache lines.
    #[display("L3 $, 4M, 8-way associative, 64-byte cache lines")]
    L3c4m8a64l                  = 0xD8,
    /// L3 $, 1.5M, 12-way associative, 64-byte cache lines.
    #[display("L3 $, 1.5M, 12-way associative, 64-byte cache lines")]
    L3c15m12a64l                = 0xDC,
    /// L3 $, 3M, 12-way associative, 64-byte cache lines.
    #[display("L3 $, 3M, 12-way associative, 64-byte cache lines")]
    L3c3m12a64l                 = 0xDD,
    /// L3 $, 6M, 12-way associative, 64-byte cache lines.
    #[display("L3 $, 6M, 12-way associative, 64-byte cache lines")]
    L3c6m12a64l2                = 0xDE,
    /// L3 $, 2M, 16-way associative, 64-byte cache lines.
    #[display("L3 $, 2M, 16-way associative, 64-byte cache lines")]
    L3c2m16a64l                 = 0xE2,
    /// L3 $, 4M, 16-way associative, 64-byte cache lines.
    #[display("L3 $, 4M, 16-way associative, 64-byte cache lines")]
    L3c4m16a64l                 = 0xE3,
    /// L3 $, 8M, 16-way associative, 64-byte cache lines.
    #[display("L3 $, 8M, 16-way associative, 64-byte cache lines")]
    L3c8m16a64l2                = 0xE4,
    /// L3 $, 12M, 24-way associative, 64-byte cache lines.
    #[display("L3 $, 12M, 24-way associative, 64-byte cache lines")]
    L3c12m24a64l                = 0xEA,
    /// L3 $, 18M, 24-way associative, 64-byte cache lines.
    #[display("L3 $, 18M, 24-way associative, 64-byte cache lines")]
    L3c18m24a64l                = 0xEB,
    /// L3 $, 24M, 24-way associative, 64-byte cache lines.
    #[display("L3 $, 24M, 24-way associative, 64-byte cache lines")]
    L3c24m24a64l                = 0xEC,
    /// 64-bit prefetch.
    #[display("64-bit prefetch")]
    Prefetch64                  = 0xF0,
    /// 128-bit prefetch.
    #[display("128-bit prefetch")]
    Prefetch128                  = 0xF1,
    /// Info in leaf 18h.
    #[display("Info in leaf 18h")]
    Leaf18                      = 0xFE,
    /// Info in leaf 4h.
    #[display("Info in leaf 4h")]
    Leaf4                       = 0xFF
}

/// Thermal & Power Management
#[flags]
pub enum ThermalPowerManagementFlags {
    /// Digital Thermal Sensor.
    DTS,
    /// Intel Turbo Boost technology.
    IntelTurboBoost,
    /// Always Running APIC Timer.
    ARAT,
    /// Power Limit Notification.
    PLN = 0x8,
    /// Extended Clock Modulation Duty.
    ECMD,
    /// Package Thermal Management.
    PTM,
    /// Hardware-controlled Perfomance states. MSRs added:
    /// - `IA32_PM_ENABLE` (770h)
    /// - `IA32_HWP_CAPBILITIES` (771h)
    /// - `IA32_HWP_REQUEST` (772h)
    /// - `IA32_HWP_STATUS` (773h)
    HWP,
    /// HWP notification of dyanmic guaranteed performance change - IA32_HWP_INTERRUPT (773h).
    HwpNotification,
    /// HWP Activity Window Control - bits 41:32 of IA32_HWP_REQUEST MSR.
    HwpActivityWindow,
    /// HWP Energy/performance preference control - bits 31:24 of IA_HWP_REQUEST MSR.
    HwpEnergyPerformancePreference,
    /// HWP Package-level control - IA32_HWP_REQUEST_PKG (772h) MSR.
    HwpPackageLevelRequest,
    /// Hardware Duty Cycling, MSRDs added:
    /// - `IA32_PKG_HDC_CTL` (D80h)
    /// - `IA32_PM_CTL1` (D81h)
    /// - `IA32_THREAD_STALL` (D82h)
    HDC = 0x1000,
    /// Intel Turbo Boost Max technology 3.0
    IntelTurboBoost3,
    /// Interrupts upon changes to `IA32_HWP_CAPABILITIES`: highest performance (bits 7:0).
    InterruptIA32,
    /// HWP PECI override - bits 63:60 of `IA32_HWP_PECI_REQUEST_INFO` (775h) MSR.
    HwpPeci,
    /// Flexible HWP - bits 63:59 of `IA32_HWP_REQUEST` MSR
    FlexibleHWP,
    /// Fast access mode for `IA32_WHP_REQUEST` MSR
    FastAccessMode,
    /// Hardware Feedback Interface. Added MSRs:
    /// - `IA32_HW_FEEDBACK_PTR` (17D0h)
    /// - `IA32_HW_FEEDBACK_CONFIG` (17D1h) (bit 0 enables HFI, bit 1 enabled Intel Thread Director)
    HwFeedback,
    /// `IA32_HWP_REQUEST` of idle logical processors ignoered when only oneof two logical processors tha share a physical processor is active.
    Ia32HwpRequest,
    /// `IA32_HWP_CTL` (776h)
    Ia32HwpCtl = 0x200000,
    /// Intel Thread Director. Added MSRs:
    /// - `IA32_THREAD_FEEDBACK_CHAR` (17D2h)
    /// - `IA32_HW_FEEDBACK_THREAD_CONFIG` (17D4h)
    IntelThreadDirector,
    /// `IA32_THERM_INTERRUPT` MSR bit 25.
    Ia32ThermInterrupt,

    // None `eax` flags
    /// Effective frequency interface. MSRs added:
    /// - `IA32_MPERF` (0E7h)
    /// - `IA32_APERF` (0E8h)
    EFI,
    /// ACNT2 Capability
    ACNT2,
    /// Performance Energy Bias, MSRs added:
    /// - `IA_ENERGY_PERF_VIAS` (1B0h)
    PEB,
    /// Hardware feedback reporting: Performance Capability Reporting supported
    HF_PC,
    /// Hardware feedback reporting: Efficiency Capability Reporting supported
    HF_EC,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ThermalPowerManagement {
    /// Thermal management flags
    pub flags: ThermalPowerManagementFlags,
    /// Number of interrupt thresholds in digital thermal sensor
    pub dts_thresholds: u8,
    /// Number of Intel Thread Directory classes supported by hardware
    pub thread_director_classes:       u8,
    /// Size of the Hardware Feeback interface structure (in 4K blocks)
    pub hardware_feedback_struct_size: u8,
}

impl fmt::Display for ThermalPowerManagement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Thermal & Power Management:");

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Flags:");
        indenter.set_spaces(8);
        writeln!(indenter, "{} Digital Thermal Sensor", if self.flags.contains(ThermalPowerManagementFlags::DTS) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Intel Turbo Boost technology", if self.flags.contains(ThermalPowerManagementFlags::IntelTurboBoost) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Always Running APIC Timer", if self.flags.contains(ThermalPowerManagementFlags::ARAT) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} ower Limit Notification", if self.flags.contains(ThermalPowerManagementFlags::PLN) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Extended Clock Modulation Duty", if self.flags.contains(ThermalPowerManagementFlags::ECMD) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Package Thermal Management", if self.flags.contains(ThermalPowerManagementFlags::PTM) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Hardware-controlled Perfomance states", if self.flags.contains(ThermalPowerManagementFlags::HWP) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} HWP notification of dyanmic guaranteed performance change - IA32_HWP_INTERRUPT", if self.flags.contains(ThermalPowerManagementFlags::HwpNotification) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} HWP Activity Window Control", if self.flags.contains(ThermalPowerManagementFlags::HwpActivityWindow) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} HWP Energy/performance preference control", if self.flags.contains(ThermalPowerManagementFlags::HwpEnergyPerformancePreference) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} HWP Package-level control - IA32_HWP_REQUEST_PKG", if self.flags.contains(ThermalPowerManagementFlags::HwpPackageLevelRequest) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Hardware Duty Cycling", if self.flags.contains(ThermalPowerManagementFlags::HDC) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Intel Turbo Boost Max technology 3.0", if self.flags.contains(ThermalPowerManagementFlags::IntelTurboBoost3) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Interrupts upon changes to `IA32_HWP_CAPABILITIES`", if self.flags.contains(ThermalPowerManagementFlags::InterruptIA32) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} HWP PECI override", if self.flags.contains(ThermalPowerManagementFlags::HwpPeci) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Flexible HWP", if self.flags.contains(ThermalPowerManagementFlags::FlexibleHWP) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Fast access mode for `IA32_WHP_REQUEST` MSR", if self.flags.contains(ThermalPowerManagementFlags::FastAccessMode) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Hardware Feedback Interface", if self.flags.contains(ThermalPowerManagementFlags::HwFeedback) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} `IA32_HWP_REQUEST` of idle logical processors ignoered when only oneof two logical processors tha share a physical processor is active", if self.flags.contains(ThermalPowerManagementFlags::Ia32HwpRequest) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} `IA32_HWP_CTL`", if self.flags.contains(ThermalPowerManagementFlags::Ia32HwpCtl) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} Intel Thread Director", if self.flags.contains(ThermalPowerManagementFlags::IntelThreadDirector) { "[X]" } else { "[ ]" });
        writeln!(indenter, "{} `IA32_THERM_INTERRUPT` MSR bit 25", if self.flags.contains(ThermalPowerManagementFlags::Ia32ThermInterrupt) { "[X]" } else { "[ ]" });
        indenter.set_spaces(4);

        writeln!(indenter, "Number of Digital Thermal Sensor thresholds:       {}", self.dts_thresholds);
        writeln!(indenter, "Number of supported Intel Thread Director classes: {}", self.thread_director_classes);
        writeln!(indenter, "Size of the Hardware Feedback interface structure: {} x 4KiB", self.hardware_feedback_struct_size);

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CpuidInfo {
    /// CPU manufacturer
    pub manufacturer:            Manufacturer,
    /// CPU familty info
    pub family_info:             FamilyInfo,
    /// Additional feature info
    pub additional_feature_info: AdditionalFeatureInfo,
    /// Feature flags
    pub feature_flags:           FeatureFlags,
    /// Feature flags
    pub extended_feature_flags:  ExtendedFeatureFlags,
    /// Feature flags 2
    pub feature_flags2:          FeatureFlags2,
    /// Feature flags 3
    pub feature_flags3:          FeatureFlags3,
    /// Feature flags 4
    pub feature_flags4:          FeatureFlags4,
    /// Feature flags 5
    pub feature_flags5:          FeatureFlags5,
    /// Cache/TLB descriptors
    pub cache_tlb_descriptors:   Option<[CacheTlbDescriptor; 15]>,
    /// Thermal power management
    pub thermal_power_mgmt:      ThermalPowerManagement,
    /// MAWAU bits: The value of userspace MPC Address-Width Adjust used by the `BNDLSX` and `BNDSTX` Intel MPX instruction in 64-bit mode.
    pub mpx_addr_width_adjust:   u8,
}

impl CpuidInfo {
    pub fn get() -> Self {
        #[cfg(target_arch = "x86_64")]
        return unsafe { get_features() };

        #[cfg(not(target_arch = "x86_64"))]
        CpuidInfo::default()
    }
}

impl fmt::Display for CpuidInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CPUID info:")?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Manufacturer:  {}", self.manufacturer)?;
        writeln!(indenter, "{}", self.family_info)?;
        writeln!(indenter, "Brand index:   {}", self.additional_feature_info.brand_index)?;
        writeln!(indenter, "Local APIC ID: {}", self.additional_feature_info.local_apic_id)?;

        writeln!(indenter, "Features:")?;
        indenter.set_spaces(8);
        writeln!(indenter, "CPUID(EAX=1).EDX:")?;
        writeln!(indenter, "{} fpu                   : Onboard x87 FPU", if self.feature_flags.contains(FeatureFlags::FPU) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} vme                   : Virtual 8086 mode extensions (such as VIF, VIP, and PVI)", if self.feature_flags.contains(FeatureFlags::VME) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} de                    : Debugging Extensions (`CR4` bit 3)", if self.feature_flags.contains(FeatureFlags::DE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pse                   : Page Size Extensions (4MiB pages)", if self.feature_flags.contains(FeatureFlags::PSE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tsc                   : Time Stamp Counter", if self.feature_flags.contains(FeatureFlags::TSC) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} msr                   : Model-specfic registers and RDMSR/WRMSR instuctions", if self.feature_flags.contains(FeatureFlags::MSR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pea                   : Physical Address Extension", if self.feature_flags.contains(FeatureFlags::PAE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mce                   : Machine Check Exception", if self.feature_flags.contains(FeatureFlags::MCE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cx8                   : `CMPXCHG8B`` (compare-and-swap) instruction", if self.feature_flags.contains(FeatureFlags::CX8) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} apic                  : Onboard Advanced Programmable Interrupt Controller", if self.feature_flags.contains(FeatureFlags::APIC) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sep                   : `SYSENTER` and `SYSEXIT` fast system call instructions", if self.feature_flags.contains(FeatureFlags::SEP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mtrr                  : Memory Type Range Registers", if self.feature_flags.contains(FeatureFlags::MTRR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pge                   : Page Global Enable bit in `CR4`", if self.feature_flags.contains(FeatureFlags::PGE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mca                   : Machine Check Architecture", if self.feature_flags.contains(FeatureFlags::MCA) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cmov                  : Conditional move `CMOV`, `FCMOV`, and `FCOMI` instructions", if self.feature_flags.contains(FeatureFlags::CMOV) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pat                   : Page Attribute Table", if self.feature_flags.contains(FeatureFlags::PAT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pse-36                : 36-bit Page Size Extension", if self.feature_flags.contains(FeatureFlags::PSE36) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} psn                   : Processor Serial Number supported and enabled", if self.feature_flags.contains(FeatureFlags::PSN) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} clfsh                 : `CLFLUSH` cache line flush instruction (SSE2)", if self.feature_flags.contains(FeatureFlags::CLFSH) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} (nx)                  : No-execute (NX) bit", if self.feature_flags.contains(FeatureFlags::NX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} dx                    : Debug store: save trace of executed jump", if self.feature_flags.contains(FeatureFlags::DS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} acpi                  : Onboard thermal control MSRs ofor ACPI", if self.feature_flags.contains(FeatureFlags::ACPI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mmx                   : MMX instructions (64-bit SIMD)", if self.feature_flags.contains(FeatureFlags::MMX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fxsr                  : `FXSAVE` and `FXRSTOR` instructions, `CR4` bit 9", if self.feature_flags.contains(FeatureFlags::FXSR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sse                   : Streaming SIMD Extensions (SSE) instructions (aka \"Katmai New Instrucitons\", 128-bit SIMD)", if self.feature_flags.contains(FeatureFlags::SSE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sse2                  : SSE2 instructions", if self.feature_flags.contains(FeatureFlags::SSE2) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ss                    : CPU cache implements self-snoop", if self.feature_flags.contains(FeatureFlags::SS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} htt                   : Max APIC IDs reserved field is valid", if self.feature_flags.contains(FeatureFlags::HTT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tm                    : Thermal monitor automatically limits temperature", if self.feature_flags.contains(FeatureFlags::TM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ia64                  : IA64 processor emulating x86", if self.feature_flags.contains(FeatureFlags::IA64) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pbe                   : Pending Break Enable (PBE# pin) wakeup capability", if self.feature_flags.contains(FeatureFlags::PBE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "CPUID(EAX=1).ECX:")?;
        writeln!(indenter, "{} sse3                  : SSE3 (Prescott new instructions - PNI)", if self.feature_flags.contains(FeatureFlags::SSE3) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pclmulqdq             : `PCLMULQDQ` (carry-less multiply) instuctions", if self.feature_flags.contains(FeatureFlags::PCLMULQDQ) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} dtex64                : 64-bit debug store (`edx` bit 21)", if self.feature_flags.contains(FeatureFlags::DTES64) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} monitor               : `MONITOR` and `MWAIT` instructions (PNI)", if self.feature_flags.contains(FeatureFlags::Monitor) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ds-cpl                : CPL qualified debug store", if self.feature_flags.contains(FeatureFlags::DS_CPL) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} vmx                   : Virtual Machine eXtensions", if self.feature_flags.contains(FeatureFlags::VMX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} smx                   : Safer Mode eXtensions (LaGrance) (`GETSEC` instruction)", if self.feature_flags.contains(FeatureFlags::SMX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} est                   : Enhanced SpeedStep", if self.feature_flags.contains(FeatureFlags::EST) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tm2                   : Thermal Monitor 2", if self.feature_flags.contains(FeatureFlags::TM2) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ssse3                 : Supplemental SSE3 instructions", if self.feature_flags.contains(FeatureFlags::SSSE3) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cnxt-id               : L1 Context ID", if self.feature_flags.contains(FeatureFlags::CnxtId) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sdbg                  : Silicon Debug Interface", if self.feature_flags.contains(FeatureFlags::SDBG) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fma                   : Fused Multiply-Add (FMA3)", if self.feature_flags.contains(FeatureFlags::FMA) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cs16                  : CMPXCHG12B` instruction", if self.feature_flags.contains(FeatureFlags::CX16) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} xtpr                  : Can disable sending task priority messages", if self.feature_flags.contains(FeatureFlags::XTPR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pdcm                  : Perfmon & debug capability", if self.feature_flags.contains(FeatureFlags::PDCM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pcid                  : Process context identifiers (`CR4` it 17)", if self.feature_flags.contains(FeatureFlags::PCID) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} dca                   : Direct Cache Access for DMA writes", if self.feature_flags.contains(FeatureFlags::DCA) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sse4.1                : SSE 4.1 instructions", if self.feature_flags.contains(FeatureFlags::SSE41) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sse4.2                : SSE 4.2 instructions", if self.feature_flags.contains(FeatureFlags::SSE42) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} x2apic                : x2APIC (enhanced APIC)", if self.feature_flags.contains(FeatureFlags::X2APIC) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} movbe                 : `MOVBE` instruction (big-endian)", if self.feature_flags.contains(FeatureFlags::MOVBE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} popcnt                : `POPCNT` instruction", if self.feature_flags.contains(FeatureFlags::POPCNT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tsc-deadline          : APIC implements one-shot operation usig a TSC deadline value", if self.feature_flags.contains(FeatureFlags::TscDeadline) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} aes-ni                : AES instruction set", if self.feature_flags.contains(FeatureFlags::AES_NI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} xsave                 : eXtensible processor state save/restore: `XSAVE`, `XRSTOR`, `XSETBV`, and `XGETBV` instructions", if self.feature_flags.contains(FeatureFlags::XSAVE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} osxsave               : `XSAVE` enabled by the OS", if self.feature_flags.contains(FeatureFlags::OSXSAVE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avs                   : Advanced Vector Extensions (256-bit SIMD)", if self.feature_flags.contains(FeatureFlags::AVX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} f16c                  : Floating point conversion instruction to/from FP16 format", if self.feature_flags.contains(FeatureFlags::F16C) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rdrnd                 : `RDRND` (on-chip random number generator) feature", if self.feature_flags.contains(FeatureFlags::RDRND) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} hypervisor            : Hypervisor present (always zero on physical CPUs)", if self.feature_flags.contains(FeatureFlags::Hypervisor) { "[X]" } else { "[ ]" })?;
        
        writeln!(indenter, "CPUID(EAX=80000001h).EDX (duplicate flags skipped):")?;
        writeln!(indenter, "{} syscall (K6)          : `SYSCALL/SYSRET` (K6 only)", if self.extended_feature_flags.contains(ExtendedFeatureFlags::SyscallK6) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} syscall               : `SYSCALL/SYSRET`", if self.extended_feature_flags.contains(ExtendedFeatureFlags::Syscall) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ecc                   : 'Athlon MP`/`Sempron` CPU brand identification", if self.extended_feature_flags.contains(ExtendedFeatureFlags::ECC) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} nx                    : NX (no-execute) bit", if self.extended_feature_flags.contains(ExtendedFeatureFlags::NX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mmxext                : Extended MMX", if self.extended_feature_flags.contains(ExtendedFeatureFlags::MmxExt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fxsr_opt              : `FXSAVE` and `FXSTOR` optimizations", if self.extended_feature_flags.contains(ExtendedFeatureFlags::FxsrOpt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pdpe1gb               : Gigabyte pages", if self.extended_feature_flags.contains(ExtendedFeatureFlags::PDPE1GB) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rdtscp                : `RDTSCP` instruction", if self.extended_feature_flags.contains(ExtendedFeatureFlags::RDTSCP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} lm                    : Long Mode", if self.extended_feature_flags.contains(ExtendedFeatureFlags::LM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} 3dnowext              : Extended 3DNow", if self.extended_feature_flags.contains(ExtendedFeatureFlags::_3DNowExt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} 3dnow                 : 3DNow!", if self.extended_feature_flags.contains(ExtendedFeatureFlags::_3DNow) { "[X]" } else { "[ ]" })?;
        
        writeln!(indenter, "CPUID(EAX=80000001h).ECX:")?;
        writeln!(indenter, "{} lahf_lm               : `LAHF` in Long Mode", if self.extended_feature_flags.contains(ExtendedFeatureFlags::LAHF_LM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cmp_legacy            : Hyperthreading not valid", if self.extended_feature_flags.contains(ExtendedFeatureFlags::CmpLegacy) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} svm                   : Secure Virtual Machine", if self.extended_feature_flags.contains(ExtendedFeatureFlags::SVM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} extapic               : Extended APIC space", if self.extended_feature_flags.contains(ExtendedFeatureFlags::ExtApic) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cr8_legacy            : CR8 in 32-bit mode", if self.extended_feature_flags.contains(ExtendedFeatureFlags::Cr8Legacy) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} abm/lzcnt             : Advanced Bit Manipulation (`LZCNT` and `POPCNT`)", if self.extended_feature_flags.contains(ExtendedFeatureFlags::AmbLzcnt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sse4a                 : SSE4a", if self.extended_feature_flags.contains(ExtendedFeatureFlags::Sse4a) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} misalignsse           : Misaligned SSE mode", if self.extended_feature_flags.contains(ExtendedFeatureFlags::MisalignSse) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} 3dnowprefetch         : PREFETCH` and `PREFETCHW` instructions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::_3DNowPrefetch) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} osvw                  : OS Visible Workaround", if self.extended_feature_flags.contains(ExtendedFeatureFlags::OSVW) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ibs                   : Instruction Based Sampling", if self.extended_feature_flags.contains(ExtendedFeatureFlags::IBS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} xop                   : XOP instruction set", if self.extended_feature_flags.contains(ExtendedFeatureFlags::XOP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} skinit                : `SKINIT/STGI` instructions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::SKINIT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} wdt                   : Watchdog timer", if self.extended_feature_flags.contains(ExtendedFeatureFlags::WDT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} lwp                   : Light Weight Profiling", if self.extended_feature_flags.contains(ExtendedFeatureFlags::FMA4) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fma4                  : 4-operand Fused Multiply-Add", if self.extended_feature_flags.contains(ExtendedFeatureFlags::FMA4) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tce                   :Translation Cache Extension ", if self.extended_feature_flags.contains(ExtendedFeatureFlags::TCE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} nodeid_msr            : NodeID MSR (C001_100C)", if self.extended_feature_flags.contains(ExtendedFeatureFlags::NodeIdMsr) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tbm                   : Trailing Bit Manipulation", if self.extended_feature_flags.contains(ExtendedFeatureFlags::TBM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} topoext               : Topology extensions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::TopoExt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} perfctr_core          : Core performance counter extensions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::PerfCtrCore) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} perctr_nb             : Northbridge performance coutner extensions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::PerfCtrNb) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} StreamPerfMon         : Streaming Performnce Monitor Architecture", if self.extended_feature_flags.contains(ExtendedFeatureFlags::StreamPerfMon) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} dbx                   : Data breakpoint extensions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::DBX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} perftcs               : Performance timestamp counter (PTSC)", if self.extended_feature_flags.contains(ExtendedFeatureFlags::PerfTSC) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pcx_l2i               : L2i Perf Counter eXtensions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::PcxL2i) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} monitorx              : `MONITORX` and `MWAITX` instructions", if self.extended_feature_flags.contains(ExtendedFeatureFlags::MonitorX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} addr_mask_ext         : Address mask extensions to 32 bits for instruction breakpoints.", if self.extended_feature_flags.contains(ExtendedFeatureFlags::AddrMaskExt) { "[X]" } else { "[ ]" })?;
        
        writeln!(indenter, "CPUID(EAX=7,ECX=0).EBX:");
        writeln!(indenter, "{} fsgsbase              : Access to base %fs and %gs", if self.feature_flags2.contains(FeatureFlags2::FsGsBase) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{}                       : IA32_TSC_ADJUST` MSR", if self.feature_flags2.contains(FeatureFlags2::Ia32TscAdjust) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sgx                   : Software Guard Extensions", if self.feature_flags2.contains(FeatureFlags2::SGX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} bmi1                  : Bit Manipulation Instructions set 1", if self.feature_flags2.contains(FeatureFlags2::BMI1) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} hle                   : TSX Hardware Lock Elision", if self.feature_flags2.contains(FeatureFlags2::HLE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx2                  : Advanced Vector Extension 2", if self.feature_flags2.contains(FeatureFlags2::AVX2) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fdp-excptn-only       : x87 FPU dat pointer register updated on exceptions only", if self.feature_flags2.contains(FeatureFlags2::FdpExceptnOnly) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} smep                  : Supervisor Mode Execution Prevention", if self.feature_flags2.contains(FeatureFlags2::SMEP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} bmi2                  : Bit Manipulation Instructions set 2", if self.feature_flags2.contains(FeatureFlags2::BMI2) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} erms                  : Enhanced `REP MOVSB/STOSB`", if self.feature_flags2.contains(FeatureFlags2::ERMS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} invpcid               : `INVPCID` instruction", if self.feature_flags2.contains(FeatureFlags2::INVPCID) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rtm                   : TSX Restricted Transactional Memory", if self.feature_flags2.contains(FeatureFlags2::RTM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rdt-m/pqm             : Intel Resource Director (RDT) Monitoring or AMD Platform QOS Monitoring", if self.feature_flags2.contains(FeatureFlags2::RdtMPqm) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{}                       : X86 FPU CD and DS deprecated", if self.feature_flags2.contains(FeatureFlags2::X86FpuCsAndDsDepricated) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mpx                   : Intel MPX (Memory Protection Extensions)", if self.feature_flags2.contains(FeatureFlags2::MPX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rdt-a/pqe             : Intel Resource Director (RDT) Allocation or AMD Platform QOS Enforcement", if self.feature_flags2.contains(FeatureFlags2::RdtMPqm) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-f              : AVX-512 Foundation", if self.feature_flags2.contains(FeatureFlags2::AVX512F) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-dq             : AVX-512 Doubleword and Quadword instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512DQ) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rdseed                : `RDSEED` instruction", if self.feature_flags2.contains(FeatureFlags2::RDSEED) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} adx                   : Intel ADX (Multi-precision Add-Cary instruction eXtensions)", if self.feature_flags2.contains(FeatureFlags2::ADX) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} smap                  : Supervisor Mode Access Prevention", if self.feature_flags2.contains(FeatureFlags2::SMAP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-idma           : AVX512 Integer Fused Multiply-Add instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512IFMA) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} (pcommit)             : PCOMMIT` instruction (deprecated)", if self.feature_flags2.contains(FeatureFlags2::PCOMMIT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} clflushopt            : CLFLUSHOPT` instruction", if self.feature_flags2.contains(FeatureFlags2::ClFlushOpt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} clwb                  : `CLWB`: Cash-Line WriteBack instruction", if self.feature_flags2.contains(FeatureFlags2::CLWB) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pt                    : Intel Processor Trace", if self.feature_flags2.contains(FeatureFlags2::PT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-pf             : AVX-512 PreFetch instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512PF) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-er             : AVX-512 Exponential and Reciprocal instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512ER) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-cd             : AVX-512 Coflict Detection instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512CD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sha                   : SHA-1 and SHA-256 extensions", if self.feature_flags2.contains(FeatureFlags2::SHA) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-bw             : AVX-512 Byte and Word instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512BW) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-vl             : AVX-512 Vector Length instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512VL) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "CPUID(EAX=7,ECX=0).ECX:");
        writeln!(indenter, "{} prefetchwt1           : `PREFETCHWT1` instruction", if self.feature_flags2.contains(FeatureFlags2::PrefetchWT1) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-vbmi           : AVX-512 Vector Bit Manipulation Instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512BVMI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} umip                  : User-Mode Instruction Prevention", if self.feature_flags2.contains(FeatureFlags2::UMIP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pku                   : Memory Protection Keys for User-mode pages", if self.feature_flags2.contains(FeatureFlags2::PKU) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ospku                 : PKU enabled by OS", if self.feature_flags2.contains(FeatureFlags2::OSPKE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} waitpkg               : Times paused and uer-level monitor/wait instructions (`TPAUSE`, `UMONITOR`, `UMWAIT`)", if self.feature_flags2.contains(FeatureFlags2::WaitPkg) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-vbmi2          : AVX-512 Vector Bit Manipulation Instructions 2", if self.feature_flags2.contains(FeatureFlags2::AVX512VBMI2) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cet_ss/shstk          : Control flow enforcement (`CET`): Shadow stack (`SHSTK` alternative name)", if self.feature_flags2.contains(FeatureFlags2::CetSsShstk) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} gfni                  : Galois Field instructions", if self.feature_flags2.contains(FeatureFlags2::GFNI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} vaes                  : Vector AES instruction set (VEX-256/EVEX)", if self.feature_flags2.contains(FeatureFlags2::VAES) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} vpclmulqdq            : `CLMUL` instruction set (VEX-256/EVEX)", if self.feature_flags2.contains(FeatureFlags2::VPCLMULQDQ) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-vnni           : AVX-512 Vector Neural Networks Instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512VNNI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-bitalg         : AVX-512 BITALG instructions", if self.feature_flags2.contains(FeatureFlags2::AVX512BITALG) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tme_en                : Total Memory Encryption MSRs available", if self.feature_flags2.contains(FeatureFlags2::TME_EN) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-vpopcntdq      : AVX-512 Vector Population Count Double and Quad-word", if self.feature_flags2.contains(FeatureFlags2::AVX512VPOPCNTDQ) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fzm                   : FZM", if self.feature_flags2.contains(FeatureFlags2::FZM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} la57                  : 5-level page (57 address bits)", if self.feature_flags2.contains(FeatureFlags2::LA57) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rdpid                 : `RDPID` (Read Processor ID) instruction and IA32_TSC_AUX MSR", if self.feature_flags2.contains(FeatureFlags2::RDPID) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} kl                    : AES Key Locker", if self.feature_flags2.contains(FeatureFlags2::KL) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} bus-lock-detect       : Bus lock debug exceptions", if self.feature_flags2.contains(FeatureFlags2::BusLockDetect) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cidemote              : `CLDEMOTE` (Cache Line Demote) instruction", if self.feature_flags2.contains(FeatureFlags2::CLDEMOTE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mprr                  : MPRR", if self.feature_flags2.contains(FeatureFlags2::MPRR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} movdiri               : `MOVEDIRI` instruction", if self.feature_flags2.contains(FeatureFlags2::MOVDIRI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} movdir64b             : `MOVDIR64B` (64-byte direct store) instructions", if self.feature_flags2.contains(FeatureFlags2::MOVDIR64B) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} enqcmd                : Enqueue Stores and `EMQCMD/EMQCMDS` instructions", if self.feature_flags2.contains(FeatureFlags2::ENQCMD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sgx-lc                : SGX Launch Configurations", if self.feature_flags2.contains(FeatureFlags2::SGX_LC) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pks                   : Protection Keys for supervisor-mode pages", if self.feature_flags2.contains(FeatureFlags2::PKS) { "[X]" } else { "[ ]" })?;

        writeln!(indenter, "CPUID(EAX=7,ECX=0).EDX:");
        writeln!(indenter, "{} sgx-tem               : sgx-tem", if self.feature_flags3.contains(FeatureFlags3::SGX_TEM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sgx-keys              : Attestation Services for Intel SGX", if self.feature_flags3.contains(FeatureFlags3::SGX_KEYS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-4vnniw         : AVX-512 4-register Neural Network Instructions", if self.feature_flags3.contains(FeatureFlags3::AVX512_4VNNIW) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-4fmaps         : AVX-512 4-register Multiply Accumulation Signlee Precision", if self.feature_flags3.contains(FeatureFlags3::AVX512_4FMAPS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fsrm                  : Fast Short `REP MOVSB`", if self.feature_flags3.contains(FeatureFlags3::FSRM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} uintr                 : User Inter-preocess Interrupts", if self.feature_flags3.contains(FeatureFlags3::UINTR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-vp4intersect   : AVX-512 Vector Intersection instructions on 32/64-bit integers", if self.feature_flags3.contains(FeatureFlags3::AVX512_VP2INTERSECT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} srbds-ctrl            : Special Register Buffer Data Sampling Mitigations", if self.feature_flags3.contains(FeatureFlags3::SRBDS_CTRL) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} md-clear              : `VERW` instruction clears CPU buffers", if self.feature_flags3.contains(FeatureFlags3::MD_CLEAR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rmt-always-abort      : All TSX transactions are aborted", if self.feature_flags3.contains(FeatureFlags3::RTM_ALWAYS_ABORT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{}                       : `TSX_FORCE_ABORT` MSR is available", if self.feature_flags3.contains(FeatureFlags3::TSX_FORCE_ABORT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} serialize             : `SERIALIZE` instruction", if self.feature_flags3.contains(FeatureFlags3::Serialize) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} hybrid                : Mixture of CPU types in rocessor topology (e.g. Alder Lake)", if self.feature_flags3.contains(FeatureFlags3::Hybrid) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} tscldtrk              : TSX load address tracking suspend/resumre instructions (`TSUSLDTRK` and `TRESLDTRK`)", if self.feature_flags3.contains(FeatureFlags3::TSXLDTRK) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pconfig               : Platform Configuration (Memory Encryption Technologies Instructions)", if self.feature_flags3.contains(FeatureFlags3::PCONFIG) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} lbr                   : Architectural Last Branch Records", if self.feature_flags3.contains(FeatureFlags3::LBR) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} det-ibt               : Control flow enforcement (CET): Indirect Branch Tracking", if self.feature_flags3.contains(FeatureFlags3::CET_IBT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} amx-bf16              : AMX tile computation on bfloat16 numbers", if self.feature_flags3.contains(FeatureFlags3::AMX_BF16) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-fp16           : AVX-512 half-precision arithmetic instructons", if self.feature_flags3.contains(FeatureFlags3::AVX512_FP16) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} amx-tile              : AMX tile load/store instuctions", if self.feature_flags3.contains(FeatureFlags3::AMX_TILE) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} amx-int8              : AMX tile computation on 8-bit integers", if self.feature_flags3.contains(FeatureFlags3::AMX_INT8) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ibrs/spec_ctrl        : Speculation Control, part of Indirect Branch Control (IBC): Indirect Branch Restricted Speculation (IBRS) and Indirect Brach Prediciton Barrier (IBPB)", if self.feature_flags3.contains(FeatureFlags3::IbrsSpecCtrl) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} stibp                 : Single Thread Indreict Branch Predictor, part of IBC", if self.feature_flags3.contains(FeatureFlags3::STIBP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} L1D_FLUSH             : `IA32_FLUSH_CMD` MSR", if self.feature_flags3.contains(FeatureFlags3::L1dFlush) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{}                       : `IA32_ARCH_CAPABILITIES` MSR (lists speculative side channel migrations)", if self.feature_flags3.contains(FeatureFlags3::IA32_ARCH_CAPABILITIES) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{}                       : `IA32_CORE_CAPABILITIES` MSR (lists model-specific core capabilities)", if self.feature_flags3.contains(FeatureFlags3::IA32_CORE_CAPABILITIES) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ssbd                  : Speculative Store Bypass Disable, as mitigation for Speculative Store Bypass (IA32_SPEC_CTRL)", if self.feature_flags3.contains(FeatureFlags3::SSBD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "CPUID(EAX=7,ECX=1).EAX:");
        writeln!(indenter, "{} sha512                : SHA-512 extensions", if self.feature_flags3.contains(FeatureFlags3::SHA512) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sm3                   : SM3 hash extensions", if self.feature_flags3.contains(FeatureFlags3::SM3) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} sm4                   : SM4 cipher extensions", if self.feature_flags3.contains(FeatureFlags3::SM4) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rao-int               : Remote Atomic Operations on INTegers: `AADD`, `AAND`, `AOR`, and `AXOR` instructions", if self.feature_flags3.contains(FeatureFlags3::RAO_INT) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx-vnni              : AVX Vector Neural Network Instructions (VNNI) (VEX encoded)", if self.feature_flags3.contains(FeatureFlags3::AVX_VNNI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx512-bf16           : AVX512 instructions for bfloat16 numbers", if self.feature_flags3.contains(FeatureFlags3::AVX512_BF16) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} lass                  : Linear Address Space Separation (CR4 bit 27)", if self.feature_flags3.contains(FeatureFlags3::LASS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cmpccxadd             : `CMPccXASS` instructions", if self.feature_flags3.contains(FeatureFlags3::CMPCCXADD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} archpermonext         : Architectural Performance Monitoring Extended Lead (EAX=23h)", if self.feature_flags3.contains(FeatureFlags3::ArchPerfMonExt) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} dedup                 : DEDUP", if self.feature_flags3.contains(FeatureFlags3::DEDUP) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fzrm                  : Fast zero-length `REP STOSB`", if self.feature_flags3.contains(FeatureFlags3::FZRM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fsrs                  : Fast short `REP STOSB`", if self.feature_flags3.contains(FeatureFlags3::FSRS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rsrcs                 : Fast short `REP CMPSB` and `REP SCASB`", if self.feature_flags3.contains(FeatureFlags3::RSRCS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} fred                  : Flexible Return and Event Delivery", if self.feature_flags3.contains(FeatureFlags3::FRED) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} lkgs                  : `LKGS` instruction", if self.feature_flags3.contains(FeatureFlags3::LKGS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} wrmsms                : `WRMSRNS` instruction (non-serializing write to MSRs)", if self.feature_flags3.contains(FeatureFlags3::WRMSRNS) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} amx-fp16              : AMX instructions for FP16 numbers", if self.feature_flags3.contains(FeatureFlags3::AMX_FP16) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} hreset                : `HRESET` instruction, `IA32_HRESET_ENABLE` (17DAh) MSR, and processor history reset leaf (EAX=20h)", if self.feature_flags3.contains(FeatureFlags3::HRESET) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx-ifma              : `AVX IFMA instructions", if self.feature_flags3.contains(FeatureFlags3::AMX_IFMA) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} lam                   : Linear Address Masking", if self.feature_flags3.contains(FeatureFlags3::LAM) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mrlist                : `RDMSRLIST` and `WRMSRLIST` instructions, and the `IA32_BARRIER` (02Fh) MSR", if self.feature_flags3.contains(FeatureFlags3::MsrLists) { "[X]" } else { "[ ]" })?;
        
        writeln!(indenter, "CPUID(EAX=7,ECX=1).EBX:");
        writeln!(indenter, "{} ppin                  : Intel PPIN (Protected processor inventory number): `IA32_PPIN_CTL` (04Eh) and `IA32_PPIN` (04Fh)", if self.feature_flags4.contains(FeatureFlags4::PPIN) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} pbndkb                : Total storage encryption: `PBNDKB` instruction and `TSE_CAPABILITY` (9F1h) MSR", if self.feature_flags4.contains(FeatureFlags4::PBNDKB) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "CPUID(EAX=7,ECX=1).EDX:");
        writeln!(indenter, "{} avx-vnni-int8         : AVX VNNI INT8 instructions", if self.feature_flags4.contains(FeatureFlags4::AvxVnniInt8) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx-ne-convert        : AVX no-exception FP conversion instruction (bfloat16<->fp32 and fp16<->fp32)", if self.feature_flags4.contains(FeatureFlags4::AvxNeConvert) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} amx-complex           : AMX support for 'complex' tiles (`TCMMIMFP16PS` and `TCMMRLFP16PS`)", if self.feature_flags4.contains(FeatureFlags4::AmxComplex) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avs-vnni-int16        : AVX VNNI INT16 instructions", if self.feature_flags4.contains(FeatureFlags4::AvxVnniInt16) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} prefetchi             : Instruction-cache prefetch instructions (`PREFETCHIT0` and `PREFETCHIT1`)", if self.feature_flags4.contains(FeatureFlags4::PrefetchI) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} user_msr              : User-mode MSR access instructions (`USDMSR` and `UWRMSR`)", if self.feature_flags4.contains(FeatureFlags4::UserMsr) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} uiret-uif-from-rflags : `UIRET` (User Interrupt Return) instruction will set UIF (User Interrupt Flag) to the value of bit 1 of the RFLAGS image popped of the stack", if self.feature_flags4.contains(FeatureFlags4::UiretUifFromRflags) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} cet-sss               : Control-flow enforcement (CET) supervisor shadow stack (SSS) are guaranteed not to become prematurely busy as long as shadow stack switching does not cause page faults on the stack being switched to", if self.feature_flags4.contains(FeatureFlags4::CetSss) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} avx10                 : AVX10 converged vector ISA", if self.feature_flags4.contains(FeatureFlags4::AVX10) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} apx_f                 : Advanced Perforamnce Extnesions, Foundation (adds REX2 and extended EVEX prefix encoding to support 32 GPRs, as well as some new instructions)", if self.feature_flags4.contains(FeatureFlags4::APX_F) { "[X]" } else { "[ ]" })?;
        
        writeln!(indenter, "CPUID(EAX=7,ECX=1).EBX:");
        writeln!(indenter, "{} psfd                  : Fast Store Forwarding Predictor disable supported (`SPEC_CTRL` (MSR 48h) bit 7)", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ipred_ctrl            : IPRED_DIS controls supported (`SPEC_CTRL` bits 3 and 4)", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} rrsba_ctrl            : RRSBA behavior disable supported (`SPEC_CTRL` bits 5 and 6)", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} ddpd_u                : Data Dependent Prefetcher disable support (`SPEC_CTRL` bit 8)", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} bhi_ctrl              : BHI_DIS_S behavior enable supported (`SPEC_CTRL` bits 10)", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{} mcdt_no               : The processor does not exhibit MXCSR configuration dependent timing", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        writeln!(indenter, "{}                       : UC-lock disbale feature supported", if self.feature_flags5.contains(FeatureFlags5::PSFD) { "[X]" } else { "[ ]" })?;
        indenter.set_spaces(4);

        writeln!(indenter, "MPX Address-Width Adjust: {}", self.mpx_addr_width_adjust)?;

        if let Some(cache_tlb_descs) = self.cache_tlb_descriptors {
            writeln!(indenter, "Cache/TLB descriptors:")?;
            indenter.set_spaces(8);
            for (i, desc) in cache_tlb_descs.iter().enumerate() {
                writeln!(indenter, "[{:>2}]: {}", i, desc)?;

            }

            indenter.set_spaces(4);
        } else {
            writeln!(indenter, "Cache/TLB descriptors: None")?;
        }

        writeln!(indenter, "{}", self.thermal_power_mgmt)?;

        Ok(())
    }
}


#[cfg(target_arch = "x86_64")]
unsafe fn get_features() -> CpuidInfo {
    use core::arch::x86_64::{__cpuid, __cpuid_count, __get_cpuid_max};

    use onca_base::EnumFromIndexT;

    use crate::{utils::is_flag_set, KiB};

    let (highest_leaf, _) = __get_cpuid_max(0);
    
    // Manifacturer
    let res = __cpuid(0);
    let mut manifacturer_name = [0u8; 12];
    copy_nonoverlapping(&res.ebx as *const _ as *const _, manifacturer_name.as_mut_ptr(), 4);
    copy_nonoverlapping(&res.edx as *const _ as *const _, manifacturer_name.as_mut_ptr().add(4), 4);
    copy_nonoverlapping(&res.ecx as *const _ as *const _, manifacturer_name.as_mut_ptr().add(8), 4);

    let manufacturer = match manifacturer_name {
        [b'A', b'M', b'D', b'i', b's', b'b', b'e', b't', b't', b'e', b'r', b'!'] => Manufacturer::EarlyAMD,
        [b'A', b'u', b't', b'h', b'e', b'n', b't', b'i', b'c', b'A', b'M', b'D'] => Manufacturer::AMD,
        [b'C', b'e', b'n', b't', b'a', b'u', b'r', b'H', b'a', b'u', b'l', b's'] => Manufacturer::Centaur,
        [b'C', b'y', b'r', b'i', b'x', b'I', b'n', b's', b't', b'e', b'a', b'd'] => Manufacturer::Cyrix,
        [b'G', b'e', b'n', b'u', b'i', b'n', b'e', b'I', b'n', b't', b'e', b'l'] => Manufacturer::Intel,
        [b'G', b'e', b'n', b'u', b'i', b'n', b'e', b'I', b'o', b't', b'e', b'l'] => Manufacturer::Iotel,
        [b'T', b'r', b'a', b'n', b's', b'm', b'e', b't', b'a', b'C', b'P', b'U'] => Manufacturer::Transmeta,
        [b'G', b'e', b'n', b'u', b'i', b'n', b'e', b'T', b'M', b'x', b'8', b'6'] => Manufacturer::Transmeta2,
        [b'G', b'e', b'o', b'd', b'e', b' ', b'b', b'y', b' ', b'N', b'S', b'C'] => Manufacturer::NationalSemiconductor,
        [b'N', b'e', b'x', b'G', b'e', b'n', b'D', b'r', b'i', b'v', b'e', b'n'] => Manufacturer::NexGen,
        [b'R', b'i', b's', b'e', b'R', b'i', b's', b'e', b'R', b'i', b's', b'e'] => Manufacturer::Rise,
        [b'S', b'i', b'S', b' ', b'S', b'i', b'S', b' ', b'S', b'i', b'S', b' '] => Manufacturer::SIS,
        [b'U', b'M', b'C', b' ', b'U', b'M', b'C', b' ', b'U', b'M', b'C', b' '] => Manufacturer::UMC,
        [b'V', b'I', b'A', b' ', b'V', b'I', b'A', b' ', b'V', b'I', b'A', b' '] => Manufacturer::VIA,
        [b'V', b'o', b'r', b't', b'e', b'x', b'8', b'6', b' ', b'S', b'o', b'X'] => Manufacturer::DmPVortex86,
        [b' ', b' ', b'S', b'h', b'a', b'n', b'g', b'h', b'a', b'i', b' ', b' '] => Manufacturer::Zhaoxin,
        [b'H', b'y', b'g', b'o', b'n', b'G', b'e', b'n', b'u', b'i', b'n', b'e'] => Manufacturer::Hygon,
        [b'G', b'e', b'n', b'u', b'i', b'n', b'e', b' ', b' ', b'R', b'D', b'C'] => Manufacturer::RDC,
        [b'E', b'2', b'K', b' ', b'M', b'A', b'C', b'H', b'I', b'N', b'E', b'\0'] => Manufacturer::MCST,
        [b'M', b'i', b'S', b'T', b'e', b'r', b' ', b'A', b'O', b'4', b'8', b'6'] => Manufacturer::AO486,
        [b'b', b'h', b'y', b'v', b'e', b' ', b'b', b'h', b'y', b'v', b'w', b' '] => Manufacturer::Bhyve,
        [b'K', b'V', b'M', b'K', b'V', b'M', b'K', b'V', b'M', b'\0', b'\0', b'\0'] => Manufacturer::KVM,
        [b'T', b'C', b'G', b'T', b'C', b'G', b'T', b'C', b'G', b'T', b'C', b'G'] => Manufacturer::QEMU,
        [b'M', b'i', b'c', b'r', b'o', b's', b'o', b'f', b't', b' ', b'H', b'v'] => Manufacturer::HyperV,
        [b'M', b'i', b'c', b'r', b'o', b's', b'o', b'f', b't', b'X', b'T', b'A'] => Manufacturer::MsXTM,
        [b' ', b'l', b'r', b'p', b'e', b'p', b'y', b'h', b' ', b' ', b'v', b'r'] => Manufacturer::Parallels,
        [b'V', b'M', b'w', b'a', b'r', b'e', b'V', b'M', b'w', b'a', b'r', b'e'] => Manufacturer::VMware,
        [b'X', b'e', b'n', b'V', b'M', b'M', b'X', b'e', b'n', b'V', b'M', b'M'] => Manufacturer::XenHVM,
        [b'A', b'C', b'R', b'N', b'A', b'C', b'R', b'N', b'A', b'C', b'R', b'N'] => Manufacturer::ProjectACRN,
        [b' ', b'Q', b'N', b'X', b'Q', b'V', b'M', b'B', b'S', b'Q', b'G', b' '] => Manufacturer::QNX,
        [b'V', b'i', b'r', b't', b'u', b'a', b'l', b'A', b'p', b'p', b'l', b'e'] => Manufacturer::AppleRosetta,
        _ => Manufacturer::Unknown(manifacturer_name),
    };

    let (family_info, additional_feature_info, feature_flags, extended_feature_flags) = if 1 <= highest_leaf {
        let res1 = __cpuid(1);
        let res8_1 = __cpuid(0x8000_0001);

        let extended_family_id = ((res1.eax >> 20) & 0xFF) as u16;
        let family_id = ((res1. eax >> 8) & 0xF) as u16;
        
        let extended_model_id = ((res1.eax >> 16) & 0xF) as u16;
        let model_id = ((res1.eax >> 4) & 0xF) as u16;
        let model_id = if family_id == 6 || family_id == 15 {
            (extended_model_id << 4) | model_id
        } else {
            model_id
        };

        let family_id = if family_id == 15 { extended_family_id + family_id } else { family_id };
        
        let processor_type = ProcessorType::from_idx_unchecked(((res1.eax >> 12) & 0x3) as usize);
        let stepping_id = (res1.eax & 0xF) as u8;

        let flags = ((res1.ecx as u64) << 32) | (res1.edx as u64);
        let flags = core::mem::transmute(flags);

        let ext_flags = ((res8_1.ecx as u64) << 32) | (res8_1.edx as u64);
        let ext_flags = core::mem::transmute(ext_flags);

        (FamilyInfo {
            family_id,
            model_id,
            processor_type,
            stepping_id,
        },
        AdditionalFeatureInfo {
            brand_index: res1.ebx as u8,
            clflush_line_size: (res1.ebx >> 8) as u8,
            max_addressable_ids: (res1.ebx >> 16) as u8,
            local_apic_id: (res1.ebx >> 24) as u8,
        },
        flags, ext_flags)

    } else {
        (FamilyInfo::default(), AdditionalFeatureInfo::default(), FeatureFlags::None, ExtendedFeatureFlags::None)
    };

    let cache_tlb_descriptors = if 2 <= highest_leaf {
        let res2 = __cpuid(2);
        
        let supported = res2.eax & 0xFF == 0x01;
        if supported {
            let regs = [res2.eax, res2.ebx, res2.ecx, res2.edx];

            let mut descs = [CacheTlbDescriptor::Null; 15];
            for i in 1..16 {
                let reg = regs[i / 0x3];
                let idx = i & 0x3;
                let byte = (reg << idx) & 0xFF;

                descs[i - 1] = CacheTlbDescriptor::from_idx_or(idx, CacheTlbDescriptor::Null);
            }
            Some(descs)
        } else {
            None
        }
    } else {
        None
    };

    let thermal_power_mgmt = if 6 <= highest_leaf {
        let res6 = __cpuid(6);

        let mut flags: ThermalPowerManagementFlags = core::mem::transmute(res6.eax);
        flags.set(ThermalPowerManagementFlags::EFI, is_flag_set(res6.ecx, 0x1));
        flags.set(ThermalPowerManagementFlags::ACNT2, is_flag_set(res6.ecx, 0x2));
        flags.set(ThermalPowerManagementFlags::PEB, is_flag_set(res6.ecx, 0x8));
        flags.set(ThermalPowerManagementFlags::HF_PC, is_flag_set(res6.edx, 0x1));
        flags.set(ThermalPowerManagementFlags::HF_EC, is_flag_set(res6.edx, 0x2));

        ThermalPowerManagement {
            flags,
            dts_thresholds: (res6.ebx & 0xF) as u8,
            thread_director_classes: ((res6.ecx >> 8) & 0xF) as u8,
            hardware_feedback_struct_size: ((res6.edx >> 8) & 0xF) as u8 + 1,
        }
    } else {
        ThermalPowerManagement::default()
    };

    let (feature_flags2, feature_flags3, feature_flags4, feature_flags5, mpx_addr_width_adjust) = if 7 <= highest_leaf {
        let res7_0 = __cpuid(7);
        
        let mpx_addr_width_adjust = ((res7_0.ecx >> 16) & 0x1F) as u8;
        
        let flags2 = (((res7_0.ecx & 0xFFE0_FFFF) as u64) << 32) | (res7_0.ebx as u64);
        let flags2 = core::mem::transmute(flags2);
        
        let mut flags3 = (res7_0.edx as u64);
        
        let flags4 = if 1 <= res7_0.eax {
            let res7_1 = __cpuid_count(7, 1);
            flags3 |= ((res7_1.eax as u64) << 32);
            
            let flags4 = ((res7_1.edx as u64) << 32) | (res7_1.ebx as u64);
            core::mem::transmute(flags4)
        } else {
            FeatureFlags4::None
        };

        let flags5 = if 2 <= res7_0.eax {
            let res7_2 = __cpuid_count(7, 2);
            let flags5 = res7_2.edx;
            core::mem::transmute(flags5 as u8)
        } else {
            FeatureFlags5::None
        };
        
        let flags3 = core::mem::transmute(flags3);
        
        (flags2, flags3, flags4, flags5, mpx_addr_width_adjust)
    } else {
        (FeatureFlags2::None, FeatureFlags3::None, FeatureFlags4::None, FeatureFlags5::None, 0)
    };

    CpuidInfo {
        manufacturer,
        family_info,
        additional_feature_info,
        feature_flags,
        extended_feature_flags,
        feature_flags2,
        feature_flags3,
        feature_flags4,
        feature_flags5,
        cache_tlb_descriptors,
        thermal_power_mgmt,
        mpx_addr_width_adjust,
    }
}