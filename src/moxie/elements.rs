use moxie::*;

macro_rules! element_macro {
    (
        $(#[$outer:meta])*
        $name:ident, $class:ident, $handle_name:ident, $event_trait:path
    ) => {
        pub struct $handle_name($crate::dom::Node);

        impl $handle_name {
            pub fn new(node: $crate::dom::Node) -> Self {
                Self(node)
            }

            pub fn create(with_elem: impl FnOnce(Self) -> ()) {
                topo::call!(
                    {
                        let parent = &*topo::Env::expect::<$crate::dom::Node>();
                        let storage = &*topo::Env::expect::<$crate::runtime::Dom>();
                        let elem;
                        {
                            let mut storage = storage.borrow_mut();
                            elem = once!(|| storage.create_element($crate::dom::$name::$class::default()).to_inner());
                            storage.clear_children(elem);
                            storage.add_child(*parent, elem);
                        }
                        let elem = Self::new(elem);
                        with_elem(elem)
                    }
                )
            }

            pub fn attr(&self, key: &str, value: &str) -> &Self {
                topo::call!(
                    {
                        let storage = &*topo::Env::expect::<$crate::runtime::Dom>();
                        let mut storage = storage.borrow_mut();
                        storage.set_attribute(self.0, key, Some(value.to_owned().into()));
                    }
                );
                self
            }

            pub fn on<Event>(&self, callback: impl FnMut(&Event) + 'static) -> &Self
            where
                Event: $event_trait + 'static,
            {
                topo::call!(
                    {
                        let storage = &*topo::Env::expect::<$crate::runtime::Dom>();
                        let mut storage = storage.borrow_mut();
                        let element = storage.get_element_mut(self.0);
                        if let $crate::dom::Element::$class(element) = element {
                            element.on(callback);
                        }
                    }
                );
                self
            }

            pub fn inner(&self, children: impl FnOnce()) {
                topo::call!(
                    { children() },
                    env! {
                        Node => self.0,
                    }
                )
            }
        }

        $(#[$outer])*
        #[macro_export]
        macro_rules! $name {
            ($with_elem:expr) => {
                $crate::moxie::elements::$handle_name::create($with_elem)
            };
        }
    };
}

element_macro! {
    /// A top-level window
    window, Window, WindowHandle, crate::dom::window::WindowEvent
}

element_macro! {
    /// A generic layout container
    view, View, ViewHandle, crate::dom::view::ViewEvent
}
