use super::Element;
use crate::layout::{LayoutOptions, LayoutType};
use crate::render::PaintDetails;
use crate::Color;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Span {
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
}

impl Span {
    pub fn new() -> Span {
        Span {
            class_name: None,
            color: None,
        }
    }
}

impl Element for Span {
    type Child = String;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            "color" => self.color = value.and_then(|s| Color::parse(s.as_ref()).ok()),
            _ => (),
        }
    }

    fn paint(&self) -> Option<PaintDetails> {
        None
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            layout_ty: LayoutType::Inline,
            ..Default::default()
        }
    }
}
