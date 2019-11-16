use super::{node::Node, view::View, Element};
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Window {}

impl Window {
    pub fn new() -> Window {
        Window {}
    }

    pub fn on<Event>(&mut self, _func: impl FnMut(&Event) + 'static)
    where
        Event: WindowEvent,
    {
    }
}

pub trait WindowEvent {}

impl Element for Window {
    type Child = Node<View>;

    fn set_attribute(&mut self, _key: &str, _value: Option<Cow<'static, str>>) {}
}
