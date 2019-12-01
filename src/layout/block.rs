use super::{inline, LayoutChild, LayoutTreeNode, LogicalSize, RenderData};
use crate::dom::{element::DynamicNode, node::AnyNode, node::NodeRef};
use crate::style::{BlockValues, ComputedValues, Direction, DisplayType};
use crate::util::equal_rc::EqualRc;
use euclid::{point2, size2};
use moxie::*;

fn calc_max_size(values: &BlockValues, parent_size: LogicalSize) -> LogicalSize {
    let mut outer = parent_size;
    if let Some(width) = values.width {
        outer.width = width.get();
    }
    if let Some(height) = values.height {
        outer.height = height.get();
    }
    outer - size2(values.padding.horizontal(), values.padding.vertical())
}

fn calc_block_layout(
    input: &(BlockValues, Vec<EqualRc<LayoutTreeNode>>, AnyNode),
) -> EqualRc<LayoutTreeNode> {
    let (values, children, node) = input;

    let mut width = 0.0f32;
    let mut height = 0.0f32;
    let mut child_positions = vec![];
    for child in children {
        let child = child.clone();
        let size = child.size + size2(child.margin.horizontal(), child.margin.vertical());
        if values.direction == Direction::Vertical {
            width = width.max(size.width);
            child_positions.push(LayoutChild {
                position: point2(values.padding.left, height + values.padding.top),
                layout: child,
            });
            height += size.height;
        } else {
            height = height.max(size.height);
            child_positions.push(LayoutChild {
                position: point2(width + values.padding.left, values.padding.top),
                layout: child,
            });
            width += size.width;
        }
    }

    let mut size =
        size2(width, height) + size2(values.padding.horizontal(), values.padding.vertical());

    if let Some(width) = values.width {
        size.width = width.get();
    }
    if let Some(height) = values.height {
        size.height = height.get();
    }

    let margin = values.margin;

    EqualRc::new(LayoutTreeNode {
        size,
        margin,
        children: child_positions,
        render: RenderData::Node(node.clone()),
    })
}

pub fn layout_block(
    node: NodeRef,
    values: &ComputedValues,
    block_values: &BlockValues,
    parent_max_size: LogicalSize,
) -> EqualRc<LayoutTreeNode> {
    let max_size = calc_max_size(block_values, parent_max_size);

    let mut children = vec![];
    for child in node.children() {
        topo::call! {
            {
                match child {
                    DynamicNode::Node(node) => {
                        let values = node.computed_values().get().unwrap();
                        match values.display {
                            DisplayType::Block(ref block) => {
                                children.push(layout_block(node, &values, block, max_size));
                            }
                            DisplayType::Inline(_) => {
                                children.push(inline::layout_inline(node, &values, max_size));
                            }
                        }
                    }
                    DynamicNode::Text(text) => {
                        children.push(inline::layout_text(node.to_owned(), text, max_size.width, values));
                    }
                }
            }
        }
    }

    moxie::memo!(
        (block_values.clone(), children, node.to_owned()),
        calc_block_layout
    )
}
