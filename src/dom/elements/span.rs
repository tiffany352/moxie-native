use crate::dom::element::Element;
use crate::dom::AttrStyle;
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

impl Element for Span {
    type Child = String;
    type Handlers = ();
    type States = ();

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
