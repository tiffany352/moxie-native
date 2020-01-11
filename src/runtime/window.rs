use crate::dom::{Node, Window as DomWindow};
use crate::render::Context;
use gleam::gl;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{EventLoopProxy, EventLoopWindowTarget},
    window::{Window as WinitWindow, WindowBuilder, WindowId},
};

/// Wrapper around a `winit::Window` and a `Context` for rendering the
/// DOM.
pub struct Window {
    gl_context: ContextWrapper<PossiblyCurrent, WinitWindow>,
    context: Context,
}

impl Window {
    pub fn new(
        dom_window: Node<DomWindow>,
        event_loop: &EventLoopWindowTarget<()>,
        proxy: EventLoopProxy<()>,
    ) -> Window {
        let window_builder = WindowBuilder::new()
            .with_title(&dom_window.element().title[..])
            .with_decorations(true)
            .with_transparent(true);

        let gl_context = ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (3, 0),
            })
            .build_windowed(window_builder, &event_loop)
            .unwrap();

        let gl_context = unsafe { gl_context.make_current().unwrap() };

        let gl = match gl_context.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| gl_context.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| gl_context.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        };

        let mut context = Context::new(gl, gl_context.window(), proxy, dom_window);
        context.render();
        gl_context.swap_buffers().unwrap();

        Window {
            gl_context,
            context,
        }
    }

    pub fn window_id(&self) -> WindowId {
        self.gl_context.window().id()
    }

    pub fn set_dom_window(&mut self, new_node: Node<DomWindow>) {
        self.gl_context
            .window()
            .set_title(&new_node.element().title[..]);
        self.context.set_dom_window(new_node);
    }

    pub fn render(&mut self) {
        self.context.render();
        self.gl_context.swap_buffers().unwrap();
    }

    pub fn process(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(size) => {
                println!("resize {}x{}", size.width, size.height);
                let factor = self.gl_context.window().scale_factor();
                self.context.resize(size, factor as f32);
                self.render();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.gl_context.window().scale_factor();
                let element = self.context.element_at(position.to_logical(scale));
                return self.context.document.mouse_move(element);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                return self.context.document.mouse_button1(pressed);
            }
            WindowEvent::CursorLeft { .. } => {
                return self.context.document.mouse_move(None);
            }
            _ => (),
        }
        false
    }
}
