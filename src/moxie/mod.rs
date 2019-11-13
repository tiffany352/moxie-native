pub use moxie::*;

use crate::dom::Node;
use crate::runtime::Dom;
use moxie;

pub mod elements;

#[topo::nested]
#[topo::from_env(parent: &Node, storage: &Dom)]
pub fn text(s: impl ToString) {
    let mut storage = storage.borrow_mut();
    storage.add_text(*parent, s.to_string().into());
}
