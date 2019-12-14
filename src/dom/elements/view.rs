use crate::dom::element::Element;
use crate::dom::AttrStyle;
use crate::style::{ComputedValues, Style};
use crate::Color;

/// Corresponds to <view>. Generic frame for layout purposes.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct View {
    style: Option<Style>,
}

element_attributes! {
    View {
        style: AttrStyle,
    }
}

impl Element for View {
    type Child = super::BlockChild;
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
