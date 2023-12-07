use std::{
    fmt::{self, Write},
    num::NonZeroU8
};

use onca_common_macros::EnumDisplay;

use crate::{
    os,
    collections::BitSet, fmt::Indenter, MiB,
};

#[derive(Clone, Debug)]
pub struct SystemInfo {
    /// Page size and granularity of page protextions and commitment
    pub page_size:         u32,
    /// Granularity for the starting address at which vrtual memor ycan be allocated.
    pub alloc_granularity: u32,
    /// Lowest memory address accessible to applications and dynamic-link libraries (DLLs).
    pub min_app_address:   *const (),
    /// Highest memory address accessible to applications and dynamic-link libraries (DLLs).
    pub max_app_address:   *const (),
    /// Identifiable system info
    pub ident_info:        Option<IdentifiableSystemInfo>,
    /// CPU info
    pub cpu_info:          ProcessorInfo,
}

impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "System Info:")?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Page size:            {}", self.page_size)?;
        writeln!(indenter, "Alloc granularity:    {}", self.alloc_granularity)?;
        writeln!(indenter, "Virtual memory range: 0x{:016X}-0x{:016X}", self.min_app_address as usize, self.max_app_address as usize);

        if let Some(identifiable_info) = &self.ident_info {
            writeln!(indenter, "{}", identifiable_info);
        }

        write!(indenter, "{}", self.cpu_info)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct IdentifiableSystemInfo {
    pub computer_dns_domain_name:                  String,
    pub computer_dns_fully_qualifed_name:          String,
    pub computer_dns_host_name:                    String,
    pub computer_net_bios_name:                    String,
    pub computer_physical_dns_domain_name:         String,
    pub computer_physical_dns_fully_qualifed_name: String,
    pub computer_physical_dns_host_name:           String,
    pub computer_physical_net_bios_name:           String,
}

impl fmt::Display for IdentifiableSystemInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Identifiable System Name:")?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "DNS domain name:                   {}", self.computer_dns_domain_name)?;
        writeln!(indenter, "DNS fully qualified name:          {}", self.computer_dns_fully_qualifed_name)?;
        writeln!(indenter, "DNS host name:                     {}", self.computer_dns_host_name)?;
        writeln!(indenter, "DNS net bios name:                 {}", self.computer_net_bios_name)?;
        writeln!(indenter, "physical DNS domain name:          {}", self.computer_physical_dns_domain_name)?;
        writeln!(indenter, "physical DNS fully qualified name: {}", self.computer_physical_dns_fully_qualifed_name)?;
        writeln!(indenter, "physical DNS host name:            {}", self.computer_physical_dns_host_name)?;
        write!(indenter, "physical DNS net bios name:        {}", self.computer_physical_net_bios_name)?;
        Ok(())
    }
}

/// CPU archicture.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum ProcessorArchitecture {
    #[display("x86-64")]
    X86_64,
    AArch64,
    Unknown,
}

/// Processor core efficiency class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoreEfficiency {
    Performance,
    Efficient(NonZeroU8),
}

impl CoreEfficiency {
    pub fn new(class: u8) -> Self {
        NonZeroU8::new(class).map_or(CoreEfficiency::Performance, |val| CoreEfficiency::Efficient(val))
    }
}

impl fmt::Display for CoreEfficiency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreEfficiency::Performance => f.write_str("Performance"),
            CoreEfficiency::Efficient(class) => write!(f, "Efficiency (class {class})"),
        }
    }
}

/// Processor cache level
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum CacheLevel {
    /// Level 1 cache
    L1,
    /// Level 2 cache
    L2,
    /// Level 3 cache
    L3,
}

/// Number of cache levels
pub const CACHE_LEVELS: usize = 3;

/// Cache associativity
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CacheAssociativity {
    /// Cache is fully associative
    FullyAssociative,
    /// Cache has N-associativity
    Assocativity(NonZeroU8),
}

impl fmt::Display for CacheAssociativity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheAssociativity::FullyAssociative => f.write_str("fully-associative"),
            CacheAssociativity::Assocativity(count) => write!(f, "{}-associative", count.get()),
        }
    }
}

/// Cache type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumDisplay)]
pub enum CacheType {
    /// Cache is unified
    Unified,
    /// Cache is exclusivly for inteructions
    Instruction,
    /// Cache is exclusivly for data
    Data,
    /// Cache is exclusibly for traces
    Trace,
}

#[derive(Clone, Copy, Debug)]
pub struct CpuCache {
    /// Cache level
    pub level:         CacheLevel,
    /// Cache associativity
    pub associativity: CacheAssociativity,
    /// Cache line size
    pub line_size:     u16,
    /// Cache size
    pub size:          u32,
    /// Cache type
    pub ty:            CacheType
}

impl fmt::Display for CpuCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size = self.size / 1024;
        write!(f, "{size}KiB {} {} cache ({}, {} byte line)", self.level, self.ty, self.associativity, self.line_size)
    }
}

/// Physical processor core
#[derive(Clone, Debug)]
pub struct CpuCore {
    /// Core index
    pub idx:          u16,
    /// Bitset that specifies the affinity of the core withing the specific group.
    pub mask:         BitSet<64>,
    /// Id of the group this core is part of
    pub group_id:     u16,
    /// Does the core support SMT (Simultaneous Mutli-Threading)
    pub supports_smt: bool,
    /// Core efficiency class
    pub efficiency:   CoreEfficiency,
    /// Cache owned by the specific core
    pub caches:       [Vec<CpuCache>; CACHE_LEVELS]
}

impl fmt::Display for CpuCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Core {}:", self.idx)?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Mask:            {:064b}", self.mask.bits()[0])?;
        writeln!(indenter, "Group:           {}", self.group_id)?;
        writeln!(indenter, "Supports SMT:    {}", self.supports_smt)?;
        writeln!(indenter, "Efficiency clas: {}", self.efficiency)?;

        writeln!(indenter, "Core caches:")?;
        indenter.set_spaces(8);
        let mut cache_written = false;
        for (idx, cache_level) in self.caches.iter().enumerate() {
            if cache_written && idx != 0 && !cache_level.is_empty() {
                write!(indenter, "\n");
            }

            for (idx, cache) in cache_level.iter().enumerate() {
                if idx != 0 {
                    write!(indenter, "\n");
                }
                write!(indenter, "{}", cache)?;
                cache_written = true;
            }
        }
        
        Ok(())
    }
}

/// Processor group
#[derive(Clone, Debug)]
pub struct PackageGroup {
    /// Group id
    pub id:                   u16,
    /// Bitmask that specifies the affinity of all cores in this group
    pub mask:                 BitSet<64>,
    
}

impl fmt::Display for PackageGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Group {} mask: {:064b}", self.id, self.mask.bits()[0])?;

        Ok(())
    }
}

/// Processor package.
/// 
/// A package consists of 1 or more groups
#[derive(Clone, Debug)]
pub struct ProcessorPackage {
    /// Groups in the package
    pub groups: Vec<PackageGroup>,
    /// Cores in this package
    pub cores:  Vec<CpuCore>,
    /// Processor caches
    pub caches: [Vec<(u16, BitSet<64>, CpuCache)>; CACHE_LEVELS],
}

impl ProcessorPackage {
    /// Get a group in the package based on it's ID
    pub fn get_group(&self, id: u16) -> Option<&PackageGroup> {
        self.groups.iter().find(|group| group.id == id)
    }
    
    /// Get a group in the package based on it's ID
    pub fn get_mut_group(&mut self, id: u16) -> Option<&mut PackageGroup> {
        self.groups.iter_mut().find(|group| group.id == id)
    }
}

impl fmt::Display for ProcessorPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Package info:");

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Groups:");
        indenter.set_spaces(8);
        for group in &self.groups {
            writeln!(indenter, "{}", group);
        }

        indenter.set_spaces(4);
        writeln!(indenter, "Cores:");
        indenter.set_spaces(8);
        for (idx, core) in self.cores.iter().enumerate() {
            writeln!(indenter, "{}", core);
        }
        
        indenter.set_spaces(4); 
        writeln!(indenter, "Caches:");
        indenter.set_spaces(8);
        let mut cache_written = false;
        for (idx, cache_level) in self.caches.iter().enumerate() {
            if cache_written && idx != 0 && !cache_level.is_empty() {
                write!(indenter, "\n");
            }

            for (idx, (group_id, mask, cache)) in cache_level.iter().enumerate() {
                if idx != 0 {
                    write!(indenter, "\n");
                }
                writeln!(indenter, "group {}, mask {:064b}", group_id, mask.bits()[0])?;
                write!(indenter, "    {}", cache)?;
                cache_written = true;
            }
        }
        Ok(())
    }
}

/// Active processor group.
#[derive(Clone, Debug)]
pub struct ActiveGroup {
    /// Maximum number of processors in this group
    pub max_logical_cores:    u8,
    /// Maximum number of active processors in this group
    pub active_logical_cores: u8,
    /// Mask of all active processors in this group
    pub active_mask:          BitSet<64>,
}

impl fmt::Display for ActiveGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Active group:")?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Max logical cores:    {}", self.max_logical_cores)?;
        writeln!(indenter, "Active logical cores: {}", self.active_logical_cores)?;
        write!(indenter,   "Active core mask:     {:064b}", self.active_mask.bits()[0])
    }
}

/// NUMA (Non-Unifrom Memory Access) node.
#[derive(Clone, Debug)]
pub struct NumaNode {
    /// NUMA node id
    pub id:     u32,
    /// Groups in the NUMA node
    pub groups: Vec<(u16, BitSet<64>)>,
}

impl fmt::Display for NumaNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Node {}", self.id)?;

        let mut indenter = Indenter::new(f);
        for (idx, (group_idx, mask)) in self.groups.iter().enumerate() {
            if idx != 0 {
                writeln!(indenter, "");
            }
            write!(indenter, "Group {} mask: {:064b}", group_idx, mask.bits()[0]);
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ProcessorInfo {
    /// CPU architecture
    pub arch:          ProcessorArchitecture,
    /// CPU packages in the system
    pub packages:      Vec<ProcessorPackage>,
    /// Maximum number of processor groups
    pub max_groups:    u16,
    /// Active groups in th system
    pub active_groups: Vec<ActiveGroup>,
    /// NUMA nodes
    pub numa_nodes:    Vec<NumaNode>,
}

impl fmt::Display for ProcessorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CPU info:")?;

        let mut indenter = Indenter::new(f);
        writeln!(indenter, "Architecture: {}", self.arch)?;
        for package in &self.packages {
            write!(indenter, "{package}")?;
        }

        writeln!(indenter, "Maximum groups: {}", self.max_groups)?;
        writeln!(indenter, "Active groups:")?;

        indenter.set_spaces(8);
        for group in &self.active_groups {
            writeln!(indenter, "{group}")?;
        }

        indenter.set_spaces(4);
        writeln!(indenter, "NUMA nodes:")?;
        indenter.set_spaces(8);
        for node in &self.numa_nodes {
            writeln!(indenter, "{node}")?;
        }

        Ok(())
    }
}

/// System memory info
#[derive(Clone, Copy, Debug)]
pub struct MemoryInfo {
    /// Size of physical memory installed in the system
    pub memory_size:       u64,
    /// Approximate percentage of physical memory that is currently in use by the program.
    pub memory_load:         u8,
    /// The amount of actual physical memory
    pub total_physical:      u64,
    /// The amount of the physical memory currently available.
    /// 
    /// This is the amount of physical memory that can be immediately reused without having to write its contents to disk first.
    /// It is the sum of the size of the standby, free, and zero lists.
    pub available_physical:  u64,
    /// The current committed memory limit for the system  or the current process, whichever is smaller.
    pub total_page_file:     u64,
    /// The maximum amount of memory the current process can commit.
    /// 
    /// This value is equal to or smaller than the system-wide available commit value.
    pub available_page_file: u64,
    /// The size of the user-mode portion of the virtual address space of the calling process.
    pub total_virtual:       u64,
    /// The amount of unreserved and uncommited memory currently in the user-mode poriton of he virtual address space of the calling process.
    pub available_virtual:   u64
}

impl fmt::Display for MemoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Memory info:")?;

        let mut indenter = Indenter::new(f);
        const MIB : u64 = MiB(1) as u64;
        writeln!(indenter, "Installed memory size:    {}MiB", self.memory_size / MIB)?;
        writeln!(indenter, "Memory load:              {}%", self.memory_load)?;
        writeln!(indenter, "Total physical memory     {}MiB", self.total_physical / MIB)?;
        writeln!(indenter, "Available physical memory {}MiB", self.available_physical / MIB)?;
        writeln!(indenter, "Total page file           {}MiB", self.total_page_file / MIB)?;
        writeln!(indenter, "Available page file       {}MiB", self.available_page_file / MIB)?;
        writeln!(indenter, "Total virtual memory      {}MiB", self.total_physical / MIB)?;
        write!(indenter, "Available virtual memory  {}MiB", self.available_virtual / MIB)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PerformanceInfo {
    /// The number of pages currently commited by the system.
    pub total_committed_pages:    u64,
    /// The maximum number of pages tha can be comitted by the system without extending the paging fie(s).
    /// 
    /// This number can change if memory is added or deleted, or if pagefiles have grown, shrunk, or been added.
    /// If the paging file can be extended, this is a soft limit. 
    pub max_committed_pages:      u64,
    /// Peak number of pages that were simutlaneously in the comitted state since the last system reboot.
    pub peak_committed_pages:      u64,
    /// Amount of actual physical memory, in pages.
    pub physical_total_pages:     u64,
    /// Amount of physical memory currently available, in pages.
    /// 
    /// This is the amount of physical memory that can be immediately reused without having to write its contents to disk first.
    /// It is the sum of the size of the standby, free, and zero lists.
    pub physical_available_pages: u64,
    /// The amount of system cache memory, in pages.
    /// 
    /// This is the size of the standby lists plus the system working set.
    pub system_cache_pages:       u64,
    /// The sum of the memory currently in the paged and nonpaged kernel pools, in pages
    pub kernel_total_pages:       u64,
    /// The memory currently in the paged kernel pools, in pages.
    pub kernel_paged_pages:       u64,
    /// The memory currently in the unpaged kernel pool, in pages.
    pub kernal_unpaged_pages:     u64,
    /// The eisze of a page.
    pub page_size:                u64,
    /// The current number of open handles.
    pub handle_count:             u32,
    /// The current number of processes.
    pub process_count:            u32,
    /// The current number of threads
    pub thread_count:             u32,
}

impl fmt::Display for PerformanceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "System performance info:")?;

        let mut indenter = Indenter::new(f);
        const MIB : u64 = MiB(1) as u64;
        writeln!(indenter, "Total committed pages:    {} ({} MiB)", self.total_committed_pages   , self.total_committed_pages    * self.page_size / MIB)?;
        writeln!(indenter, "Maximum committed pages:  {} ({} MiB)", self.max_committed_pages     , self.max_committed_pages      * self.page_size / MIB)?;
        writeln!(indenter, "Peak committed pages:     {} ({} MiB)", self.peak_committed_pages    , self.peak_committed_pages     * self.page_size / MIB)?;
        writeln!(indenter, "Physical total pages:     {} ({} MiB)", self.physical_total_pages    , self.physical_total_pages     * self.page_size / MIB)?;
        writeln!(indenter, "Physical available pages: {} ({} MiB)", self.physical_available_pages, self.physical_available_pages * self.page_size / MIB)?;
        writeln!(indenter, "System cache pages:       {} ({} MiB)", self.system_cache_pages      , self.system_cache_pages       * self.page_size / MIB)?;
        writeln!(indenter, "Total kernel pages:       {} ({} MiB)", self.kernel_total_pages      , self.kernel_total_pages       * self.page_size / MIB)?;
        writeln!(indenter, "Paged kernel pages:       {} ({} MiB)", self.kernel_paged_pages      , self.kernel_paged_pages       * self.page_size / MIB)?;
        writeln!(indenter, "Unpaged kernel pages:     {} ({} MiB)", self.kernal_unpaged_pages    , self.kernal_unpaged_pages     * self.page_size / MIB)?;
        writeln!(indenter, "Page size:                {}", self.page_size)?;
        writeln!(indenter, "System handles:           {}", self.handle_count)?;
        writeln!(indenter, "System processes:         {}", self.process_count)?;
        write!  (indenter, "System threads:           {}", self.thread_count)
    }
}

/// Get the current system information.
/// 
/// `identifiable_info` determines of any identifiable info will be included.
/// 
/// # Errors
/// 
/// Returns an error if the system failed to retrieve the system information, the returned code is OS-specific.
pub fn get_system_info(identifiable_info: bool) -> Result<SystemInfo, i32> {
    os::sys_info::get_system_info(identifiable_info)
}

/// Get the current process memory info.
/// 
/// # Errors
/// 
/// Returns an error if the system failed to retrieve the system information, the returned code is OS-specific.
pub fn get_current_process_memory_info() -> Result<MemoryInfo, i32> {
    os::sys_info::get_current_process_memory_info()
}

/// Get the current system performance info
pub fn get_current_performance_info() -> Result<PerformanceInfo, i32> {
    os::sys_info::get_current_performance_info()
}