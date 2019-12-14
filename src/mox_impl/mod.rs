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

pub mod attributes;
pub mod elements;
pub mod builder;

/// Text node
pub fn text(s: impl ToString) -> String {
    s.to_string()
}
