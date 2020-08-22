use crate::dom::element::{Attribute, Element, Event, HasAttribute, HasEvent};
use crate::dom::node::Node;
use crate::util::event_handler::EventHandler;
use std::marker::PhantomData;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Builder pattern for creating a DOM node, typically used from the
/// mox! macro.
pub struct Builder<Elt: Element> {
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

impl<Parent> IntoChildren<Parent> for Fragment<Parent>
where
    Parent: Element,
{
    type Item = Parent::Child;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_children(self) -> Self::IntoIter {
        self.children.into_iter()
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

impl<'a, Parent> IntoChildren<Parent> for &'a str
where
    Parent: Element,
    Parent::Child: From<String>,
{
    type Item = String;
    type IntoIter = std::iter::Once<String>;

    fn into_children(self) -> Self::IntoIter {
        std::iter::once(self.to_owned())
    }
}

impl<Elt> Builder<Elt>
where
    Elt: Element,
{
    /// Creates a new builder.
    pub(crate) fn new() -> Self {
        Builder {
            element: Elt::default(),
            handlers: Elt::Handlers::default(),
            children: vec![],
        }
    }

    /// Set an attribute on the element.
    pub fn set_attr<Attr>(mut self, _phantom: Attr, value: impl Into<Attr::Value>) -> Self
    where
        Attr: Attribute,
        Elt: HasAttribute<Attr>,
    {
        self.element.set_attribute(value.into());
        self
    }

    /// Register an event handler on the element. The event type has to
    /// be supported by the element, see `HasEvent`.
    pub fn on_event<Ev>(
        mut self,
        _event: PhantomData<Ev>,
        func: impl FnMut(&Ev) -> () + 'static + Sync + Send,
    ) -> Self
    where
        Ev: Event,
        Elt: HasEvent<Ev>,
    {
        Elt::set_handler(&mut self.handlers, EventHandler::with_func(func));
        self
    }

    /// Adds a child node.
    pub fn add_child(mut self, children: impl IntoChildren<Elt>) -> Self {
        for child in children.into_children() {
            self.children.push(child.into());
        }
        self
    }

    /// Build the actual node. This attempts some memoization so that a
    /// node won't necessarily always be created.
    #[topo::nested]
    pub fn build(self) -> Node<Elt> {
        let Self {
            element,
            children,
            handlers,
        } = self;

        let id = moxie::once(|| ID_COUNTER.fetch_add(1, Ordering::Acquire));

        let node = moxie::cache(
            &(element, children),
            |(elt, children): &(Elt, Vec<Elt::Child>)| Node::new(id, elt.clone(), children.clone()),
        );

        *node.handlers().lock().unwrap() = handlers;

        node
    }
}

#[derive(Default)]
pub struct Fragment<Elt>
where
    Elt: Element,
{
    children: Vec<Elt::Child>,
}

impl<Elt> Fragment<Elt>
where
    Elt: Element,
{
    pub fn new() -> Fragment<Elt> {
        Fragment { children: vec![] }
    }

    pub fn add_child(mut self, children: impl IntoChildren<Elt>) -> Self {
        for child in children.into_children() {
            self.children.push(child.into());
        }
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
