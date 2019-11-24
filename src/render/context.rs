use super::engine::{PaintTreeNode, RenderEngine};
use crate::dom::{ClickEvent, Node, Window};
use crate::layout::{LayoutEngine, LayoutText, LayoutTreeNode, LogicalPixel, LogicalPoint};
use crate::style::StyleEngine;
use crate::Color;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use gleam::gl;
use skribo::{FontCollection, FontFamily, FontRef, LayoutSession, TextStyle};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use webrender::{
    api::{
        units::Au, units::DeviceIntRect, units::DevicePixel, units::LayoutPixel, BorderRadius,
        ClipMode, ColorF, CommonItemProperties, ComplexClipRegion, DisplayListBuilder, DocumentId,
        Epoch, FontInstanceKey, FontKey, GlyphInstance, PipelineId, PrimitiveFlags, RenderApi,
        RenderNotifier, SpaceAndClipInfo, SpatialId, Transaction,
    },
    euclid::{point2, size2, vec2, Point2D, Rect, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{
    dpi::{LogicalPosition as WinitLogicalPosition, PhysicalSize},
    event_loop::EventLoopProxy,
    window::Window as WinitWindow,
};

/// Used to wait for frames to be ready in Webrender.
#[derive(Clone)]
struct Notifier {
    events_proxy: EventLoopProxy<()>,
    tx: mpsc::Sender<()>,
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Clone::clone(self))
    }

    fn wake_up(&self) {
        self.tx.send(()).unwrap();
        let _ = self.events_proxy.send_event(());
    }

    fn new_frame_ready(&self, _: DocumentId, _: bool, _: bool, _: Option<u64>) {
        self.wake_up();
    }
}

/// Contains everything needed to display the DOM. It creates an
/// Angle-based GL context, a Webrender rendering context, and manages
/// the `LayoutEngine` and `RenderEngine` for creating the DOM's layout
/// and paint trees. It handles bubbling input events through the DOM as
/// well.
pub struct Context {
    api: RenderApi,
    document: DocumentId,
    rx: mpsc::Receiver<()>,
    renderer: Renderer,
    layout_engine: LayoutEngine,
    render_engine: RenderEngine,
    style_engine: StyleEngine,
    window: Node<Window>,
    client_size: Size2D<i32, DevicePixel>,
    dpi_scale: f32,
    fonts: HashMap<String, FontKey>,
    font_instances: HashMap<(FontKey, usize), FontInstanceKey>,
}

impl Context {
    pub fn new(
        gl: Rc<dyn gl::Gl>,
        parent_window: &WinitWindow,
        events_proxy: EventLoopProxy<()>,
        window: Node<Window>,
    ) -> Context {
        let (tx, rx) = mpsc::channel();
        let notifier = Box::new(Notifier { events_proxy, tx });

        let dpi_scale = parent_window.hidpi_factor() as f32;
        let inner_size = parent_window.inner_size().to_physical(dpi_scale as f64);
        let client_size =
            Size2D::<i32, DevicePixel>::new(inner_size.width as i32, inner_size.height as i32);

        let (renderer, sender) = Renderer::new(
            gl,
            notifier.clone(),
            RendererOptions {
                clear_color: Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
                device_pixel_ratio: dpi_scale,
                ..Default::default()
            },
            None,
            client_size,
        )
        .unwrap();
        let api = sender.create_api();
        let document = api.add_document(client_size, 0);

        Context {
            api,
            document,
            rx,
            renderer,
            window,
            layout_engine: LayoutEngine::new(),
            render_engine: RenderEngine::new(),
            style_engine: StyleEngine::new(),
            client_size,
            dpi_scale,
            fonts: HashMap::new(),
            font_instances: HashMap::new(),
        }
    }

    pub fn set_dom_window(&mut self, new_node: Node<Window>) {
        if new_node != self.window {
            self.window = new_node;
        }
    }

    pub fn resize(&mut self, size: PhysicalSize, dpi_scale: f32) {
        self.client_size = size2(size.width as i32, size.height as i32);
        self.dpi_scale = dpi_scale;
    }

    fn get_font(&mut self, font: &FontRef, txn: &mut Transaction) -> FontKey {
        let full_name = font.font.full_name();
        if let Some(&key) = self.fonts.get(&full_name) {
            return key;
        }
        let key = self.api.generate_font_key();
        let font_data = font.font.copy_font_data().unwrap().to_vec();
        txn.add_raw_font(key, font_data, 0);
        self.fonts.insert(full_name, key);

        key
    }

    fn get_font_instance(
        &mut self,
        key: FontKey,
        size: usize,
        txn: &mut Transaction,
    ) -> FontInstanceKey {
        if let Some(&instance) = self.font_instances.get(&(key, size)) {
            return instance;
        }
        let instance = self.api.generate_font_instance_key();
        txn.add_font_instance(
            instance,
            key,
            Au::from_f64_px(size as f64),
            None,
            None,
            vec![],
        );
        self.font_instances.insert((key, size), instance);

        instance
    }

    fn render_child(
        &mut self,
        pipeline_id: PipelineId,
        builder: &mut DisplayListBuilder,
        transaction: &mut Transaction,
        position: Point2D<f32, LogicalPixel>,
        paint: &PaintTreeNode,
        layout: &Rc<LayoutTreeNode>,
    ) {
        let rect = Rect::new(position, layout.size) * Scale::new(1.0);

        if let Some(ref values) = paint.values {
            if values.background_color.alpha > 0 {
                let region =
                    ComplexClipRegion::new(rect, BorderRadius::uniform(20.), ClipMode::Clip);
                let clip = builder.define_clip(
                    &SpaceAndClipInfo::root_scroll(pipeline_id),
                    rect,
                    vec![region],
                    None,
                );
                builder.push_rect(
                    &CommonItemProperties::new(
                        rect,
                        SpaceAndClipInfo {
                            spatial_id: SpatialId::root_scroll_node(pipeline_id),
                            clip_id: clip,
                        },
                    ),
                    values.background_color.into(),
                );
            }
        }

        if let Some(LayoutText { ref text, size }) = layout.render_text {
            let mut collection = FontCollection::new();
            let source = SystemSource::new();
            let font = source
                .select_best_match(&[FamilyName::SansSerif], &Properties::new())
                .unwrap()
                .load()
                .unwrap();
            collection.add_family(FontFamily::new_from_font(font));

            let mut layout = LayoutSession::create(text, &TextStyle { size }, &collection);
            let color = Color::new(0, 0, 0, 255);
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);
            builder.push_simple_stacking_context(
                point2(0.0, 0.0),
                space_and_clip.spatial_id,
                PrimitiveFlags::IS_BACKFACE_VISIBLE,
            );
            for run in layout.iter_substr(0..text.len()) {
                let font = run.font();
                let metrics = font.font.metrics();
                let units_per_px = metrics.units_per_em as f32 / size;
                let baseline_offset = metrics.ascent / units_per_px;
                let mut glyphs = vec![];
                for glyph in run.glyphs() {
                    let pos = position + vec2(glyph.offset.x, glyph.offset.y + baseline_offset);
                    glyphs.push(GlyphInstance {
                        index: glyph.glyph_id,
                        point: pos * Scale::new(1.0),
                    })
                }
                let font_key = self.get_font(font, transaction);
                let key = self.get_font_instance(font_key, size as usize, transaction);
                builder.push_text(
                    &CommonItemProperties::new(rect, space_and_clip),
                    rect,
                    &glyphs[..],
                    key,
                    color.into(),
                    None,
                );
            }
            builder.pop_stacking_context();
        }

        for layout in &layout.children {
            let paint = &paint.children[layout.index];
            self.render_child(
                pipeline_id,
                builder,
                transaction,
                position + layout.position.to_vector(),
                paint,
                &layout.layout,
            );
        }
    }

    pub fn render(&mut self) {
        let client_size = self.client_size;
        let dpi_scale = Scale::new(self.dpi_scale);
        let content_size = client_size.to_f32() / dpi_scale;

        println!("render()");
        let pipeline_id = PipelineId(0, 0);
        let mut builder = DisplayListBuilder::new(pipeline_id, content_size);
        let mut transaction = Transaction::new();

        self.style_engine
            .update(self.window.clone(), content_size * Scale::new(1.0));

        let root_layout = self
            .layout_engine
            .layout(self.window.clone(), content_size * Scale::new(1.0));

        let root_paint = self
            .render_engine
            .render(self.window.clone(), root_layout.clone());

        for layout in &root_layout.children {
            let details = &root_paint.children[layout.index];
            self.render_child(
                pipeline_id,
                &mut builder,
                &mut transaction,
                layout.position,
                details,
                &layout.layout,
            );
        }

        transaction.set_display_list(Epoch(0), None, content_size, builder.finalize(), true);
        transaction.set_root_pipeline(pipeline_id);
        transaction.generate_frame();
        self.api.set_document_view(
            self.document,
            DeviceIntRect::new(Point2D::zero(), client_size.to_i32()),
            dpi_scale.get(),
        );
        self.api.send_transaction(self.document, transaction);
        self.rx.recv().unwrap();
        self.renderer.update();
        let _ = self.renderer.render(client_size.to_i32());
        let _ = self.renderer.flush_pipeline_info();
    }

    pub fn process_child(
        &mut self,
        cursor: LogicalPoint,
        position: Point2D<f32, LogicalPixel>,
        paint: &PaintTreeNode,
        layout: &Rc<LayoutTreeNode>,
    ) -> bool {
        let rect = Rect::new(position, layout.size) * Scale::new(1.0);

        for layout in &layout.children {
            let paint = &paint.children[layout.index];
            if self.process_child(
                cursor,
                position + layout.position.to_vector(),
                paint,
                &layout.layout,
            ) {
                return true;
            }
        }

        if let Some(ref details) = paint.details {
            if rect.contains(cursor) && details.on_click.present() {
                details.on_click.invoke(&ClickEvent);
                return true;
            }
        }

        false
    }

    pub fn process_click(&mut self, position: WinitLogicalPosition) -> bool {
        let client_size = self.client_size;
        let dpi_scale = Scale::new(self.dpi_scale);
        let content_size: Size2D<f32, LayoutPixel> = client_size.to_f32() / dpi_scale;
        let position: LogicalPoint = point2(position.x as f32, position.y as f32);

        self.style_engine
            .update(self.window.clone(), content_size * Scale::new(1.0));

        let root_layout = self
            .layout_engine
            .layout(self.window.clone(), content_size * Scale::new(1.0));

        let root_paint = self
            .render_engine
            .render(self.window.clone(), root_layout.clone());

        for layout in &root_layout.children {
            let details = &root_paint.children[layout.index];
            if self.process_child(position, layout.position, details, &layout.layout) {
                return true;
            }
        }

        false
    }
}
