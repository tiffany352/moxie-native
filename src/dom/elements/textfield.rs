use crate::dom::element::{Element, ElementStates, HasEvent, NoChildren};
use crate::dom::input::{InputEvent, State};
use crate::dom::{AttrStyle, AttrTextState, ClickEvent, TextEvent};
use crate::style::Style;
use crate::util::event_handler::EventHandler;
use moxie::{__once_impl, once};
use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

pub trait TextStateMachine: fmt::Debug + 'static {
    fn text(&self) -> String;
    fn cursor_pos(&self) -> usize;

    fn input_text(&mut self, character: char);
}

#[derive(Debug)]
struct HistoryItem {
    text: String,
    cursor_pos: usize,
}

#[derive(Debug)]
struct TextState {
    history: Vec<HistoryItem>,
    current: usize,
}

impl TextState {
    pub fn new(initial: String) -> TextState {
        TextState {
            history: vec![HistoryItem {
                text: initial,
                cursor_pos: 0,
            }],
            current: 0,
        }
    }

    fn current(&self) -> &HistoryItem {
        &self.history[self.current]
    }

    fn append(&mut self, func: impl FnOnce(&String, usize) -> (String, usize)) {
        let (text, cursor_pos) = {
            let current = self.current();
            func(&current.text, current.cursor_pos)
        };
        println!("append {:?}", text);
        self.history.truncate(self.current + 1);
        self.current += 1;
        println!("??? {:?}", self);
        self.history.push(HistoryItem { text, cursor_pos })
    }
}

impl TextStateMachine for TextState {
    fn text(&self) -> String {
        self.current().text.clone()
    }

    fn cursor_pos(&self) -> usize {
        self.current().cursor_pos
    }

    fn input_text(&mut self, character: char) {
        self.append(|text, cursor_pos| {
            let mut text = text.clone();
            text.insert(cursor_pos, character);
            (text, cursor_pos + character.len_utf8())
        });
    }
}

/*impl fmt::Debug for TextState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("TextState")
            .field("current_text", &self.current().text)
            .field("cursor_pos", &self.current().cursor_pos)
            .finish()
    }
}*/

#[derive(Clone)]
pub struct TextStateRef(Rc<RefCell<dyn TextStateMachine>>);

impl TextStateRef {
    pub fn with_text(text: String) -> TextStateRef {
        TextStateRef::with_state(TextState::new(text))
    }

    pub fn with_state(state: impl TextStateMachine) -> TextStateRef {
        TextStateRef(once!(|| Rc::new(RefCell::new(state))))
    }

    pub fn text(&self) -> String {
        self.0.borrow().text()
    }

    pub fn cursor_pos(&self) -> usize {
        self.0.borrow().cursor_pos()
    }
}

impl fmt::Debug for TextStateRef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl Default for TextStateRef {
    fn default() -> Self {
        TextStateRef::with_text("".to_owned())
    }
}

impl PartialEq for TextStateRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Deref for TextStateRef {
    type Target = RefCell<dyn TextStateMachine>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<State> From<State> for TextStateRef
where
    State: TextStateMachine,
{
    fn from(state: State) -> Self {
        TextStateRef::with_state(state)
    }
}

/// Corresponds to <textfield>. This element accepts text input.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct TextField {
    style: Option<Style>,
    state: TextStateRef,
}

element_attributes! {
    TextField {
        style: AttrStyle,
        state: AttrTextState,
    }
}

element_handlers! {
    TextFieldHandlers for TextField {
        on_click: ClickEvent,
        on_text: TextEvent,
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct TextFieldStates {
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl ElementStates for TextFieldStates {
    fn has_state(&self, name: &str) -> bool {
        match name {
            "hover" => self.hovered,
            "press" => self.pressed,
            "focus" => self.focused,
            _ => false,
        }
    }
}

impl Element for TextField {
    type Child = NoChildren;
    type Handlers = TextFieldHandlers;
    type States = TextFieldStates;

    const ELEMENT_NAME: &'static str = "textfield";

    fn interactive(&self) -> bool {
        true
    }

    fn focusable(&self) -> bool {
        true
    }

    fn process(
        &self,
        states: Self::States,
        handlers: &mut Self::Handlers,
        event: &InputEvent,
    ) -> (bool, Self::States) {
        match event {
            InputEvent::Hovered {
                state: State::Begin,
            } => (
                true,
                TextFieldStates {
                    hovered: true,
                    ..states
                },
            ),
            InputEvent::Hovered { state: State::End } => (
                true,
                TextFieldStates {
                    hovered: false,
                    ..states
                },
            ),
            InputEvent::MouseLeft {
                state: State::Begin,
                ..
            } => (
                true,
                TextFieldStates {
                    pressed: true,
                    ..states
                },
            ),
            InputEvent::MouseLeft {
                state: State::End, ..
            } if states.pressed => (
                true,
                TextFieldStates {
                    pressed: false,
                    focused: true,
                    ..states
                },
            ),
            InputEvent::TextChar { character } => {
                let event = {
                    let mut state = self.state.borrow_mut();
                    state.input_text(*character);

                    let text = state.text();
                    let cursor_pos = state.cursor_pos();
                    TextEvent { text, cursor_pos }
                };
                handlers.on_text.invoke(&event);
                (true, states)
            }
            _ => (false, states),
        }
    }

    fn content(&self) -> Option<String> {
        println!("{:?}", self.state);
        Some(self.state.text())
    }

    fn style(&self) -> Option<Style> {
        self.style
    }
}
