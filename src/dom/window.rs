use super::{Element, Node, View};
use crate::layout::LayoutOptions;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Window {
    class_name: Option<Cow<'static, str>>,
}

impl Window {
    pub fn new() -> Window {
        Window { class_name: None }
    }
}

impl Element for Window {
    type Child = Node<View>;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            _ => (),
        }
    }

    fn create_layout_opts(&self, _parent_opts: &LayoutOptions) -> LayoutOptions {
        LayoutOptions {
            ..Default::default()
        }
    }

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }
}
