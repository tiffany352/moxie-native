use crate::dom::storage::DomStorage;
use crate::dom::view::View;
use crate::dom::{Node, TypedNode};
use euclid::{point2, size2, Length, Point2D, SideOffsets2D, Size2D};
use moxie::*;
use std::ptr;
use std::rc::Rc;
pub struct LogicalPixel;

pub type LogicalPoint = Point2D<f32, LogicalPixel>;
pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

pub struct Layout {
    max_size: LogicalSize,
    min_size: LogicalSize,
    child_positions: Vec<LogicalPoint>,
}

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

impl Layout {
    pub fn new() -> Layout {
        Layout {
            max_size: LogicalSize::new(0.0, 0.0),
            min_size: LogicalSize::new(0.0, 0.0),
            child_positions: vec![],
        }
    }

    pub fn calc_max_size(&mut self, opts: &LayoutOptions, parent_size: LogicalSize) -> LogicalSize {
        let mut outer = parent_size;
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        self.max_size = outer - size2(opts.padding.horizontal(), opts.padding.vertical());
        self.max_size
    }

    pub fn calc_min_size(
        &mut self,
        opts: &LayoutOptions,
        child_sizes: &[LogicalSize],
    ) -> LogicalSize {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        for child in child_sizes {
            width = width.max(child.width);
            self.child_positions
                .push(point2(opts.padding.left, height + opts.padding.top));
            height += child.height;
        }
        let mut outer =
            size2(width, height) + size2(opts.padding.horizontal(), opts.padding.vertical());
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        self.min_size = outer;
        outer
    }

    pub fn size(&self) -> LogicalSize {
        self.min_size
    }

    pub fn child_positions(&self) -> &[LogicalPoint] {
        &self.child_positions[..]
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LayoutEngine {
    node: Node,
}

pub struct LayoutChild {
    position: LogicalPoint,
    layout: Rc<LayoutTreeNode>,
}

pub struct LayoutTreeNode {
    node: Node,
    size: LogicalSize,
    children: Vec<LayoutChild>,
}

struct LayoutInputs {
    node: TypedNode<View>,
    opts: LayoutOptions,
    children: Vec<Rc<LayoutTreeNode>>,
}

impl PartialEq for LayoutInputs {
    fn eq(&self, other: &LayoutInputs) -> bool {
        if self.node != other.node {
            return false;
        }
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
    pub fn new(node: Node) -> LayoutEngine {
        LayoutEngine { node }
    }

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
            node: input.node.to_inner(),
            size: outer,
            children: child_positions,
        })
    }

    fn layout_inner(
        storage: &mut DomStorage,
        node: TypedNode<View>,
        parent_max_size: LogicalSize,
    ) -> Rc<LayoutTreeNode> {
        topo::call!({
            let view = storage.get_element_typed(node.clone());
            let opts = view.create_layout_opts();

            let max_size = Self::calc_max_size(&opts, parent_max_size);
            let mut children = vec![];
            for child in storage.get_children_of_type::<View>(node.to_inner()) {
                children.push(Self::layout_inner(storage, child, max_size));
            }

            moxie::memo!(
                LayoutInputs {
                    node,
                    children,
                    opts
                },
                Self::calc_layout
            )
        })
    }

    pub fn layout(&self, storage: &mut DomStorage, size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::root!({
            let mut child_nodes = vec![];

            for child in storage.get_children_of_type::<View>(self.node) {
                child_nodes.push(LayoutChild {
                    position: point2(0.0, 0.0),
                    layout: Self::layout_inner(storage, child, size),
                });
            }

            Rc::new(LayoutTreeNode {
                node: self.node,
                size: size,
                children: child_nodes,
            })
        })
    }
}
