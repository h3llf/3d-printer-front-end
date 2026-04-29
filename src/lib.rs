mod interface;
use interface::{UI, UIAction};

mod gcode_parser;
use gcode_parser::GCodeParser;

mod gfx;
use gfx::render::*;

use winit::{
    application::ApplicationHandler, event::*, event_loop::*, window::{Window, WindowAttributes, WindowId}
};
use std::sync::Arc;
use pollster;

#[derive(Default)]
pub struct MainContext {
    window : Option<Arc<Window>>,
    renderer : Option<GFXRenderer>,
    ui : UI,
    gcode_parser : GCodeParser
}

impl MainContext {
    pub async fn init_context(&mut self) {
        if let Some(window) = self.window.as_ref() {
            self.renderer = Some(GFXRenderer::new(window.clone()).await);
        }   

        self.ui.init_imgui(&self.renderer.as_ref().unwrap().gfx_ctx);
    }

    pub fn render(&mut self) {
        let action : UIAction = self.ui.draw_ui(self.window.as_ref().unwrap());
        self.process_ui_action(action);
        self.renderer.as_mut().unwrap().render(self.ui.interface_context.as_mut().unwrap());
    }

    pub fn process_ui_action(&mut self, action : UIAction) {
        match action {
            UIAction::None => {
                return;
            } UIAction::LoadFile(dir) => {
                self.gcode_parser.load_gcode(&dir);
                self.renderer.as_mut().unwrap().update_gcode_points(&self.gcode_parser.render_data);
                println!("Selected: {}", dir.to_str().unwrap());
            }
        }
    }
}

#[derive(Default)]
pub struct AppMain {
    main_context : MainContext,
    orbit_cam : gfx::camera::OrbitCamera,
}

impl ApplicationHandler for AppMain {
    fn resumed(&mut self, event_loop : &ActiveEventLoop) {
        let window_attributes : WindowAttributes = Window::default_attributes()
            .with_title("Winit test");

        self.main_context.window = Some(Arc::new(
                event_loop.create_window(window_attributes)
                .unwrap()
            ));
        println!("Window created");

        pollster::block_on(self.main_context.init_context());
        println!("Context initialized");

        self.orbit_cam = gfx::camera::OrbitCamera::new(0.0, 0.0);
    }   

    fn window_event(&mut self, event_loop : &ActiveEventLoop, _id : WindowId, event : WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window closed");
                event_loop.exit();
            } WindowEvent::RedrawRequested => {
                let aspect : f32 = self.main_context.renderer.as_ref().unwrap().get_aspect();
                self.main_context.renderer.as_ref().unwrap().update_camera(
                    &self.orbit_cam.construct_camera(aspect));
                self.main_context.window.as_ref().unwrap().request_redraw();
                self.main_context.render();
            } WindowEvent::Resized(new_size) => {
                self.main_context.renderer.as_mut().unwrap().reconfigure_surface(new_size);
            } WindowEvent::MouseInput { device_id , state, button } => {
                match button {
                    MouseButton::Left => {
                        let just_pressed : bool = state == ElementState::Pressed;
                        self.orbit_cam.just_pressed = just_pressed;
                        self.orbit_cam.pressed = just_pressed;
                    } MouseButton::Middle => {
                        let just_pressed : bool = state == ElementState::Pressed;
//                        self.orbit_cam.middle_just_pressed = just_pressed;
                        self.orbit_cam.just_pressed = just_pressed;
                        self.orbit_cam.middle_pressed = just_pressed;
                    } _=> {

                    }
                }
                if button == MouseButton::Left {

                }
            } WindowEvent::CursorMoved { device_id, position } => {
                if self.orbit_cam.just_pressed {
                    self.orbit_cam.reset_mouse_pos(position.x as f32, position.y as f32);
                }
                self.orbit_cam.just_pressed = false;
                self.orbit_cam.update_mouse_pos(position.x as f32, position.y as f32);
            } WindowEvent::MouseWheel { device_id, delta, phase } => {
                let mut scroll_amt : f32 =
                    match delta {
                        MouseScrollDelta::LineDelta(_, y) => -y,
                        MouseScrollDelta::PixelDelta(p) => p.y as f32 / 50.0,
                    };
                
    
                self.orbit_cam.zoom_factor += scroll_amt / 5.0;
                self.orbit_cam.zoom_factor = self.orbit_cam.zoom_factor.max(0.2);
            }
            _=> { }
        }

        let imgui = self.main_context.ui.interface_context.as_mut().unwrap();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            self.main_context.window.as_ref().unwrap(),
            &Event::WindowEvent { window_id : _id, event : event });
    }

    // TODO: Other events
}

pub fn start_application() {
   let event_loop = EventLoop::new().unwrap();
    let mut app_main : AppMain = AppMain::default();
    event_loop.run_app(&mut app_main).unwrap(); 
}

