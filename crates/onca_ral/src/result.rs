use core::fmt;
use onca_core::prelude::*;

use crate::{Format, CommandListType, TextureLayout, ClearColor, QueueIndex};

/// RAL error
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    /// Unknown error
    Unknown,
    /// Dynlib load failed
    DynLib(String),
    /// Function load failed
    LoadFunction(&'static str),
    /// An expected features is missing
    MissingFeature(&'static str),
    /// A requirement wasn't met
    UnmetRequirement(String),
    /// Out of host memory
    OutOfHostMemory,
    /// Out of device memory
    OutOfDeviceMemory,
    /// Device lost
    DeviceLost,
    /// Format error
    Format(String),
    /// Unsupported formats for swapchain
    UnsupportedSwapchainFormats(Vec<Format>),
    /// Unsupported format
    UnsupportedFormat(Format),
    /// Unsupported format for a given view
    UnsupportedViewFormat{ texture: Format, view: Format },
    /// Unsupported memory topology
    UnsupportedMemoryTopology(&'static str),
    /// Use after the device was dropped
    UseAfterDeviceDropped,
    /// Operation has timed out
    Timeout,
    /// Command list  type
    InvalidCommandPoolType{ found: CommandListType, expected: CommandListType },
    /// Command list error
    CommandList(&'static str),
    /// Invalid transition barrier
    InvalidBarrier(&'static str),
    /// Invalid texture layout
    InvalidTextureLayout(TextureLayout, &'static str),
    /// Invalid clear color for a given format
    InvalidClearColor(ClearColor, Format),
    /// Invalid count
    InvalidCount(&'static str, usize),
    /// Invalid queue submit
    InvalidQueueSubmit{ found: QueueIndex, expected: QueueIndex },
    /// Invalid shader code
    InvalidShaderCode(&'static str),
    /// Descriptor out of range
    DescriptorOutOfRange{ index: u32, max: u32 },
    /// Descriptor heap is not shader visible
    DescriptorHeapNotShaderVisible,
    /// Handle has expired (trying to use handle to object that doesn't exist)
    ExpiredHandle(&'static str),

    /// Generic invalid parameter
    InvalidParameter(String),
    /// Feature is not implemented
    NotImplemented(&'static str),
    /// Other error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unknown                                    => f.write_str("Unknown"),
            Error::DynLib(name)                               => f.write_fmt(format_args!("Failed to load dynamic library: '{name}'")),
            Error::LoadFunction(name)                         => f.write_fmt(format_args!("Failed to load function from dynamic library: '{name}'")),
            Error::MissingFeature(name)                       => f.write_fmt(format_args!("Missing expected feature: '{name}'")),
            Error::UnmetRequirement(req)                      => f.write_fmt(format_args!("Unmet requirement: {req}")),
            Error::OutOfHostMemory                            => f.write_str("Out of host memory"),
            Error::OutOfDeviceMemory                          => f.write_str("Out of device memory"),
            Error::DeviceLost                                 => f.write_str("Device lost"),
            Error::Format(name)                               => f.write_fmt(format_args!("Format error: '{name}'")),
            Error::UnsupportedSwapchainFormats(formats)       => {
                f.write_str("No supported swapchain format, provided formats:\n")?;
                for format in formats {
                    f.write_fmt(format_args!("- {format}"))?;
                }
                Ok(())
            },
            Error::UnsupportedFormat(format)                  => f.write_fmt(format_args!("Unsupported format: {format}")),
            Error::UnsupportedViewFormat { texture, view }    => f.write_fmt(format_args!("Unsupported view format {view} for texture format {texture}, formats must be in the same component family")),
            Error::UnsupportedMemoryTopology(info)            => f.write_fmt(format_args!("Unsupported memory topologies: {info}")),
            Error::UseAfterDeviceDropped                      => f.write_str("Tried to use the device after it was dropped"),
            Error::Timeout                                    => f.write_str("Operation has timed out"),
            Error::InvalidCommandPoolType { found, expected } => f.write_fmt(format_args!("Invalid command list type, found '{found}', expected `{expected}`")),
            Error::CommandList(s)                             => f.write_fmt(format_args!("Command list error: {s}")),
            Error::InvalidBarrier(s)                          => f.write_fmt(format_args!("Invalid barrier: {s}")),
            Error::InvalidTextureLayout(layout, reason)       => f.write_fmt(format_args!("Invalid texture layout: {layout} -> {reason}")),
            Error::InvalidClearColor(clear_color, format)     => f.write_fmt(format_args!("Invalid clear color '{clear_color}' for format '{format}'")),
            Error::InvalidCount(reason, count)                => f.write_fmt(format_args!("Invalid count '{reason}', found {count}")),
            Error::InvalidQueueSubmit{ found, expected }      => f.write_fmt(format_args!("Command list submitted to unsupported queue, found '{found}', expected '{expected}'")),
            Error::InvalidShaderCode(s)                       => f.write_fmt(format_args!("Invalid shader code: {s}")),
            Error::DescriptorOutOfRange { index, max }        => f.write_fmt(format_args!("Descriptor index out of range: {index} (max: {max})")),
            Error::DescriptorHeapNotShaderVisible             => f.write_str("Descriptor heap is not shader visible"),
            Error::ExpiredHandle(reason)                      => f.write_fmt(format_args!("Expired handle: {reason}")),

            Error::InvalidParameter(s)                        => f.write_fmt(format_args!("Invalid paramter: {s}")),
            Error::NotImplemented(s)                          => f.write_fmt(format_args!("Not implemented: {s}")),
            Error::Other(s)                                   => f.write_str(&s),
            
            
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

// Helper macros

#[macro_export]
macro_rules! check_invalid_parameter {
    ($expected:expr, $($args:tt)*) => {
        if !$expected {
            return Err(Error::InvalidParameter(format!($($args)*)));
        }
    };
}