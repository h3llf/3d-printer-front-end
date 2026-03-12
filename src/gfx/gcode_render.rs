use super::graphics_context::GraphicsContext;
use super::camera;
use std::borrow::Cow;
use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};
use rand;

pub const SEGMENT_COUNT : usize = 100;

pub struct GCodeBuffers {
    line_buffer : wgpu::Buffer,
    pub vertex_buffer : wgpu::Buffer,
    pub index_buffer : wgpu::Buffer,
    params_buffer : wgpu::Buffer,
    comp_bind_group : wgpu::BindGroup,
}

pub struct RenderBuffers {
    pub camera_buffer : wgpu::Buffer,
    pub render_bind_group : wgpu::BindGroup
}

pub struct GCodePass {
    pub vertex_shader : wgpu::ShaderModule,
    pub fragment_shader : wgpu::ShaderModule,
    pub render_bind_group_layout : wgpu::BindGroupLayout,
    pub pipeline_layout : wgpu::PipelineLayout,
    pub render_pipeline : wgpu::RenderPipeline,
    
    pub comp_shader : wgpu::ShaderModule,
    pub comp_bind_group_layout : wgpu::BindGroupLayout,
    pub comp_pipeline_layout : wgpu::PipelineLayout,
    pub comp_pipeline : wgpu::ComputePipeline,

    pub render_buffers : Option<RenderBuffers>,
    pub gcode_buffers : Option<GCodeBuffers>,
    pub line_segments : Option<[LineSegment; SEGMENT_COUNT]>,
}

// TODO: Use points not line segments
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct LineSegment {
    pub p0 : [f32; 3],
    pub p1 : [f32; 3],
}

pub struct SegmentRange {
    pub start_index : u32,
    pub end_index : u32
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos : [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QuadGenParam {
    segment_count : u32,
}

// Placeholder vertex & index buffer
impl GCodePass {
    pub fn new(gfx_ctx : &GraphicsContext) -> Self {
        // Compute pipeline
        let comp_shader_desc = wgpu::ShaderModuleDescriptor {
            label : None,
            source : wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("quad_gen.wgsl"))),
        };
        let comp_shader : wgpu::ShaderModule = gfx_ctx.device.create_shader_module(comp_shader_desc);

        let comp_bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
            label : Some("Comp layout"),
            entries : &[
                // Line segments
                wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::COMPUTE,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset : false,
                        min_binding_size : None
                    },
                    count : None
                },
                // Vertex buffer 
                wgpu::BindGroupLayoutEntry {
                    binding : 1,
                    visibility : wgpu::ShaderStages::COMPUTE,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset : false,
                        min_binding_size : None
                    },
                    count : None
                },
                // Index buffer
                wgpu::BindGroupLayoutEntry {
                    binding : 2,
                    visibility : wgpu::ShaderStages::COMPUTE,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset : false,
                        min_binding_size : None
                    },
                    count : None
                },
                // Parameter uniform
                wgpu::BindGroupLayoutEntry {
                    binding : 3,
                    visibility : wgpu::ShaderStages::COMPUTE,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset : false,
                        min_binding_size : None
                    },
                    count : None
                },
            ],
        };

        let comp_bind_group_layout : wgpu::BindGroupLayout = 
            gfx_ctx.device.create_bind_group_layout(&comp_bind_group_layout_desc);

        let comp_pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label : Some("Comp pipeline layout"),
            bind_group_layouts : &[&comp_bind_group_layout],
            push_constant_ranges : &[],
        };

        let comp_pipeline_layout : wgpu::PipelineLayout = gfx_ctx.device.create_pipeline_layout(&comp_pipeline_layout_desc);

        let comp_pipeline_desc = wgpu::ComputePipelineDescriptor {
            label : Some("Compute pipeline"),
            layout : Some(&comp_pipeline_layout),
            module : &comp_shader,
            entry_point : Some("main"),
            compilation_options : wgpu::PipelineCompilationOptions::default(),
            cache : None
        };

        let comp_pipeline : wgpu::ComputePipeline = 
            gfx_ctx.device.create_compute_pipeline(&comp_pipeline_desc);

        // Render pipeline
        let render_buffer_layout_desc = wgpu::BindGroupLayoutDescriptor {
            label : Some("Comp layout"),
            entries : &[
                // Line segments
                wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::VERTEX,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset : false,
                        min_binding_size : None
                    },
                    count : None,
                }
            ]
        };
        let render_buffer_layout = 
            gfx_ctx.device.create_bind_group_layout(&render_buffer_layout_desc);

        let vs_shader_desc = wgpu::ShaderModuleDescriptor {
            label : None,
            source : wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("quad_render_vs.wgsl"))),
        };

        let fs_shader_desc = wgpu::ShaderModuleDescriptor {
            label : None,
            source : wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("quad_render_fs.wgsl"))),
        };

        let vertex_shader : wgpu::ShaderModule = gfx_ctx.device.create_shader_module(vs_shader_desc);
        let fragment_shader : wgpu::ShaderModule = gfx_ctx.device.create_shader_module(fs_shader_desc);

        const ATTRIBUTES : &[wgpu::VertexAttribute] = &[
            wgpu::VertexAttribute {
                offset : 0,
                shader_location : 0,
                format : wgpu::VertexFormat::Float32x3
            }
        ];

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride : size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode : wgpu::VertexStepMode::Vertex,
            attributes : ATTRIBUTES,
        };

        let render_pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label : Some("Render pipeline layout"),
            bind_group_layouts : &[&render_buffer_layout],
            push_constant_ranges : &[],
        };
        let render_pipeline_layout : wgpu::PipelineLayout = 
            gfx_ctx.device.create_pipeline_layout(&render_pipeline_layout_desc);

        let mut primitive_state = wgpu::PrimitiveState::default();
        primitive_state.cull_mode = None;
        
        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label : Some("Render pipeline descriptor"),
            layout : Some(&render_pipeline_layout),
             vertex : wgpu::VertexState {
                module : &vertex_shader,
                entry_point : Some("vs_main"),
                buffers : &[vertex_buffer_layout],
                compilation_options : Default::default(),
            },
            fragment : Some(wgpu::FragmentState {
                module : &fragment_shader,
                entry_point : Some("fs_main"),
                compilation_options : Default::default(),
                targets : &[Some(gfx_ctx.surface_format.into())],
            }),
            primitive : primitive_state,
            depth_stencil : None,
            multisample : wgpu::MultisampleState::default(),
            multiview : None,
            cache : None           
        };

        let render_pipeline : wgpu::RenderPipeline = 
            gfx_ctx.device.create_render_pipeline(&render_pipeline_desc);

        GCodePass{
            vertex_shader : vertex_shader,
            fragment_shader : fragment_shader,
            render_bind_group_layout : render_buffer_layout,
            pipeline_layout : render_pipeline_layout,
            render_pipeline : render_pipeline,

            comp_shader : comp_shader,
            comp_bind_group_layout : comp_bind_group_layout,
            comp_pipeline_layout : comp_pipeline_layout,
            comp_pipeline : comp_pipeline,

            render_buffers : None,
            gcode_buffers : None,
            line_segments : None
        }
    }

    pub fn rengenerate_geometry(&mut self, gfx_ctx : &GraphicsContext)
    {
        // Line segments -> Compute
        let line_buf_desc = wgpu::BufferDescriptor {
            label : None,
            size : (SEGMENT_COUNT * size_of::<LineSegment>()) as u64,
            usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        };

        // Params -> Compute
        let param_buf_desc = wgpu::BufferDescriptor {
            label : None,
            size : size_of::<QuadGenParam>() as u64,
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        };

        // Quads -> Vertex shader
        let vert_buf_desc = wgpu::BufferDescriptor {
            label : None,
            size : (size_of::<Vertex>() * SEGMENT_COUNT * 4) as u64,
            usage : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation : false
        };

        let index_buf_desc = wgpu::BufferDescriptor {
            label : None,
            size : (size_of::<u32>() * SEGMENT_COUNT * 6) as u64,
            usage : wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation : false
        };

        let line_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&line_buf_desc);
        let params_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&param_buf_desc);
        let vertex_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&vert_buf_desc);
        let index_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&index_buf_desc);

        let bind_group_desc = wgpu::BindGroupDescriptor {
            label : Some("Comp bind group descriptor"),
            layout : &self.comp_bind_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 2,
                    resource : index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 3,
                    resource : params_buffer.as_entire_binding(),
                },
            ],
        };

        let bind_group : wgpu::BindGroup = gfx_ctx.device.create_bind_group(&bind_group_desc);

        // TODO: Remove, testing
        let mut lines : [LineSegment; SEGMENT_COUNT] = [LineSegment{p0 : [0.0, 0.0, 0.0], p1 : [0.0, 1.0, 0.0] }; SEGMENT_COUNT];
        for i in 0..SEGMENT_COUNT {
            let rx1 : f32 = rand::random_range(-5.0..5.0);
            let ry1 : f32 = rand::random_range(-1.0..1.0);
            let rz1 : f32 = rand::random_range(-5.0..5.0);

            let rx2 : f32 = rand::random_range(-5.0..5.0);
            let ry2 : f32 = rand::random_range(-1.0..1.0);
            let rz2 : f32 = rand::random_range(-5.0..5.0);
            let line = LineSegment{p0 : [rx1, ry1, rz1], p1 : [rx2, ry2, rz2] };
            println!("{:?}", line);
            lines[i] = line;
        }

        self.line_segments = Some(lines);

        gfx_ctx.queue.write_buffer(&line_buffer, 0, bytemuck::cast_slice(&lines));

        let line_count : u32 = lines.len() as u32;

        gfx_ctx.queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&line_count));

        self.gcode_buffers = Some(GCodeBuffers{
            line_buffer : line_buffer,
            vertex_buffer : vertex_buffer,
            index_buffer : index_buffer,
            params_buffer : params_buffer,
            comp_bind_group : bind_group,
        });

        self.run_compute_pipeline(gfx_ctx);
    }

    fn run_compute_pipeline(&self, gfx_ctx : &GraphicsContext) {
        let mut encoder = gfx_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let compute_pass_desc = wgpu::ComputePassDescriptor {
            label : Some("Compute pass desc"),
            timestamp_writes : None,
        };

        let mut compute_pass = encoder.begin_compute_pass(&compute_pass_desc);
        compute_pass.set_pipeline(&self.comp_pipeline);
        compute_pass.set_bind_group(0, Some(&self.gcode_buffers.as_ref().unwrap().comp_bind_group), &[]);
        compute_pass.dispatch_workgroups(10, 1, 1);
        drop(compute_pass);

        gfx_ctx.queue.submit([encoder.finish()]);
        println!("queue submit");
    }

    pub fn create_camera_buffer(&mut self, gfx_ctx : &GraphicsContext) {
        let camera_buf_desc = wgpu::BufferDescriptor {
            label : None,
            size : size_of::<camera::Camera>() as u64,
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        };
        
        let camera_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&camera_buf_desc);

        let camera : camera::Camera = camera::Camera::build_camera_matrix(
            Vec3{x : 6.0, y : 6.0, z : 6.0}, 
            Vec3{x : 0.0, y : 0.0, z : 0.0}, 
            1.0); // TODO: Use correct aspect ratio

        gfx_ctx.queue.write_buffer(&camera_buffer, 0, 
            bytemuck::cast_slice(camera.view_proj.as_ref()));

        let bind_group_desc = wgpu::BindGroupDescriptor {
            label : Some("Camera bind group"),
            layout : &self.render_bind_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : camera_buffer.as_entire_binding(),
                },
            ],
        };

        let bind_group : wgpu::BindGroup = gfx_ctx.device.create_bind_group(&bind_group_desc);

        self.render_buffers = Some(RenderBuffers { 
                camera_buffer : camera_buffer, 
                render_bind_group: bind_group,
        });
    }
}
