pub mod app;
pub use app::App;

use direct_composition::DirectComposition;
use std::sync::mpsc;
use webrender::{
    api::{
        units::DevicePixel, BorderRadius, ClipMode, ColorF, CommonItemProperties,
        ComplexClipRegion, DisplayListBuilder, DocumentId, Epoch, PipelineId, RenderNotifier,
        SpaceAndClipInfo, SpatialId, Transaction,
    },
    euclid::{Point2D, Rect, Scale, Size2D},
    Renderer, RendererOptions,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::WindowBuilder,
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

pub fn start<T>(app: T)
where
    T: App,
{
    let event_loop = EventLoop::new();
    let (tx, rx) = mpsc::channel();
    let notifier = Box::new(Notifier {
        events_proxy: event_loop.create_proxy(),
        tx,
    });
    let window = WindowBuilder::new()
        .with_title("UI Lib")
        .with_decorations(true)
        .with_transparent(true)
        .with_no_redirection_bitmap(true)
        .build(&event_loop)
        .unwrap();
    let hwnd = window.hwnd();
    println!("hwnd {:?}", hwnd);
    let composition = unsafe { DirectComposition::new(hwnd as _) };
    let factor = window.hidpi_factor() as f32;
    let size = Size2D::<i32, DevicePixel>::new(
        window.inner_size().width as i32,
        window.inner_size().height as i32,
    );
    println!("size {} factor {}", size, factor);
    let visual = composition.create_angle_visual(size.width as u32, size.height as u32);
    visual.make_current();
    let (mut renderer, sender) = Renderer::new(
        composition.gleam.clone(),
        notifier.clone(),
        RendererOptions {
            clear_color: Some(ColorF::new(0.2, 0.7, 0.8, 1.0)),
            device_pixel_ratio: factor,
            ..Default::default()
        },
        None,
        size,
    )
    .unwrap();
    let api = sender.create_api();
    let document = api.add_document(size, 0);

    let mut render = move || {
        println!("render()");
        visual.make_current();
        let pipeline_id = PipelineId(0, 0);
        let layout_size = size.to_f32() / Scale::new(factor);
        let mut builder = DisplayListBuilder::new(pipeline_id, layout_size);
        let rect = Rect::new(Point2D::zero(), layout_size);
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
        transaction.set_display_list(
            Epoch(0),
            Some(ColorF::new(0.2, 0.7, 0.8, 1.0)),
            layout_size,
            builder.finalize(),
            true,
        );
        transaction.set_root_pipeline(pipeline_id);
        transaction.generate_frame();
        api.send_transaction(document, transaction);
        rx.recv().unwrap();
        renderer.update();
        let _ = renderer.render(size);
        let _ = renderer.flush_pipeline_info();
        visual.present();
    };

    render();

    composition.commit();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::EventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //window.request_redraw();
            }
            Event::UserEvent(_) => {
                println!("event: {:?}", event);
                composition.commit();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: winit::event::MouseButton::Left,
                        ..
                    },
                ..
            } => {
                println!("event: {:?}", event);
                render();
                composition.commit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Redraw the application.
                //
                // It's preferrable to render in this event rather than in EventsCleared, since
                // rendering in here allows the program to gracefully handle redraws requested
                // by the OS.
                println!("event: {:?}", event);
                render();
                composition.commit();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("event: {:?}", event);
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            }
            _ => *control_flow = ControlFlow::Wait,
        }
    });
}
