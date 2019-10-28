macro_rules! element_macro {
    (
        $(#[$outer:meta])*
        $name:ident
    ) => {
        $(#[$outer])*
        #[macro_export]
        macro_rules! $name {
            ($with_elem:expr) => {
                $crate::element!(stringify!($name), $with_elem)
            };
        }
    };
}

element_macro! {
    /// A top-level window
    window
}

element_macro! {
    /// A generic layout container
    view
}
