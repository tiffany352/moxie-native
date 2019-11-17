use super::{CanSetEvent, ClickEvent, Element, EventHandler, Node, Span, View};
use crate::layout::{LayoutOptions, LogicalLength, LogicalSideOffsets};
use crate::render::PaintDetails;
use crate::Color;
use std::borrow::Cow;

#[derive(Default, Clone, PartialEq)]
pub struct Button {
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
    width: Option<f32>,
    height: Option<f32>,
    padding: Option<f32>,
    on_click: EventHandler<ClickEvent>,
}

impl CanSetEvent<ClickEvent> for Button {
    fn set_handler(&mut self, handler: EventHandler<ClickEvent>) {
        self.on_click = handler;
    }
}

crate::multiple_children! {
    enum ButtonChild {
        Button(Node<Button>),
        View(Node<View>),
        Span(Node<Span>),
    }
}

impl Element for Button {
    type Child = ButtonChild;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            "color" => self.color = value.and_then(|string| Color::parse(&string[..]).ok()),
            "width" => self.width = value.and_then(|string| string.parse::<f32>().ok()),
            "height" => self.height = value.and_then(|string| string.parse::<f32>().ok()),
            "padding" => self.padding = value.and_then(|string| string.parse::<f32>().ok()),
            _ => (),
        }
    }

    fn paint(&self) -> Option<PaintDetails> {
        Some(PaintDetails {
            background_color: Some(self.color.unwrap_or(Color::new(50, 180, 200, 255))),
            on_click: self.on_click.clone(),
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
}
