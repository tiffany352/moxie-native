use super::builder::Builder;
use crate::dom::*;

/// The root of the DOM.
pub fn app() -> Builder<App> {
    Builder::new()
}

/// Top level window.
pub fn window() -> Builder<Window> {
    Builder::new()
}

/// Basic flow container.
pub fn view() -> Builder<View> {
    Builder::new()
}

/// An interactible button.
pub fn button() -> Builder<Button> {
    Builder::new()
}

/// Container for inline text.
pub fn span() -> Builder<Span> {
    Builder::new()
}
