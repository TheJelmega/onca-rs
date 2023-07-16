use core::{ffi::c_void, mem::ManuallyDrop};

use onca_core::{collections::ByteBuffer, prelude::DynArray};
use onca_ral as ral;
use ral::HandleImpl;
use windows::{Win32::Graphics::{Direct3D12::*, Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC, DXGI_FORMAT}}, core::PCSTR};

use crate::{device::Device, shader::Shader, utils::{ToDx, ToRalError}};

pub struct PipelineLayout {
    pub root_sig: ID3D12RootSignature
}

impl PipelineLayout {
    pub unsafe fn new(device: &Device, desc: &ral::PipelineLayoutDesc) -> ral::Result<ral::PipelineLayoutInterfaceHandle> {

        let mut flags = D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS |
                        D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS |
                        D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS;

        if desc.flags.is_set(ral::PipelineLayoutFlags::ContainsInputLayout) {
            flags |= D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT;
        }

        let root_desc = D3D12_ROOT_SIGNATURE_DESC1 {
            NumParameters: 0, // TODO
            pParameters: core::ptr::null(), // TODO
            NumStaticSamplers: 0, // TODO
            pStaticSamplers: core::ptr::null(), // TODO
            Flags: flags,
        };

        let versioned_root_desc = D3D12_VERSIONED_ROOT_SIGNATURE_DESC {
            Version: D3D_ROOT_SIGNATURE_VERSION_1_1,
            Anonymous: D3D12_VERSIONED_ROOT_SIGNATURE_DESC_0 {
                Desc_1_1: root_desc
            },
        };

        let mut signature_blob = None;

        match D3D12SerializeVersionedRootSignature(&versioned_root_desc, &mut signature_blob, None) {
            Ok(_) => (),
            Err(_) => todo!(),
        }

        // If we get here, the blob contains a valid serialized root signature 
        let signature_blob = signature_blob.unwrap_unchecked();
        let serialized_data = core::slice::from_raw_parts(signature_blob.GetBufferPointer() as *const u8, signature_blob.GetBufferSize());

        let root_sig = device.device.CreateRootSignature(0, serialized_data).map_err(|err| err.to_ral_error())?;

        Ok(ral::PipelineLayoutInterfaceHandle::new(PipelineLayout {
            root_sig
        }))
    }
}

impl ral::PipelineLayoutInterface for PipelineLayout {

}

//==============================================================================================================================

pub struct Pipeline {
    pub pso: ID3D12PipelineState
}

impl Pipeline {
    pub unsafe fn new_graphics(device: &Device, desc: &ral::GraphicsPipelineDesc) -> ral::Result<ral::PipelineInterfaceHandle> {
        let mut pipeline_stream = PipelineStream::default();

        pipeline_stream.set_root_signature(&desc.pipeline_layout);
        pipeline_stream.set_vertex_shader(&desc.vertex_shader);
        pipeline_stream.set_pixel_shader(&desc.pixel_shader);
        pipeline_stream.set_blend_desc(&desc.blend_state, desc.multisample_state.alpha_to_coverage);
        pipeline_stream.set_sample_mask(desc.multisample_state.sample_mask as u32);
        pipeline_stream.set_raster_desc(&desc.rasterizer_state, desc.multisample_state.samples != ral::SampleCount::Sample1);
        pipeline_stream.set_strip_cut(desc.primitive_restart.to_dx());
        pipeline_stream.set_topology_type(desc.topology.get_type().to_dx());
        pipeline_stream.set_sample_desc(desc.multisample_state);
        pipeline_stream.set_depth_stencil_state(&desc.depth_stencil_state);
        pipeline_stream.set_render_target_formats(desc.rendertarget_formats);

        if let Some(format) = desc.depth_stencil_format {
            pipeline_stream.set_depth_stencil_format(format.to_dx());
        }

        let mut input_layout = DynArray::with_capacity(desc.vertex_input_layout.elements.len());
        for element in &desc.vertex_input_layout.elements {
            input_layout.push(element.to_dx());
        }
        pipeline_stream.set_input_layout(&input_layout);
        
        let mut stream = pipeline_stream.build();
        let dx_desc = D3D12_PIPELINE_STATE_STREAM_DESC {
            SizeInBytes: stream.len(),
            pPipelineStateSubobjectStream: stream.as_mut_ptr() as *mut c_void,
        };

        let pso = device.device.CreatePipelineState(&dx_desc).map_err(|err| err.to_ral_error())?;
        
        Ok(ral::PipelineInterfaceHandle::new(Self {
            pso
        }))
    }
}

impl ral::PipelineInterface for Pipeline {

}

//==============================================================================================================================
// HELPERS
//==============================================================================================================================

#[derive(Clone, Copy)]
#[repr(C)]
struct PipelineSubObject<T: Copy> {
    pub subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    pub data:      T
}

#[derive(Default)]
pub struct PipelineStream {
    root_signature: Option<PipelineSubObject<*const ID3D12RootSignature>>,
    vs_shader:      Option<PipelineSubObject<D3D12_SHADER_BYTECODE>>,
    ps_shader:      Option<PipelineSubObject<D3D12_SHADER_BYTECODE>>,
    blend_desc:     Option<PipelineSubObject<D3D12_BLEND_DESC>>,
    sample_mask:    Option<PipelineSubObject<u32>>,
    raster_desc:    Option<PipelineSubObject<D3D12_RASTERIZER_DESC1>>,
    strip_cut:      Option<PipelineSubObject<D3D12_INDEX_BUFFER_STRIP_CUT_VALUE>>,
    topology_type:  Option<PipelineSubObject<D3D12_PRIMITIVE_TOPOLOGY_TYPE>>,
    rt_formats:     Option<PipelineSubObject<D3D12_RT_FORMAT_ARRAY>>,
    dsv_format:     Option<PipelineSubObject<DXGI_FORMAT>>,
    sample_desc:    Option<PipelineSubObject<DXGI_SAMPLE_DESC>>,
    depth_stecnil:  Option<PipelineSubObject<D3D12_DEPTH_STENCIL_DESC2>>,
    input_layout:   Option<PipelineSubObject<D3D12_INPUT_LAYOUT_DESC>>,
}

impl PipelineStream {
    fn set_root_signature(&mut self, pipeline_layout: &ral::PipelineLayoutHandle) {
        unsafe {
            let pipeline_layout = pipeline_layout.interface().as_concrete_type::<PipelineLayout>();
            let root_sig = ManuallyDrop::new(core::ptr::read(&pipeline_layout.root_sig));
            let root_sig_ptr : *const ID3D12RootSignature = core::mem::transmute_copy(&root_sig);
            self.root_signature = Some(PipelineSubObject {
                subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE,
                data: root_sig_ptr,
            })
        }
    }

    fn set_vertex_shader(&mut self, shader: &ral::ShaderHandle) {
        let bytecode = unsafe { shader.interface().as_concrete_type::<Shader>().get_dx_bytecode() };
        self.vs_shader = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VS,
            data: bytecode
        });
    }

    fn set_pixel_shader(&mut self, shader: &ral::ShaderHandle) {
        let bytecode = unsafe { shader.interface().as_concrete_type::<Shader>().get_dx_bytecode() };
        self.ps_shader = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS,
            data: bytecode
        });
    }

    fn set_blend_desc(&mut self, blend_state: &ral::BlendState, alpha_to_coverage: bool) {
        let mut blend_desc = blend_state.to_dx();
        blend_desc.AlphaToCoverageEnable = alpha_to_coverage.into();

        self.blend_desc = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND,
            data: blend_desc
        });
    }

    fn set_sample_mask(&mut self, sample_mask: u32) {
        self.sample_mask = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK,
            data: sample_mask
        });
    }

    fn set_raster_desc(&mut self, raster_state: &ral::RasterizerState, multisample_enable: bool) {
        let (depth_bias, depth_slope, depth_clamp) = raster_state.depth_bias.map_or((0.0, 0.0, 0.0), |bias| (bias.scale, bias.slope, bias.clamp));

        let raster_desc = D3D12_RASTERIZER_DESC1 {
            FillMode: raster_state.fill_mode.to_dx(),
            CullMode: raster_state.cull_mode.to_dx(),
            FrontCounterClockwise: (raster_state.winding_order == ral::WindingOrder::CCW).into(),
            DepthBias: depth_bias,
            DepthBiasClamp: depth_clamp,
            SlopeScaledDepthBias: depth_slope,
            DepthClipEnable: raster_state.depth_clip_enable.into(),
            MultisampleEnable: multisample_enable.into(),
            AntialiasedLineEnable: false.into(),
            ForcedSampleCount: 0,
            ConservativeRaster: raster_state.conservative_raster.to_dx(),
        };

        self.raster_desc = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER1,
            data: raster_desc
        });
    }

    fn set_strip_cut(&mut self, strip_cut: D3D12_INDEX_BUFFER_STRIP_CUT_VALUE) {
        self.strip_cut = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_IB_STRIP_CUT_VALUE,
            data: strip_cut
        });
    }

    fn set_topology_type(&mut self, topology_type: D3D12_PRIMITIVE_TOPOLOGY_TYPE) {
        self.topology_type = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY,
            data: topology_type
        });
    }

    fn set_render_target_formats(&mut self, rendertarget_formats: [Option<ral::Format>; ral::constants::MAX_RENDERTARGETS as usize]) {
        let mut rt_formats = [DXGI_FORMAT_UNKNOWN; 8];
        let mut rt_count = 0;

        for (idx, format) in rendertarget_formats.iter().enumerate() {
            if let Some(format) = format {
                rt_formats[idx] = format.to_dx();
                rt_count = idx as u32 + 1;
            }
        }

        let rt_format_array = D3D12_RT_FORMAT_ARRAY {
            RTFormats: rt_formats,
            NumRenderTargets: rt_count,
        };

        self.rt_formats = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS,
            data: rt_format_array
        });
    }

    fn set_depth_stencil_format(&mut self, dsv_format: DXGI_FORMAT) {
        self.dsv_format = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT,
            data: dsv_format
        });
    }

    fn set_sample_desc(&mut self, multisample_state: ral::MultisampleState) {
        let sample_desc = DXGI_SAMPLE_DESC {
            Count: multisample_state.samples.get_count(),
            Quality: 0,
        };

        self.sample_desc = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC,
            data: sample_desc
        });
    }

    fn set_depth_stencil_state(&mut self, depth_stencil: &ral::DepthStencilState) {
        let depth_stencil_desc = depth_stencil.to_dx();

        self.depth_stecnil = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL2,
            data: depth_stencil_desc
        });
    }
    
    fn set_input_layout(&mut self, input_layout: &[D3D12_INPUT_ELEMENT_DESC]) {
        let input_layout_desc = D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: input_layout.as_ptr(),
            NumElements: input_layout.len() as u32,
        };

        self.input_layout = Some(PipelineSubObject {
            subobject: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_INPUT_LAYOUT,
            data: input_layout_desc,
        })
    }

    fn build(self) -> ByteBuffer {
        let mut buffer = ByteBuffer::new();
        Self::write_sub_object(&mut buffer, self.root_signature);
        Self::write_sub_object(&mut buffer, self.vs_shader);
        Self::write_sub_object(&mut buffer, self.ps_shader);
        Self::write_sub_object(&mut buffer, self.blend_desc);
        Self::write_sub_object(&mut buffer, self.raster_desc);
        Self::write_sub_object(&mut buffer, self.strip_cut);
        Self::write_sub_object(&mut buffer, self.topology_type);
        Self::write_sub_object(&mut buffer, self.rt_formats);
        Self::write_sub_object(&mut buffer, self.dsv_format);
        Self::write_sub_object(&mut buffer, self.sample_desc);
        Self::write_sub_object(&mut buffer, self.depth_stecnil);
        Self::write_sub_object(&mut buffer, self.input_layout);
        buffer
    }


    fn write_sub_object<T: Copy>(stream: &mut ByteBuffer, subobject: Option<PipelineSubObject<T>>) {
        if let Some(subobject) = subobject {
            stream.write_raw(subobject);
            stream.pad_to_multiple(core::mem::align_of::<*const c_void>());
        }
    }
}