use crate::dom::element::Element;
use crate::dom::{AttrClassName, AttrStyles, Button, Node, Span};
use crate::style::{ComputedValues, Style};
use crate::Color;
use std::borrow::Cow;

/// Corresponds to <view>. Generic frame for layout purposes.
#[derive(Default, Clone, PartialEq)]
pub struct View {
    styles: Cow<'static, [&'static Style]>,
    class_name: Option<Cow<'static, str>>,
}

multiple_children! {
    enum ViewChild {
        Button(Node<Button>),
        View(Node<View>),
        Span(Node<Span>),
    }
}

element_attributes! {
    View {
        styles: AttrStyles,
        class_name: AttrClassName,
    }
}

impl Element for View {
    type Child = ViewChild;
    type Handlers = ();

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }

    fn styles(&self) -> &[&'static Style] {
        self.styles.as_ref()
    }

    fn create_computed_values(&self) -> ComputedValues {
        ComputedValues {
            background_color: Color::new(50, 180, 200, 255),
            ..Default::default()
        }
    }
}
