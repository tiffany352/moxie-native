use crate::dom::input;
use crate::dom::{Node, Window as DomWindow};
use crate::render::Context;
use gleam::gl;
use glutin::{ContextBuilder, ContextWrapper, PossiblyCurrent};
use winit::{
    dpi::LogicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{EventLoopProxy, EventLoopWindowTarget},
    window::{Window as WinitWindow, WindowBuilder, WindowId},
};

/// Wrapper around a `winit::Window` and a `Context` for rendering the
/// DOM.
pub struct Window {
    gl_context: ContextWrapper<PossiblyCurrent, WinitWindow>,
    context: Context,
    cursor_pos: LogicalPosition,
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
            cursor_pos: LogicalPosition::new(0.0, 0.0),
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
            WindowEvent::RedrawRequested => {
                self.context.render();
            }
            WindowEvent::Resized(size) => {
                println!("resize {}x{}", size.width, size.height);
                let factor = self.gl_context.window().hidpi_factor();
                self.context.resize(size.to_physical(factor), factor as f32);
                self.render();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
                let event = input::InputEvent::MouseMove {
                    x: self.cursor_pos.x as f32,
                    y: self.cursor_pos.y as f32,
                };
                return self.context.process(&event);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let event = input::InputEvent::MouseLeft {
                    state: match state {
                        ElementState::Pressed => input::State::Begin,
                        ElementState::Released => input::State::End,
                    },
                    x: self.cursor_pos.x as f32,
                    y: self.cursor_pos.y as f32,
                };
                return self.context.process(&event);
            }
            _ => (),
        }
        false
    }
}
