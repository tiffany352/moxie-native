mod direct_composition;
pub mod moxie;
mod node;
mod window;

pub use node::Node;

use crate::moxie::MemoElement;
use ::moxie::embed::Runtime as MoxieRuntime;
use std::collections::HashMap;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

pub struct Runtime {
    root_node: Node,
    moxie_runtime: MoxieRuntime<Box<dyn FnMut() + 'static>, ()>,
    windows: HashMap<WindowId, window::Window>,
}

impl Runtime {
    pub fn new(mut root: impl FnMut() + 'static) -> Runtime {
        let root_node = Node::create_root();
        Runtime {
            root_node: root_node.clone(),
            moxie_runtime: MoxieRuntime::new(Box::new(move || {
                topo::call!(
                    { root() },
                    env! {
                        MemoElement => MemoElement::new(root_node.clone()),
                    }
                )
            })),
            windows: HashMap::new(),
        }
    }

    fn process(
        &mut self,
        event: Event<()>,
        _target: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::WindowEvent { event, window_id } => {
                self.windows.get_mut(&window_id).unwrap().process(event)
            }
            _ => *control_flow = ControlFlow::Wait,
        }
    }

    pub fn start(mut self) {
        let event_loop = EventLoop::new();

        self.moxie_runtime.run_once();

        println!("{:#?}", self.root_node);

        event_loop
            .run(move |event, target, control_flow| self.process(event, target, control_flow));
    }
}
