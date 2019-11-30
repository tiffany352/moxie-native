use super::{
    block,
    text::{TextLayoutInfo, TextState},
    LayoutChild, LayoutText, LayoutTreeNode, LogicalSideOffsets, LogicalSize,
};
use crate::dom::{element::DynamicNode, node::AnyNodeData};
use crate::style::{ComputedValues, DisplayType};
use crate::util::equal_rc::EqualRc;
use euclid::{point2, size2};
use moxie::*;

#[derive(PartialEq)]
enum InlineLayoutItem {
    Block {
        index: usize,
        layout: EqualRc<LayoutTreeNode>,
    },
    Text {
        index: usize,
        text: EqualRc<TextLayoutInfo>,
    },
}

struct LayoutState {
    children: Vec<LayoutChild>,
    longest_line: f32,
    height: f32,
}

impl LayoutState {
    fn add_line(&mut self, line: LineState) {
        for item in line.line_items {
            let LineItem {
                ascender,
                index,
                x,
                layout,
            } = item;
            self.children.push(LayoutChild {
                position: point2(x, self.height + line.ascender - ascender),
                index,
                layout,
            });
        }

        self.height += line.height;
        self.longest_line = self.longest_line.max(line.x);
    }
}

// Turns into LayoutChild
struct LineItem {
    ascender: f32,
    index: usize,
    x: f32,
    layout: EqualRc<LayoutTreeNode>,
}

struct LineState {
    line_items: Vec<LineItem>,
    max_width: f32,
    x: f32,
    height: f32,
    ascender: f32,
}

impl LineState {
    fn new(max_width: f32) -> Self {
        LineState {
            max_width,
            x: 0.0f32,
            height: 0.0f32,
            ascender: 0.0f32,
            line_items: vec![],
        }
    }

    fn insert_block_item(&mut self, index: usize, layout: EqualRc<LayoutTreeNode>) -> bool {
        let size = layout.size;
        if self.x + size.width > self.max_width {
            return false;
        }
        self.line_items.push(LineItem {
            x: self.x,
            ascender: size.height,
            index,
            layout,
        });
        self.x += size.width;
        self.height = self.height.max(size.height);
        true
    }

    fn insert_text_item(&mut self, index: usize, state: &mut TextState) -> bool {
        if let Some(line) = state.fill_line(self.max_width - self.x, self.line_items.is_empty()) {
            self.line_items.push(LineItem {
                index,
                ascender: line.ascender,
                x: self.x,
                layout: EqualRc::new(LayoutTreeNode {
                    render_text: Some(LayoutText {
                        fragments: line.fragments,
                        size: line.text_size,
                    }),
                    size: size2(line.width, line.height),
                    margin: LogicalSideOffsets::default(),
                    children: vec![],
                }),
            });

            self.x += line.width;
            self.height = self.height.max(line.height);
            self.ascender = self.ascender.max(line.ascender);

            true
        } else {
            false
        }
    }
}

fn collect_inline_items(
    node: &dyn AnyNodeData,
    parent_values: &ComputedValues,
    max_size: LogicalSize,
    items: &mut Vec<InlineLayoutItem>,
) {
    for (index, child) in node.children().enumerate() {
        topo::call! {
            {
                match child {
                    DynamicNode::Node(node) => {
                        let values = node.computed_values().get().unwrap();
                        match values.display {
                            DisplayType::Block(ref block) => {
                                let layout = block::layout_block(node, &values, block, max_size).into();
                                items.push(InlineLayoutItem::Block { index, layout });
                            }
                            DisplayType::Inline(_) => {
                                collect_inline_items(node, &values, max_size, items);
                            }
                        }
                    }
                    DynamicNode::Text(text) => items.push(InlineLayoutItem::Text {
                        text: memo!((text.to_owned(), parent_values.text_size.get()), move |(text, size)| {
                            EqualRc::new(TextLayoutInfo::new(
                                (*text).to_owned(),
                                *size,
                            ))
                        }).into(),
                        index,
                    })
                }
            }
        }
    }
}

fn calc_inline_layout(max_width: f32, items: &[InlineLayoutItem]) -> EqualRc<LayoutTreeNode> {
    let mut state = LayoutState {
        height: 0.0f32,
        longest_line: 0.0f32,
        children: vec![],
    };

    let mut line = LineState::new(max_width);

    for item in items {
        match item {
            InlineLayoutItem::Block { index, layout } => {
                if !line.insert_block_item(*index, layout.clone().into()) {
                    let old_line = std::mem::replace(&mut line, LineState::new(max_width));
                    state.add_line(old_line);
                    line.insert_block_item(*index, layout.clone().into());
                }
            }
            InlineLayoutItem::Text { index, text } => {
                let mut text_state = TextState::new(&**text);
                loop {
                    line.insert_text_item(*index, &mut text_state);
                    if text_state.finished() {
                        break;
                    }
                    let old_line = std::mem::replace(&mut line, LineState::new(max_width));
                    state.add_line(old_line);
                }
            }
        }
    }
    state.add_line(line);
    let size = size2(state.longest_line, state.height);
    let children = state.children;

    EqualRc::new(LayoutTreeNode {
        render_text: None,
        margin: LogicalSideOffsets::default(),
        size,
        children,
    })
}

pub fn layout_inline(
    node: &dyn AnyNodeData,
    values: &ComputedValues,
    max_size: LogicalSize,
) -> EqualRc<LayoutTreeNode> {
    let mut items = vec![];

    collect_inline_items(node, values, max_size, &mut items);

    memo!((max_size.width, items), |(max_width, items)| {
        calc_inline_layout(*max_width, &items[..])
    })
}

pub fn layout_text(
    index: usize,
    text: &str,
    max_width: f32,
    values: &ComputedValues,
) -> EqualRc<LayoutTreeNode> {
    let size = values.text_size;
    memo!((max_width, text.to_owned(), index, size), |(
        max_width,
        text,
        index,
        size,
    )| {
        let item = InlineLayoutItem::Text {
            index: *index,
            text: EqualRc::new(TextLayoutInfo::new(text.to_owned(), size.get())),
        };
        calc_inline_layout(*max_width, &[item])
    })
}
