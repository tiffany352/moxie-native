use crate::dom::{Element, Node};
use moxie::*;
use std::any::Any;
use std::cell::RefCell;

struct BuilderChildren(Box<dyn Any>);

struct Builder<Elt: Element>(Elt);

impl<Elt> Builder<Elt>
where
    Elt: Element,
{
    pub fn create(with_elem: impl FnOnce(Self)) {
        topo::call!({ with_elem(Self(Elt::default())) })
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        topo::call!({
            self.0.set_attribute(key, Some(value.to_owned().into()));
        });
        self
    }

    #[topo::from_env(parent_children: &BuilderChildren)]
    pub fn inner(self, create_children: impl FnOnce()) {
        let children = BuilderChildren(Box::new(RefCell::new(Vec::<Elt::Child>::new())));
        topo::call!(
            { create_children() },
            env! {
                BuilderChildren => children,
            }
        );

        let children: Vec<Elt::Child> = (*children
            .0
            .downcast::<RefCell<Vec<Elt::Child>>>()
            .expect("correct Any value"))
        .into_inner();

        let node = memo!((self.0, children), |(elt, children): &(
            Elt,
            Vec<Elt::Child>
        )| Node::new(
            elt.clone(),
            children.clone()
        ));

        parent_children
            .0
            .downcast::<RefCell<Vec<Node<Elt>>>>()
            .expect("Invalid child attached to parent")
            .borrow_mut()
            .push(node);
    }
}

pub fn root<RootChild>(create_children: &mut impl FnMut()) -> Vec<RootChild>
where
    RootChild: 'static,
{
    let children = BuilderChildren(Box::new(RefCell::new(Vec::<RootChild>::new())));
    topo::call!(
        { create_children() },
        env! {
            BuilderChildren => children,
        }
    );

    let children: Vec<RootChild> = (*children
        .0
        .downcast::<RefCell<Vec<RootChild>>>()
        .expect("correct Any value"))
    .into_inner();

    children
}

/// Top level window
#[macro_export]
macro_rules! window {
    ($with_elem:expr) => {
        $crate::moxie::elements::Builder::<$crate::dom::Window>::new($with_elem)
    };
}

/// Basic flow container
#[macro_export]
macro_rules! view {
    ($with_elem:expr) => {
        $crate::moxie::elements::Builder::<$crate::dom::View>::new($with_elem)
    };
}
