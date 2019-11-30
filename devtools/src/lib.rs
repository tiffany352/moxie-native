#![recursion_limit = "512"]

use moxie_native::dom::{
    devtools::{register_devtools, DevTools},
    element::DynamicNode,
    node::{AnyNode, AnyNodeData},
};
use moxie_native::prelude::*;

const VIEW: Style = define_style! {
    background_color: rgba(0, 0, 0, 0),
};

const CHILD_STYLE: Style = define_style! {
    padding: 10 px,
    background_color: rgb(255, 255, 255),
};

const FAKE_BORDER_STYLE: Style = define_style! {
    padding: 2 px,
    background_color: rgb(238, 238, 238),
};

const NODE_STYLE: Style = define_style! {
    padding: 4 px,
    background_color: rgb(255, 255, 255),
};

// Needs to be static to maintain object identity
static SENTINEL_STYLE: Style = define_style! {
    text_color: rgb(0, 0, 0),
};

#[topo::nested]
fn node_view(node: &dyn AnyNodeData) -> Node<View> {
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
                    {% "<{}>", node.name()}
                </span>
                <view style={CHILD_STYLE}>
                    {node.children().map(|child| match child {
                        DynamicNode::Node(child) => mox! {
                            <view style={VIEW}>
                                <node_view _=(child) />
                            </view>
                        },
                        DynamicNode::Text(text) => mox! {
                            <view style={VIEW}>
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
                <node_view _=(&**node) />
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
