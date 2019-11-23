use super::{
    AttrClassName, AttrColor, AttrHeight, AttrPadding, AttrStyles, AttrWidth, ClickEvent, Element,
    EventHandler, HasEvent, Node, Span, View,
};
use crate::layout::{LayoutOptions, LogicalLength, LogicalSideOffsets};
use crate::render::PaintDetails;
use crate::style::Style;
use crate::Color;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Button {
    styles: Cow<'static, [&'static Style]>,
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
    width: Option<f32>,
    height: Option<f32>,
    padding: Option<f32>,
}

crate::multiple_children! {
    enum ButtonChild {
        Button(Node<Button>),
        View(Node<View>),
        Span(Node<Span>),
    }
}

crate::element_attributes! {
    Button {
        styles: AttrStyles,
        class_name: AttrClassName,
        padding: AttrPadding,
        width: AttrWidth,
        height: AttrHeight,
        color: AttrColor,
    }
}

crate::element_handlers! {
    ButtonHandlers for Button {
        on_click: ClickEvent,
    }
}

impl Element for Button {
    type Child = ButtonChild;
    type Handlers = ButtonHandlers;

    fn paint(&self, handlers: &ButtonHandlers) -> Option<PaintDetails> {
        Some(PaintDetails {
            background_color: Some(self.color.unwrap_or(Color::new(50, 180, 200, 255))),
            on_click: handlers.on_click.clone(),
            ..Default::default()
        })
    }

    fn create_layout_opts(&self, parent_opts: &LayoutOptions) -> LayoutOptions {
        LayoutOptions {
            width: self.width.map(LogicalLength::new),
            height: self.height.map(LogicalLength::new),
            padding: LogicalSideOffsets::new_all_same(self.padding.unwrap_or(4.0)),
            text_size: parent_opts.text_size,
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
