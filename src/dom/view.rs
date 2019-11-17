use super::{Element, Node, NodeChild, Span};
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
    //on_test_event: Option<Rc<dyn FnMut(&TestEvent) + 'static>>,
}

impl View {
    pub fn new() -> View {
        View {
            class_name: None,
            color: None,
            width: None,
            height: None,
            padding: None,
            //on_test_event: None,
        }
    }

    pub fn on<Event>(&mut self, func: impl FnMut(&Event) + 'static)
    where
        Event: ViewEvent,
    {
        Event::set_to_view(self, func);
    }
}

pub trait ViewEvent {
    fn set_to_view(view: &mut View, func: impl FnMut(&Self) + 'static);
}

pub struct TestEvent;

impl ViewEvent for TestEvent {
    fn set_to_view(view: &mut View, func: impl FnMut(&Self) + 'static) {
        //view.on_test_event = Some(Box::new(func));
    }
}

#[derive(Clone, PartialEq)]
pub enum ViewChild {
    View(Node<View>),
    Span(Node<Span>),
}

impl From<Node<View>> for ViewChild {
    fn from(node: Node<View>) -> Self {
        ViewChild::View(node)
    }
}

impl From<Node<Span>> for ViewChild {
    fn from(node: Node<Span>) -> Self {
        ViewChild::Span(node)
    }
}

impl NodeChild for ViewChild {
    fn paint(&self) -> Option<PaintDetails> {
        match self {
            ViewChild::View(view) => view.paint(),
            ViewChild::Span(span) => span.paint(),
        }
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        match self {
            ViewChild::View(view) => view.create_layout_opts(),
            ViewChild::Span(span) => span.create_layout_opts(),
        }
    }

    fn get_child(&self, index: usize) -> Option<&dyn NodeChild> {
        match self {
            ViewChild::View(view) => {
                if let Some(child) = view.children().get(index) {
                    Some(child)
                } else {
                    None
                }
            }
            ViewChild::Span(span) => {
                if let Some(child) = span.children().get(index) {
                    Some(child)
                } else {
                    None
                }
            }
        }
    }
}

impl Element for View {
    type Child = ViewChild;

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
            ..Default::default()
        })
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            width: self.width.map(LogicalLength::new),
            height: self.height.map(LogicalLength::new),
            padding: LogicalSideOffsets::new_all_same(self.padding.unwrap_or(0.0)),
            ..Default::default()
        }
    }
}
