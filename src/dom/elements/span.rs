use crate::dom::element::Element;
use crate::dom::{AttrStyle, Button, Node, View};
use crate::style::{ComputedValues, DisplayType, InlineValues, Style};

/// Corresponds to <span>. This element is typically used for inline
/// layout of text.
#[derive(Default, Clone, PartialEq)]
pub struct Span {
    style: Option<Style>,
}

element_attributes! {
    Span {
        style: AttrStyle,
    }
}

multiple_children! {
    enum SpanChild {
        Text(String),
        Button(Node<Button>),
        View(Node<View>),
        Span(Node<Span>),
    }
}

impl Element for Span {
    type Child = SpanChild;
    type Handlers = ();
    type States = ();

    const ELEMENT_NAME: &'static str = "span";

    fn create_computed_values(&self) -> ComputedValues {
        ComputedValues {
            display: DisplayType::Inline(InlineValues {}),
            ..Default::default()
        }
    }

    fn style(&self) -> Option<Style> {
        self.style
    }
}
