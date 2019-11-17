use crate::dom::{Node, Window};
use ::moxie::embed::Runtime as MoxieRuntime;
use std::collections::HashMap;
use std::iter;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    window::WindowId,
};

mod window;

pub struct Runtime {
    moxie_runtime: MoxieRuntime<Box<dyn FnMut() -> Vec<Node<Window>> + 'static>, Vec<Node<Window>>>,
    windows: HashMap<WindowId, window::Window>,
    window_ids: Vec<WindowId>,
    proxy: Option<EventLoopProxy<()>>,
}

impl Runtime {
    pub fn new(mut root: impl FnMut() -> Vec<Node<Window>> + 'static) -> Runtime {
        Runtime {
            moxie_runtime: MoxieRuntime::new(Box::new(move || topo::call!({ root() }))),
            windows: HashMap::new(),
            window_ids: vec![],
            proxy: None,
        }
    }

    fn process(
        &mut self,
        event: Event<()>,
        target: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        let mut did_process = false;
        match event {
            Event::WindowEvent { event, window_id } => {
                let window = self.windows.get_mut(&window_id).unwrap();
                let res = window.process(event);
                did_process = res;
            }
            _ => *control_flow = ControlFlow::Wait,
        }
        if did_process {
            self.update_runtime(target);
        }
    }

    fn update_runtime(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        let windows = self.moxie_runtime.run_once();

        let first_iter = windows.into_iter().map(Some).chain(iter::repeat(None));
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
                    window.set_dom_window(dom_window);
                    window.render();
                    self.window_ids.push(window_id);
                }
                (Some(dom_window), None) => {
                    let window = window::Window::new(
                        dom_window,
                        event_loop,
                        self.proxy.as_ref().unwrap().clone(),
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

    pub fn start(mut self) {
        let event_loop = EventLoop::new();

        self.proxy = Some(event_loop.create_proxy());

        self.update_runtime(&event_loop);

        event_loop
            .run(move |event, target, control_flow| self.process(event, target, control_flow));
    }
}
