pub enum InputEvent {
    Hovered { state: State },
    MouseLeft { state: State },
    CloseRequested,
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
        None
    }
}
