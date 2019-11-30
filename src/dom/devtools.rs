use crate::dom::node::AnyNode;
use std::cell::RefCell;

pub(crate) struct DevToolsRegistry {
    current: RefCell<Box<dyn DevTools>>,
}

impl DevToolsRegistry {
    pub(crate) fn new() -> DevToolsRegistry {
        DevToolsRegistry {
            current: RefCell::new(Box::new(())),
        }
    }

    pub(crate) fn update(&self, node: AnyNode) {
        self.current.borrow_mut().on_update(node);
    }
}

pub trait DevTools: 'static {
    /// Be careful to ignore your own subtree when processing this, or an infinite loop is possible.
    fn on_update(&mut self, node: AnyNode);
}

impl DevTools for () {
    fn on_update(&mut self, _node: AnyNode) {}
}

#[topo::from_env(tools_registry: &DevToolsRegistry)]
pub fn register_devtools(tools: impl DevTools) {
    tools_registry.current.replace(Box::new(tools));
}
