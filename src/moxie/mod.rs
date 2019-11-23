//! This module implements compatibility with the mox! macro for
//! declaring UI.
//!
//! To use it, you should import it with a glob pattern like this:
//! ```rs
//! use moxie_native::moxie::*;
//! ```
//!
//! You may also wish to glob-import `moxie_native::dom::*` for
//! convenience.
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

pub use moxie::*;
pub mod elements;
pub use elements::*;
pub mod attributes;
pub use attributes::*;

/// Used by the mox! macro for free-standing text, which is then passed
/// to `Builder::add_content`.
#[macro_export]
macro_rules! text {
    ($s:expr) => {
        $crate::moxie::text($s)
    };
}

/// Text node
pub fn text(s: impl ToString) -> String {
    s.to_string()
}
