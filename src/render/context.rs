use crate::document::Document;
use crate::dom::{Node, Window};
use crate::layout::{LayoutText, LayoutTreeNode, LogicalPixel, LogicalSideOffsets, RenderData};
use crate::util::equal_rc::EqualRc;
use gleam::gl;
use skribo::FontRef;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use webrender::{
    api::{
        units::Au, units::DeviceIntRect, units::DevicePixel, units::LayoutSideOffsets,
        BorderDetails, BorderRadius, BorderSide, BorderStyle, ClipId, ClipMode, ColorF,
        CommonItemProperties, ComplexClipRegion, DisplayListBuilder, DocumentId, Epoch,
        FontInstanceKey, FontKey, GlyphInstance, HitTestFlags, NormalBorder, PipelineId,
        PrimitiveFlags, RenderApi, RenderNotifier, SpaceAndClipInfo, SpatialId, Transaction,
    },
    euclid::{point2, size2, Point2D, Rect, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{
    dpi::{LogicalPosition, PhysicalSize},
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
    document_id: DocumentId,
    rx: mpsc::Receiver<()>,
    renderer: Renderer,
    pub document: Document,
    client_size: Size2D<i32, DevicePixel>,
    dpi_scale: f32,
    fonts: HashMap<String, FontKey>,
    font_instances: HashMap<(FontKey, usize), FontInstanceKey>,
}

fn convert_offsets(input: LogicalSideOffsets) -> LayoutSideOffsets {
    LayoutSideOffsets::new(input.top, input.right, input.bottom, input.left)
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
        let document_id = api.add_document(client_size, 0);

        let document = Document::new(
            window,
            client_size.to_f32() / Scale::<f32, LogicalPixel, DevicePixel>::new(dpi_scale),
        );

        Context {
            api,
            document_id,
            rx,
            renderer,
            document,
            client_size,
            dpi_scale,
            fonts: HashMap::new(),
            font_instances: HashMap::new(),
        }
    }

    pub fn set_dom_window(&mut self, new_node: Node<Window>) {
        self.document.set_root(new_node);
    }

    pub fn resize(&mut self, size: PhysicalSize, dpi_scale: f32) {
        self.client_size = size2(size.width as i32, size.height as i32);
        self.dpi_scale = dpi_scale;
        self.document.set_size(
            self.client_size.to_f32() / Scale::<f32, LogicalPixel, DevicePixel>::new(dpi_scale),
        );
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
        layout: &EqualRc<LayoutTreeNode>,
    ) {
        let rect = Rect::new(position, layout.size) * Scale::new(1.0);

        let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

        match layout.render {
            RenderData::Node(ref node) => {
                let values = node.computed_values().get().unwrap();

                if values.border_color.alpha > 0
                    && values.border_thickness != LogicalSideOffsets::zero()
                {
                    let common = CommonItemProperties::new(rect, space_and_clip);
                    let side = BorderSide {
                        style: BorderStyle::Solid,
                        color: values.border_color.into(),
                    };
                    builder.push_border(
                        &common,
                        rect,
                        convert_offsets(values.border_thickness),
                        BorderDetails::Normal(NormalBorder {
                            left: side,
                            right: side,
                            top: side,
                            bottom: side,
                            radius: BorderRadius::uniform(values.border_radius.get()),
                            do_aa: true,
                        }),
                    )
                }

                if values.background_color.alpha > 0 || node.interactive() {
                    let clip_rect = rect.inner_rect(convert_offsets(values.border_thickness));
                    let clip_id = if values.border_radius.get() > 0.0 {
                        let region = ComplexClipRegion::new(
                            rect,
                            BorderRadius::uniform(values.border_radius.get()),
                            ClipMode::Clip,
                        );
                        builder.define_clip(
                            &SpaceAndClipInfo::root_scroll(pipeline_id),
                            rect,
                            vec![region],
                            None,
                        )
                    } else {
                        ClipId::root(pipeline_id)
                    };
                    let item_props = CommonItemProperties {
                        clip_rect,
                        clip_id,
                        spatial_id: SpatialId::root_scroll_node(pipeline_id),
                        hit_info: Some((node.id(), 0)),
                        flags: PrimitiveFlags::empty(),
                    };
                    builder.push_rect(&item_props, values.background_color.into());
                }

                for layout in &layout.children {
                    self.render_child(
                        pipeline_id,
                        builder,
                        transaction,
                        position + layout.position.to_vector(),
                        &layout.layout,
                    );
                }
            }
            RenderData::Text {
                text:
                    LayoutText {
                        ref fragments,
                        size,
                    },
                ref parent,
            } => {
                let values = parent.computed_values().get().unwrap();
                let color = values.text_color;
                builder.push_simple_stacking_context(
                    point2(0.0, 0.0),
                    space_and_clip.spatial_id,
                    PrimitiveFlags::IS_BACKFACE_VISIBLE,
                );
                for fragment in fragments {
                    let glyphs = fragment
                        .glyphs
                        .iter()
                        .map(|glyph| {
                            let pos = position + glyph.offset.to_vector();
                            GlyphInstance {
                                index: glyph.index,
                                point: pos * Scale::new(1.0),
                            }
                        })
                        .collect::<Vec<_>>();
                    let font_key = self.get_font(&fragment.font, transaction);
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
    }

    pub fn render(&mut self) {
        let client_size = self.client_size;
        let dpi_scale = Scale::new(self.dpi_scale);
        let content_size = client_size.to_f32() / dpi_scale;

        println!("render()");
        let pipeline_id = PipelineId(0, 0);
        let mut builder = DisplayListBuilder::new(pipeline_id, content_size);
        let mut transaction = Transaction::new();

        let root_layout = self.document.get_layout();

        for layout in &root_layout.children {
            self.render_child(
                pipeline_id,
                &mut builder,
                &mut transaction,
                layout.position,
                &layout.layout,
            );
        }

        transaction.set_display_list(Epoch(0), None, content_size, builder.finalize(), true);
        transaction.set_root_pipeline(pipeline_id);
        transaction.generate_frame();
        self.api.set_document_view(
            self.document_id,
            DeviceIntRect::new(Point2D::zero(), client_size.to_i32()),
            dpi_scale.get(),
        );
        self.api.send_transaction(self.document_id, transaction);
        self.rx.recv().unwrap();
        self.renderer.update();
        let _ = self.renderer.render(client_size.to_i32());
        let _ = self.renderer.flush_pipeline_info();
    }

    pub fn element_at(&mut self, position: LogicalPosition) -> Option<u64> {
        self.api
            .hit_test(
                self.document_id,
                None,
                point2(position.x as f32, position.y as f32),
                HitTestFlags::empty(),
            )
            .items
            .first()
            .map(|item| item.tag.0)
    }
}
