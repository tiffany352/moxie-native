use crate::style::ComputedValues;
use std::cell::{Cell, RefCell};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::Element;

struct NodeData<Elt>
where
    Elt: Element,
{
    element: Elt,
    handlers: RefCell<Elt::Handlers>,
    computed_values: Cell<Option<ComputedValues>>,
    children: Vec<Elt::Child>,
}

impl<Elt> NodeData<Elt>
where
    Elt: Element,
{
    fn new(element: Elt, children: Vec<Elt::Child>) -> NodeData<Elt> {
        NodeData {
            element: element,
            handlers: RefCell::new(Default::default()),
            computed_values: Cell::new(None),
            children: children,
        }
    }
}

/// Typed handle to a DOM node.
#[derive(Clone)]
pub struct Node<Elt: Element>(Rc<NodeData<Elt>>);

impl<Elt> Node<Elt>
where
    Elt: Element,
{
    /// Create a new DOM node from the given element and children vector.
    pub fn new(element: Elt, children: Vec<Elt::Child>) -> Node<Elt> {
        Node(Rc::new(NodeData::new(element, children)))
    }

    /// Returns a reference to the children vector.
    pub fn children(&self) -> &[Elt::Child] {
        &self.0.children[..]
    }

    /// Returns a reference to the element representing this node.
    pub fn element(&self) -> &Elt {
        &self.0.element
    }

    pub fn computed_values(&self) -> &Cell<Option<ComputedValues>> {
        &self.0.computed_values
    }

    pub fn handlers(&self) -> &RefCell<Elt::Handlers> {
        &self.0.handlers
    }
}

impl<Elt> PartialEq for Node<Elt>
where
    Elt: Element,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<Elt> Hash for Node<Elt>
where
    Elt: Element,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let raw: *const NodeData<Elt> = &*self.0;
        raw.hash(state);
    }
}
