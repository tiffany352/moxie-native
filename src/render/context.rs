use super::engine::{PaintTreeNode, RenderEngine};
use crate::direct_composition::{AngleVisual, DirectComposition};
use crate::dom::{Node, Window};
use crate::layout::{LayoutEngine, LayoutText, LayoutTreeNode, LogicalPixel};
use crate::Color;
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use skribo::{FontCollection, FontFamily, FontRef, LayoutSession, TextStyle};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use webrender::{
    api::{
        units::Au, units::DeviceIntRect, units::DevicePixel, BorderRadius, ClipMode, ColorF,
        CommonItemProperties, ComplexClipRegion, DisplayListBuilder, DocumentId, Epoch,
        FontInstanceKey, FontKey, GlyphInstance, PipelineId, PrimitiveFlags, RenderApi,
        RenderNotifier, SpaceAndClipInfo, SpatialId, Transaction,
    },
    euclid::{point2, size2, vec2, Point2D, Rect, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoop, EventLoopProxy},
    platform::windows::WindowExtWindows,
    window::Window as WinitWindow,
};

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

pub struct Context {
    composition: DirectComposition,
    visual: Option<AngleVisual>,
    api: RenderApi,
    document: DocumentId,
    rx: mpsc::Receiver<()>,
    renderer: Renderer,
    layout_engine: LayoutEngine,
    render_engine: RenderEngine,
    window: Node<Window>,
    client_size: Size2D<i32, DevicePixel>,
    dpi_scale: f32,
    fonts: HashMap<String, FontKey>,
    font_instances: HashMap<(FontKey, usize), FontInstanceKey>,
}

impl Context {
    pub fn new(
        parent_window: &WinitWindow,
        event_loop: &EventLoop<()>,
        window: Node<Window>,
    ) -> Context {
        let (tx, rx) = mpsc::channel();
        let notifier = Box::new(Notifier {
            events_proxy: event_loop.create_proxy(),
            tx,
        });

        let dpi_scale = parent_window.hidpi_factor() as f32;
        let inner_size = parent_window.inner_size().to_physical(dpi_scale as f64);
        let client_size =
            Size2D::<i32, DevicePixel>::new(inner_size.width as i32, inner_size.height as i32);

        let composition = unsafe { DirectComposition::new(parent_window.hwnd() as _) };
        let visual =
            composition.create_angle_visual(client_size.width as u32, client_size.height as u32);
        visual.make_current();
        let (renderer, sender) = Renderer::new(
            composition.gleam.clone(),
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
            composition,
            visual: Some(visual),
            api,
            document,
            rx,
            renderer,
            window,
            layout_engine: LayoutEngine::new(),
            render_engine: RenderEngine::new(),
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
        if let Some(visual) = self.visual.take() {
            self.composition.cleanup_angle_visual(visual);
        }
        if size.width > 0.0 && size.height > 0.0 {
            self.visual = Some(
                self.composition
                    .create_angle_visual(size.width as u32, size.height as u32),
            );
        }
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

        if let Some(ref details) = paint.details {
            if let Some(color) = details.background_color {
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
                    color.into(),
                );
            }

            if let Some(LayoutText { ref text, size }) = layout.render_text {
                println!("plain text {}", text);
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
                        println!("glyph id:{} pos:{}", glyph.glyph_id, pos);
                        glyphs.push(GlyphInstance {
                            index: glyph.glyph_id,
                            point: pos * Scale::new(1.0),
                        })
                    }
                    println!("font: {}", font.font.full_name());
                    println!("#glyphs {}", glyphs.len());
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
        self.visual.as_mut().unwrap().make_current();
        let pipeline_id = PipelineId(0, 0);
        let mut builder = DisplayListBuilder::new(pipeline_id, content_size);
        let mut transaction = Transaction::new();

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
        self.visual.as_mut().unwrap().present();
        self.composition.commit();
    }
}
