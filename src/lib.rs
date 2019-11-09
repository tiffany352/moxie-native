mod direct_composition;
pub mod dom;
pub mod moxie;
mod window;

use ::moxie::embed::Runtime as MoxieRuntime;
pub use dom::DomStorage;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowId,
};

struct Dom(RefCell<DomStorage>);

impl Dom {
    fn borrow_mut(&self) -> RefMut<DomStorage> {
        self.0.borrow_mut()
    }
}

#[topo::nested]
#[topo::from_env(storage: &Dom)]
fn debug_print() {
    let storage = storage.borrow_mut();
    println!("{}", storage.pretty_print_xml(storage.root()));
}

pub struct Runtime {
    moxie_runtime: MoxieRuntime<Box<dyn FnMut() -> () + 'static>, ()>,
    windows: HashMap<WindowId, window::Window>,
}

impl Runtime {
    pub fn new(mut root: impl FnMut() + 'static) -> Runtime {
        Runtime {
            moxie_runtime: MoxieRuntime::new(Box::new(move || {
                let storage = DomStorage::new();
                let node = storage.root();
                topo::call!(
                    {
                        root();
                        debug_print!()
                    },
                    env! {
                        Node => node,
                        Dom => Dom(RefCell::new(storage)),
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
