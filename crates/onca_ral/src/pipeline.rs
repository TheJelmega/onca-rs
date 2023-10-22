use onca_core::prelude::*;
use onca_core_macros::flags;

use crate::{
    handle::{InterfaceHandle, create_ral_handle},
    Handle, HandleImpl, Result, StaticSamplerHandle, ShaderVisibility, Error, constants, InlineDescriptorDesc, DescriptorTableLayoutHandle,
};


/// Pipeline layout flags
#[flags]
pub enum PipelineLayoutFlags {
    /// Pipelines created with this flag can contain input layouts
    /// 
    /// On certain hardware, this can allow space to be saved in the pipeline layout
    ContainsInputLayout,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PipelineConstantRange {
    /// Number of constants
    pub count:      u8,
    /// Shader visibility
    pub visibility: ShaderVisibility,
}

/// Pipeline layout description
/// 
/// Each layout only allows up to 64 available layouts slots exists, both descriptor entries and constants take up some of these slots, and must not exceed this limit
/// 
/// In shaders, entries will be laid out in the following order:
/// - Descriptor tables
/// - Inline descriptors
/// - Constants
/// - Static samplers
/// 
/// With the index of an 'entry' here referring to:
/// - DX12: A register space
/// - Vulkan: A descriptor 'set'
#[derive(Clone)]
pub struct PipelineLayoutDesc {
    /// Flags
    pub flags:              PipelineLayoutFlags,
    /// Descriptor tables
    /// 
    /// Takes up 1 slot per table
    pub descriptor_tables:  Option<Vec<DescriptorTableLayoutHandle>>,
    /// Entries
    /// 
    /// Takes up 1 or 2 slots, depending on entry type
    pub inline_descriptors: Option<Vec<InlineDescriptorDesc>>,
    /// Constant ranges
    pub constant_ranges:    Option<Vec<PipelineConstantRange>>,
    /// Static samplers
    /// 
    /// This will be bound after all entries
    pub static_samplers:    Option<Vec<StaticSamplerHandle>>
}

impl PipelineLayoutDesc {
    const MAX_SLOTS: u32 = 64;

    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "validation")]
        {
            let num_tables = self.descriptor_tables.as_ref().map_or(0, |arr| arr.len() as u32);
            let num_inline = self.inline_descriptors.as_ref().map_or(0, |arr| arr.len() as u32);
            let has_static_samplers = self.static_samplers.is_some();
            let num_entries = num_tables + num_inline + has_static_samplers as u32;

            if num_entries + has_static_samplers as u32 > constants::MAX_PIPELINE_BOUND_DESCRIPTORS {
                return Err(Error::InvalidParameter(format!(
                    "Too many pipeline layout entries ('{num_entries}'), only {} bindings are allowed", 
                    constants::MAX_PIPELINE_BOUND_DESCRIPTORS
                ) ));
            }

            
            let mut num_constants = 0;
            if let Some(constants) = &self.constant_ranges {   
                for constant_range in constants {
                    num_constants += constant_range.count as u32;
                }
            }
            

            let used_slots = num_tables + num_inline * 2 + num_constants + has_static_samplers as u32;
            if used_slots > Self::MAX_SLOTS {
                let mut needs_comma = false;
                let table_err = if num_tables == 0 {
                    String::new()
                } else {
                    needs_comma = true;
                    format!("{} descriptor tables ({} slots)", num_tables, num_tables)
                };
                let inline_err = if num_inline == 0 {
                    String::new()
                } else {
                    needs_comma = true;
                    format!("{}{} inline descriptors ({} slots, 2 slots/descriptor)", if needs_comma { ", " } else { "" }, num_inline, num_inline * 2)
                };
                let constants_err = if num_constants == 0 {
                    String::new()
                } else {
                    format!("{}{} 32-bit constants ({} slots)", if needs_comma { ", " } else { "" }, num_constants, num_constants)
                };
                let static_sampler_err = if has_static_samplers {
                    ", static samplers (1 slot)".to_string()
                } else {
                    String::new()
                };

                return Err(Error::InvalidParameter(format!(
                    "Too many layout slots used ({used_slots}), only {} slots are allowed. Slots used by:{}{}{}{}",
                    Self::MAX_SLOTS,
                    table_err,
                    inline_err,
                    constants_err,
                    static_sampler_err
                )));
            }
        }
        Ok(())
    }
}

//==============================================================================================================================

pub trait PipelineLayoutInterface {

}

pub type PipelineLayoutInterfaceHandle = InterfaceHandle<dyn PipelineLayoutInterface>;

/// Graphics of compute pipeline layout
pub struct PipelineLayout {
    handle:          PipelineLayoutInterfaceHandle,
    desc:            PipelineLayoutDesc,
    static_samplers: Vec<StaticSamplerHandle>,
}
create_ral_handle!(PipelineLayoutHandle, PipelineLayout, PipelineLayoutInterfaceHandle);

impl PipelineLayoutHandle {
    pub(crate) fn create(alloc: AllocId, handle: PipelineLayoutInterfaceHandle, desc: &PipelineLayoutDesc, static_samplers: Vec<StaticSamplerHandle>) -> PipelineLayoutHandle {
        scoped_alloc!(alloc);

        Self::new(PipelineLayout {
            handle,
            desc: desc.clone(),
            static_samplers
        })
    }

    /// Get the pipeline layout descriptor  
    pub fn desc(&self) -> &PipelineLayoutDesc {
        &self.desc
    }

    /// Get the pipeline layout flags
    pub fn flags(&self) -> PipelineLayoutFlags {
        self.desc.flags
    }

    /// Get the static samplers
    pub fn static_samplers(&self) -> &Vec<StaticSamplerHandle> {
        &self.static_samplers
    }
}

//==============================================================================================================================

pub trait PipelineInterface {

}

pub type PipelineInterfaceHandle = InterfaceHandle<dyn PipelineInterface>;

/// Graphics or compute pipeline
/// 
/// ## Dynamic state
/// 
/// The following state is always dynamic:
/// - Viewports
/// - Scissor rects
/// - Blend constants
/// - Depth Bounds
/// - Stencil reference
/// 
/// The following state allows dynamic changes but also has a default value defined in the pipeline
/// - Depth bias state (`bias`, `slope`, and `clamp`)
/// - Primitive topology
pub struct Pipeline {
    handle: PipelineInterfaceHandle,
    layout: PipelineLayoutHandle,
}
create_ral_handle!(PipelineHandle, Pipeline, PipelineInterfaceHandle);

impl PipelineHandle {
    pub(crate) fn create(handle: PipelineInterfaceHandle, layout: PipelineLayoutHandle) -> PipelineHandle {
        Self::new(Pipeline { handle, layout })
    }

    pub fn layout(&self) -> &PipelineLayoutHandle {
        &self.layout
    }
}