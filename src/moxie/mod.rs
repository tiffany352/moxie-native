pub use moxie::*;

use moxie;

pub mod elements;

#[topo::nested]
pub fn text(_s: impl ToString) {}
