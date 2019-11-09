use super::Element;
use crate::layout::{Layout, LayoutOptions, LogicalLength, LogicalPixel, LogicalSideOffsets};
use crate::Color;
use euclid::{Point2D, Rect, Scale};
use std::borrow::Cow;
use webrender::api::{
    units::LayoutPixel, BorderRadius, ClipMode, ColorF, CommonItemProperties, ComplexClipRegion,
    DisplayListBuilder, PipelineId, SpaceAndClipInfo, SpatialId,
};

#[derive(Default)]
pub struct View {
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
    width: Option<f32>,
    height: Option<f32>,
    padding: Option<f32>,
    layout: Layout,
}

impl View {
    pub fn new() -> View {
        View {
            class_name: None,
            color: None,
            width: None,
            height: None,
            padding: None,
            layout: Layout::new(),
        }
    }

    pub fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            "color" => self.color = value.and_then(|string| Color::parse(&string[..]).ok()),
            "width" => self.width = value.and_then(|string| string.parse::<f32>().ok()),
            "height" => self.height = value.and_then(|string| string.parse::<f32>().ok()),
            "padding" => self.padding = value.and_then(|string| string.parse::<f32>().ok()),
            _ => (),
        }
    }

    pub fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            width: self.width.map(LogicalLength::new),
            height: self.height.map(LogicalLength::new),
            padding: LogicalSideOffsets::new_all_same(self.padding.unwrap_or(0.0)),
            ..Default::default()
        }
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    pub fn draw(
        &self,
        position: Point2D<f32, LogicalPixel>,
        scale: Scale<f32, LogicalPixel, LayoutPixel>,
        builder: &mut DisplayListBuilder,
        pipeline_id: PipelineId,
    ) {
        let layout_size = self.layout.size();
        let rect = Rect::new(position, layout_size);
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

impl Into<Element> for View {
    fn into(self) -> Element {
        Element::View(self)
    }
}
