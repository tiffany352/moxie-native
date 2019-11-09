#[doc(hidden)]
pub use moxie::*;

use super::{Node, NodeStorage};
use moxie;

pub mod elements;

#[topo::nested]
#[topo::from_env(parent: &Node, storage: &NodeStorage)]
pub fn text(s: impl ToString) {
    let text_node = memo!(s.to_string(), |s| storage.create_text_node(s));
    storage.add_child(*parent, text_node);
}

#[topo::nested]
#[topo::from_env(parent: &Node, storage: &NodeStorage)]
pub fn element<ChildRet>(
    ty: &'static str,
    with_elem: impl FnOnce(MemoNode) -> ChildRet,
) -> ChildRet {
    let elem = memo!(ty, |ty| storage.create_element(ty));
    storage.clear_children(elem);
    storage.add_child(*parent, elem);
    let elem = MemoNode(elem);
    with_elem(elem)
}

pub struct MemoNode(Node);

impl MemoNode {
    #[topo::from_env(storage: &NodeStorage)]
    pub fn attr(&self, key: &str, value: &str) -> &Self {
        storage.set_attribute(self.0, key, value);
        self
    }

    pub fn inner(&self, children: impl FnOnce()) {
        topo::call!(
            {
                children()
            },
            env! {
                Node => self.0,
            }
        )
    }
}
