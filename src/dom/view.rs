use super::{
    AttrClassName, AttrColor, AttrHeight, AttrPadding, AttrWidth, Button, Element, Node, Span,
};
use crate::layout::{LayoutOptions, LogicalLength, LogicalSideOffsets};
use crate::render::PaintDetails;
use crate::Color;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct View {
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
    width: Option<f32>,
    height: Option<f32>,
    padding: Option<f32>,
}

crate::multiple_children! {
    enum ViewChild {
        Button(Node<Button>),
        View(Node<View>),
        Span(Node<Span>),
    }
}

crate::element_attributes! {
    View {
        class_name: AttrClassName,
        padding: AttrPadding,
        width: AttrWidth,
        height: AttrHeight,
        color: AttrColor,
    }
}

impl Element for View {
    type Child = ViewChild;

    fn paint(&self) -> Option<PaintDetails> {
        Some(PaintDetails {
            background_color: Some(self.color.unwrap_or(Color::new(50, 180, 200, 255))),
            ..Default::default()
        })
    }

    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions {
        LayoutOptions {
            width: self.width.map(LogicalLength::new),
            height: self.height.map(LogicalLength::new),
            padding: LogicalSideOffsets::new_all_same(self.padding.unwrap_or(0.0)),
            text_size: parent_opts.text_size,
            ..Default::default()
        }
    }

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }
}
