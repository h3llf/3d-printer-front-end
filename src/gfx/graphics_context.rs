use winit::window::{Window, WindowId, WindowAttributes};
use winit::{dpi::PhysicalSize};
use std::sync::Arc;

pub struct GraphicsContext {
    pub window : Arc<Window>,
    pub device : wgpu::Device,
    pub queue : wgpu::Queue,
    pub size : PhysicalSize<u32>,
    pub surface : wgpu::Surface<'static>,
    pub surface_format : wgpu::TextureFormat,
//    gcode_pass : GCodePass,
}
