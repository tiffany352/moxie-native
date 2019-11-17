use crate::dom::{CanSetEvent, Element, Event, EventHandler, Node};
use moxie::*;

pub struct Builder<Elt: Element> {
    element: Elt,
    children: Vec<Elt::Child>,
}

impl<Elt> Builder<Elt>
where
    Elt: Element,
{
    fn new() -> Self {
        Builder {
            element: Elt::default(),
            children: vec![],
        }
    }

    pub fn create(with_elem: impl FnOnce(Self) -> Node<Elt>) -> Node<Elt> {
        topo::call!({ with_elem(Self::new()) })
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        topo::call!({
            self.element
                .set_attribute(key, Some(value.to_owned().into()));
        });
        self
    }

    pub fn on<E>(mut self, func: impl FnMut(&E) + 'static) -> Self
    where
        E: Event,
        Elt: CanSetEvent<E>,
    {
        topo::call!({
            self.element.set_handler(EventHandler::with_func(func));
        });
        self
    }

    pub fn add_child(mut self, child: impl Into<Elt::Child>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn add_content(mut self, child: impl Into<Elt::Child>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn build(self) -> Node<Elt> {
        memo!((self.element, self.children), |(elt, children): &(
            Elt,
            Vec<Elt::Child>
        )| Node::new(
            elt.clone(),
            children.clone()
        ))
    }
}

/// Top level window
#[macro_export]
macro_rules! window {
    ($with_elem:expr) => {
        $crate::moxie::elements::Builder::<$crate::dom::Window>::create($with_elem)
    };
}

/// Basic flow container
#[macro_export]
macro_rules! view {
    ($with_elem:expr) => {
        $crate::moxie::elements::Builder::<$crate::dom::View>::create($with_elem)
    };
}

/// An interactible button
#[macro_export]
macro_rules! button {
    ($with_elem:expr) => {
        $crate::moxie::elements::Builder::<$crate::dom::Button>::create($with_elem)
    };
}

/// Container for inline text
#[macro_export]
macro_rules! span {
    ($with_elem:expr) => {
        $crate::moxie::elements::Builder::<$crate::dom::Span>::create($with_elem)
    };
}
