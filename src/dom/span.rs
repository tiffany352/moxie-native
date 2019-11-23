use super::{AttrClassName, AttrColor, AttrTextSize, Element};
use crate::layout::{LayoutOptions, LayoutType, LogicalLength};
use crate::render::PaintDetails;
use crate::Color;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Span {
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
    text_size: Option<f32>,
}

crate::element_attributes! {
    Span {
        class_name: AttrClassName,
        color: AttrColor,
        text_size: AttrTextSize,
    }
}

impl Element for Span {
    type Child = String;

    fn paint(&self) -> Option<PaintDetails> {
        None
    }

    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions {
        LayoutOptions {
            layout_ty: LayoutType::Inline,
            text_size: self
                .text_size
                .map(LogicalLength::new)
                .unwrap_or(parent_opts.text_size),
            ..Default::default()
        }
    }

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }
}
