use crate::dom::devtools::DevToolsRegistry;
use crate::dom::{App, Node};
use crate::util::outer_join::{outer_join, Joined};
use log::{debug, info};
use moxie::runtime::Runtime as MoxieRuntime;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    window::WindowId,
};

mod window;

/// Contains the event loop and the root component of the application.
pub struct Runtime {
    state: RuntimeState,
    windows: HashMap<WindowId, window::Window>,
    window_ids: Vec<WindowId>,
    proxy: Option<EventLoopProxy<()>>,
}

enum RuntimeState {
    Stopped {
        root_func: Box<dyn FnMut() -> Node<App> + 'static + Sync + Send>,
    },
    Running {
        sender: Sender<RuntimeEvent>,
        receiver: Receiver<MainEvent>,
    },
    Shutdown,
}

/// Events sent from the main thread to the runtime thread.
enum RuntimeEvent {
    UpdateRuntime,
    Shutdown,
}

/// Events sent from the runtime thread to the main thread.
enum MainEvent {
    UpdateRuntime(Node<App>),
    Shutdown,
}

#[derive(Debug)]
struct RuntimeMessageSender(Sender<MainEvent>);

impl Runtime {
    /// Create a new runtime based on the application's root component.
    pub fn new(mut root: impl FnMut() -> Node<App> + 'static + Sync + Send) -> Runtime {
        Runtime {
            state: RuntimeState::Stopped {
                root_func: Box::new(move || {
                    illicit::Layer::new()
                        .offer(DevToolsRegistry::new())
                        .enter(|| {
                            topo::call(|| {
                                let registry = illicit::expect::<DevToolsRegistry>();
                                let app = root();
                                registry.update(app.clone().into());
                                app
                            })
                        })
                }),
            },
            windows: HashMap::new(),
            window_ids: vec![],
            proxy: None,
        }
    }

    /// Handle events
    fn process(
        &mut self,
        event: Event<()>,
        target: &EventLoopWindowTarget<()>,
        control_flow: &mut ControlFlow,
    ) {
        let mut did_process = false;
        match event {
            Event::RedrawRequested(window_id) => {
                let window = self.windows.get_mut(&window_id).unwrap();
                window.render();
            }
            Event::WindowEvent { event, window_id } => {
                let window = self.windows.get_mut(&window_id).unwrap();
                let res = window.process(event);
                did_process = res;
            }
            _ => *control_flow = ControlFlow::Wait,
        }
        if did_process {
            self.update_runtime(target);
        }
        if let RuntimeState::Shutdown = &self.state {
            *control_flow = ControlFlow::Exit;
        }
    }

    /// Updates the moxie runtime and reconciles the DOM changes,
    /// re-rendering if things have changed.
    fn update_runtime(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        if let RuntimeState::Running {
            sender, receiver, ..
        } = &self.state
        {
            sender.send(RuntimeEvent::UpdateRuntime).unwrap();

            if let Ok(event) = receiver.recv() {
                match event {
                    MainEvent::UpdateRuntime(app) => {
                        let window_ids = self.window_ids.drain(..).collect::<Vec<_>>();
                        for joined in outer_join(app.children(), window_ids) {
                            match joined {
                                Joined::Both(dom_window, window_id) => {
                                    let window = self.windows.get_mut(&window_id).unwrap();
                                    window.set_dom_window(dom_window.clone());
                                    window.render();
                                    self.window_ids.push(window_id);
                                }
                                Joined::Left(dom_window) => {
                                    let window = window::Window::new(
                                        dom_window.clone(),
                                        event_loop,
                                        self.proxy.as_ref().unwrap().clone(),
                                    );
                                    let id = window.window_id();
                                    self.windows.insert(id, window);
                                    self.window_ids.push(id);
                                }
                                Joined::Right(window_id) => {
                                    self.windows.remove(&window_id);
                                }
                            }
                        }
                    }
                    MainEvent::Shutdown => {
                        debug!("Main thread received shutdown request");
                        sender.send(RuntimeEvent::Shutdown).unwrap();
                        self.state = RuntimeState::Shutdown;
                    }
                }
            }
        } else if let RuntimeState::Shutdown = &self.state {
        } else {
            panic!("Invalid state");
        }
    }

    /// Start up the application.
    pub fn start(mut self) {
        let Runtime {
            state,
            windows,
            window_ids,
            ..
        } = self;

        if let RuntimeState::Stopped { mut root_func } = state {
            let (sender_a, receiver_b) = channel();
            let (sender_b, receiver_a) = channel();
            let sender_c = sender_b.clone();

            let thread = std::thread::spawn(move || {
                let mut moxie_runtime = MoxieRuntime::new();

                while let Ok(event) = receiver_b.recv() {
                    match event {
                        RuntimeEvent::UpdateRuntime => {
                            let app = moxie_runtime.run_once(&mut root_func);

                            match sender_b.send(MainEvent::UpdateRuntime(app)) {
                                Ok(_) => (),
                                Err(err) => {
                                    debug!("Runtime thread send error: {}", err);
                                    break;
                                }
                            }
                        }
                        RuntimeEvent::Shutdown => {
                            debug!("Runtime thread received shutdown request");
                            break;
                        }
                    }
                }
                debug!("Runtime thread exit");
            });

            let event_loop = EventLoop::new();

            self = Runtime {
                state: RuntimeState::Running {
                    sender: sender_a,
                    receiver: receiver_a,
                },
                windows,
                window_ids,
                proxy: Some(event_loop.create_proxy()),
            };

            illicit::Layer::new()
                .offer(RuntimeMessageSender(sender_c))
                .enter(|| {
                    self.update_runtime(&event_loop);
                    event_loop.run(move |event, target, control_flow| {
                        self.process(event, target, control_flow)
                    });
                });

            debug!("Waiting for runtime thread to exit");
            // After the event loop exits (due to app shutdown), wait
            // for  the render thread to realize it's time to exit.
            thread.join().unwrap();
        } else {
            panic!("Already running");
        }
    }

    pub fn shutdown() {
        info!("Shutdown requested");
        let sender = illicit::expect::<RuntimeMessageSender>();
        sender
            .0
            .send(MainEvent::Shutdown)
            .expect("Sending shutdown request");
    }
}
