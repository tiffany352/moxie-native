use crate::dom::element::{Attribute, Element, Event, HasAttribute, HasEvent};
use crate::dom::Node;
use crate::util::event_handler::EventHandler;
use moxie::*;

/// Builder pattern for creating a DOM node, typically used from the
/// mox! macro.
pub struct Builder<Elt: Element> {
    func: &'static str,
    element: Elt,
    handlers: Elt::Handlers,
    children: Vec<Elt::Child>,
}

pub trait IntoChildren<Elt>
where
    Elt: Element,
{
    type Item: Into<Elt::Child>;
    type IntoIter: Iterator<Item = Self::Item>;

    fn into_children(self) -> Self::IntoIter;
}

impl<Parent, Child> IntoChildren<Parent> for Node<Child>
where
    Parent: Element,
    Child: Element,
    Parent::Child: From<Node<Child>>,
{
    type Item = Node<Child>;
    type IntoIter = std::iter::Once<Node<Child>>;

    fn into_children(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

pub struct ChildrenIterator<Iter>(Iter);

impl<Iter> From<Iter> for ChildrenIterator<Iter>
where
    Iter: Iterator,
{
    fn from(value: Iter) -> Self {
        Self(value)
    }
}

impl<Parent, Iter, Item> IntoChildren<Parent> for ChildrenIterator<Iter>
where
    Parent: Element,
    Parent::Child: From<Item>,
    Iter: Iterator<Item = Item>,
{
    type Item = Iter::Item;
    type IntoIter = Iter;

    fn into_children(self) -> Self::IntoIter {
        self.0
    }
}

impl<Parent, Item> IntoChildren<Parent> for Vec<Item>
where
    Parent: Element,
    Parent::Child: From<Item>,
{
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Item>;

    fn into_children(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<Parent, Item> IntoChildren<Parent> for Option<Item>
where
    Parent: Element,
    Parent::Child: From<Item>,
{
    type Item = Item;
    type IntoIter = std::option::IntoIter<Item>;

    fn into_children(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<Parent> IntoChildren<Parent> for String
where
    Parent: Element,
    Parent::Child: From<String>,
{
    type Item = String;
    type IntoIter = std::iter::Once<String>;

    fn into_children(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl<Elt> Builder<Elt>
where
    Elt: Element,
{
    /// Creates a new builder.
    fn new(func: &'static str) -> Self {
        Builder {
            func,
            element: Elt::default(),
            handlers: Elt::Handlers::default(),
            children: vec![],
        }
    }

    /// Implements the protocol used by the mox! macro to build an element.
    pub fn create(func: &'static str, with_elem: impl FnOnce(Self) -> Node<Elt>) -> Node<Elt> {
        topo::call!({ with_elem(Self::new(func)) })
    }

    /// Set an attribute on the element.
    pub fn attr<Attr>(mut self, _phantom: Attr, value: impl Into<Attr::Value>) -> Self
    where
        Attr: Attribute,
        Elt: HasAttribute<Attr>,
    {
        topo::call!({
            self.element.set_attribute(value.into());
        });
        self
    }

    /// Register an event handler on the element. The event type has to
    /// be supported by the element, see `HasEvent`.
    pub fn on<E>(mut self, func: impl FnMut(&E) + 'static) -> Self
    where
        E: Event,
        Elt: HasEvent<E>,
    {
        topo::call!({
            Elt::set_handler(&mut self.handlers, EventHandler::with_func(func));
        });
        self
    }

    /// Adds a child node.
    pub fn add_child(mut self, children: impl IntoChildren<Elt>) -> Self {
        for child in children.into_children() {
            self.children.push(child.into());
        }
        self
    }

    /// Adds free-floating content, typically text, to the node.
    pub fn add_content(mut self, children: impl IntoChildren<Elt>) -> Self {
        for child in children.into_children() {
            self.children.push(child.into());
        }
        self
    }

    /// Build the actual node. This attempts some memoization so that a
    /// node won't necessarily always be created.
    pub fn build(self) -> Node<Elt> {
        let Self {
            func,
            element,
            children,
            handlers,
        } = self;
        let node = memo!((element, children), |(elt, children): &(
            Elt,
            Vec<Elt::Child>
        )| Node::new(
            func,
            elt.clone(),
            children.clone()
        ));

        node.handlers().replace(handlers);

        node
    }
}

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

/// The root of the DOM.
#[macro_export]
macro_rules! app {
    ($with_elem:expr) => {
        $crate::moxie::Builder::<$crate::dom::App>::create($crate::function!(), $with_elem)
    };
}

/// Top level window.
#[macro_export]
macro_rules! window {
    ($with_elem:expr) => {
        $crate::moxie::Builder::<$crate::dom::Window>::create($crate::function!(), $with_elem)
    };
}

/// Basic flow container.
#[macro_export]
macro_rules! view {
    ($with_elem:expr) => {
        $crate::moxie::Builder::<$crate::dom::View>::create($crate::function!(), $with_elem)
    };
}

/// An interactible button.
#[macro_export]
macro_rules! button {
    ($with_elem:expr) => {
        $crate::moxie::Builder::<$crate::dom::Button>::create($crate::function!(), $with_elem)
    };
}

/// Container for inline text.
#[macro_export]
macro_rules! span {
    ($with_elem:expr) => {
        $crate::moxie::Builder::<$crate::dom::Span>::create($crate::function!(), $with_elem)
    };
}
