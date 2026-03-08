use super::graphics_context::GraphicsContext;
use std::borrow::Cow;

pub struct GCodePass{
    pub shader : wgpu::ShaderModule,
    pub pipeline_layout : wgpu::PipelineLayout,
    pub render_pipeline : wgpu::RenderPipeline,
}

impl GCodePass {
    pub fn new(gfx_ctx : &GraphicsContext) -> Self {
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label : None,
            source : wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("test.wgsl"))),
        };

        let shader = gfx_ctx.device.create_shader_module(shader_desc);

        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label : None,
            bind_group_layouts : &[],
            push_constant_ranges : &[],
        };
        let pipeline_layout = gfx_ctx.device.create_pipeline_layout(&pipeline_layout_desc);

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label : None,
            layout : Some(&pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : Some("vs_main"),
                buffers : &[],
                compilation_options : Default::default(),
            },
            fragment : Some(wgpu::FragmentState {
                module : &shader,
                entry_point : Some("fs_main"),
                compilation_options : Default::default(),
                targets : &[Some(gfx_ctx.surface_format.into())],
            }),
            primitive : wgpu::PrimitiveState::default(),
            depth_stencil : None,
            multisample : wgpu::MultisampleState::default(),
            multiview : None,
            cache : None
        };

        let render_pipeline = gfx_ctx.device.create_render_pipeline(&render_pipeline_desc);

        GCodePass{
            shader,
            pipeline_layout,
            render_pipeline,
        }
    }
}
