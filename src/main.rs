mod interface;
use interface::UI;

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
}

impl MainContext {
    pub async fn init_context(&mut self) {
        if let Some(window) = self.window.as_ref() {
            self.renderer = Some(GFXRenderer::new(window.clone()).await);
        }   

        self.ui.init_imgui(&self.renderer.as_ref().unwrap().gfx_ctx);
    }

    pub fn render(&mut self) {
        self.ui.draw_ui(self.window.as_ref().unwrap());
        self.renderer.as_mut().unwrap().render(self.ui.interface_context.as_mut().unwrap());
    }
}

#[derive(Default)]
pub struct AppMain {
    main_context : MainContext,
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
    }   

    fn window_event(&mut self, event_loop : &ActiveEventLoop, _id : WindowId, event : WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window closed");
                event_loop.exit();
            } WindowEvent::RedrawRequested => {
                self.main_context.render();
            }
            _=> (), // Default
        }

        let imgui = self.main_context.ui.interface_context.as_mut().unwrap();
        imgui.platform.handle_event::<()>(
            imgui.context.io_mut(),
            self.main_context.window.as_ref().unwrap(),
            &Event::WindowEvent { window_id: _id, event : event }
        );
    }

    // TODO: Other events
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app_main : AppMain = AppMain::default();
    event_loop.run_app(&mut app_main).unwrap();
}
