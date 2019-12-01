#![recursion_limit = "512"]

use moxie_native::dom::{
    devtools::{register_devtools, DevTools},
    element::DynamicNode,
    node::{AnyNode, NodeRef},
};
use moxie_native::prelude::*;

define_style! {
    static VIEW = {
        background_color: rgba(0, 0, 0, 0),
    };

    static CHILD_STYLE = {
        padding: 10 px,
        background_color: rgb(255, 255, 255),
    };

    static FAKE_BORDER_STYLE = {
        padding: 2 px,
        background_color: rgb(238, 238, 238),
    };

    static NODE_STYLE = {
        padding: 4 px,
        background_color: rgb(255, 255, 255),
    };

    // Needs to be static to maintain object identity
    static SENTINEL_STYLE = {
        text_color: rgb(0, 0, 0),
    };

    static NAME_STYLE = {
        text_color: rgb(39, 111, 156),
    };

    static CONTENT_STYLE = {
        background_color: rgba(0, 0, 0, 0),
        text_color: rgb(10, 145, 50),
    };
}

#[topo::nested]
fn node_view(node: NodeRef) -> Node<View> {
    if let Some(style) = node.style() {
        if style == SENTINEL_STYLE {
            return mox! {
                <view style={FAKE_BORDER_STYLE}>
                    <view style={NODE_STYLE}>
                        <span>
                            "<devtools />"
                        </span>
                    </view>
                </view>
            };
        }
    }

    mox! {
        <view style={FAKE_BORDER_STYLE}>
            <view style={NODE_STYLE}>
                <span>
                    "<"
                    <span style={NAME_STYLE}>
                        {% "{}", node.name()}
                    </span>
                    {node.style().map(|style| format!(" style={}", style.name()))}
                    ">"
                </span>
                <view style={CHILD_STYLE}>
                    {node.children().map(|child| match child {
                        DynamicNode::Node(child) => mox! {
                            <view style={VIEW}>
                                <node_view _=(child) />
                            </view>
                        },
                        DynamicNode::Text(text) => mox! {
                            <view style={CONTENT_STYLE}>
                                <span>{% "{:?}", text}</span>
                            </view>
                        }
                    }).collect::<Vec<_>>()}
                </view>
            </view>
        </view>
    }
}

struct Tools {
    root: Key<Option<AnyNode>>,
}

impl DevTools for Tools {
    fn on_update(&mut self, node: AnyNode) {
        println!("new node {}", node.name());
        self.root.set(Some(node));
    }
}

#[topo::nested]
pub fn devtools() -> Node<View> {
    let root = state!(|| None);

    register_devtools(Tools { root: root.clone() });

    if let Some(ref node) = *root {
        mox! {
            <view style={SENTINEL_STYLE}>
                <node_view _=(node.into()) />
            </view>
        }
    } else {
        mox! {
            <view style={SENTINEL_STYLE}>
                <span>
                    "Loading"
                </span>
            </view>
        }
    }
}
