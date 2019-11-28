use crate::style::Style;

macro_rules! attribute {
    ($name:ident, $value:ty) => {
        #[derive(Default)]
        pub struct $name;

        impl crate::dom::element::Attribute for $name {
            type Value = $value;
        }
    };
}

attribute!(AttrStyle, Option<Style>);
attribute!(AttrTitle, String);
