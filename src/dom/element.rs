use crate::dom::input::{ElementState, InputEvent};
use crate::dom::node::{Node, NodeRef};
use crate::style::{ComputedValues, Style};
use crate::util::event_handler::EventHandler;
use enumset::EnumSet;
use std::any::Any;
use std::fmt::Debug;

/// Represents the attributes and behavior of a single DOM element.
pub trait Element: Default + Clone + Debug + PartialEq + 'static {
    /// The type of children that can be parented to this element.
    type Child: NodeChild + Clone + Debug + PartialEq;
    type Handlers: HandlerList;

    const ELEMENT_NAME: &'static str;

    /// Creates default style values
    fn create_computed_values(&self) -> ComputedValues {
        Default::default()
    }

    fn interactive(&self) -> bool {
        false
    }

    fn process(
        &self,
        states: EnumSet<ElementState>,
        _event: &InputEvent,
    ) -> (
        EnumSet<ElementState>,
        Option<<Self::Handlers as HandlerList>::Message>,
    ) {
        (states, None)
    }

    /// Returns the list of styles attached to this element.
    fn style(&self) -> Option<Style>;
}

pub trait DynElement {
    fn create_computed_values(&self) -> ComputedValues;
    fn interactive(&self) -> bool;
    fn style(&self) -> Option<Style>;
    fn name(&self) -> &str;
    fn process(
        &self,
        states: EnumSet<ElementState>,
        event: &InputEvent,
    ) -> (EnumSet<ElementState>, Option<Box<dyn Any + Send>>);
}

impl<Elt> DynElement for Elt
where
    Elt: Element,
{
    fn create_computed_values(&self) -> ComputedValues {
        Element::create_computed_values(self)
    }

    fn interactive(&self) -> bool {
        Element::interactive(self)
    }

    fn style(&self) -> Option<Style> {
        Element::style(self)
    }

    fn name(&self) -> &str {
        Elt::ELEMENT_NAME
    }

    fn process(
        &self,
        states: EnumSet<ElementState>,
        event: &InputEvent,
    ) -> (EnumSet<ElementState>, Option<Box<dyn Any + Send>>) {
        let (states, message) = Element::process(self, states, event);
        (
            states,
            message.map(|message| Box::new(message) as Box<dyn Any + Send>),
        )
    }
}

/// The trait representing all events that can be invoked on an element.
pub trait Event {}

/// Statically defines the relationship between which elements can have
/// which events listened to, and also provides the mechanism for that
/// to happen via the set_handler method.
pub trait HasEvent<Ev>: Element
where
    Ev: Event,
{
    fn set_handler(list: &mut Self::Handlers, handler: EventHandler<Ev>);
    fn get_handler(list: &Self::Handlers) -> &EventHandler<Ev>;
}

pub trait Attribute: Sized {
    type Value: Sized;
}

pub trait HasAttribute<Attr>
where
    Attr: Attribute,
{
    fn set_attribute(&mut self, value: Attr::Value);
}

pub enum DynamicNode<'a> {
    Text(&'a str),
    Node(NodeRef<'a>),
}

impl<'a, Elt> From<&'a Node<Elt>> for DynamicNode<'a>
where
    Elt: Element,
{
    fn from(value: &'a Node<Elt>) -> Self {
        DynamicNode::Node(value.into())
    }
}

impl<'a> From<&'a String> for DynamicNode<'a> {
    fn from(value: &'a String) -> Self {
        DynamicNode::Text(&value[..])
    }
}

/// Because some elements need to have multiple types of elements
/// parented to them, their `Element::Child` type is actually an enum
/// (defined using the `multiple_children!` macro).
///
/// This trait abstracts over the children of an element so that these
/// enums don't have to implement Element directly. This trait provides
/// a sort of visitor pattern which lets the DOM be walked without
/// having to know the types of each element at each step.
pub trait NodeChild: 'static {
    fn get_node(&self) -> DynamicNode;
}

impl<Elt> NodeChild for Node<Elt>
where
    Elt: Element,
{
    fn get_node(&self) -> DynamicNode {
        DynamicNode::Node(self.into())
    }
}

impl NodeChild for String {
    fn get_node(&self) -> DynamicNode {
        DynamicNode::Text(&self[..])
    }
}

pub trait HandlerList: Default + 'static {
    type Message: Any + Send + 'static;

    fn handle_message(&self, message: Self::Message);
}

impl HandlerList for () {
    type Message = ();

    fn handle_message(&self, _message: ()) {}
}
