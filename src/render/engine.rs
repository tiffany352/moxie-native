use crate::dom::{
    element::children as get_children, ClickEvent, EventHandler, Node, NodeChild, Window,
};
use crate::layout::LayoutTreeNode;
use crate::style::ComputedValues;
use moxie::embed::Runtime;
use std::rc::Rc;

/// Contains information needed to display an element, and also event
/// handlers for processing input.
#[derive(Default)]
pub struct PaintDetails {
    pub on_click: EventHandler<ClickEvent>,
}

/// A node in the paint tree, which corresponds 1:1 to the DOM.
pub struct PaintTreeNode {
    /// Not all nodes are visible, so not all have associated paint
    /// information. Stores an Rc pointer for caching.
    pub details: Option<Rc<PaintDetails>>,
    pub values: Option<ComputedValues>,
    /// Children are stored directly since caching of the tree nodes is
    /// not necessary.
    pub children: Vec<PaintTreeNode>,
}

/// Persistent structure for building the paint tree.
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
        let values = node
            .computed_values()
            .ok()
            .map(|value| value.get().unwrap());
        let mut children = vec![];

        for (child, layout) in get_children(node).zip(layout.children.iter()) {
            children.push(Self::render_child(child, &layout.layout));
        }

        PaintTreeNode {
            details,
            values,
            children,
        }
    }

    #[topo::from_env(node: &Node<Window>, layout: &Rc<LayoutTreeNode>)]
    fn run_render() -> PaintTreeNode {
        let values = node.computed_values().get();

        let mut children = vec![];
        for (child, layout) in node.children().iter().zip(layout.children.iter()) {
            children.push(Self::render_child(child, &layout.layout));
        }

        PaintTreeNode {
            details: None,
            values,
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
