pub enum InputEvent {
    Hovered { state: State },
    MouseLeft { state: State },
    TextChar { character: char },
    Focused { state: State },
}

#[derive(Copy, Clone)]
pub enum State {
    Begin,
    End,
    Resume,
    Cancel,
}
