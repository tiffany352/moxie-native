use super::Node;
use crate::dom::input::InputEvent;
use crate::style::{ComputedValues, Style};
use crate::util::event_handler::EventHandler;
use std::any::TypeId;
use std::cell::Cell;

/// Represents the attributes and behavior of a single DOM element.
pub trait Element: Default + Clone + PartialEq + 'static {
    /// The type of children that can be parented to this element.
    type Child: NodeChild + Clone + PartialEq;
    type Handlers: HandlerList;
    type States: ElementStates + Clone + Copy + Default + PartialEq;

    /// Creates default style values
    fn create_computed_values(&self) -> ComputedValues {
        Default::default()
    }

    fn process(
        &self,
        states: Self::States,
        _handlers: &mut Self::Handlers,
        _event: &InputEvent,
    ) -> (bool, Self::States) {
        (false, states)
    }

    /// Returns the class_name attribute.
    fn class_name(&self) -> Option<&str>;

    /// Returns the list of styles attached to this element.
    fn styles(&self) -> &[&'static Style];
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

pub trait ElementStates {
    fn has_state(&self, name: &str) -> bool;
}

impl ElementStates for () {
    fn has_state(&self, _name: &str) -> bool {
        false
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
    /// Returns a trait object for the child at the given index. If the
    /// index is out of bounds, return None. Typically maps to
    /// `Element::children().get(index)`.
    fn get_child(&self, child: usize) -> Option<&dyn NodeChild>;

    fn computed_values(&self) -> Result<&Cell<Option<ComputedValues>>, &str>;

    fn type_id(&self) -> TypeId;

    fn process(&self, event: &InputEvent) -> bool;

    fn has_state(&self, name: &str) -> bool;

    fn class_name(&self) -> Option<&str>;

    fn styles(&self) -> &[&'static Style];

    fn create_computed_values(&self) -> ComputedValues;
}

/// A helper to walk through the children of a `NodeChild`, creating an
/// iterator over the children so that you don't have to call
/// `get_child` manually.
pub fn children(node: &dyn NodeChild) -> impl Iterator<Item = &dyn NodeChild> {
    struct Iter<'a> {
        node: &'a dyn NodeChild,
        index: usize,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a dyn NodeChild;

        fn next(&mut self) -> Option<Self::Item> {
            let child = self.node.get_child(self.index);
            self.index += 1;
            child
        }
    }

    Iter {
        node: node,
        index: 0,
    }
}

impl<Elt> NodeChild for Node<Elt>
where
    Elt: Element,
{
    fn get_child(&self, child: usize) -> Option<&dyn NodeChild> {
        if let Some(child) = self.children().get(child) {
            Some(child)
        } else {
            None
        }
    }

    fn computed_values(&self) -> Result<&Cell<Option<ComputedValues>>, &str> {
        Ok(self.computed_values())
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<Elt>()
    }

    fn process(&self, event: &InputEvent) -> bool {
        let mut handlers = self.handlers().borrow_mut();
        let states = self.states().get();
        let (sink, states) = self.element().process(states, &mut *handlers, event);
        self.states().set(states);
        sink
    }

    fn has_state(&self, name: &str) -> bool {
        self.states().get().has_state(name)
    }

    fn class_name(&self) -> Option<&str> {
        self.element().class_name()
    }

    fn styles(&self) -> &[&'static Style] {
        self.element().styles()
    }

    fn create_computed_values(&self) -> ComputedValues {
        self.element().create_computed_values()
    }
}

impl NodeChild for String {
    fn get_child(&self, _child: usize) -> Option<&dyn NodeChild> {
        None
    }

    fn computed_values(&self) -> Result<&Cell<Option<ComputedValues>>, &str> {
        Err(&self[..])
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<String>()
    }

    fn process(&self, _event: &InputEvent) -> bool {
        false
    }

    fn has_state(&self, _name: &str) -> bool {
        false
    }

    fn class_name(&self) -> Option<&str> {
        None
    }

    fn styles(&self) -> &[&'static Style] {
        &[]
    }

    fn create_computed_values(&self) -> ComputedValues {
        ComputedValues::default()
    }
}

pub trait HandlerList: Default + 'static {}

impl HandlerList for () {}
