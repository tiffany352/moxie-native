use crate::dom::element::Element;
use crate::dom::{AttrStyle, Button, Node, Span};
use crate::style::{ComputedValues, Style};
use crate::Color;

/// Corresponds to <view>. Generic frame for layout purposes.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct View {
    style: Option<Style>,
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
        style: AttrStyle,
    }
}

impl Element for View {
    type Child = ViewChild;
    type Handlers = ();
    type States = ();

    const ELEMENT_NAME: &'static str = "view";

    fn create_computed_values(&self) -> ComputedValues {
        ComputedValues {
            background_color: Color::new(50, 180, 200, 255),
            ..Default::default()
        }
    }

    fn style(&self) -> Option<Style> {
        self.style
    }
}
