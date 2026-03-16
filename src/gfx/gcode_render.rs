use super::graphics_context::GraphicsContext;
use super::camera;
use std::borrow::Cow;
use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};
use rand;

pub const SEGMENT_COUNT : usize = 500;

pub struct GCodeBuffers {
    points_buffer : wgpu::Buffer,
    range_buffer : wgpu::Buffer,
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

    pub index_count : u32,
}

// TODO: Can add additional information such as feed rate
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Point {
    pub p : [f32; 3],
    pub _padding : f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, Default)]
pub struct SegmentRange {
    pub start_vertex : u32,
    pub end_vertex : u32,
    pub start_index : u32,
}

#[derive(Default)]
pub struct GCodeRenderData {
    pub points : Vec<Point>,
    pub segment_ranges : Vec<SegmentRange>,
    pub vertex_count : u32,
    pub index_count : u32,
}

// TODO: Maybe add bounding box for automatic camera placement
impl GCodeRenderData {
    pub fn default() -> Self {
        Self {
            points : Vec::<Point>::new(),
            segment_ranges : Vec::<SegmentRange>::new(),
            vertex_count : 0,
            index_count : 0,
        }
    }
}

// TODO: Normals
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos : [f32; 3],
    pub dist : f32,
    pub normal : [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct QuadGenParam {
    point_count : u32,
    range_count : u32,
    line_width : f32,
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
                // Points
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
                wgpu::BindGroupLayoutEntry {
                    binding : 1,
                    visibility : wgpu::ShaderStages::COMPUTE,
                    ty : wgpu::BindingType::Buffer {
                        ty : wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset : false,
                        min_binding_size : None
                    },
                    count : None
                },
                // Segment ranges
                // Vertex buffer 
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
                // Index buffer
                wgpu::BindGroupLayoutEntry {
                    binding : 3,
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
                    binding : 4,
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
            },
            wgpu::VertexAttribute {
                offset : 12,
                shader_location : 1,
                format : wgpu::VertexFormat::Float32
            },
            wgpu::VertexAttribute {
                offset : 16,
                shader_location : 2,
                format : wgpu::VertexFormat::Float32x3
            },
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
        
        let depth_stencil = wgpu::DepthStencilState {
            format : wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled : true,
            depth_compare : wgpu::CompareFunction::Less,
            stencil : wgpu::StencilState::default(),
            bias : wgpu::DepthBiasState::default(),
        };

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
            depth_stencil : Some(depth_stencil),
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

            index_count : 0,   
        }
    }

    pub fn rengenerate_geometry(&mut self, gfx_ctx : &GraphicsContext, render_data : &GCodeRenderData)
    {
        // Line segment points -> Compute
        let points_buf_desc = wgpu::BufferDescriptor {
            label : Some("point buffer"),
            size : (render_data.points.len() * size_of::<Point>()) as u64,
            usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        };

        // Line segment ranges -> Compute
        let range_buf_desc = wgpu::BufferDescriptor {
            label : Some("range buffer"),
            size : (render_data.segment_ranges.len() * size_of::<SegmentRange>()) as u64,
            usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        };

        // Params -> Compute
        let param_buf_desc = wgpu::BufferDescriptor {
            label : Some("params buffer"),
            size : size_of::<QuadGenParam>() as u64,
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        };

        // Quads -> Vertex shader
        let vert_buf_desc = wgpu::BufferDescriptor {
            label : Some("vertex buffer"),
            size : (size_of::<Vertex>() * render_data.vertex_count as usize) as u64,
            usage : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation : false
        };

        let index_buf_desc = wgpu::BufferDescriptor {
            label : Some("index buffer"),
            size : (size_of::<u32>() * render_data.index_count as usize) as u64,
            usage : wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation : false
        };

        let points_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&points_buf_desc);
        let range_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&range_buf_desc);
        let params_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&param_buf_desc);
        let vertex_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&vert_buf_desc);
        let index_buffer : wgpu::Buffer = gfx_ctx.device.create_buffer(&index_buf_desc);

        let bind_group_desc = wgpu::BindGroupDescriptor {
            label : Some("Comp bind group descriptor"),
            layout : &self.comp_bind_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : points_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : range_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 2,
                    resource : vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 3,
                    resource : index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding : 4,
                    resource : params_buffer.as_entire_binding(),
                },
            ],
        };

        let bind_group : wgpu::BindGroup = gfx_ctx.device.create_bind_group(&bind_group_desc);

        gfx_ctx.queue.write_buffer(&points_buffer, 0, bytemuck::cast_slice(&render_data.points));
        gfx_ctx.queue.write_buffer(&range_buffer, 0, bytemuck::cast_slice(&render_data.segment_ranges));

        self.index_count = render_data.index_count as u32;

        let params = QuadGenParam {
            point_count : render_data.points.len() as u32,
            range_count : render_data.segment_ranges.len() as u32,
            line_width : 0.01,
        };
        println!("{:?}", params);
        gfx_ctx.queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&params));

        self.gcode_buffers = Some(GCodeBuffers{
            points_buffer : points_buffer,
            range_buffer : range_buffer,
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
        compute_pass.dispatch_workgroups(100, 1, 1);
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
