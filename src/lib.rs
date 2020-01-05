//! This is a framework for building GUI applications, in a similar vein
//! to React Native. It lets you write application UI code in native
//! rust, using the mox! macro to cleanly express a declarative DOM. The
//! UI runtime is entirely native, using webrender for rendering with a
//! custom layout engine.
//!
//! See the `dom` module for an API reference of the DOM interface.
//! `moxie` contains information about the mox! macro used for declaring
//! UI.
//!
//! To start up your application, define a component like so:
//! ```rs
//! use moxie_native::prelude::*;
//!
//! #[topo::nested]
//! fn app() -> Vec<Node<Window>> {
//!     vec![mox! {
//!         <window>
//!             // Your application logic
//!         </window>
//!     }]
//! }
//! ```
//!
//! Then, in your main function, create a `Runtime` and start it, like this:
//!
//! ```rs
//! let runtime = moxie_native::Runtime::new(|| app!());
//! runtime.start();
//! ```

#[proc_macro_hack::proc_macro_hack(support_nested)]
pub use moxie_native_mox::mox;
pub use moxie_native_style::define_style;

mod document;
pub mod dom;
mod layout;
#[doc(hidden)]
pub mod moxie;
#[doc(hidden)]
pub mod style_impl;
pub mod prelude;
mod render;
mod runtime;
pub mod style;
mod util;

pub use runtime::Runtime;
pub use topo;
pub use util::color::Color;
