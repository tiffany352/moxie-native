pub use moxie::*;

use crate::dom::{Element, Node};
use crate::runtime::Dom;
use moxie;
use std::marker::PhantomData;

pub mod elements;

#[topo::nested]
#[topo::from_env(parent: &Node, storage: &Dom)]
pub fn text(s: impl ToString) {
    let mut storage = storage.borrow_mut();
    storage.add_text(*parent, s.to_string().into());
}
