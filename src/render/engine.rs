use crate::dom::{element::children as get_children, Node, NodeChild, Window};
use crate::layout::LayoutTreeNode;
use crate::Color;
use moxie::embed::Runtime;
use std::rc::Rc;

#[derive(Default)]
pub struct PaintDetails {
    pub background_color: Option<Color>,
    pub text: Option<String>,
}

pub struct PaintTreeNode {
    pub details: Option<Rc<PaintDetails>>,
    pub children: Vec<PaintTreeNode>,
}

pub struct RenderEngine {
    runtime: Runtime<fn() -> PaintTreeNode, PaintTreeNode>,
}

impl RenderEngine {
    pub fn new() -> RenderEngine {
        RenderEngine {
            runtime: Runtime::new(Self::run_render),
        }
    }

    fn render_child(node: &dyn NodeChild, layout: &Rc<LayoutTreeNode>) -> PaintTreeNode {
        let details = node.paint().map(Rc::new);
        let mut children = vec![];

        for (child, layout) in get_children(node).zip(layout.children.iter()) {
            children.push(Self::render_child(child, &layout.layout));
        }

        PaintTreeNode { details, children }
    }

    #[topo::from_env(node: &Node<Window>, layout: &Rc<LayoutTreeNode>)]
    fn run_render() -> PaintTreeNode {
        let mut children = vec![];
        for (child, layout) in node.children().iter().zip(layout.children.iter()) {
            children.push(Self::render_child(child, &layout.layout));
        }

        PaintTreeNode {
            details: None,
            children,
        }
    }

    pub fn render(&mut self, window: Node<Window>, layout: Rc<LayoutTreeNode>) -> PaintTreeNode {
        topo::call!(
            { self.runtime.run_once() },
            env! {
                Node<Window> => window,
                Rc<LayoutTreeNode> => layout,
            }
        )
    }
}
