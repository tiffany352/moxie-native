use super::Node;
use crate::layout::{LayoutOptions, LogicalPixel, LogicalPoint, LogicalSize};
use euclid::Scale;
use std::borrow::Cow;
use webrender::api::{units::LayoutPixel, DisplayListBuilder, PipelineId, RenderApi, Transaction};

pub struct DrawContext<'a> {
    pub position: LogicalPoint,
    pub size: LogicalSize,
    pub scale: Scale<f32, LogicalPixel, LayoutPixel>,
    pub pipeline_id: PipelineId,
    pub builder: &'a mut DisplayListBuilder,
    pub transaction: &'a mut Transaction,
    pub api: &'a RenderApi,
}

pub trait Element: Default + Clone + PartialEq + 'static {
    type Child: NodeChild + Clone + PartialEq;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>);

    fn draw(&self, _context: DrawContext) {}
    fn create_layout_opts(&self) -> LayoutOptions;
}

pub trait NodeChild: 'static {
    fn draw(&self, context: DrawContext);
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
    fn draw(&self, context: DrawContext) {
        Element::draw(self.element(), context);
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
    fn draw(&self, _context: DrawContext) {}

    fn create_layout_opts(&self) -> LayoutOptions {
        panic!()
    }

    fn get_child(&self, _child: usize) -> Option<&dyn NodeChild> {
        None
    }
}
