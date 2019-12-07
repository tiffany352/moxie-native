use crate::dom::devtools::DevToolsRegistry;
use crate::dom::{element::Element, element::HandlerList, App, Node};
use crate::window_runtime::{Message as WindowMessage, WindowRuntimeNotifier};
use futures::task::ArcWake;
use moxie::embed::Runtime as MoxieRuntime;
use parking_lot::{Condvar, Mutex};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

pub enum Message {
    InvokeHandler {
        node_id: usize,
        payload: Box<dyn Any + Send>,
    },
}

pub struct RuntimeWaker {
    var: Condvar,
    messages: Mutex<Vec<Message>>,
}

impl ArcWake for RuntimeWaker {
    fn wake_by_ref(arc: &Arc<Self>) {
        arc.var.notify_all();
    }

    fn wake(self: Arc<Self>) {
        self.var.notify_all();
    }
}

impl RuntimeWaker {
    pub fn new() -> Arc<RuntimeWaker> {
        Arc::new(RuntimeWaker {
            var: Condvar::new(),
            messages: Mutex::new(vec![]),
        })
    }

    pub fn send_message(&self, message: Message) {
        self.messages.lock().push(message);
        self.var.notify_all();
    }
}

trait AnyHandlers {
    fn invoke(&self, message: Box<dyn Any + Send>);
}

struct Handlers<Elt>
where
    Elt: Element,
{
    handlers: Elt::Handlers,
}

impl<Elt> AnyHandlers for Handlers<Elt>
where
    Elt: Element,
{
    fn invoke(&self, message: Box<dyn Any + Send>) {
        let message = message
            .downcast::<<Elt::Handlers as HandlerList>::Message>()
            .unwrap();
        self.handlers.handle_message(*message);
    }
}

pub struct HandlersStorage {
    map: RefCell<HashMap<usize, Box<dyn AnyHandlers>>>,
}

impl fmt::Debug for HandlersStorage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "HandlersStorage {{ ... }}")
    }
}

impl HandlersStorage {
    fn new() -> HandlersStorage {
        HandlersStorage {
            map: RefCell::new(HashMap::new()),
        }
    }

    pub fn set_handlers<Elt>(&self, id: usize, handlers: Elt::Handlers)
    where
        Elt: Element,
    {
        self.map
            .borrow_mut()
            .insert(id, Box::new(Handlers::<Elt> { handlers }));
    }

    pub fn invoke(&self, id: usize, payload: Box<dyn Any + Send>) {
        self.map.borrow_mut().get(&id).unwrap().invoke(payload);
    }
}

type RootFunc = Box<dyn FnMut() -> Node<App> + 'static>;

/// Contains the event loop and the root component of the application.
pub struct Runtime {
    moxie_runtime: MoxieRuntime<RootFunc>,
    handlers: Rc<HandlersStorage>,
    waker: Arc<RuntimeWaker>,
}

impl Runtime {
    pub fn with_waker(
        waker: Arc<RuntimeWaker>,
        mut root: impl FnMut() -> Node<App> + 'static,
    ) -> Runtime {
        let handlers = Rc::new(HandlersStorage::new());
        let handlers2 = handlers.clone();
        let mut moxie_runtime: MoxieRuntime<RootFunc> = MoxieRuntime::new(Box::new(move || {
            illicit::child_env!(
                DevToolsRegistry => DevToolsRegistry::new(),
                Rc<HandlersStorage> => handlers2.clone()
            )
            .enter(|| {
                topo::call!({
                    let registry = illicit::Env::expect::<DevToolsRegistry>();
                    let app = root();
                    registry.update(app.clone().into());
                    app
                })
            })
        }));
        moxie_runtime.set_state_change_waker(futures::task::waker(waker.clone()));
        Runtime {
            moxie_runtime,
            handlers,
            waker,
        }
    }

    /// Create a new runtime based on the application's root component.
    pub fn new(root: impl FnMut() -> Node<App> + 'static) -> Runtime {
        Self::with_waker(RuntimeWaker::new(), root)
    }

    pub fn waker(&self) -> Arc<RuntimeWaker> {
        self.waker.clone()
    }

    /// Start up the application.
    pub fn start(mut self, notifier: WindowRuntimeNotifier) {
        let mut messages = self.waker.messages.lock();

        loop {
            for message in messages.drain(..) {
                match message {
                    Message::InvokeHandler { node_id, payload } => {
                        self.handlers.invoke(node_id, payload);
                    }
                }
            }

            let app = self.moxie_runtime.run_once();
            notifier.send_message(WindowMessage::UpdateDom(app));

            self.waker.var.wait(&mut messages);
        }
    }
}
