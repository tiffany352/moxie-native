pub use moxie::*;

use crate::dom::{Element, Node};
use crate::Dom;
use moxie;
use std::marker::PhantomData;

pub mod elements;

#[topo::nested]
#[topo::from_env(parent: &Node, storage: &Dom)]
pub fn text(s: impl ToString) {
    let mut storage = storage.borrow_mut();
    storage.add_text(*parent, s.to_string().into());
}

#[topo::nested]
#[topo::from_env(parent: &Node, storage: &Dom)]
pub fn element<Class: Into<Element> + Default>(
    _phantom: PhantomData<Class>,
    with_elem: impl FnOnce(MemoNode) -> (),
) {
    let elem;
    {
        let mut storage = storage.borrow_mut();
        elem = once!(|| storage.create_element(Class::default()));
        storage.clear_children(elem);
        storage.add_child(*parent, elem);
    }
    let elem = MemoNode(elem);
    with_elem(elem)
}

pub struct MemoNode(Node);

impl MemoNode {
    #[topo::from_env(storage: &Dom)]
    pub fn attr(&self, key: &str, value: &str) -> &Self {
        let mut storage = storage.borrow_mut();
        storage.set_attribute(self.0, key, Some(value.to_owned().into()));
        self
    }

    pub fn inner(&self, children: impl FnOnce()) {
        topo::call!(
            { children() },
            env! {
                Node => self.0,
            }
        )
    }
}
