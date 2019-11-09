macro_rules! element_macro {
    (
        $(#[$outer:meta])*
        $name:ident, $class:ident
    ) => {
        $(#[$outer])*
        #[macro_export]
        macro_rules! $name {
            ($with_elem:expr) => {
                $crate::element!(::std::marker::PhantomData::<$crate::dom::$name::$class>, $with_elem)
            };
        }
    };
}

element_macro! {
    /// A top-level window
    window, Window
}

element_macro! {
    /// A generic layout container
    view, View
}
