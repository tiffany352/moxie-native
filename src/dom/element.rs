use super::view::View;
use super::window::Window;
use std::borrow::Cow;

pub enum Element {
    View(View),
    Window(Window),
    Root,
}

impl Element {
    pub fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>) {
        match *self {
            Element::View(ref mut view) => view.set_attribute(key, value),
            _ => unimplemented!(),
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            Element::View(_) => "view",
            Element::Window(_) => "window",
            Element::Root => "root",
        }
    }
}

pub trait ElementType: Into<Element> {
    fn from_element(elt: &Element) -> Option<&Self>;
}
