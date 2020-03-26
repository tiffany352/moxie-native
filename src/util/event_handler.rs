use std::sync::{Arc, Mutex};

struct HandlerInner<Func> {
    func: Mutex<Func>,
}

trait Handler<Event>: Sync + Send {
    fn invoke(&self, event: &Event);
}

impl<Event, Func> Handler<Event> for HandlerInner<Func>
where
    Func: FnMut(&Event) + 'static + Sync + Send,
{
    fn invoke(&self, event: &Event) {
        (&mut *self.func.lock().unwrap())(event);
    }
}

/// Represents the callback attached to an event. Allows the handler to
/// be more easily passed around and invoked than directly storing a
/// boxed FnMut.
pub struct EventHandler<Event>(Option<Arc<dyn Handler<Event>>>);

impl<Event> EventHandler<Event> {
    pub fn new() -> EventHandler<Event> {
        EventHandler(None)
    }

    pub fn with_func(func: impl FnMut(&Event) + 'static + Sync + Send) -> EventHandler<Event> {
        EventHandler(Some(Arc::new(HandlerInner {
            func: Mutex::new(func),
        })))
    }

    pub fn invoke(&self, event: &Event) {
        if let Some(ref handler) = self.0 {
            handler.invoke(event);
        }
    }

    pub fn present(&self) -> bool {
        self.0.is_some()
    }
}

impl<Event> Default for EventHandler<Event> {
    fn default() -> EventHandler<Event> {
        EventHandler::new()
    }
}

impl<Event> Clone for EventHandler<Event> {
    fn clone(&self) -> EventHandler<Event> {
        EventHandler(self.0.clone())
    }
}
