mod direct_composition;
pub mod moxie;
mod node;
mod window;

pub use node::{Node, NodeStorage};

use ::moxie::embed::Runtime as MoxieRuntime;
use std::collections::HashMap;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

#[topo::nested]
#[topo::from_env(storage: &NodeStorage)]
fn debug_print() {
    println!("{}", storage.pretty_print_xml(NodeStorage::root()));
}

pub struct Runtime {
    moxie_runtime: MoxieRuntime<Box<dyn FnMut() -> () + 'static>, ()>,
    windows: HashMap<WindowId, window::Window>,
}

impl Runtime {
    pub fn new(mut root: impl FnMut() + 'static) -> Runtime {
        Runtime {
            moxie_runtime: MoxieRuntime::new(Box::new(move || {
                topo::call!(
                    {
                        root();
                        debug_print!()
                    },
                    env! {
                        Node => NodeStorage::root(),
                        NodeStorage => NodeStorage::new(),
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

        event_loop
            .run(move |event, target, control_flow| self.process(event, target, control_flow));
    }
}
