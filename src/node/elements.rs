use std::fmt;

#[derive(Clone)]
pub enum Element {
    Root,
    Window,
    View,
    Unknown(String),
}

impl Element {
    pub fn from_str(name: &str) -> Element {
        match name {
            "view" => Element::View,
            "window" => Element::Window,
            _ => Element::Unknown(name.to_owned()),
        }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Element::Root => write!(fmt, "root"),
            Element::Window => write!(fmt, "window"),
            Element::View => write!(fmt, "view"),
            Element::Unknown(name) => write!(fmt, "{}", name),
        }
    }
}
