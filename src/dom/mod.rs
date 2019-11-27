//! This module defines the main part of how applications interact with
//! moxie-native. It implements the DOM hierarchy which is used to
//! represent the UI.

pub mod attributes;
pub mod element;
pub mod elements;
pub mod events;
pub mod node;

pub use attributes::*;
pub use elements::{button::Button, span::Span, view::View, window::Window};
pub use events::*;
pub use node::Node;
