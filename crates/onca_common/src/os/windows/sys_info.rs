use std::{ptr::null_mut, mem::{size_of, size_of_val}, num::NonZeroU8};

use windows::{
    core::PSTR,
    Win32::{
        System::{SystemInformation::{SYSTEM_INFO, GetSystemInfo, GetComputerNameExA, ComputerNameDnsDomain, COMPUTER_NAME_FORMAT, ComputerNameDnsFullyQualified, ComputerNameDnsHostname, ComputerNameNetBIOS, ComputerNamePhysicalDnsDomain, ComputerNamePhysicalDnsFullyQualified, ComputerNamePhysicalDnsHostname, ComputerNamePhysicalNetBIOS, PROCESSOR_ARCHITECTURE_AMD64, PROCESSOR_ARCHITECTURE_ARM64, LOGICAL_PROCESSOR_RELATIONSHIP, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX, GetLogicalProcessorInformationEx, RelationProcessorCore, RelationAll, RelationNumaNode, RelationCache, RelationProcessorPackage, RelationProcessorDie, RelationNumaNodeEx, RelationProcessorModule, GROUP_AFFINITY, RelationGroup, CacheUnified, CacheInstruction, CacheData, CacheTrace, GetPhysicallyInstalledSystemMemory, MEMORYSTATUSEX, GlobalMemoryStatusEx}, SystemServices::LTP_PC_SMT, ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION}},
        Foundation::ERROR_INSUFFICIENT_BUFFER,
    }
};

use crate::{
    sys::{SystemInfo, ProcessorArchitecture, self, IdentifiableSystemInfo, ProcessorPackage, PackageGroup, CpuCore, ProcessorInfo, CoreEfficiency, CacheLevel, CpuCache, CacheAssociativity, CacheType, ActiveGroup, NumaNode, MemoryInfo, PerformanceInfo},
    collections::BitSet, utils::is_flag_set, KiB,
};



pub(crate) fn get_system_info(identifiable_info: bool) -> Result<SystemInfo, i32> {
    let mut sys_info = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut sys_info) };

    let arch = match unsafe { sys_info.Anonymous.Anonymous.wProcessorArchitecture } {
        PROCESSOR_ARCHITECTURE_AMD64 => ProcessorArchitecture::X86_64,
        PROCESSOR_ARCHITECTURE_ARM64 => ProcessorArchitecture::AArch64,
        _ => ProcessorArchitecture::Unknown,
    };

    

    let identifiable_info = if identifiable_info {
        Some(get_identifiable_info())
    } else {
        None
    };

    let cpu_info = get_processor_info(arch)?;

    Ok(SystemInfo {
        page_size: sys_info.dwPageSize,
        alloc_granularity: sys_info.dwAllocationGranularity,
        ident_info: identifiable_info,
        min_app_address: sys_info.lpMinimumApplicationAddress as *const _,
        max_app_address: sys_info.lpMaximumApplicationAddress as *const _,
        
        cpu_info,
    })
}

pub(crate) fn get_current_process_memory_info() -> Result<MemoryInfo, i32> {
    let mut memory_size = 0;
    unsafe { GetPhysicallyInstalledSystemMemory(&mut memory_size) }.map_err(|err| err.code().0);

    let mut mem_status = MEMORYSTATUSEX::default();
    mem_status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut mem_status) }.map_err(|err| err.code().0)?;

    Ok(MemoryInfo {
        memory_size: memory_size * KiB(1) as u64,
        memory_load: mem_status.dwMemoryLoad as u8,
        total_physical: mem_status.ullTotalPhys,
        available_physical: mem_status.ullAvailPhys,
        total_page_file: mem_status.ullTotalPageFile,
        available_page_file: mem_status.ullAvailPageFile,
        total_virtual: mem_status.ullTotalVirtual,
        available_virtual: mem_status.ullAvailVirtual,
    })
}

pub(crate) fn get_current_performance_info() -> Result<PerformanceInfo, i32> {
    let mut perf_info = PERFORMANCE_INFORMATION::default();
    unsafe { GetPerformanceInfo(&mut perf_info, size_of_val(&perf_info) as u32) }.map_err(|err| err.code().0)?;

    Ok(PerformanceInfo {
        total_committed_pages: perf_info.CommitTotal as u64,
        max_committed_pages: perf_info.CommitLimit as u64,
        peak_committed_pages: perf_info.CommitPeak as u64,
        physical_total_pages: perf_info.PhysicalTotal as u64,
        physical_available_pages: perf_info.PhysicalAvailable as u64,
        system_cache_pages: perf_info.SystemCache as u64,
        kernel_total_pages: perf_info.KernelTotal as u64,
        kernel_paged_pages: perf_info.KernelPaged as u64,
        kernal_unpaged_pages: perf_info.KernelNonpaged as u64,
        page_size: perf_info.PageSize as u64,
        handle_count: perf_info.HandleCount,
        process_count: perf_info.ProcessCount,
        thread_count: perf_info.ThreadCount,
    })
}

fn get_identifiable_info() -> IdentifiableSystemInfo {
    IdentifiableSystemInfo {
        computer_dns_domain_name: get_computer_name(ComputerNameDnsDomain),
        computer_dns_fully_qualifed_name: get_computer_name(ComputerNameDnsFullyQualified),
        computer_net_bios_name: get_computer_name(ComputerNameDnsHostname),
        computer_dns_host_name: get_computer_name(ComputerNameNetBIOS),
        computer_physical_dns_domain_name: get_computer_name(ComputerNamePhysicalDnsDomain),
        computer_physical_dns_fully_qualifed_name: get_computer_name(ComputerNamePhysicalDnsFullyQualified),
        computer_physical_dns_host_name: get_computer_name(ComputerNamePhysicalDnsHostname),
        computer_physical_net_bios_name: get_computer_name(ComputerNamePhysicalNetBIOS),
    }
}

#[allow(non_upper_case_globals)]
fn get_processor_info(arch: ProcessorArchitecture) -> Result<ProcessorInfo, i32> {
    let mut size = 0;
    unsafe { GetLogicalProcessorInformationEx(RelationAll, None, &mut size) }.map_or_else(|err| 
        if err.code() == ERROR_INSUFFICIENT_BUFFER.to_hresult() {
            Ok(())
        } else {
            Err(err.code().0)
        }, 
        |_| Ok(())
    );

    let mut buf = Vec::<u8>::with_capacity(size as usize);
    unsafe { GetLogicalProcessorInformationEx(RelationAll, Some(buf.as_mut_ptr() as *mut _), &mut size) }.map_err(|err| err.code().0);
    unsafe { buf.set_len(size as usize) };

    let mut packages = Vec::<ProcessorPackage>::new();
    let mut active_groups = Vec::new();
    let mut numa_nodes = Vec::new();
    let mut max_groups = 0;

    let mut offset = 0;
    let mut group_idx = 0;
    while offset < buf.len() {
        let info = &unsafe { *buf.as_mut_ptr().add(offset).cast::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>() };
        match info.Relationship {
            RelationProcessorCore => {
                let processor = &unsafe { info.Anonymous.Processor };
                debug_assert!(processor.GroupCount == 1);

                let group_id = processor.GroupMask[0].Group;
                let package = packages.iter_mut()
                    .find(|package| package.get_group(group_id).is_some())
                    .map_or_else(|| Err(0), |package| Ok(package))?;

                let idx = package.cores.len() as u16;
                package.cores.push(CpuCore {
                    idx,
                    mask: BitSet::from_bits([processor.GroupMask[0].Mask as u64]),
                    group_id: processor.GroupMask[0].Group,
                    supports_smt: is_flag_set(processor.Flags, LTP_PC_SMT as u8),
                    efficiency: CoreEfficiency::new(processor.EfficiencyClass),
                    caches: [Vec::new(), Vec::new(), Vec::new()],
                });
            },
            RelationNumaNode => {
                let numa = unsafe { info.Anonymous.NumaNode };
                debug_assert!(numa.NodeNumber as usize == numa_nodes.len());

                let mut groups = Vec::new();
                for i in 0..numa.GroupCount as usize {
                    let group = &unsafe { *numa.Anonymous.GroupMasks.as_ptr().add(i) };
                    groups.push((group.Group, BitSet::from_bits([group.Mask as u64])));
                }

                numa_nodes.push(NumaNode {
                    id: numa.NodeNumber,
                    groups
                })
            },
            RelationCache => {
                let cache = &unsafe { info.Anonymous.Cache };

                let group_id = unsafe { cache.Anonymous.GroupMask.Group };
                let package = packages.iter_mut()
                    .find(|package| package.get_group(group_id).is_some())
                    .map_or_else(|| Err(0), |package| Ok(package))?;

                let level = match cache.Level {
                    1 => CacheLevel::L1,
                    2 => CacheLevel::L2,
                    3 => CacheLevel::L3,
                    _ => unreachable!(),
                };

                let associativity = if cache.Associativity == 0xFF {
                    CacheAssociativity::FullyAssociative
                } else {
                    CacheAssociativity::Assocativity(NonZeroU8::new(cache.Associativity).expect("Win32 should never return an associativity of 0"))
                };

                let ty = match cache.Type {
                    CacheUnified     => CacheType::Unified,
                    CacheInstruction => CacheType::Instruction,
                    CacheData        => CacheType::Data,
                    CacheTrace       => CacheType::Trace,
                    _ => unreachable!()
                };

                let mut cache_mask = BitSet::from_bits([unsafe { cache.Anonymous.GroupMask.Mask as u64 }]);
                let mut group_id = unsafe { cache.Anonymous.GroupMask.Group };
                let cache = CpuCache {
                    level,
                    associativity,
                    line_size: cache.LineSize,
                    size: cache.CacheSize,
                    ty
                };

                let mut core_specific = false;
                for core in &mut package.cores {
                    if core.group_id == group_id && core.mask == cache_mask {
                        core.caches[level as usize].push(cache);
                        core_specific = true;
                        break;
                    }
                }

                if !core_specific {
                    package.caches[level as usize].push((group_id, cache_mask, cache));
                }
            },
            RelationProcessorPackage => {
                let processor = &unsafe { info.Anonymous.Processor };

                let mut groups = Vec::new();
                for i in 0..processor.GroupCount as usize {
                    let group = processor.GroupMask[i];

                    groups.push(PackageGroup {
                        id: group.Group,
                        mask: BitSet::from_bits([group.Mask as u64]),
                    });
                }
                
                packages.push(ProcessorPackage {
                    groups,
                    cores: Vec::new(),
                    caches: [Vec::new(), Vec::new(), Vec::new()],
                });
            },
            RelationGroup => {
                let group = &unsafe { info.Anonymous.Group };

                max_groups = group.MaximumGroupCount;
                for i in 0..group.ActiveGroupCount as usize {
                    let active_group = &unsafe { *group.GroupInfo.as_ptr().add(i) };

                    active_groups.push(ActiveGroup {
                        max_logical_cores: active_group.MaximumProcessorCount,
                        active_logical_cores: active_group.ActiveProcessorCount,
                        active_mask: BitSet::from_bits([active_group.ActiveProcessorMask as u64]),
                    });
                }

                group_idx += 1;
            }
            RelationProcessorDie => {
                // Unhandled
            },
            RelationNumaNodeEx => {
                // Unhandled
            },
            RelationProcessorModule => {
                // Unhandled
            },
            _ => unreachable!()
        }
        offset += info.Size as usize;
    }

    Ok(ProcessorInfo {
        arch: ProcessorArchitecture::X86_64,
        packages,
        max_groups,
        active_groups,
        numa_nodes,
    })

}

fn get_computer_name(format: COMPUTER_NAME_FORMAT) -> String {
    let mut len = 0;
        _ = unsafe { GetComputerNameExA(format, PSTR(null_mut()), &mut len) };
        let mut name = String::with_capacity(len as usize);
        unsafe { GetComputerNameExA(format, PSTR(name.as_mut_ptr()), &mut len) };
        unsafe { name.as_mut_vec().set_len(len as usize) };
        name
}