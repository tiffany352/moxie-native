use crate::dom::element::{DynamicNode, Element, ElementStates, NodeChild};
use crate::dom::input::InputEvent;
use crate::style::{ComputedValues, Style};
use std::any::{type_name, TypeId};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub struct NodeData<Elt>
where
    Elt: Element,
{
    id: u64,
    element: Elt,
    handlers: Mutex<Elt::Handlers>,
    children: Vec<Elt::Child>,
}

impl<Elt> Debug for NodeData<Elt>
where
    Elt: Element,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let name = format!("NodeData<{}>", type_name::<Elt>());
        f.debug_struct(&name)
            .field("id", &self.id)
            .field("element", &self.element())
            .field("children", &self.children)
            .field("handlers", &"...")
            .finish()
    }
}

impl<Elt> NodeData<Elt>
where
    Elt: Element,
{
    fn new(id: u64, element: Elt, children: Vec<Elt::Child>) -> NodeData<Elt> {
        NodeData {
            id,
            element,
            children,
            handlers: Mutex::new(Default::default()),
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

    pub fn handlers(&self) -> &Mutex<Elt::Handlers> {
        &self.handlers
    }

    pub fn id(&self) -> u64 {
        self.id
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

pub trait AnyNodeData: Debug {
    fn get_child(&self, index: usize) -> Option<DynamicNode>;
    fn children(&self) -> NodeDataChildrenIter;
    fn process(&self, states: ElementStates, event: &InputEvent) -> ElementStates;
    fn create_computed_values(&self) -> ComputedValues;
    fn style(&self) -> Option<Style>;
    fn type_id(&self) -> TypeId;
    fn attributes(&self) -> Vec<(&'static str, String)>;
    fn name(&self) -> &'static str;
    fn id(&self) -> u64;
    fn interactive(&self) -> bool;
}

impl<Elt> AnyNodeData for NodeData<Elt>
where
    Elt: Element,
{
    fn get_child(&self, index: usize) -> Option<DynamicNode> {
        self.children.get(index).map(|child| child.get_node())
    }

    fn children(&self) -> NodeDataChildrenIter {
        NodeDataChildrenIter {
            node: self,
            index: 0,
        }
    }

    fn process(&self, states: ElementStates, event: &InputEvent) -> ElementStates {
        let mut handlers = self.handlers.lock().unwrap();
        let (_sink, new_states) = self.element.process(states, &mut *handlers, event);
        new_states
    }

    fn create_computed_values(&self) -> ComputedValues {
        self.element.create_computed_values()
    }

    fn style(&self) -> Option<Style> {
        self.element.style()
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Elt>()
    }

    fn attributes(&self) -> Vec<(&'static str, String)> {
        self.element.attributes()
    }

    fn name(&self) -> &'static str {
        Elt::ELEMENT_NAME
    }

    fn id(&self) -> u64 {
        self.id
    }

    fn interactive(&self) -> bool {
        self.element.interactive()
    }
}

/// Typed handle to a DOM node.
#[derive(Clone, Debug)]
pub struct Node<Elt: Element>(Arc<NodeData<Elt>>);

impl<Elt> Node<Elt>
where
    Elt: Element,
{
    /// Create a new DOM node from the given element and children vector.
    pub fn new(id: u64, element: Elt, children: Vec<Elt::Child>) -> Node<Elt> {
        let data = Arc::new(NodeData::new(id, element, children));
        Node(data)
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
        Arc::ptr_eq(&self.0, &other.0)
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

impl<'a> PartialEq for NodeRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            self.0.node_data() as *const dyn AnyNodeData,
            other.0.node_data() as *const dyn AnyNodeData,
        )
    }
}

impl<'a> NodeRef<'a> {
    pub fn to_owned(&self) -> AnyNode {
        self.0.to_owned()
    }
}

#[derive(Clone, Debug)]
pub struct AnyNode(Arc<dyn AnyNodeData>);

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
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for AnyNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let raw: *const dyn AnyNodeData = &*self.0;
        raw.hash(state);
    }
}
