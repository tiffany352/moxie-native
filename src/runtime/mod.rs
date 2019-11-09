use crate::dom::{DomStorage, Element, Node, NodeOrText};
use ::moxie::embed::Runtime as MoxieRuntime;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowId,
};
mod window;

pub struct Dom(Rc<RefCell<DomStorage>>);

impl Dom {
    pub fn borrow_mut(&self) -> RefMut<DomStorage> {
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
    node_to_window: HashMap<Node, WindowId>,
    dom: Rc<RefCell<DomStorage>>,
}

impl Runtime {
    pub fn new(mut root: impl FnMut() + 'static) -> Runtime {
        let storage = DomStorage::new();
        let root_node = storage.root();
        let storage = Rc::new(RefCell::new(storage));
        Runtime {
            dom: storage.clone(),
            moxie_runtime: MoxieRuntime::new(Box::new(move || {
                topo::call!(
                    {
                        root();
                        debug_print!()
                    },
                    env! {
                        Node => root_node,
                        Dom => Dom(storage.clone()),
                    }
                )
            })),
            windows: HashMap::new(),
            node_to_window: HashMap::new(),
        }
    }

    fn process(
        &mut self,
        event: Event<()>,
        _target: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        match event {
            Event::WindowEvent { event, window_id } => self
                .windows
                .get_mut(&window_id)
                .unwrap()
                .process(&mut *self.dom.borrow_mut(), event),
            _ => *control_flow = ControlFlow::Wait,
        }
    }

    fn update_runtime(&mut self, event_loop: &EventLoop<()>) {
        self.moxie_runtime.run_once();

        let mut storage = self.dom.borrow_mut();
        let root = storage.root();
        let mut create_windows = vec![];
        for child in storage.get_children(root) {
            if let NodeOrText::Node(node) = child {
                if let Element::Window(_dom_window) = storage.get_element(*node) {
                    if !self.node_to_window.contains_key(node) {
                        create_windows.push(*node);
                    }
                }
            }
        }

        for node in create_windows {
            let window = window::Window::new(node, &mut *storage, &event_loop);
            let id = window.window_id();
            self.windows.insert(id, window);
            self.node_to_window.insert(node, id);
        }
    }

    pub fn start(mut self) {
        let event_loop = EventLoop::new();

        self.update_runtime(&event_loop);

        event_loop
            .run(move |event, target, control_flow| self.process(event, target, control_flow));
    }
}
