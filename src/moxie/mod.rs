pub use moxie::*;
pub mod elements;

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
