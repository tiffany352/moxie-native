use crate::direct_composition::{AngleVisual, DirectComposition};
use crate::dom::{DomStorage, Element, Node, NodeOrText};
use crate::layout::{LogicalPixel, LogicalSize};
use std::sync::mpsc;
use webrender::{
    api::{
        units::DeviceIntRect, units::DevicePixel, ColorF, DisplayListBuilder, DocumentId, Epoch,
        PipelineId, RenderApi, RenderNotifier, Transaction,
    },
    euclid::{Point2D, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{Window as WinitWindow, WindowBuilder, WindowId},
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

pub struct Window {
    winit_window: WinitWindow,
    composition: DirectComposition,
    visual: Option<AngleVisual>,
    api: RenderApi,
    document: DocumentId,
    rx: mpsc::Receiver<()>,
    renderer: Renderer,
    dom_window: Node,
}

impl Window {
    pub fn new(dom_window: Node, storage: &mut DomStorage, event_loop: &EventLoop<()>) -> Window {
        let (tx, rx) = mpsc::channel();
        let notifier = Box::new(Notifier {
            events_proxy: event_loop.create_proxy(),
            tx,
        });

        let winit_window = WindowBuilder::new()
            .with_title("UI Lib")
            .with_decorations(true)
            .with_transparent(true)
            .with_no_redirection_bitmap(true)
            .build(event_loop)
            .unwrap();

        let factor = winit_window.hidpi_factor() as f32;
        let inner_size = winit_window.inner_size().to_physical(factor as f64);
        let size =
            Size2D::<i32, DevicePixel>::new(inner_size.width as i32, inner_size.height as i32);

        let composition = unsafe { DirectComposition::new(winit_window.hwnd() as _) };
        let visual = composition.create_angle_visual(size.width as u32, size.height as u32);
        visual.make_current();
        let (renderer, sender) = Renderer::new(
            composition.gleam.clone(),
            notifier.clone(),
            RendererOptions {
                clear_color: Some(ColorF::new(1.0, 1.0, 1.0, 1.0)),
                device_pixel_ratio: factor,
                ..Default::default()
            },
            None,
            size,
        )
        .unwrap();
        let api = sender.create_api();
        let document = api.add_document(size, 0);

        let mut window = Window {
            winit_window,
            composition,
            visual: Some(visual),
            api,
            document,
            rx,
            renderer,
            dom_window,
        };

        let factor = window.winit_window.hidpi_factor() as f32;
        let inner_size = window.winit_window.inner_size().to_physical(factor as f64);
        window.render(storage, inner_size);
        window
    }

    pub fn window_id(&self) -> WindowId {
        self.winit_window.id()
    }

    fn layout_child(
        &mut self,
        storage: &mut DomStorage,
        node: Node,
        parent_max_size: LogicalSize,
    ) -> Option<LogicalSize> {
        let max_size = {
            let element = storage.get_element_mut(node);
            if let Element::View(view) = element {
                let opts = view.create_layout_opts();
                let max_size = view.layout_mut().calc_max_size(&opts, parent_max_size);
                max_size
            } else {
                return None;
            }
        };

        let children: Vec<NodeOrText> = storage.get_children(node).to_vec();
        let mut child_sizes = vec![];
        child_sizes.reserve(children.len());
        for child in children {
            if let NodeOrText::Node(node) = child {
                if let Some(size) = self.layout_child(storage, node, max_size) {
                    child_sizes.push(size);
                }
            }
        }

        {
            let element = storage.get_element_mut(node);
            if let Element::View(view) = element {
                let opts = view.create_layout_opts();
                let min_size = view.layout_mut().calc_min_size(&opts, &child_sizes[..]);
                Some(min_size)
            } else {
                return None;
            }
        }
    }

    fn render_child(
        &mut self,
        storage: &mut DomStorage,
        pipeline_id: PipelineId,
        builder: &mut DisplayListBuilder,
        position: Point2D<f32, LogicalPixel>,
        node: Node,
    ) {
        let element = storage.get_element(node);
        match element {
            Element::View(view) => {
                view.draw(position, Scale::new(1.0), builder, pipeline_id);

                let children: Vec<NodeOrText> = storage.get_children(node).to_vec();
                let child_positions = view.layout().child_positions().to_vec();
                let mut i = 0;
                for child in children {
                    if let NodeOrText::Node(node) = child {
                        self.render_child(
                            storage,
                            pipeline_id,
                            builder,
                            position + child_positions[i].to_vector(),
                            node,
                        );
                        i += 1;
                    }
                }
            }
            _ => (),
        }
    }

    pub fn render(&mut self, storage: &mut DomStorage, inner_size: PhysicalSize) {
        println!("render()");
        let factor = self.winit_window.hidpi_factor() as f32;
        let size =
            Size2D::<i32, DevicePixel>::new(inner_size.width as i32, inner_size.height as i32);
        self.visual.as_mut().unwrap().make_current();
        let pipeline_id = PipelineId(0, 0);
        let layout_size = size.to_f32() / Scale::new(factor);
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);

        let children: Vec<NodeOrText> = storage.get_children(self.dom_window).to_vec();
        for child in children {
            if let NodeOrText::Node(node) = child {
                self.layout_child(storage, node, layout_size * Scale::new(1.0));

                self.render_child(
                    storage,
                    pipeline_id,
                    &mut builder,
                    Point2D::new(0.0, 0.0),
                    node,
                );
            }
        }

        let mut transaction = Transaction::new();
        transaction.set_display_list(Epoch(0), None, layout_size, builder.finalize(), true);
        transaction.set_root_pipeline(pipeline_id);
        transaction.generate_frame();
        self.api.set_document_view(
            self.document,
            DeviceIntRect::new(Point2D::zero(), size),
            factor,
        );
        self.api.send_transaction(self.document, transaction);
        self.rx.recv().unwrap();
        self.renderer.update();
        let _ = self.renderer.render(size);
        let _ = self.renderer.flush_pipeline_info();
        self.visual.as_mut().unwrap().present();
        self.composition.commit();
    }

    pub fn process(&mut self, storage: &mut DomStorage, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let factor = self.winit_window.hidpi_factor() as f32;
                let inner_size = self.winit_window.inner_size().to_physical(factor as f64);
                self.render(storage, inner_size);
            }
            WindowEvent::Resized(size) => {
                println!("resize {}x{}", size.width, size.height);
                if let Some(visual) = self.visual.take() {
                    self.composition.cleanup_angle_visual(visual);
                }
                if size.width > 0.0 && size.height > 0.0 {
                    let factor = self.winit_window.hidpi_factor() as f32;
                    let inner_size = size.to_physical(factor as f64);
                    self.visual = Some(
                        self.composition
                            .create_angle_visual(inner_size.width as u32, inner_size.height as u32),
                    );
                    self.render(storage, inner_size);
                }
            }
            _ => (),
        }
    }
}
