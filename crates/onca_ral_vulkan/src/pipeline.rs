use std::sync::{Weak, Arc};

use onca_common::prelude::*;
use onca_ral as ral;

use ash::vk;
use ral::{HandleImpl, TextureAspect};

use crate::{device::Device, utils::{ToRalError, ToVulkan}, vulkan::AllocationCallbacks, shader::Shader, sampler::Sampler, descriptor::{MUTABLE_DESCRIPTOR_TYPES, DescriptorTableLayout}};


pub struct PipelineLayout {
    pub layout:          vk::PipelineLayout,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

impl PipelineLayout {
    pub unsafe fn new(device: &Device, desc: &ral::PipelineLayoutDesc) -> ral::Result<ral::PipelineLayoutInterfaceHandle> {
        let mut layouts = Vec::new();

        if let Some(tables) = &desc.descriptor_tables {
            for table in tables {
                let table_layout = table.interface().as_concrete_type::<DescriptorTableLayout>();
                layouts.push(table_layout.handle);
            }
        }

        if let Some(inlines) = &desc.inline_descriptors {
            for inline in inlines {
                let binding = [vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_type(inline.descriptor_type.to_vulkan())
                    .descriptor_count(1)
                    .stage_flags(inline.visibility.to_vulkan())
                    .build()    
                ];
                
                let mutable_type_list = [vk::MutableDescriptorTypeListEXT::builder()
                    .descriptor_types(&MUTABLE_DESCRIPTOR_TYPES)
                    .build()];
                let mut mutable_types = vk::MutableDescriptorTypeCreateInfoEXT::builder()
                    .mutable_descriptor_type_lists(&mutable_type_list);

                let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .push_next(&mut mutable_types)
                .flags(vk::DescriptorSetLayoutCreateFlags::DESCRIPTOR_BUFFER_EXT  | vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR)
                .bindings(&binding);
                
                let layout = device.device.create_descriptor_set_layout(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
                layouts.push(layout);
            }
        }

        let mut push_constant_offset = 0;
        let mut push_constants = Vec::new();

        if let Some(constants) = &desc.constant_ranges {
            for constant in constants {
                    let size = constant.count as u32 * 4;
                    let range = vk::PushConstantRange::builder()
                    .stage_flags(constant.visibility.to_vulkan())
                    .size(size)
                    .offset(push_constant_offset)
                    .build();
                push_constants.push(range);
                push_constant_offset += size;
            }
        }

        match &desc.static_samplers {
            Some(samplers) => {
                let mut bindings = Vec::new();
                for (idx, sampler) in samplers.iter().enumerate() {
                    let vk_sampler = sampler.interface().as_concrete_type::<Sampler>().sampler;

                    sampler.desc();

                    bindings.push(vk::DescriptorSetLayoutBinding::builder()
                        .binding(idx as u32)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .immutable_samplers(&[vk_sampler])
                        .stage_flags(sampler.desc().visibility.to_vulkan())
                        .build()
                    );
                }

                let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                    .flags(vk::DescriptorSetLayoutCreateFlags::DESCRIPTOR_BUFFER_EXT | vk::DescriptorSetLayoutCreateFlags::EMBEDDED_IMMUTABLE_SAMPLERS_EXT)
                    .bindings(&bindings);

                let layout = device.device.create_descriptor_set_layout(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;
                layouts.push(layout);
            },
            None => {},
        }

        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&layouts)
            .push_constant_ranges(&push_constants);

        let layout = device.device.create_pipeline_layout(&create_info, device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.to_ral_error())?;

        Ok(ral::PipelineLayoutInterfaceHandle::new(
            PipelineLayout {
                layout,
                device: Arc::downgrade(&device.device),
                alloc_callbacks: device.alloc_callbacks.clone(),
            }
        ))
    }
}

impl ral::PipelineLayoutInterface for PipelineLayout {

}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let device = Weak::upgrade(&self.device).unwrap();
        unsafe { device.destroy_pipeline_layout(self.layout, self.alloc_callbacks.get_some_vk_callbacks()) };
    }
}

//==============================================================================================================================

pub struct Pipeline {
    pub pipeline:        vk::Pipeline,
    pub device:          Weak<ash::Device>,
    pub alloc_callbacks: AllocationCallbacks,
}

// TODO: Look into more flags: https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPipelineCreateFlagBits.html

impl Pipeline {
    pub unsafe fn new_graphics(device: &Device, desc: &ral::GraphicsPipelineDesc) -> ral::Result<ral::PipelineInterfaceHandle> {
        scoped_alloc!(AllocId::TlsTemp);

        let vertex_shader = desc.vertex_shader.interface().as_concrete_type::<Shader>();
        let pixel_shader = desc.pixel_shader.interface().as_concrete_type::<Shader>();

        let mut shader_stages = Vec::with_capacity(2);
        shader_stages.push(vertex_shader.get_shader_stage_info(ral::ShaderType::Vertex));
        shader_stages.push(pixel_shader.get_shader_stage_info(ral::ShaderType::Pixel));

        
        let mut vertex_bindings = Vec::new();
        let mut vertex_binding_divisors = Vec::new();
        let mut vertex_attributes = Vec::new();
        
        if let Some(input_layout) = &desc.input_layout {
            vertex_bindings.reserve(input_layout.elements.len());
            vertex_binding_divisors.reserve(input_layout.elements.len());
            vertex_attributes.reserve(input_layout.elements.len());

            let mut location_counters = [0; ral::constants::MAX_VERTEX_INPUT_BUFFERS as usize];
            let mut bindings_created  = [false; ral::constants::MAX_VERTEX_INPUT_BUFFERS as usize];

            let strides = input_layout.calculate_strides();
            for element in &input_layout.elements {
                let step_rate = element.step_rate.to_vulkan();

                let location = location_counters[element.input_slot as usize];
                location_counters[element.input_slot as usize] += 1;
                
                vertex_attributes.push(vk::VertexInputAttributeDescription::builder()
                // TODO: Currently we just expect vulkan locations to match the order of vertex attributes, but in the future this should be handled by either shader reflection or a custom shader system (shaders written in rust)
                .location(location)
                .binding(element.input_slot as u32)
                .format(element.format.to_vulkan())
                .offset(element.offset as u32)
                .build());

                if !bindings_created[element.input_slot as usize] {
                    bindings_created[element.input_slot as usize] = true;

                    vertex_bindings.push(vk::VertexInputBindingDescription::builder()
                    .binding(element.input_slot as u32)
                    .stride(strides[element.input_slot as usize] as u32)
                    .input_rate(step_rate.0)
                    .build());

                    if step_rate.0 == vk::VertexInputRate::INSTANCE {
                        vertex_binding_divisors.push(vk::VertexInputBindingDivisorDescriptionEXT::builder()
                        .binding(element.input_slot as u32)
                        .divisor(step_rate.1)
                        .build());
                    }
                }

                
            }
        }

        let mut vertex_input_devisors = vk::PipelineVertexInputDivisorStateCreateInfoEXT::builder()
            .vertex_binding_divisors(&vertex_binding_divisors);

        let mut vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attributes);

        if !vertex_binding_divisors.is_empty() {
            vertex_input_state = vertex_input_state.push_next(&mut vertex_input_devisors);
        }

        let vertex_input_state = vertex_input_state.build();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(desc.topology.to_vulkan())
            .primitive_restart_enable(desc.primitive_restart != ral::PrimitiveRestart::None);

        // We handle viewport dynamically, but we still need to pass this struct, so just create the default one
        let viewport_state = vk::PipelineViewportStateCreateInfo::default();

        let (depth_bias_enable, depth_bias, depth_bias_slope, depth_bias_clamp) = match desc.rasterizer_state.depth_bias {
            Some(bias) => (true, bias.scale, bias.slope, bias.clamp),
            None => (false, 0.0, 0.0, 0.0),
        };

        let mut conservative_rasterization_state = vk::PipelineRasterizationConservativeStateCreateInfoEXT::builder()
            .conservative_rasterization_mode(desc.rasterizer_state.conservative_raster.to_vulkan())
            .extra_primitive_overestimation_size(1.0 / 256.0); // This matches Tier 3 for DX12

        let mut clip_enable_state = vk::PipelineRasterizationDepthClipStateCreateInfoEXT::builder()
            .depth_clip_enable(desc.rasterizer_state.depth_clip_enable);

        // We don't use line stipple
        let mut line_raster_state = vk::PipelineRasterizationLineStateCreateInfoEXT::builder()
            .line_rasterization_mode(desc.rasterizer_state.line_raster_mode.to_vulkan());

        let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(true) // Is always enabled to match the functionality of other APIs
            .rasterizer_discard_enable(false) // Is always turned off, as we don't support stream output, so we don't need this
            .polygon_mode(desc.rasterizer_state.fill_mode.to_vulkan())
            .cull_mode(desc.rasterizer_state.cull_mode.to_vulkan())
            .front_face(desc.rasterizer_state.winding_order.to_vulkan())
            .depth_bias_enable(depth_bias_enable)
            .depth_bias_constant_factor(depth_bias)
            .depth_bias_slope_factor(depth_bias_slope)
            .depth_bias_clamp(depth_bias_clamp)
            .line_width(if desc.rasterizer_state.line_raster_mode == ral::LineRasterizationMode::RectangularWide { 1.4 } else { 1.0 })
            .push_next(&mut conservative_rasterization_state)
            .push_next(&mut clip_enable_state)
            .push_next(&mut line_raster_state);

        let depth_stencil_state = desc.depth_stencil_state.to_vulkan();

        let mut rendertarget_formats = [vk::Format::UNDEFINED; ral::constants::MAX_RENDERTARGETS as usize];
        let mut rendertarget_count = ral::constants::MAX_RENDERTARGETS as usize;

        for (idx, format_opt) in desc.rendertarget_formats.iter().enumerate() {
            if let Some(format) = format_opt {
                rendertarget_formats[idx] = format.to_vulkan();
                rendertarget_count = idx + 1;
            }
        }

        let depth_format = desc.depth_stencil_format.map_or(vk::Format::UNDEFINED, |format| if format.aspect().contains(TextureAspect::Depth) { format.to_vulkan() } else { vk::Format::UNDEFINED });
        let stencil_format = desc.depth_stencil_format.map_or(vk::Format::UNDEFINED, |format| if format.aspect().contains(TextureAspect::Stencil) { format.to_vulkan() } else { vk::Format::UNDEFINED });

        let mut blend_attachments = Vec::new();
        let blend_state = match desc.blend_state {
            ral::BlendState::None => vk::PipelineColorBlendStateCreateInfo::default(),
            ral::BlendState::LogicOp(logic_op) => vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op_enable(true)
                .logic_op(logic_op.to_vulkan())
                .build(),
            ral::BlendState::Blend(blend_states) => {
                for state in blend_states.iter() {
                    blend_attachments.push(state.to_vulkan());
                }

                vk::PipelineColorBlendStateCreateInfo::builder()
                    .attachments(&blend_attachments[..rendertarget_count])
                    .build()
            },
        };

        let dynamic_states = [
            vk::DynamicState::VIEWPORT_WITH_COUNT,
            vk::DynamicState::SCISSOR_WITH_COUNT,
            vk::DynamicState::DEPTH_BIAS,
            vk::DynamicState::BLEND_CONSTANTS,
            vk::DynamicState::DEPTH_BOUNDS,
            vk::DynamicState::STENCIL_REFERENCE,
            vk::DynamicState::PRIMITIVE_TOPOLOGY,
            vk::DynamicState::VERTEX_INPUT_BINDING_STRIDE,
        ];
    
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        let layout = desc.pipeline_layout.interface().as_concrete_type::<PipelineLayout>().layout;

        let mut rendering_create_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(&rendertarget_formats[..rendertarget_count])
            .depth_attachment_format(depth_format)
            .stencil_attachment_format(stencil_format)
            .view_mask(desc.view_mask.unwrap_or_default() as u32);
    
        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut rendering_create_info)
            .flags(vk::PipelineCreateFlags::DESCRIPTOR_BUFFER_EXT)
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .build();

        let pipeline = device.device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], device.alloc_callbacks.get_some_vk_callbacks()).map_err(|err| err.1.to_ral_error())?[0];

        Ok(ral::PipelineInterfaceHandle::new( Pipeline {
            pipeline,
            device: Arc::downgrade(&device.device),
            alloc_callbacks: device.alloc_callbacks.clone(),
        }))
    }
}

impl ral::PipelineInterface for Pipeline {

}

impl Drop for Pipeline {
    fn drop(&mut self) {
        let device = Weak::upgrade(&self.device).unwrap();
        unsafe { device.destroy_pipeline(self.pipeline, self.alloc_callbacks.get_some_vk_callbacks()) };
    }
}