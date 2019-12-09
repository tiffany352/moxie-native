pub enum InputEvent {
    Hovered { state: State },
    MouseLeft { state: State },
}

#[derive(Copy, Clone)]
pub enum State {
    Begin,
    End,
    Resume,
    Cancel,
}

impl InputEvent {
    pub fn get_position(&self) -> Option<(f32, f32)> {
        match self {
            _ => None,
        }
    }
}
