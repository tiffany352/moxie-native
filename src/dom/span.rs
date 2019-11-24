use super::{AttrClassName, AttrStyles, Element};
use crate::render::PaintDetails;
use crate::style::{ComputedValues, DisplayType, InlineValues, Style};
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Span {
    styles: Cow<'static, [&'static Style]>,
    class_name: Option<Cow<'static, str>>,
}

crate::element_attributes! {
    Span {
        styles: AttrStyles,
        class_name: AttrClassName,
    }
}

impl Element for Span {
    type Child = String;
    type Handlers = ();

    fn paint(&self, _handlers: &()) -> Option<PaintDetails> {
        None
    }

    fn create_computed_values(&self) -> ComputedValues {
        ComputedValues {
            display: DisplayType::Inline(InlineValues {}),
            ..Default::default()
        }
    }

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }

    fn styles(&self) -> &[&'static Style] {
        self.styles.as_ref()
    }
}
