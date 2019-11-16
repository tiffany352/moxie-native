use super::Element;
use crate::dom::node::Node;
use crate::layout::{
    LayoutOptions, LogicalLength, LogicalPixel, LogicalPoint, LogicalSideOffsets, LogicalSize,
};
use crate::Color;
use euclid::{Rect, Scale};
use std::borrow::Cow;
use webrender::api::{
    units::LayoutPixel, BorderRadius, ClipMode, ColorF, CommonItemProperties, ComplexClipRegion,
    DisplayListBuilder, PipelineId, SpaceAndClipInfo, SpatialId,
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

    pub fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            width: self.width.map(LogicalLength::new),
            height: self.height.map(LogicalLength::new),
            padding: LogicalSideOffsets::new_all_same(self.padding.unwrap_or(0.0)),
            ..Default::default()
        }
    }

    pub fn draw(
        &self,
        position: LogicalPoint,
        size: LogicalSize,
        scale: Scale<f32, LogicalPixel, LayoutPixel>,
        builder: &mut DisplayListBuilder,
        pipeline_id: PipelineId,
    ) {
        let rect = Rect::new(position, size);
        let region =
            ComplexClipRegion::new(rect * scale, BorderRadius::uniform(20.), ClipMode::Clip);
        let clip = builder.define_clip(
            &SpaceAndClipInfo::root_scroll(pipeline_id),
            rect * scale,
            vec![region],
            None,
        );
        let color = self.color.unwrap_or(Color::new(50, 180, 200, 255));
        builder.push_rect(
            &CommonItemProperties::new(
                rect * scale,
                SpaceAndClipInfo {
                    spatial_id: SpatialId::root_scroll_node(pipeline_id),
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

impl Element for View {
    type Child = Node<View>;

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
}
