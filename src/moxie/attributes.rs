use crate::dom::attributes::*;

macro_rules! attribute {
    ($name:ident -> $class:ty) => {
        pub fn $name() -> $class {
            Default::default()
        }
    };
}

attribute!(attr_class_name -> AttrClassName);
attribute!(attr_styles -> AttrStyles);
attribute!(attr_title -> AttrTitle);
