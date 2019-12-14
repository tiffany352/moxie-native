use crate::dom::{App, Window, View, Button, Span};
use crate::dom::node::Node;
use super::builder::Builder;

macro_rules! define_element {
    (
        $( #[$outer:meta] )*
        $name:ident($class:ident)
    ) => {
        $( #[$outer] )+
        pub fn $name(with_elem: impl FnOnce(Builder<$class>) -> Node<$class>) -> Node<$class> {
            Builder::create(with_elem)
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
