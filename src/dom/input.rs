pub enum InputEvent {
    MouseLeft { state: State, x: f32, y: f32 },
    MouseMove { x: f32, y: f32 },
}

#[derive(Copy, Clone)]
pub enum State {
    Begin,
    End,
    Resume,
    Cancel,
}

#[derive(enumset::EnumSetType)]
pub enum ElementState {
    Hovered,
    Pressed,
    Focused,
}

impl InputEvent {
    pub fn get_position(&self) -> Option<(f32, f32)> {
        match self {
            InputEvent::MouseLeft { x, y, .. } => Some((*x, *y)),
            InputEvent::MouseMove { x, y } => Some((*x, *y)),
        }
    }
}
