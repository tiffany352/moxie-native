use super::{DrawContext, Element};
use crate::layout::LayoutOptions;
use crate::Color;
use euclid::{point2, vec2, Rect};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use skribo::{FontCollection, FontFamily, LayoutSession, TextStyle};
use std::borrow::Cow;
use std::cell::Cell;
use webrender::api::{
    units::Au, BorderRadius, ClipMode, ColorF, CommonItemProperties, ComplexClipRegion,
    FontInstanceKey, GlyphInstance, PrimitiveFlags, SpaceAndClipInfo, SpatialId,
};

#[derive(Default, Clone, PartialEq)]
pub struct Span {
    class_name: Option<Cow<'static, str>>,
    color: Option<Color>,
    font_instance_key: Cell<Option<FontInstanceKey>>,
}

impl Span {
    pub fn new() -> Span {
        Span {
            class_name: None,
            color: None,
            font_instance_key: Cell::new(None),
        }
    }

    pub fn on<Event>(&mut self, func: impl FnMut(&Event) + 'static)
    where
        Event: SpanEvent,
    {
        Event::set_to_span(self, func);
    }
}

pub trait SpanEvent {
    fn set_to_span(span: &mut Span, func: impl FnMut(&Self) + 'static);
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

    fn draw(&self, context: DrawContext) {
        let rect = Rect::new(context.position, context.size);

        println!("text rect {}", rect);

        let inner_text = "the quick brown fox jumps over the lazy dog";
        let len = inner_text.len();
        let size = 32.0;

        let mut collection = FontCollection::new();
        let source = SystemSource::new();
        let font = source
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
        collection.add_family(FontFamily::new_from_font(font));

        let mut layout = LayoutSession::create(&inner_text, &TextStyle { size }, &collection);

        let color = self.color.unwrap_or(Color::new(0, 0, 0, 255));

        let space_and_clip = SpaceAndClipInfo::root_scroll(context.pipeline_id);
        context.builder.push_simple_stacking_context(
            point2(0.0, 0.0),
            space_and_clip.spatial_id,
            PrimitiveFlags::IS_BACKFACE_VISIBLE,
        );

        for run in layout.iter_substr(0..len) {
            let font = run.font();
            let mut glyphs = vec![];
            for glyph in run.glyphs() {
                let pos =
                    context.position * context.scale + vec2(glyph.offset.x, glyph.offset.y + size);
                println!("glyph id:{} pos:{}", glyph.glyph_id, pos);
                glyphs.push(GlyphInstance {
                    index: glyph.glyph_id,
                    point: pos,
                })
            }

            println!("font: {}", font.font.full_name());
            println!("#glyphs {}", glyphs.len());

            let key = if let Some(key) = self.font_instance_key.get() {
                key
            } else {
                let font_key = context.api.generate_font_key();
                let font_data = font.font.copy_font_data().unwrap().to_vec();
                println!("font data: {} bytes", font_data.len());
                context.transaction.add_raw_font(font_key, font_data, 0);

                let key = context.api.generate_font_instance_key();
                context.transaction.add_font_instance(
                    key,
                    font_key,
                    Au::from_f64_px(size as f64),
                    None,
                    None,
                    vec![],
                );
                self.font_instance_key.set(Some(key));
                key
            };

            println!("key {:?}", key);

            context.builder.push_text(
                &CommonItemProperties::new(rect * context.scale, space_and_clip),
                rect * context.scale,
                &glyphs[..],
                key,
                ColorF::new(
                    color.red as f32 / 255.0,
                    color.green as f32 / 255.0,
                    color.blue as f32 / 255.0,
                    color.alpha as f32 / 255.0,
                ),
                None,
            );
        }

        context.builder.pop_stacking_context();
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            ..Default::default()
        }
    }
}
