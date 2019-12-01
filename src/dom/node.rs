use crate::dom::element::{DynamicNode, Element, ElementStates, NodeChild};
use crate::dom::input::InputEvent;
use crate::style::{ComputedValues, Style};
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

pub struct NodeData<Elt>
where
    Elt: Element,
{
    element: Elt,
    handlers: RefCell<Elt::Handlers>,
    states: Cell<Elt::States>,
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
            states: Cell::new(Default::default()),
            computed_values: Cell::new(None),
            children: children,
        }
    }

    /// Returns a reference to the children vector.
    pub fn children(&self) -> &[Elt::Child] {
        &self.children[..]
    }

    /// Returns a reference to the element representing this node.
    pub fn element(&self) -> &Elt {
        &self.element
    }

    pub fn states(&self) -> &Cell<Elt::States> {
        &self.states
    }

    pub fn computed_values(&self) -> &Cell<Option<ComputedValues>> {
        &self.computed_values
    }

    pub fn handlers(&self) -> &RefCell<Elt::Handlers> {
        &self.handlers
    }
}

pub struct NodeDataChildrenIter<'a> {
    node: &'a dyn AnyNodeData,
    index: usize,
}

impl<'a> Iterator for NodeDataChildrenIter<'a> {
    type Item = DynamicNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.node.get_child(self.index);
        self.index += 1;
        result
    }
}

pub trait AnyNodeData {
    fn computed_values(&self) -> &Cell<Option<ComputedValues>>;
    fn get_child(&self, index: usize) -> Option<DynamicNode>;
    fn children(&self) -> NodeDataChildrenIter;
    fn process(&self, event: &InputEvent) -> bool;
    fn create_computed_values(&self) -> ComputedValues;
    fn style(&self) -> Option<Style>;
    fn has_state(&self, key: &str) -> bool;
    fn type_id(&self) -> TypeId;
    fn name(&self) -> &'static str;
}

impl<Elt> AnyNodeData for NodeData<Elt>
where
    Elt: Element,
{
    fn computed_values(&self) -> &Cell<Option<ComputedValues>> {
        &self.computed_values
    }

    fn get_child(&self, index: usize) -> Option<DynamicNode> {
        self.children.get(index).map(|child| child.get_node())
    }

    fn children(&self) -> NodeDataChildrenIter {
        NodeDataChildrenIter {
            node: self,
            index: 0,
        }
    }

    fn process(&self, event: &InputEvent) -> bool {
        let mut handlers = self.handlers.borrow_mut();
        let (sink, new_states) = self
            .element
            .process(self.states.get(), &mut *handlers, event);
        self.states.set(new_states);
        sink
    }

    fn create_computed_values(&self) -> ComputedValues {
        self.element.create_computed_values()
    }

    fn style(&self) -> Option<Style> {
        self.element.style()
    }

    fn has_state(&self, key: &str) -> bool {
        self.states.get().has_state(key)
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Elt>()
    }

    fn name(&self) -> &'static str {
        Elt::ELEMENT_NAME
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
}

impl<Elt> Deref for Node<Elt>
where
    Elt: Element,
{
    type Target = NodeData<Elt>;

    fn deref(&self) -> &Self::Target {
        &*self.0
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

trait AnyNodeHandle {
    fn node_data(&self) -> &dyn AnyNodeData;
    fn to_owned(&self) -> AnyNode;
}

impl<Elt> AnyNodeHandle for Node<Elt>
where
    Elt: Element,
{
    fn node_data(&self) -> &dyn AnyNodeData {
        &**self
    }

    fn to_owned(&self) -> AnyNode {
        self.clone().into()
    }
}

impl AnyNodeHandle for AnyNode {
    fn node_data(&self) -> &dyn AnyNodeData {
        &*self.0
    }

    fn to_owned(&self) -> AnyNode {
        self.clone()
    }
}

#[derive(Copy, Clone)]
pub struct NodeRef<'a>(&'a dyn AnyNodeHandle);

impl<'a, Elt> From<&'a Node<Elt>> for NodeRef<'a>
where
    Elt: Element,
{
    fn from(node: &'a Node<Elt>) -> Self {
        NodeRef(node)
    }
}

impl<'a> From<&'a AnyNode> for NodeRef<'a> {
    fn from(node: &'a AnyNode) -> Self {
        NodeRef(node)
    }
}

impl<'a> Deref for NodeRef<'a> {
    type Target = dyn AnyNodeData + 'a;

    fn deref(&self) -> &Self::Target {
        self.0.node_data()
    }
}

#[derive(Clone)]
pub struct AnyNode(Rc<dyn AnyNodeData>);

impl<Elt> From<Node<Elt>> for AnyNode
where
    Elt: Element,
{
    fn from(node: Node<Elt>) -> Self {
        AnyNode(node.0)
    }
}

impl Deref for AnyNode {
    type Target = dyn AnyNodeData;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl PartialEq for AnyNode {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for AnyNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let raw: *const dyn AnyNodeData = &*self.0;
        raw.hash(state);
    }
}
