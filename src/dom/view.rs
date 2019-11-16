use super::{DrawContext, Element, Node, NodeChild, Span};
use crate::layout::{LayoutOptions, LogicalLength, LogicalSideOffsets};
use crate::Color;
use euclid::Rect;
use std::borrow::Cow;
use webrender::api::{
    BorderRadius, ClipMode, ColorF, CommonItemProperties, ComplexClipRegion, SpaceAndClipInfo,
    SpatialId,
};

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
    fn draw(&self, context: DrawContext) {
        match self {
            ViewChild::View(view) => view.draw(context),
            ViewChild::Span(span) => span.draw(context),
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

    fn draw(&self, context: DrawContext) {
        let rect = Rect::new(context.position, context.size);
        let region = ComplexClipRegion::new(
            rect * context.scale,
            BorderRadius::uniform(20.),
            ClipMode::Clip,
        );
        let clip = context.builder.define_clip(
            &SpaceAndClipInfo::root_scroll(context.pipeline_id),
            rect * context.scale,
            vec![region],
            None,
        );
        let color = self.color.unwrap_or(Color::new(50, 180, 200, 255));
        context.builder.push_rect(
            &CommonItemProperties::new(
                rect * context.scale,
                SpaceAndClipInfo {
                    spatial_id: SpatialId::root_scroll_node(context.pipeline_id),
                    clip_id: clip,
                },
            ),
            ColorF::new(
                color.red as f32 / 255.0,
                color.green as f32 / 255.0,
                color.blue as f32 / 255.0,
                color.alpha as f32 / 255.0,
            ),
        );
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
