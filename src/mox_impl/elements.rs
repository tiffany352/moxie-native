use super::builder::Builder;
use crate::dom::{App, Button, Span, View, Window};

macro_rules! define_element {
    (
        $( #[$outer:meta] )*
        $name:ident($class:ident)
    ) => {
        $( #[$outer] )+
        pub fn $name() -> Builder<$class> {
            Builder::new()
        }
    };
}

define_element! {
    /// The root of the DOM.
    app(App)
}

define_element! {
    /// Top level window.
    window(Window)
}

define_element! {
    /// Basic flow container.
    view(View)
}

define_element! {
    /// An interactible button.
    button(Button)
}

define_element! {
    /// Container for inline text.
    span(Span)
}
