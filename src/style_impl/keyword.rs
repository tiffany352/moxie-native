macro_rules! keyword {
    ($name:ident : $class:ident) => {
        pub struct $class;

        pub fn $name() -> $class {
            $class
        }
    };
}

keyword!(block: Block);
keyword!(inline: Inline);
keyword!(horizontal: Horizontal);
keyword!(vertical: Vertical);
keyword!(solid: Solid);
