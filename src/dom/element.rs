use super::{EventHandler, Node};
use crate::render::PaintDetails;
use crate::style::{ComputedValues, Style};
use std::any::TypeId;
use std::cell::Cell;

/// Represents the attributes and behavior of a single DOM element.
pub trait Element: Default + Clone + PartialEq + 'static {
    /// The type of children that can be parented to this element.
    type Child: NodeChild + Clone + PartialEq;
    type Handlers: HandlerList;

    /// Creates default style values
    fn create_computed_values(&self) -> ComputedValues {
        Default::default()
    }

    /// Returns the class_name attribute.
    fn class_name(&self) -> Option<&str>;

    /// Returns the list of styles attached to this element.
    fn styles(&self) -> &[&'static Style];

    /// Describes how this element should be displayed on the screen.
    /// Return None for this element to only affect layout.
    fn paint(&self, _handlers: &Self::Handlers) -> Option<PaintDetails> {
        None
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

/// Because some elements need to have multiple types of elements
/// parented to them, their `Element::Child` type is actually an enum
/// (defined using the `multiple_children!` macro).
///
/// This trait abstracts over the children of an element so that these
/// enums don't have to implement Element directly. This trait provides
/// a sort of visitor pattern which lets the DOM be walked without
/// having to know the types of each element at each step.
pub trait NodeChild: 'static {
    /// Typically a pass-through to `Element::paint()`.
    fn paint(&self) -> Option<PaintDetails>;
    /// Returns a trait object for the child at the given index. If the
    /// index is out of bounds, return None. Typically maps to
    /// `Element::children().get(index)`.
    fn get_child(&self, child: usize) -> Option<&dyn NodeChild>;

    fn computed_values(&self) -> Result<&Cell<Option<ComputedValues>>, &str>;

    fn type_id(&self) -> TypeId;

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
    fn paint(&self) -> Option<PaintDetails> {
        self.element().paint(&*self.handlers().borrow())
    }

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
    fn paint(&self) -> Option<PaintDetails> {
        Some(PaintDetails::default())
    }

    fn get_child(&self, _child: usize) -> Option<&dyn NodeChild> {
        None
    }

    fn computed_values(&self) -> Result<&Cell<Option<ComputedValues>>, &str> {
        Err(&self[..])
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<String>()
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
