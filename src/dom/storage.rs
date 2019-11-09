use super::Element;
use super::Node;
use slotmap::DenseSlotMap;
use std::borrow::Cow;

#[derive(Clone)]
pub enum NodeOrText {
    Text(Cow<'static, str>),
    Node(Node),
}

struct NodeData {
    children: Vec<NodeOrText>,
    element: Element,
}

pub struct DomStorage {
    nodes: DenseSlotMap<Node, NodeData>,
    root: Node,
}

impl DomStorage {
    pub fn new() -> DomStorage {
        let mut nodes = DenseSlotMap::with_key();
        let root = nodes.insert(NodeData {
            children: vec![],
            element: Element::Root,
        });
        DomStorage { nodes, root }
    }

    pub fn root(&self) -> Node {
        self.root
    }

    pub fn create_element(&mut self, element: impl Into<Element>) -> Node {
        self.nodes.insert(NodeData {
            children: vec![],
            element: element.into(),
        })
    }

    pub fn set_attribute(&mut self, node: Node, key: &str, value: Option<Cow<'static, str>>) {
        self.nodes
            .get_mut(node)
            .unwrap()
            .element
            .set_attribute(key, value);
    }

    pub fn clear_children(&mut self, node: Node) {
        self.nodes.get_mut(node).unwrap().children.clear();
    }

    pub fn add_child(&mut self, node: Node, child: Node) {
        self.nodes
            .get_mut(node)
            .unwrap()
            .children
            .push(NodeOrText::Node(child));
    }

    pub fn add_text(&mut self, node: Node, text: Cow<'static, str>) {
        self.nodes
            .get_mut(node)
            .unwrap()
            .children
            .push(NodeOrText::Text(text));
    }

    pub fn get_children(&self, node: Node) -> &[NodeOrText] {
        &self.nodes.get(node).unwrap().children[..]
    }

    pub fn get_element(&self, node: Node) -> &Element {
        &self.nodes.get(node).unwrap().element
    }

    pub fn pretty_print_xml(&self, node: Node) -> String {
        use std::fmt::Write;
        let data = self.nodes.get(node).unwrap();
        let mut out = String::new();
        writeln!(out, "<{}>", data.element.get_name()).unwrap();
        for child in data.children.iter() {
            match child {
                NodeOrText::Text(text) => writeln!(out, "  \"{}\"", text).unwrap(),
                NodeOrText::Node(node) => {
                    for line in self.pretty_print_xml(*node).lines() {
                        writeln!(out, "  {}", line).unwrap();
                    }
                }
            }
        }
        write!(out, "</{}>", data.element.get_name()).unwrap();
        out
    }
}
