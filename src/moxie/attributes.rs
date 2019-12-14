use crate::dom::attributes::*;

macro_rules! attribute {
    ($name:ident -> $class:ty) => {
        pub fn $name() -> $class {
            Default::default()
        }
    };
}

attribute!(attr_style -> AttrStyle);
attribute!(attr_title -> AttrTitle);
attribute!(attr_text_state -> AttrTextState);
