use crate::style::{BorderStyle, Direction, Display};

macro_rules! keyword {
    ($name:ident : $class:ident => $enum:ty as $variant:ident) => {
        pub struct $class;

        pub fn $name() -> $class {
            $class
        }

        impl Into<$enum> for $class {
            fn into(self) -> $enum {
                <$enum>::$variant
            }
        }
    };
}

keyword!(block: Block => Display as Block);
keyword!(inline: Inline => Display as Inline);
keyword!(horizontal: Horizontal => Direction as Horizontal);
keyword!(vertical: Vertical => Direction as Vertical);
keyword!(none: None => BorderStyle as None);
keyword!(solid: Solid => BorderStyle as Solid);
keyword!(double: Double => BorderStyle as Double);
keyword!(dotted: Dotted => BorderStyle as Dotted);
keyword!(dashed: Dashed => BorderStyle as Dashed);
keyword!(hidden: Hidden => BorderStyle as Hidden);
keyword!(groove: Groove => BorderStyle as Groove);
keyword!(ridge: Ridge => BorderStyle as Ridge);
keyword!(inset: Inset => BorderStyle as Inset);
keyword!(outset: Outset => BorderStyle as Outset);
