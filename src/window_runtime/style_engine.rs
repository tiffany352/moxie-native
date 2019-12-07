use super::node::LocalNodeStorage;
use crate::dom::{element::DynamicNode, node::NodeRef, Node, Window};
use crate::layout::LogicalSize;
use crate::style::{ComputedValues, Style};
use moxie::embed::Runtime;
use std::rc::Rc;

/// Used to annotate the node tree with computed values from styling.
pub struct StyleEngine {
    runtime: Runtime<fn()>,
}

impl StyleEngine {
    pub fn new() -> StyleEngine {
        StyleEngine {
            runtime: Runtime::new(StyleEngine::run_styling),
        }
    }

    #[illicit::from_env(node_storage: &Rc<LocalNodeStorage>)]
    fn update_style(node: NodeRef, parent: Option<&ComputedValues>) {
        let mut computed = node.create_computed_values();

        if let Some(parent) = parent {
            computed.text_size = parent.text_size;
            computed.text_color = parent.text_color;
        }

        let style = node.style();
        if let Some(Style(style)) = style {
            style.attributes.apply(&mut computed);
            for sub_style in style.sub_styles {
                if (sub_style.selector)(node) {
                    sub_style.attributes.apply(&mut computed);
                }
            }
        }

        *node_storage.values_mut(node.id()) = computed;

        for child in node.children() {
            if let DynamicNode::Node(node) = child {
                Self::update_style(node, Some(&computed));
            }
        }
    }

    #[illicit::from_env(node: &Node<Window>)]
    fn run_styling() {
        Self::update_style(node.into(), None);
    }

    /// Update the node tree with computed values.
    pub fn update(&mut self, node: Node<Window>, size: LogicalSize, storage: Rc<LocalNodeStorage>) {
        illicit::child_env!(
            Node<Window> => node,
            LogicalSize => size,
            Rc<LocalNodeStorage> => storage
        )
        .enter(|| topo::call!(self.runtime.run_once()))
    }
}
