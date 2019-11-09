pub mod element;
pub mod storage;
pub mod view;
pub mod window;

pub use element::Element;
pub use storage::{DomStorage, NodeOrText};

slotmap::new_key_type! {
    pub struct Node;
}
