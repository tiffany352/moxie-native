use crate::dom::{Node, Window as DomWindow};
use crate::render::Context;
use winit::{
    event::WindowEvent,
    event_loop::EventLoop,
    platform::windows::WindowBuilderExtWindows,
    window::{Window as WinitWindow, WindowBuilder, WindowId},
};

pub struct Window {
    winit_window: WinitWindow,
    context: Context,
}

impl Window {
    pub fn new(dom_window: Node<DomWindow>, event_loop: &EventLoop<()>) -> Window {
        let winit_window = WindowBuilder::new()
            .with_title("UI Lib")
            .with_decorations(true)
            .with_transparent(true)
            .with_no_redirection_bitmap(true)
            .build(event_loop)
            .unwrap();

        let mut context = Context::new(&winit_window, event_loop, dom_window);
        context.render();

        Window {
            winit_window,
            context,
        }
    }

    pub fn window_id(&self) -> WindowId {
        self.winit_window.id()
    }

    pub fn set_dom_window(&mut self, new_node: Node<DomWindow>) {
        self.context.set_dom_window(new_node)
    }

    pub fn process(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.context.render();
            }
            WindowEvent::Resized(size) => {
                println!("resize {}x{}", size.width, size.height);
                let factor = self.winit_window.hidpi_factor();
                self.context.resize(size.to_physical(factor), factor as f32);
                self.context.render();
            }
            _ => (),
        }
    }
}
