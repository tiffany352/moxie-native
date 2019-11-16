use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::Element;

struct NodeData<Elt>
where
    Elt: Element,
{
    element: Elt,
    children: Vec<Elt::Child>,
}

impl<Elt> NodeData<Elt>
where
    Elt: Element,
{
    fn new(element: Elt, children: Vec<Elt::Child>) -> NodeData<Elt> {
        NodeData {
            element: element,
            children: children,
        }
    }
}

#[derive(Clone)]
pub struct Node<Elt: Element>(Rc<NodeData<Elt>>);

impl<Elt> Node<Elt>
where
    Elt: Element,
{
    pub fn new(element: Elt, children: Vec<Elt::Child>) -> Node<Elt> {
        Node(Rc::new(NodeData::new(element, children)))
    }

    pub fn children(&self) -> &[Elt::Child] {
        &self.0.children[..]
    }

    pub fn element(&self) -> &Elt {
        &self.0.element
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
