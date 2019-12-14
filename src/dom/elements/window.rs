use crate::dom::element::Element;
use crate::dom::{AttrStyle, AttrTitle};
use crate::style::Style;

/// Corresponds to <window>. This is the top-level container for UI and
/// corresponds to an OS window.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    style: Option<Style>,
    pub title: String,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            style: None,
            title: "Untitled Window".to_owned(),
        }
    }
}

element_attributes! {
    Window {
        style: AttrStyle,
        title: AttrTitle,
    }
}

impl Element for Window {
    type Child = super::BlockChild;
    type Handlers = ();
    type States = ();

    const ELEMENT_NAME: &'static str = "window";

    fn style(&self) -> Option<Style> {
        self.style
    }

    fn attributes(&self) -> Vec<(&'static str, String)> {
        vec![("title", format!("{:?}", self.title))]
    }
}
