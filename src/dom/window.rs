use super::Element;

#[derive(Default)]
pub struct Window {}

impl Window {
    pub fn on<Event>(&mut self, func: impl FnMut(&Event) + 'static)
    where
        Event: WindowEvent,
    {
    }
}

impl Into<Element> for Window {
    fn into(self) -> Element {
        Element::Window(self)
    }
}

pub trait WindowEvent {}
