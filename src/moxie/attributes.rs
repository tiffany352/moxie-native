use crate::dom::attributes::*;

macro_rules! attribute {
    ($name:ident -> $class:ty) => {
        pub fn $name() -> $class {
            Default::default()
        }
    };
}

attribute!(attr_class_name -> AttrClassName);
attribute!(attr_padding -> AttrPadding);
attribute!(attr_text_size -> AttrTextSize);
attribute!(attr_color -> AttrColor);
attribute!(attr_width -> AttrWidth);
attribute!(attr_height -> AttrHeight);
