use super::DocumentState;
use crate::dom::element::{DynamicNode, ElementState};
use crate::dom::node::NodeRef;
use crate::style::{ComputedValues, NodeSelect, Style};
use std::any::TypeId;

struct NodeProxy<'a> {
    node: NodeRef<'a>,
    state: &'a DocumentState,
}

impl<'a> NodeSelect for NodeProxy<'a> {
    fn has_type(&self, ty: TypeId) -> bool {
        self.node.type_id() == ty
    }

    fn has_state(&self, state: ElementState) -> bool {
        self.state.node_states(self.node.id()).contains(state)
    }
}

impl DocumentState {
    pub fn update_style(&mut self, node: NodeRef, parent: Option<&ComputedValues>) {
        let mut computed = node.create_computed_values();

        let default_values = ComputedValues::default();
        let parent = parent.unwrap_or(&default_values);

        // Default-inherited attributes
        computed.text_color = parent.text_color;
        computed.text_size = parent.text_size;

        illicit::child_env!(
            ComputedValues => parent.clone()
        )
        .enter(|| {
            let style = node.style();
            if let Some(Style(style)) = style {
                (style.attributes.apply)(&mut computed);

                let node = NodeProxy { node, state: self };
                for sub_style in style.sub_styles {
                    if (sub_style.selector)(&node) {
                        (sub_style.attributes.apply)(&mut computed);
                    }
                }
            }
        });

        self.states.get_mut(&node.id()).unwrap().computed_values = Some(computed);

        for child in node.children() {
            if let DynamicNode::Node(node) = child {
                self.update_style(node, Some(&computed));
            }
        }
    }
}
