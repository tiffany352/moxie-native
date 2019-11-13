use super::{Element, ElementType};

#[derive(Default)]
pub struct Window {}

impl Window {
    pub fn on<Event>(&mut self, _func: impl FnMut(&Event) + 'static)
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

impl ElementType for Window {
    fn from_element(elt: &Element) -> Option<&Self> {
        match elt {
            Element::Window(window) => Some(window),
            _ => None,
        }
    }
}
