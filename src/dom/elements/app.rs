use crate::dom::element::Element;
use crate::dom::{Node, Window};
use crate::style::Style;

/// Corresponds to <app>. This is the root of the DOM and contains
/// windows.
#[derive(Clone, Debug, PartialEq)]
pub struct App {}

impl Default for App {
    fn default() -> Self {
        App {}
    }
}

impl Element for App {
    type Child = Node<Window>;
    type Handlers = ();

    const ELEMENT_NAME: &'static str = "app";

    fn style(&self) -> Option<Style> {
        None
    }
}
