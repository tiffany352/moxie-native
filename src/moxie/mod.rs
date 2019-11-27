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

mod attributes;
mod elements;

pub use attributes::*;
pub use elements::Builder;

/// Used by the mox! macro for free-standing text, which is then passed
/// to `Builder::add_content`.
#[doc(hidden)]
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
