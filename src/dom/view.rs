use super::Element;
use std::borrow::Cow;

#[derive(Default)]
pub struct View {
    class_name: Option<Cow<'static, str>>,
}

impl View {
    pub fn new() -> View {
        View { class_name: None }
    }

    pub fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match key {
            "className" => self.class_name = value,
            _ => (),
        }
    }
}

impl Into<Element> for View {
    fn into(self) -> Element {
        Element::View(self)
    }
}
