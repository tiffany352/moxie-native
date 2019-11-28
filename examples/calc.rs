#![recursion_limit = "512"]

use moxie_native::prelude::*;

const ROW_STYLE: Style = define_style! {
    width: 100 vw,
    height: 20 vh,
    direction: horizontal,
    background_color: rgba(0, 0, 0, 0),
    text_size: 25 px,
    padding: 4 px,
};

const BUTTON_STYLE: Style = define_style! {
    width: 25 vw - 8 px,
    height: 20 vh - 8 px,
    background_color: rgb(200, 200, 200),
    padding: 10 px,
    margin: 4 px,

    if state: hover {
        background_color: rgb(238, 238, 238),
    }
};

#[derive(Clone, PartialEq, Copy)]
enum Message {
    Op(Op),
    Equ,
    Cls,
    Digit(i64),
}

#[derive(Clone, PartialEq, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn apply(self, left: i64, right: i64) -> i64 {
        match self {
            Op::Add => left + right,
            Op::Sub => left - right,
            Op::Mul => left * right,
            Op::Div => left / right,
        }
    }
}

#[derive(Clone, PartialEq)]
struct CalcState {
    previous: i64,
    value: i64,
    op: Option<Op>,
}

impl CalcState {
    fn new() -> CalcState {
        CalcState {
            previous: 0,
            value: 0,
            op: None,
        }
    }

    fn process(&self, message: Message) -> CalcState {
        match message {
            Message::Op(op) => {
                if let Some(prev_op) = self.op {
                    CalcState {
                        op: Some(op),
                        previous: prev_op.apply(self.previous, self.value),
                        value: 0,
                    }
                } else if self.value > 0 {
                    CalcState {
                        op: Some(op),
                        previous: self.value,
                        value: 0,
                    }
                } else {
                    CalcState {
                        op: Some(op),
                        ..*self
                    }
                }
            }
            Message::Equ => {
                if let Some(op) = self.op {
                    CalcState {
                        op: None,
                        previous: op.apply(self.previous, self.value),
                        value: 0,
                    }
                } else {
                    CalcState { ..*self }
                }
            }
            Message::Cls => CalcState::new(),
            Message::Digit(digit) => CalcState {
                value: self.value * 10 + digit,
                ..*self
            },
        }
    }

    fn display(&self) -> String {
        if let Some(op) = self.op {
            let op_str = match op {
                Op::Mul => "*",
                Op::Div => "/",
                Op::Add => "+",
                Op::Sub => "-",
            };
            format!("{} {} {}", self.previous, op_str, self.value)
        } else if self.value > 0 {
            format!("{}", self.value)
        } else {
            format!("{}", self.previous)
        }
    }
}

#[topo::nested]
fn calc_function(state: Key<CalcState>, message: Message) -> Node<Button> {
    let on_click = move |_event: &ClickEvent| state.update(|state| Some(state.process(message)));

    let text = match message {
        Message::Cls => "C".to_owned(),
        Message::Equ => "=".to_owned(),
        Message::Digit(digit) => digit.to_string(),
        Message::Op(Op::Add) => "+".to_owned(),
        Message::Op(Op::Sub) => "-".to_owned(),
        Message::Op(Op::Mul) => "*".to_owned(),
        Message::Op(Op::Div) => "/".to_owned(),
    };

    mox!(
        <button style={BUTTON_STYLE} on={on_click}>
            <span>{text}</span>
        </button>
    )
}

#[topo::nested]
fn app() -> Vec<Node<Window>> {
    let state: Key<CalcState> = state!(|| CalcState::new());

    vec![mox! {
        <window title="Moxie-Native Calculator">
            <view style={ROW_STYLE}>
                <span>{% "{}", state.display()}</span>
            </view>
            <view style={ROW_STYLE}>
                <calc_function _=(state.clone(), Message::Digit(7)) />
                <calc_function _=(state.clone(), Message::Digit(8)) />
                <calc_function _=(state.clone(), Message::Digit(9)) />
                <calc_function _=(state.clone(), Message::Op(Op::Mul)) />
            </view>
            <view style={ROW_STYLE}>
                <calc_function _=(state.clone(), Message::Digit(4)) />
                <calc_function _=(state.clone(), Message::Digit(5)) />
                <calc_function _=(state.clone(), Message::Digit(6)) />
                <calc_function _=(state.clone(), Message::Op(Op::Div)) />
            </view>
            <view style={ROW_STYLE}>
                <calc_function _=(state.clone(), Message::Digit(1)) />
                <calc_function _=(state.clone(), Message::Digit(2)) />
                <calc_function _=(state.clone(), Message::Digit(3)) />
                <calc_function _=(state.clone(), Message::Op(Op::Add)) />
            </view>
            <view style={ROW_STYLE}>
                <calc_function _=(state.clone(), Message::Digit(0)) />
                <calc_function _=(state.clone(), Message::Equ) />
                <calc_function _=(state.clone(), Message::Cls) />
                <calc_function _=(state.clone(), Message::Op(Op::Sub)) />
            </view>
        </window>
    }]
}

fn main() {
    let runtime = moxie_native::Runtime::new(|| app!());
    runtime.start();
}
