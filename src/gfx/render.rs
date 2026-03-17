pub use wgpu;
use crate::gfx::gcode_render::SEGMENT_COUNT;

use super::gcode_render::{GCodePass, Point, GCodeRenderData};
use super::graphics_context::GraphicsContext;
use super::camera;
use super::super::interface::InterfaceContext;

use winit::window::{Window, WindowId, WindowAttributes};
use winit::{dpi::PhysicalSize};
use std::sync::Arc;
use std::time::Instant;
use glam::Vec3;

pub struct GFXRenderer {
    pub gfx_ctx : GraphicsContext,
    gcode_pass : GCodePass,
    camera : camera::Camera,
}

impl GFXRenderer {
    pub async fn new(window : Arc<Window>) -> Self {
        let mut instance_desc = wgpu::InstanceDescriptor::default();
        let instance = wgpu::Instance::new(&instance_desc);

        let surface : wgpu::Surface = instance.create_surface(window.clone()).unwrap();
        let adapter : wgpu::Adapter = 
            instance.request_adapter(&wgpu::RequestAdapterOptions{
                compatible_surface : Some(&surface),
                ..Default::default()
                })
            .await
            .unwrap();

        let device_desc = wgpu::DeviceDescriptor::default();
        let (device, queue) = 
            adapter.request_device(&device_desc)
            .await
            .unwrap();
        
        let size : PhysicalSize<u32> = window.inner_size();
        let capabilities : wgpu::SurfaceCapabilities = surface.get_capabilities(&adapter);
        let surface_format : wgpu::TextureFormat = capabilities.formats[0];

        let (depth_texture, depth_view) = Self::create_depth_texture(&device, &size);

        let gfx_ctx : GraphicsContext = GraphicsContext{
                window,
                device,
                queue,
                size,
                surface,
                surface_format,
                depth_texture,
                depth_view,
        };

        GFXRenderer::configure_surface(&gfx_ctx);

        let mut gcode_pass : GCodePass = GCodePass::new(&gfx_ctx);
        gcode_pass.create_camera_buffer(&gfx_ctx);

        GFXRenderer {
            gfx_ctx,
            gcode_pass,
            camera : camera::Camera::build_camera_matrix(
                Vec3{x : 6.0, y : 6.0, z : 6.0}, 
                Vec3{x : 0.0, y : 0.0, z : 0.0}, 
                1.0),
        }
    }

    pub fn reconfigure_surface(&mut self, new_size : PhysicalSize<u32>) {
        self.gfx_ctx.size = new_size;
        GFXRenderer::configure_surface(&self.gfx_ctx);
        let (depth_texture, depth_view) = 
            Self::create_depth_texture(&self.gfx_ctx.device, &self.gfx_ctx.size);

        self.gfx_ctx.depth_texture = depth_texture;
        self.gfx_ctx.depth_view = depth_view;
    }

    fn create_depth_texture(device : &wgpu::Device, size : &PhysicalSize<u32>) ->
        (wgpu::Texture, wgpu::TextureView)
    {
        let size = wgpu::Extent3d {
            width : size.width,
            height : size.height,
            depth_or_array_layers : 1,
        };

        let texture_desc = wgpu::TextureDescriptor {
            label : Some("Depth texture"),
            size : size,
            mip_level_count : 1,
            sample_count : 1,
            dimension : wgpu::TextureDimension::D2,
            format : wgpu::TextureFormat::Depth24Plus,
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats : &[],
        };

        let texture : wgpu::Texture = device.create_texture(&texture_desc);

        let view : wgpu::TextureView = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    fn configure_surface(gfx_ctx : &GraphicsContext) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: gfx_ctx.surface_format,
            // Request compatibility with the sRGB-format texture for later
            view_formats: vec![gfx_ctx.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: gfx_ctx.size.width,
            height: gfx_ctx.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        gfx_ctx.surface.configure(&gfx_ctx.device, &surface_config);

        println!("Surface created");
    }

    pub fn get_aspect(&self) -> f32 {
        (self.gfx_ctx.size.width as f32) / (self.gfx_ctx.size.height as f32)
    }

    pub fn update_camera(&self, camera : &camera::Camera)
    {
        self.gfx_ctx.queue.write_buffer(
            &self.gcode_pass.render_buffers.as_ref().unwrap().camera_buffer, 
            0, 
            //bytemuck::cast_slice(camera.view_proj.as_ref()));
            bytemuck::bytes_of(camera));
    }

    pub fn update_gcode_points(&mut self, render_data : &GCodeRenderData) {
        self.gcode_pass.rengenerate_geometry(&self.gfx_ctx, render_data);
    }

    pub fn render(&mut self, interface_context : &mut InterfaceContext) {
        let gfx_ctx : &mut GraphicsContext = &mut self.gfx_ctx;
        let surface_texture = gfx_ctx.surface.get_current_texture()
            .expect("Failed to acuire swapchain image");

        let mut encoder = gfx_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // This should probably be moved to initialization?
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor{
            format : Some(gfx_ctx.surface_format.add_srgb_suffix()),
            ..Default::default()
        });

        let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
            view : &gfx_ctx.depth_view,
            depth_ops : Some(wgpu::Operations {
                load : wgpu::LoadOp::Clear(1.0),
                store : wgpu::StoreOp::Store,
            }),
            stencil_ops : None,
        };

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label : None,
            color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                view : &texture_view,
                resolve_target : None,
                ops : wgpu::Operations {
                    load : wgpu::LoadOp::Clear(interface_context.clear_colour),
                    store : wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment : Some(depth_stencil_attachment),
            timestamp_writes : None,
            occlusion_query_set : None,
        });

        if self.gcode_pass.index_count > 0 {
            let g_buffers = self.gcode_pass.gcode_buffers.as_ref().unwrap();
            let r_buffers = self.gcode_pass.render_buffers.as_ref().unwrap();

            renderpass.set_pipeline(&self.gcode_pass.render_pipeline);
            renderpass.set_vertex_buffer(0, g_buffers.vertex_buffer.slice(..));
            renderpass.set_index_buffer(g_buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            renderpass.set_bind_group(0, Some(&r_buffers.render_bind_group), &[]);
            renderpass.draw_indexed(0..self.gcode_pass.index_count, 0, 0..1);
//            println!("{}", self.gcode_pass.index_count);
        }

        drop(renderpass);

        let mut renderpass2 = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label : None,
            color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                view : &texture_view,
                resolve_target : None,
                ops : wgpu::Operations {
                    load : wgpu::LoadOp::Load,
                    store : wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment : None,
            timestamp_writes : None,
            occlusion_query_set : None,
        });

        interface_context.renderer.render(
            interface_context.context.render(),
            &gfx_ctx.queue,
            &gfx_ctx.device,
            &mut renderpass2
        ).expect("imgui rendering failed");

        drop(renderpass2);

        gfx_ctx.queue.submit([encoder.finish()]);
        gfx_ctx.window.pre_present_notify();
        surface_texture.present();
    }
}

