use super::gfx::graphics_context::GraphicsContext;
use imgui;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use std::time::Instant;
use winit::{window::Window, event::*};
use rfd::{FileDialog, AsyncFileDialog};
use std::path::PathBuf;

pub enum UIAction {
    LoadFile(PathBuf),
    None,
}

pub struct InterfaceContext {
    pub context : imgui::Context,
    pub platform : WinitPlatform,
    pub renderer : Renderer,
    pub clear_colour : wgpu::Color,
    pub last_frame : Instant,
    pub last_cursor : Option<imgui::MouseCursor>,
}

#[derive(Default)]
pub struct UI {
    pub interface_context : Option<InterfaceContext>,
    frame_count : u32,
}

impl UI {
    pub fn init_imgui(&mut self, gfx_context : &GraphicsContext) {
        let mut context = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);

        platform.attach_window(
            context.io_mut(), 
            gfx_context.window.as_ref(), 
            imgui_winit_support::HiDpiMode::Default);
        context.set_ini_filename(None);
        let font_size : f32 = 13.0 * gfx_context.window.scale_factor() as f32;

        let font_conf = imgui::FontConfig{
            oversample_h : 1,
            pixel_snap_h : true,
            size_pixels : font_size,
            ..Default::default()
        };

        context.fonts().add_font(&[imgui::FontSource::DefaultFontData {
            config : Some(font_conf)
        }]);

        let clear_colour = wgpu::Color {
            r : 0.003,
            g : 0.004,
            b : 0.008,
            a : 1.0
        };

        let renderer_conf = RendererConfig {
            texture_format : gfx_context.surface_format,
            ..Default::default()
        };

        let renderer = Renderer::new(
            &mut context, 
            &gfx_context.device,
            &gfx_context.queue,
            renderer_conf);

        let last_frame = Instant::now();

        self.interface_context = Some(InterfaceContext{
            context,
            platform,
            renderer,
            clear_colour,
            last_frame,
            last_cursor : None,
        });

        self.frame_count = 0;

        println!("Imgui initialized");
    }

    pub fn draw_ui(&mut self, window : &Window) -> UIAction {
        let ctx : &mut InterfaceContext = self.interface_context.as_mut().unwrap();

        let now : Instant = Instant::now();
        ctx.context.io_mut().update_delta_time(now - ctx.last_frame);
        ctx.last_frame = now;

        ctx.platform
            .prepare_frame(ctx.context.io_mut(), window)
            .expect("Failed to prepare imgui frame");

        self.frame_count += 1;

        let mut file_dialog_btn : bool = false;

        let imgui_frame = ctx.context.frame();
        imgui_frame.window("Test")
            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
            .build(|| {
                imgui_frame.text(format!{"Hello world: {}", self.frame_count});
                file_dialog_btn = imgui_frame.button("Open G-Code");
            });

        // TODO: This should be moved to a dedicated gcode loader/parser
        if file_dialog_btn {
            let file_dir = FileDialog::new()
                .add_filter("G-code files", &["gcode"])
                .pick_file();

            if let Some(file_str) = file_dir {
                    return UIAction::LoadFile(file_str);
            }
        }

        if ctx.last_cursor != imgui_frame.mouse_cursor() {
            ctx.last_cursor = imgui_frame.mouse_cursor();
        }

        ctx.platform.prepare_render(imgui_frame, window);

        UIAction::None
    }

    pub fn handle_event<T>(&mut self, window : &Window, event : &Event<T>) {
        let ctx : &mut InterfaceContext = self.interface_context.as_mut().unwrap();
        ctx.platform.handle_event(ctx.context.io_mut(), window, event);
    }
}

