use crate::dom::{element::children as get_children, Node, NodeChild, Window};
use euclid::{point2, size2, Length, Point2D, SideOffsets2D, Size2D};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use moxie::embed::Runtime;
use moxie::*;
use skribo::{FontCollection, FontFamily, LayoutSession, TextStyle};
use std::ops::Range;
use std::ptr;
use std::rc::Rc;

mod word_break_iter;

pub struct LogicalPixel;
pub type LogicalPoint = Point2D<f32, LogicalPixel>;
pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

#[derive(PartialEq)]
pub enum LayoutType {
    List,
    Inline,
    Text(String),
}

#[derive(PartialEq)]
pub struct LayoutOptions {
    pub padding: LogicalSideOffsets,
    pub width: Option<LogicalLength>,
    pub height: Option<LogicalLength>,
    pub layout_ty: LayoutType,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        LayoutOptions {
            padding: LogicalSideOffsets::new_all_same(0.0f32),
            width: None,
            height: None,
            layout_ty: LayoutType::List,
        }
    }
}

pub struct LayoutChild {
    pub index: usize,
    pub position: LogicalPoint,
    pub layout: Rc<LayoutTreeNode>,
}

pub struct LayoutTreeNode {
    pub size: LogicalSize,
    pub render_text: Option<Range<usize>>,
    pub children: Vec<LayoutChild>,
}

#[derive(Clone)]
struct TextLayoutInfo {
    text: String,
    max_width: f32,
}

#[derive(Clone)]
enum UnresolvedLayout {
    Resolved(Rc<LayoutTreeNode>),
    Text(TextLayoutInfo),
}

impl TextLayoutInfo {
    #[topo::from_env(collection: &Rc<FontCollection>)]
    fn fill_line(&self, width: f32, offset: usize) -> (usize, f32) {
        let mut session = LayoutSession::create(&self.text, &TextStyle { size: 32.0 }, collection);

        let mut x = 0.0;
        let mut last_word_end = 0;
        let mut last_word_x = 0.0;
        for word in word_break_iter::WordBreakIterator::new(&self.text[offset..]) {
            let start = word.as_ptr() as usize - self.text.as_ptr() as usize;
            let end = start + word.len();
            for run in session.iter_substr(start..end) {
                let font = run.font();
                let metrics = font.font.metrics();
                let units_per_px = metrics.units_per_em as f32 / 32.0;
                for glyph in run.glyphs() {
                    let new_x = glyph.offset.x
                        + font.font.advance(glyph.glyph_id).unwrap().x / units_per_px;
                    if last_word_x + new_x > width {
                        return (last_word_end, last_word_x);
                    }
                    x = last_word_x + new_x;
                }
            }
            last_word_end = end - offset;
            last_word_x = x;
        }

        (last_word_end, last_word_x)
    }
}

impl UnresolvedLayout {
    fn resolve(self) -> Rc<LayoutTreeNode> {
        match self {
            UnresolvedLayout::Resolved(layout) => layout,
            UnresolvedLayout::Text(text) => {
                let mut height = 0.0;
                let mut offset = 0;
                let mut longest_line_width = 0.0f32;
                let len = text.text.len();
                while offset < len {
                    let (end, width) = text.fill_line(text.max_width, offset);
                    longest_line_width = longest_line_width.max(width);
                    height += 32.0;
                    offset += end;
                }
                let size = size2(longest_line_width, height);
                Rc::new(LayoutTreeNode {
                    size,
                    render_text: None,
                    children: vec![],
                })
            }
        }
    }
}

struct LayoutInputs {
    opts: LayoutOptions,
    max_size: LogicalSize,
    children: Vec<UnresolvedLayout>,
}

impl PartialEq for LayoutInputs {
    fn eq(&self, other: &LayoutInputs) -> bool {
        if self.opts != other.opts {
            return false;
        }
        if self.max_size != other.max_size {
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

pub struct LayoutEngine {
    runtime: Runtime<fn() -> Rc<LayoutTreeNode>, Rc<LayoutTreeNode>>,
}

impl LayoutEngine {
    pub fn new() -> LayoutEngine {
        LayoutEngine {
            runtime: Runtime::new(LayoutEngine::run_layout),
        }
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

    fn calc_layout(input: &LayoutInputs) -> UnresolvedLayout {
        let opts = &input.opts;
        let children = &input.children;
        let max_size = input.max_size;

        let mut child_positions = vec![];
        child_positions.reserve(children.len());
        let min_size = match opts.layout_ty {
            LayoutType::Text(ref text) => {
                return UnresolvedLayout::Text(TextLayoutInfo {
                    text: text.clone(),
                    max_width: max_size.width,
                })
            }
            LayoutType::Inline => {
                let mut x = 0.0f32;
                let mut height = 0.0f32;
                let mut line_height = 0.0f32;
                let mut longest_line = 0.0f32;
                println!("layout inline, max_size:{}", max_size);
                for (index, child) in children.iter().enumerate() {
                    match child {
                        UnresolvedLayout::Resolved(child) => {
                            let size = child.size;
                            if x + size.width > max_size.width {
                                height += line_height;
                                longest_line = longest_line.max(x);
                                x = 0.0;
                                line_height = 0.0;
                            }
                            child_positions.push(LayoutChild {
                                index,
                                position: point2(opts.padding.left + x, opts.padding.top + height),
                                layout: child.clone(),
                            });
                            x += size.width;
                            line_height = line_height.max(size.height);
                        }
                        UnresolvedLayout::Text(text) => {
                            let mut offset = 0;
                            while offset < text.text.len() {
                                let remaining = max_size.width - x;
                                let (end, width) = text.fill_line(remaining, offset);
                                let start = offset;
                                offset += end;
                                let width = if end == 0 {
                                    height += line_height;
                                    longest_line = longest_line.max(x);
                                    x = 0.0;
                                    line_height = 0.0;
                                    let (end, width) = text.fill_line(max_size.width, offset);
                                    offset += end;
                                    if end == 0 {
                                        // overflow
                                        let (end, width) = text.fill_line(99999999.0, offset);
                                        offset += end;
                                        width
                                    } else {
                                        width
                                    }
                                } else {
                                    width
                                };
                                child_positions.push(LayoutChild {
                                    index,
                                    position: point2(
                                        opts.padding.left + x,
                                        opts.padding.top + height,
                                    ),
                                    layout: Rc::new(LayoutTreeNode {
                                        render_text: Some(start..offset),
                                        size: size2(width, 32.0),
                                        children: vec![],
                                    }),
                                });
                                x += width;
                                line_height = line_height.max(32.0);
                            }
                        }
                    }
                }
                size2(longest_line.max(x), height + line_height)
            }
            LayoutType::List => {
                let mut width = 0.0f32;
                let mut height = 0.0f32;
                for (index, child) in children.iter().enumerate() {
                    let child = child.clone().resolve();
                    let size = child.size;
                    width = width.max(size.width);
                    let size = child.size;
                    child_positions.push(LayoutChild {
                        index,
                        position: point2(opts.padding.left, height + opts.padding.top),
                        layout: child,
                    });
                    height += size.height;
                }
                size2(width, height)
            }
        };

        let mut outer = min_size + size2(opts.padding.horizontal(), opts.padding.vertical());
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        UnresolvedLayout::Resolved(Rc::new(LayoutTreeNode {
            render_text: None,
            size: outer,
            children: child_positions,
        }))
    }

    fn layout_child(node: &dyn NodeChild, parent_max_size: LogicalSize) -> UnresolvedLayout {
        topo::call!({
            let opts = node.create_layout_opts();

            let max_size = Self::calc_max_size(&opts, parent_max_size);
            let mut children = vec![];
            for child in get_children(node) {
                children.push(Self::layout_child(child, max_size));
            }

            moxie::memo!(
                LayoutInputs {
                    children,
                    opts,
                    max_size
                },
                Self::calc_layout
            )
        })
    }

    #[topo::from_env(node: &Node<Window>, size: &LogicalSize)]
    fn run_layout() -> Rc<LayoutTreeNode> {
        let collection = once!(|| {
            let mut collection = FontCollection::new();
            let source = SystemSource::new();
            let font = source
                .select_best_match(&[FamilyName::SansSerif], &Properties::new())
                .unwrap()
                .load()
                .unwrap();
            collection.add_family(FontFamily::new_from_font(font));

            Rc::new(collection)
        });

        topo::call!(
            {
                let mut child_nodes = vec![];

                for (index, child) in node.children().iter().enumerate() {
                    child_nodes.push(LayoutChild {
                        index,
                        position: point2(0.0, 0.0),
                        layout: Self::layout_child(child, *size).resolve(),
                    });
                }

                Rc::new(LayoutTreeNode {
                    render_text: None,
                    size: *size,
                    children: child_nodes,
                })
            },
            env! {
                Rc<FontCollection> => collection,
            }
        )
    }

    pub fn layout(&mut self, node: Node<Window>, size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!(
            { self.runtime.run_once() },
            env! {
                Node<Window> => node,
                LogicalSize => size,
            }
        )
    }
}
