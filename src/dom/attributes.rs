use crate::style::Style;
use std::borrow::Cow;

macro_rules! attribute {
    ($name:ident, $value:ty) => {
        #[derive(Default)]
        pub struct $name;

        impl crate::dom::element::Attribute for $name {
            type Value = $value;
        }
    };
}

attribute!(AttrStyles, Cow<'static, [&'static Style]>);
attribute!(AttrClassName, Cow<'static, str>);
attribute!(AttrTitle, String);
