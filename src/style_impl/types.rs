pub struct Edges<Value> {
    pub left: Option<Value>,
    pub right: Option<Value>,
    pub top: Option<Value>,
    pub bottom: Option<Value>,
}

impl<Value> Edges<Value> {
    pub fn new() -> Edges<Value> {
        Edges {
            left: None,
            right: None,
            top: None,
            bottom: None,
        }
    }

    pub fn left(mut self, value: Value) -> Self {
        self.left = Some(value);
        self
    }

    pub fn right(mut self, value: Value) -> Self {
        self.right = Some(value);
        self
    }

    pub fn top(mut self, value: Value) -> Self {
        self.top = Some(value);
        self
    }

    pub fn bottom(mut self, value: Value) -> Self {
        self.bottom = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
