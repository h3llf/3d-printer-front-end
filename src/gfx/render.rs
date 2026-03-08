pub use wgpu;
use super::gcode_render::GCodePass;
use super::graphics_context::GraphicsContext;

use winit::window::{Window, WindowId, WindowAttributes};
use winit::{dpi::PhysicalSize};
use std::sync::Arc;
use std::time::Instant;
use super::super::interface::InterfaceContext;

pub struct GFXRenderer {
    pub gfx_ctx : GraphicsContext,
    gcode_pass : GCodePass,
}

impl GFXRenderer {
    pub async fn new(window : Arc<Window>) -> Self {
        let instance_desc = wgpu::InstanceDescriptor::default();
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
        
        let size /*: PhysicalSize<u32>*/ = window.inner_size();
        let capabilities : wgpu::SurfaceCapabilities = surface.get_capabilities(&adapter);
        let surface_format : wgpu::TextureFormat = capabilities.formats[0];

        let gfx_ctx : GraphicsContext = GraphicsContext{
                window,
                device,
                queue,
                size,
                surface,
                surface_format,
        };

        GFXRenderer::configure_surface(&gfx_ctx);

        let gcode_pass : GCodePass = GCodePass::new(&gfx_ctx);

        GFXRenderer {
            gfx_ctx,
            gcode_pass

        }
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

    pub fn render(&mut self, interface_context : &mut InterfaceContext) {
        let gfx_ctx : &mut GraphicsContext = &mut self.gfx_ctx;
        let surface_texture = gfx_ctx.surface.get_current_texture()
            .expect("Failed to acuire swapchain image");

        let mut encoder = gfx_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor{
            format : Some(gfx_ctx.surface_format.add_srgb_suffix()),
            ..Default::default()
        });

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
            depth_stencil_attachment : None,
            timestamp_writes : None,
            occlusion_query_set : None,
        });

        renderpass.set_pipeline(&self.gcode_pass.render_pipeline);
        renderpass.draw(0..3, 0..1);

        interface_context.renderer.render(
            interface_context.context.render(),
            &gfx_ctx.queue,
            &gfx_ctx.device,
            &mut renderpass
        ).expect("imgui rendering failed");

        drop(renderpass);

        gfx_ctx.queue.submit([encoder.finish()]);
        gfx_ctx.window.pre_present_notify();
        surface_texture.present();
    }
}

