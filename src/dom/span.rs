use super::Element;
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

impl Span {
    pub fn new() -> Span {
        Span {
            class_name: None,
            color: None,
            text_size: None,
        }
    }
}

impl Element for Span {
    type Child = String;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            "color" => self.color = value.and_then(|s| Color::parse(s.as_ref()).ok()),
            "textSize" => self.text_size = value.and_then(|s| s.parse::<f32>().ok()),
            _ => (),
        }
    }

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
}
