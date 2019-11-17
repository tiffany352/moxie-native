//! This module defines the main part of how applications interact with
//! moxie-native. It implements the DOM hierarchy which is used to
//! represent the UI.

pub mod button;
pub mod element;
pub mod event_handler;
pub mod events;
pub mod node;
pub mod span;
pub mod view;
pub mod window;

pub use button::Button;
pub use element::{CanSetEvent, Element, Event, NodeChild};
pub use event_handler::EventHandler;
pub use events::ClickEvent;
pub use node::Node;
pub use span::Span;
pub use view::View;
pub use window::Window;

/// Defines an enum over multiple types that implement NodeChild and
/// then implements NodeChild for that enum.
#[macro_export]
macro_rules! multiple_children {
    (enum $name:ident { $( $var_name:ident ( $var_ty:ty ) ),+ $(,)* }) => {
        #[derive(Clone, PartialEq)]
        pub enum $name {
            $(
                $var_name ($var_ty)
            ),+
        }

        $(
            impl From<$var_ty> for $name {
                fn from(elt: $var_ty) -> Self {
                    $name::$var_name(elt)
                }
            }
        )+

        impl $crate::dom::NodeChild for $name {
            fn paint(&self) -> Option<$crate::render::PaintDetails> {
                match self {
                    $(
                        $name::$var_name(elt) => elt.paint()
                    ),+
                }
            }

            fn create_layout_opts(&self, parent_opts: &$crate::layout::LayoutOptions) -> $crate::layout::LayoutOptions {
                match self {
                    $(
                        $name::$var_name(elt) => elt.create_layout_opts(parent_opts)
                    ),+
                }
            }

            fn get_child(&self, child: usize) -> Option<&dyn $crate::dom::NodeChild> {
                match self {
                    $(
                        $name::$var_name(elt) => elt.get_child(child)
                    ),+
                }
            }
        }
    }
}
