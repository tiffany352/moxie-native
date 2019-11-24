use super::{AttrClassName, AttrStyles, Element, Node, View};
use crate::style::Style;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Window {
    styles: Cow<'static, [&'static Style]>,
    class_name: Option<Cow<'static, str>>,
}

crate::element_attributes! {
    Window {
        styles: AttrStyles,
        class_name: AttrClassName,
    }
}

impl Element for Window {
    type Child = Node<View>;
    type Handlers = ();

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }

    fn styles(&self) -> &[&'static Style] {
        self.styles.as_ref()
    }
}
