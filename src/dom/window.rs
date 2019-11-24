use super::{AttrClassName, AttrStyles, AttrTitle, Element, Node, View};
use crate::style::Style;
use std::borrow::Cow;

#[derive(Clone, PartialEq)]
pub struct Window {
    styles: Cow<'static, [&'static Style]>,
    class_name: Option<Cow<'static, str>>,
    pub title: String,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            styles: Cow::default(),
            class_name: None,
            title: "Untitled Window".to_owned(),
        }
    }
}

crate::element_attributes! {
    Window {
        styles: AttrStyles,
        class_name: AttrClassName,
        title: AttrTitle,
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
