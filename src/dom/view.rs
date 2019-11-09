use super::Element;
use crate::layout::{Layout, LayoutOptions, LogicalLength, LogicalPixel, LogicalSize};
use euclid::{Point2D, Rect, Scale};
use std::borrow::Cow;
use webrender::api::{
    units::LayoutPixel, BorderRadius, ClipMode, ColorF, CommonItemProperties, ComplexClipRegion,
    DisplayListBuilder, PipelineId, SpaceAndClipInfo, SpatialId,
};

#[derive(Default)]
pub struct View {
    class_name: Option<Cow<'static, str>>,
    layout: Layout,
}

impl View {
    pub fn new() -> View {
        View {
            class_name: None,
            layout: Layout::new(),
        }
    }

    pub fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            _ => (),
        }
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            width: Some(LogicalLength::new(200.0)),
            height: Some(LogicalLength::new(200.0)),
            ..Default::default()
        }
    }

    pub fn draw(
        &self,
        position: Point2D<f32, LogicalPixel>,
        scale: Scale<f32, LogicalPixel, LayoutPixel>,
        builder: &mut DisplayListBuilder,
        pipeline_id: PipelineId,
        child_sizes: &[LogicalSize],
    ) {
        let layout_size = self
            .layout
            .calc_min_size(&self.create_layout_opts(), child_sizes);
        let rect = Rect::new(position, layout_size);
        let region =
            ComplexClipRegion::new(rect * scale, BorderRadius::uniform(20.), ClipMode::Clip);
        let clip = builder.define_clip(
            &SpaceAndClipInfo::root_scroll(pipeline_id),
            rect * scale,
            vec![region],
            None,
        );
        builder.push_rect(
            &CommonItemProperties::new(
                rect * scale,
                SpaceAndClipInfo {
                    spatial_id: SpatialId::root_scroll_node(pipeline_id),
                    clip_id: clip,
                },
            ),
            ColorF::new(0.2, 0.7, 0.8, 1.0),
        );
    }
}

impl Into<Element> for View {
    fn into(self) -> Element {
        Element::View(self)
    }
}
