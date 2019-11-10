use super::Element;

#[derive(Default)]
pub struct Window {}

impl Into<Element> for Window {
    fn into(self) -> Element {
        Element::Window(self)
    }
}

pub trait WindowEvent {}

pub struct TestEvent;

impl WindowEvent for TestEvent {}
