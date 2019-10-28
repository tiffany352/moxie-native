use crate::direct_composition::{AngleVisual, DirectComposition};
use std::sync::mpsc;
use webrender::{
    api::{
        units::DeviceIntRect, units::DevicePixel, BorderRadius, ClipMode, ColorF,
        CommonItemProperties, ComplexClipRegion, DisplayListBuilder, DocumentId, Epoch, PipelineId,
        RenderApi, RenderNotifier, SpaceAndClipInfo, SpatialId, Transaction,
    },
    euclid::{Point2D, Rect, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{Window as WinitWindow, WindowBuilder},
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
}

impl Window {
    pub fn new(event_loop: &EventLoop<()>) -> Window {
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
        };

        let factor = window.winit_window.hidpi_factor() as f32;
        let inner_size = window.winit_window.inner_size().to_physical(factor as f64);
        window.render(inner_size);
        window
    }

    pub fn render(&mut self, inner_size: PhysicalSize) {
        println!("render()");
        let factor = self.winit_window.hidpi_factor() as f32;
        let size =
            Size2D::<i32, DevicePixel>::new(inner_size.width as i32, inner_size.height as i32);
        self.visual.as_mut().unwrap().make_current();
        let pipeline_id = PipelineId(0, 0);
        let layout_size = size.to_f32() / Scale::new(factor);
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
        let rect = Rect::new(
            Point2D::new(20.0, 20.0),
            layout_size - Size2D::new(40.0, 40.0),
        );
        let region = ComplexClipRegion::new(rect, BorderRadius::uniform(20.), ClipMode::Clip);
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
            ColorF::new(0.2, 0.7, 0.8, 1.0),
        );
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

    pub fn process(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let factor = self.winit_window.hidpi_factor() as f32;
                let inner_size = self.winit_window.inner_size().to_physical(factor as f64);
                self.render(inner_size);
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
                    self.render(inner_size);
                }
            }
            _ => (),
        }
    }
}
