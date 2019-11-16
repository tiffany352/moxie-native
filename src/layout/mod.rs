use crate::dom::{Node, View, Window};
use euclid::{point2, size2, Length, Point2D, SideOffsets2D, Size2D};
use moxie::*;
use std::ptr;
use std::rc::Rc;
pub struct LogicalPixel;

pub type LogicalPoint = Point2D<f32, LogicalPixel>;
pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

#[derive(PartialEq)]
pub struct LayoutOptions {
    pub padding: LogicalSideOffsets,
    pub width: Option<LogicalLength>,
    pub height: Option<LogicalLength>,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        LayoutOptions {
            padding: LogicalSideOffsets::new_all_same(0.0f32),
            width: None,
            height: None,
        }
    }
}

pub struct LayoutEngine;

pub struct LayoutChild {
    pub position: LogicalPoint,
    pub layout: Rc<LayoutTreeNode>,
}

pub struct LayoutTreeNode {
    pub size: LogicalSize,
    pub children: Vec<LayoutChild>,
}

struct LayoutInputs {
    opts: LayoutOptions,
    children: Vec<Rc<LayoutTreeNode>>,
}

impl PartialEq for LayoutInputs {
    fn eq(&self, other: &LayoutInputs) -> bool {
        if self.opts != other.opts {
            return false;
        }
        if self.children.len() != other.children.len() {
            return false;
        }
        for (a, b) in self.children.iter().zip(other.children.iter()) {
            if !ptr::eq(a, b) {
                return false;
            }
        }
        true
    }
}

impl LayoutEngine {
    fn calc_max_size(opts: &LayoutOptions, parent_size: LogicalSize) -> LogicalSize {
        let mut outer = parent_size;
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        outer - size2(opts.padding.horizontal(), opts.padding.vertical())
    }

    fn calc_layout(input: &LayoutInputs) -> Rc<LayoutTreeNode> {
        let opts = &input.opts;
        let children = &input.children;

        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let mut child_positions = vec![];
        child_positions.reserve(children.len());
        for child in children {
            let size = child.size;
            width = width.max(size.width);
            child_positions.push(LayoutChild {
                position: point2(0.0, height),
                layout: child.clone(),
            });
            height += size.height;
        }
        let mut outer =
            size2(width, height) + size2(opts.padding.horizontal(), opts.padding.vertical());
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        Rc::new(LayoutTreeNode {
            size: outer,
            children: child_positions,
        })
    }

    fn layout_inner(node: &Node<View>, parent_max_size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!({
            let view = node.element();
            let opts = view.create_layout_opts();

            let max_size = Self::calc_max_size(&opts, parent_max_size);
            let mut children = vec![];
            for child in node.children() {
                children.push(Self::layout_inner(child, max_size));
            }

            moxie::memo!(LayoutInputs { children, opts }, Self::calc_layout)
        })
    }

    pub fn layout(node: &Node<Window>, size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::root!({
            let mut child_nodes = vec![];

            for child in node.children() {
                child_nodes.push(LayoutChild {
                    position: point2(0.0, 0.0),
                    layout: Self::layout_inner(child, size),
                });
            }

            Rc::new(LayoutTreeNode {
                size: size,
                children: child_nodes,
            })
        })
    }
}
