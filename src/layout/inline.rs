use super::{
    block,
    text::{TextLayoutInfo, TextState},
    LayoutChild, LayoutText, LayoutTreeNode, LogicalSideOffsets, LogicalSize, RenderData,
};
use crate::dom::{element::DynamicNode, node::AnyNode, node::NodeRef};
use crate::style::{ComputedValues, DisplayType};
use crate::util::equal_rc::EqualRc;
use euclid::{point2, size2};

#[derive(PartialEq)]
enum InlineLayoutItem {
    Block(EqualRc<LayoutTreeNode>),
    Text {
        text: EqualRc<TextLayoutInfo>,
        parent: AnyNode,
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
                x,
                layout,
            } = item;
            self.children.push(LayoutChild {
                position: point2(x, self.height + line.ascender - ascender),
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

    fn insert_block_item(&mut self, layout: EqualRc<LayoutTreeNode>) -> bool {
        let size = layout.size;
        if self.x + size.width > self.max_width {
            return false;
        }
        self.line_items.push(LineItem {
            x: self.x,
            ascender: size.height,
            layout,
        });
        self.x += size.width;
        self.height = self.height.max(size.height);
        true
    }

    fn insert_text_item(&mut self, parent: AnyNode, state: &mut TextState) -> bool {
        if let Some(line) = state.fill_line(self.max_width - self.x, self.line_items.is_empty()) {
            self.line_items.push(LineItem {
                ascender: line.ascender,
                x: self.x,
                layout: EqualRc::new(LayoutTreeNode {
                    render: RenderData::Text {
                        text: LayoutText {
                            fragments: line.fragments,
                            size: line.text_size,
                        },
                        parent,
                    },
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
    node: NodeRef,
    parent_values: &ComputedValues,
    max_size: LogicalSize,
    items: &mut Vec<InlineLayoutItem>,
) {
    for child in node.children() {
        topo::call(|| match child {
            DynamicNode::Node(node) => {
                let values = node.computed_values().get().unwrap();
                match values.display {
                    DisplayType::Block(ref block) => {
                        let layout = block::layout_block(node, &values, block, max_size).into();
                        items.push(InlineLayoutItem::Block(layout));
                    }
                    DisplayType::Inline(_) => {
                        collect_inline_items(node, &values, max_size, items);
                    }
                }
            }
            DynamicNode::Text(text) => items.push(InlineLayoutItem::Text {
                text: moxie::memo::memo(
                    (text.to_owned(), parent_values.text_size.get()),
                    move |(text, size)| {
                        EqualRc::new(TextLayoutInfo::new((*text).to_owned(), *size))
                    },
                )
                .into(),
                parent: node.to_owned(),
            }),
        })
    }
}

fn calc_inline_layout(
    node: AnyNode,
    max_width: f32,
    items: &[InlineLayoutItem],
) -> EqualRc<LayoutTreeNode> {
    let mut state = LayoutState {
        height: 0.0f32,
        longest_line: 0.0f32,
        children: vec![],
    };

    let mut line = LineState::new(max_width);

    for item in items {
        match item {
            InlineLayoutItem::Block(layout) => {
                if !line.insert_block_item(layout.clone().into()) {
                    let old_line = std::mem::replace(&mut line, LineState::new(max_width));
                    state.add_line(old_line);
                    line.insert_block_item(layout.clone().into());
                }
            }
            InlineLayoutItem::Text { text, parent } => {
                let mut text_state = TextState::new(&**text);
                loop {
                    line.insert_text_item(parent.clone(), &mut text_state);
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
        render: RenderData::Node(node),
        margin: LogicalSideOffsets::default(),
        size,
        children,
    })
}

pub fn layout_inline(
    node: NodeRef,
    values: &ComputedValues,
    max_size: LogicalSize,
) -> EqualRc<LayoutTreeNode> {
    let mut items = vec![];

    collect_inline_items(node, values, max_size, &mut items);

    moxie::memo::memo(
        (node.to_owned(), max_size.width, items),
        |(node, max_width, items)| calc_inline_layout(node.clone(), *max_width, &items[..]),
    )
}

pub fn layout_text(
    node: AnyNode,
    text: &str,
    max_width: f32,
    values: &ComputedValues,
) -> EqualRc<LayoutTreeNode> {
    let size = values.text_size;
    moxie::memo::memo(
        (max_width, text.to_owned(), node, size),
        |(max_width, text, node, size)| {
            let item = InlineLayoutItem::Text {
                text: EqualRc::new(TextLayoutInfo::new(text.to_owned(), size.get())),
                parent: node.clone(),
            };
            calc_inline_layout(node.clone(), *max_width, &[item])
        },
    )
}
