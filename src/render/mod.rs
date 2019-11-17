//! This module handles creating the paint tree, as well as rendering it
//! and processing user input queries against it.

pub mod context;
pub mod engine;

pub use context::Context;
pub use engine::{PaintDetails, PaintTreeNode, RenderEngine};
