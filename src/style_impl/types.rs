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

pub struct Corners<Value> {
    pub top_left: Option<Value>,
    pub top_right: Option<Value>,
    pub bottom_left: Option<Value>,
    pub bottom_right: Option<Value>,
}

impl<Value> Corners<Value> {
    pub fn new() -> Corners<Value> {
        Corners {
            top_left: None,
            top_right: None,
            bottom_left: None,
            bottom_right: None,
        }
    }

    pub fn top_left(mut self, value: Value) -> Self {
        self.top_left = Some(value);
        self
    }

    pub fn top_right(mut self, value: Value) -> Self {
        self.top_right = Some(value);
        self
    }

    pub fn bottom_left(mut self, value: Value) -> Self {
        self.bottom_left = Some(value);
        self
    }

    pub fn bottom_right(mut self, value: Value) -> Self {
        self.bottom_right = Some(value);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
