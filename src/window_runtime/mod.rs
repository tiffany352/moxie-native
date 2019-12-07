use crate::dom::{App, Node};
use crate::runtime::RuntimeWaker;
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    window::WindowId,
};

mod context;
mod document;
mod node;
mod style_engine;
mod window;

pub use node::LocalNodeStorage;

pub enum Message {
    WebrenderReady,
    UpdateDom(Node<App>),
}

pub struct WindowRuntimeNotifier {
    proxy: EventLoopProxy<Message>,
}

impl WindowRuntimeNotifier {
    pub fn send_message(&self, message: Message) {
        self.proxy.send_event(message).unwrap();
    }
}

pub struct WindowRuntime {
    event_loop: EventLoop<Message>,
}

struct Runtime {
    windows: HashMap<WindowId, window::Window>,
    window_ids: Vec<WindowId>,
    proxy: EventLoopProxy<Message>,
    waker: Arc<RuntimeWaker>,
}

impl WindowRuntime {
    /// Create a new runtime based on the application's root component.
    pub fn new() -> WindowRuntime {
        let event_loop = EventLoop::<Message>::with_user_event();
        WindowRuntime { event_loop }
    }

    pub fn notifier(&self) -> WindowRuntimeNotifier {
        WindowRuntimeNotifier {
            proxy: self.event_loop.create_proxy(),
        }
    }

    pub fn start(self, waker: Arc<RuntimeWaker>) {
        let mut runtime = Runtime {
            windows: HashMap::new(),
            window_ids: vec![],
            proxy: self.event_loop.create_proxy(),
            waker,
        };
        self.event_loop
            .run(move |event, target, control_flow| runtime.process(event, target, control_flow));
    }
}

impl Runtime {
    /// Handle events
    fn process(
        &mut self,
        event: Event<Message>,
        target: &EventLoopWindowTarget<Message>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::UserEvent(Message::UpdateDom(dom)) => {
                self.update_dom(target, dom);
            }
            Event::WindowEvent { event, window_id } => {
                let window = self.windows.get_mut(&window_id).unwrap();
                window.process(event);
            }
            _ => *control_flow = ControlFlow::Wait,
        }
    }

    /// Updates the moxie runtime and reconciles the DOM changes,
    /// re-rendering if things have changed.
    fn update_dom(&mut self, event_loop: &EventLoopWindowTarget<Message>, app: Node<App>) {
        let first_iter = app.children().iter().map(Some).chain(iter::repeat(None));
        let second_iter = self
            .window_ids
            .drain(..)
            .collect::<Vec<_>>()
            .into_iter()
            .map(Some)
            .chain(iter::repeat(None));

        for (dom_window, window_id) in first_iter.zip(second_iter) {
            match (dom_window, window_id) {
                (Some(dom_window), Some(window_id)) => {
                    let window = self.windows.get_mut(&window_id).unwrap();
                    window.set_dom_window(dom_window.clone());
                    window.render();
                    self.window_ids.push(window_id);
                }
                (Some(dom_window), None) => {
                    let window = window::Window::new(
                        dom_window.clone(),
                        event_loop,
                        self.proxy.clone(),
                        self.waker.clone(),
                    );
                    let id = window.window_id();
                    self.windows.insert(id, window);
                    self.window_ids.push(id);
                }
                (None, Some(window_id)) => {
                    self.windows.remove(&window_id);
                }
                (None, None) => break,
            }
        }
    }
}
