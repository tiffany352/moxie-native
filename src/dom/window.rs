use super::{Element, Node, View};
use crate::layout::LayoutOptions;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Window {}

impl Window {
    pub fn new() -> Window {
        Window {}
    }
}

impl Element for Window {
    type Child = Node<View>;

    fn set_attribute(&mut self, _key: &str, _value: Option<Cow<'static, str>>) {}

    fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            ..Default::default()
        }
    }
}
