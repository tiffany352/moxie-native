use std::cell::{RefCell, RefMut};
use std::rc::Rc;

struct HandlerInner<Func> {
    func: RefCell<Func>,
}

trait Handler<Event> {
    fn invoke(&self, event: &Event);
}

impl<Event, Func> Handler<Event> for HandlerInner<Func>
where
    Func: FnMut(&Event) + 'static,
{
    fn invoke(&self, event: &Event) {
        RefMut::map(self.func.borrow_mut(), |func| {
            (*func)(event);
            func
        });
    }
}

/// Represents the callback attached to an event. Allows the handler to
/// be more easily passed around and invoked than directly storing a
/// boxed FnMut.
pub struct EventHandler<Event>(Option<Rc<dyn Handler<Event>>>);

impl<Event> EventHandler<Event> {
    pub fn new() -> EventHandler<Event> {
        EventHandler(None)
    }

    pub fn with_func(func: impl FnMut(&Event) + 'static) -> EventHandler<Event> {
        EventHandler(Some(Rc::new(HandlerInner {
            func: RefCell::new(func),
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

impl<Event> PartialEq for EventHandler<Event> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(ref left), Some(ref right)) => Rc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}
