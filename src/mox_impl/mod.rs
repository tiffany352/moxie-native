//! This module implements compatibility with the mox! macro for
//! declaring UI.
//!
//! Each of the macros declared by this module can be referenced in the mox macro like so:
//!
//! ```rs
//! mox! {
//!     <view color="255,255,255">
//!         Hello, moxie-native
//!     </view>
//! }
//! ```

use crate::dom::element::Element;

pub mod attr;
mod builder;
pub mod elt;
pub mod event;

pub fn fragment<Elt>() -> builder::Fragment<Elt>
where
    Elt: Element,
{
    builder::Fragment::new()
}
