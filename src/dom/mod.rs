pub mod element;
pub mod storage;
pub mod view;
pub mod window;

use std::marker::PhantomData;

pub use element::{Element, ElementType};
pub use storage::{DomStorage, NodeOrText};

slotmap::new_key_type! {
    pub struct Node;
}

pub struct TypedNode<Type: ElementType> {
    node: Node,
    _phantom: PhantomData<Type>,
}

impl<Type> TypedNode<Type>
where
    Type: ElementType,
{
    pub(crate) fn from_raw(node: Node) -> TypedNode<Type> {
        TypedNode {
            node,
            _phantom: PhantomData,
        }
    }

    pub fn to_inner(&self) -> Node {
        self.node
    }
}

impl<Elt> PartialEq for TypedNode<Elt>
where
    Elt: ElementType,
{
    fn eq(&self, other: &Self) -> bool {
        return self.node == other.node;
    }
}

impl<Elt> Clone for TypedNode<Elt>
where
    Elt: ElementType,
{
    fn clone(&self) -> Self {
        TypedNode {
            node: self.node,
            _phantom: PhantomData,
        }
    }
}
