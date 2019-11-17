use super::{EventHandler, Node};
use crate::layout::{LayoutOptions, LayoutType};
use crate::render::PaintDetails;
use std::borrow::Cow;

pub trait Element: Default + Clone + PartialEq + 'static {
    type Child: NodeChild + Clone + PartialEq;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>);

    fn create_layout_opts(&self) -> LayoutOptions;

    fn paint(&self) -> Option<PaintDetails> {
        None
    }
}

pub trait Event {}

pub trait CanSetEvent<Ev>
where
    Ev: Event,
{
    fn set_handler(&mut self, handler: EventHandler<Ev>);
}

pub trait NodeChild: 'static {
    fn paint(&self) -> Option<PaintDetails>;
    fn create_layout_opts(&self) -> LayoutOptions;
    fn get_child(&self, child: usize) -> Option<&dyn NodeChild>;
}

pub fn children(node: &dyn NodeChild) -> impl Iterator<Item = &dyn NodeChild> {
    struct Iter<'a> {
        node: &'a dyn NodeChild,
        index: usize,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a dyn NodeChild;

        fn next(&mut self) -> Option<Self::Item> {
            let child = self.node.get_child(self.index);
            self.index += 1;
            child
        }
    }

    Iter {
        node: node,
        index: 0,
    }
}

impl<Elt> NodeChild for Node<Elt>
where
    Elt: Element,
{
    fn paint(&self) -> Option<PaintDetails> {
        Element::paint(self.element())
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        Element::create_layout_opts(self.element())
    }

    fn get_child(&self, child: usize) -> Option<&dyn NodeChild> {
        if let Some(child) = self.children().get(child) {
            Some(child)
        } else {
            None
        }
    }
}

impl NodeChild for String {
    fn paint(&self) -> Option<PaintDetails> {
        Some(PaintDetails {
            text: Some(self.clone()),
            ..Default::default()
        })
    }

    fn create_layout_opts(&self) -> LayoutOptions {
        LayoutOptions {
            layout_ty: LayoutType::Text(self.clone()),
            ..Default::default()
        }
    }

    fn get_child(&self, _child: usize) -> Option<&dyn NodeChild> {
        None
    }
}
