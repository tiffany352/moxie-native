use crate::dom::devtools::DevToolsRegistry;
use crate::dom::{App, Node};
use crate::util::outer_join::{outer_join, Joined};
use moxie::runtime::Runtime as MoxieRuntime;
use std::collections::HashMap;
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
        thread: std::thread::JoinHandle<()>,
        sender: std::sync::mpsc::Sender<RuntimeEvent>,
        receiver: std::sync::mpsc::Receiver<MainEvent>,
    },
}

/// Events sent from the main thread to the runtime thread.
enum RuntimeEvent {
    UpdateRuntime,
}

/// Events sent from the runtime thread to the main thread.
enum MainEvent {
    UpdateRuntime(Node<App>),
}

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
    }

    /// Updates the moxie runtime and reconciles the DOM changes,
    /// re-rendering if things have changed.
    fn update_runtime(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        if let RuntimeState::Running {
            sender, receiver, ..
        } = &self.state
        {
            sender.send(RuntimeEvent::UpdateRuntime).unwrap();

            while let Ok(event) = receiver.recv() {
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
                        break;
                    }
                }
            }
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
            let (sender_a, receiver_b) = std::sync::mpsc::channel();
            let (sender_b, receiver_a) = std::sync::mpsc::channel();

            let thread = std::thread::spawn(move || {
                let mut moxie_runtime = MoxieRuntime::new();

                while let Ok(event) = receiver_b.recv() {
                    match event {
                        RuntimeEvent::UpdateRuntime => {
                            let app = moxie_runtime.run_once(&mut root_func);

                            sender_b.send(MainEvent::UpdateRuntime(app)).unwrap();
                        }
                    }
                }
            });

            let event_loop = EventLoop::new();

            self = Runtime {
                state: RuntimeState::Running {
                    thread,
                    sender: sender_a,
                    receiver: receiver_a,
                },
                windows,
                window_ids,
                proxy: Some(event_loop.create_proxy()),
            };

            self.update_runtime(&event_loop);
            event_loop
                .run(move |event, target, control_flow| self.process(event, target, control_flow));
        } else {
            panic!("Already running");
        }
    }
}
