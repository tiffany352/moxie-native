use crate::dom::{Node, Window as DomWindow};
use crate::render::Context;
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{EventLoopProxy, EventLoopWindowTarget},
    platform::windows::WindowBuilderExtWindows,
    window::{Window as WinitWindow, WindowBuilder, WindowId},
};

/// Wrapper around a `winit::Window` and a `Context` for rendering the
/// DOM.
pub struct Window {
    winit_window: WinitWindow,
    context: Context,
    cursor_pos: LogicalPosition,
}

impl Window {
    pub fn new(
        dom_window: Node<DomWindow>,
        event_loop: &EventLoopWindowTarget<()>,
        proxy: EventLoopProxy<()>,
    ) -> Window {
        let winit_window = WindowBuilder::new()
            .with_title("UI Lib")
            .with_decorations(true)
            .with_transparent(true)
            .with_no_redirection_bitmap(true)
            .build(event_loop)
            .unwrap();

        let mut context = Context::new(&winit_window, proxy, dom_window);
        context.render();

        Window {
            winit_window,
            context,
            cursor_pos: LogicalPosition::new(0.0, 0.0),
        }
    }

    pub fn window_id(&self) -> WindowId {
        self.winit_window.id()
    }

    pub fn set_dom_window(&mut self, new_node: Node<DomWindow>) {
        self.context.set_dom_window(new_node)
    }

    pub fn render(&mut self) {
        self.context.render();
    }

    pub fn process(&mut self, event: WindowEvent) -> bool {
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
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && button == MouseButton::Left {
                    return self.context.process_click(self.cursor_pos);
                }
            }
            _ => (),
        }
        false
    }
}
