use crate::Color;
use std::borrow::Cow;

macro_rules! attribute {
    ($name:ident, $value:ty) => {
        #[derive(Default)]
        pub struct $name;

        impl crate::dom::Attribute for $name {
            type Value = $value;
        }
    };
}

attribute!(AttrClassName, Cow<'static, str>);
attribute!(AttrPadding, f32);
attribute!(AttrTextSize, f32);
attribute!(AttrColor, Color);
attribute!(AttrWidth, f32);
attribute!(AttrHeight, f32);
