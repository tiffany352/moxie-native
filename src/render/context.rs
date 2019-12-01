use crate::dom::input::InputEvent;
use crate::dom::{element::DynamicNode, node::NodeRef, Node, Window};
use crate::layout::{LayoutEngine, LayoutText, LayoutTreeNode, LogicalPixel};
use crate::style::{ComputedValues, StyleEngine};
use crate::util::equal_rc::EqualRc;
use gleam::gl;
use skribo::FontRef;
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
    euclid::{point2, size2, Point2D, Rect, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window as WinitWindow};

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
        node: DynamicNode,
        layout: &EqualRc<LayoutTreeNode>,
        parent_values: ComputedValues,
    ) {
        let rect = Rect::new(position, layout.size) * Scale::new(1.0);

        let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

        let values = if let DynamicNode::Node(node) = node {
            node.computed_values().get()
        } else {
            None
        };

        if let Some(values) = values {
            if values.background_color.alpha > 0 {
                let item_props = if values.border_radius.get() > 0.0 {
                    let region = ComplexClipRegion::new(
                        rect,
                        BorderRadius::uniform(values.border_radius.get()),
                        ClipMode::Clip,
                    );
                    let clip = builder.define_clip(
                        &SpaceAndClipInfo::root_scroll(pipeline_id),
                        rect,
                        vec![region],
                        None,
                    );
                    CommonItemProperties::new(
                        rect,
                        SpaceAndClipInfo {
                            spatial_id: SpatialId::root_scroll_node(pipeline_id),
                            clip_id: clip,
                        },
                    )
                } else {
                    CommonItemProperties::new(rect, space_and_clip)
                };
                builder.push_rect(&item_props, values.background_color.into());
            }
        }

        if let Some(LayoutText {
            ref fragments,
            size,
        }) = layout.render_text
        {
            let values = values.as_ref().unwrap_or(&parent_values);
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

        if let DynamicNode::Node(node) = node {
            for layout in &layout.children {
                let child = node.get_child(layout.index).expect("child to exist");
                self.render_child(
                    pipeline_id,
                    builder,
                    transaction,
                    position + layout.position.to_vector(),
                    child,
                    &layout.layout,
                    node.computed_values().get().unwrap(),
                );
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

        self.style_engine
            .update(self.window.clone(), content_size * Scale::new(1.0));

        let root_layout = self
            .layout_engine
            .layout(self.window.clone(), content_size * Scale::new(1.0));

        for layout in &root_layout.children {
            let child = self.window.children()[layout.index].clone();
            self.render_child(
                pipeline_id,
                &mut builder,
                &mut transaction,
                layout.position,
                DynamicNode::Node((&child).into()),
                &layout.layout,
                self.window.computed_values().get().unwrap(),
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
        &self,
        event: &InputEvent,
        position: Point2D<f32, LogicalPixel>,
        node: NodeRef,
        layout: &EqualRc<LayoutTreeNode>,
    ) -> bool {
        let rect = Rect::new(position, layout.size);

        for layout in &layout.children {
            let child = node.get_child(layout.index).expect("child to exist");
            if let DynamicNode::Node(node) = child {
                if self.process_child(
                    event,
                    position + layout.position.to_vector(),
                    node,
                    &layout.layout,
                ) {
                    return true;
                }
            }
        }

        let do_process = match event.get_position() {
            Some((x, y)) => rect.contains(point2(x, y)),
            None => true,
        };

        if do_process {
            if node.process(event) {
                return true;
            }
        }

        false
    }

    pub fn process(&mut self, event: &InputEvent) -> bool {
        let client_size = self.client_size;
        let dpi_scale = Scale::new(self.dpi_scale);
        let content_size: Size2D<f32, LayoutPixel> = client_size.to_f32() / dpi_scale;

        self.style_engine
            .update(self.window.clone(), content_size * Scale::new(1.0));

        let root_layout = self
            .layout_engine
            .layout(self.window.clone(), content_size * Scale::new(1.0));

        for layout in &root_layout.children {
            let child = &self.window.children()[layout.index];
            if self.process_child(event, layout.position, child.into(), &layout.layout) {
                return true;
            }
        }

        false
    }
}
