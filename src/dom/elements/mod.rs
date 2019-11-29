/// Defines an enum over multiple types that implement NodeChild and
/// then implements NodeChild for that enum.
macro_rules! multiple_children {
    (enum $name:ident { $( $var_name:ident ( $var_ty:ty ) ),+ $(,)* }) => {
        #[derive(Clone, PartialEq)]
        pub enum $name {
            $(
                $var_name ($var_ty)
            ),+
        }

        $(
            impl From<$var_ty> for $name {
                fn from(elt: $var_ty) -> Self {
                    $name::$var_name(elt)
                }
            }
        )+

        impl $crate::dom::element::NodeChild for $name {
            fn get_node(&self) -> $crate::dom::element::DynamicNode {
                match self {
                    $(
                        $name::$var_name(elt) => $crate::dom::element::DynamicNode::Node(&**elt)
                    ),+
                }
            }
        }
    }
}

macro_rules! element_attributes {
    ( $element:ty { $( $name:ident : $class:ty ),+ $(,)* } ) => {
        $(
            impl $crate::dom::element::HasAttribute<$class> for $element {
                fn set_attribute(&mut self, value: <$class as crate::dom::element::Attribute>::Value) {
                    self.$name = value.into();
                }
            }
        )+
    };
}

macro_rules! element_handlers {
    ( $handler_name:ident for $element:ty { $( $name:ident : $class:ty ),+ $(,)* } ) => {
        #[derive(Default)]
        pub struct $handler_name {
            $(
                $name : EventHandler<$class>
            ),+
        }

        $(
            impl HasEvent<$class> for $element {
                fn set_handler(list: &mut $handler_name, handler: EventHandler<$class>) {
                    list.$name = handler;
                }

                fn get_handler(list: &$handler_name) -> &EventHandler<$class> {
                    &list.$name
                }
            }
        )+

        impl $crate::dom::element::HandlerList for $handler_name {}
    };
}

pub mod button;
pub mod span;
pub mod view;
pub mod window;
