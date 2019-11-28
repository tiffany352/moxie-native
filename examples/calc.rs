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

    fn digit(&self, digit: i64) -> CalcState {
        CalcState {
            value: self.value * 10 + digit,
            ..*self
        }
    }

    fn op(&self, op: Op) -> CalcState {
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

    fn equals(&self) -> CalcState {
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
fn app() -> Vec<Node<Window>> {
    let state: Key<CalcState> = state!(|| CalcState::new());

    let state_copy = state.clone();
    let make_digit_handler = move |digit: i64| {
        let state_copy = state_copy.clone();
        move |_: &ClickEvent| {
            state_copy.update(|state| Some(state.digit(digit)));
        }
    };

    let state_copy = state.clone();
    let make_op_handler = move |op: Op| {
        let state_copy = state_copy.clone();
        move |_: &ClickEvent| {
            state_copy.update(|state| Some(state.op(op)));
        }
    };

    let state_copy = state.clone();
    let on_clear = move |_: &ClickEvent| {
        state_copy.update(|_state| Some(CalcState::new()));
    };

    let state_copy = state.clone();
    let on_equals = move |_: &ClickEvent| {
        state_copy.update(|state| Some(state.equals()));
    };

    vec![mox! {
        <window title="Moxie-Native Calculator">
            <view style={ROW_STYLE}>
                <span>{% "{}", state.display()}</span>
            </view>
            <view style={ROW_STYLE}>
                <button style={BUTTON_STYLE} on={make_digit_handler(7)}>
                    <span>"7"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_digit_handler(8)}>
                    <span>"8"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_digit_handler(9)}>
                    <span>"9"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_op_handler(Op::Mul)}>
                    <span>"*"</span>
                </button>
            </view>
            <view style={ROW_STYLE}>
                <button style={BUTTON_STYLE} on={make_digit_handler(4)}>
                    <span>"4"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_digit_handler(5)}>
                    <span>"5"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_digit_handler(6)}>
                    <span>"6"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_op_handler(Op::Div)}>
                    <span>"/"</span>
                </button>
            </view>
            <view style={ROW_STYLE}>
                <button style={BUTTON_STYLE} on={make_digit_handler(1)}>
                    <span>"1"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_digit_handler(2)}>
                    <span>"2"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_digit_handler(3)}>
                    <span>"3"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_op_handler(Op::Add)}>
                    <span>"+"</span>
                </button>
            </view>
            <view style={ROW_STYLE}>
                <button style={BUTTON_STYLE} on={make_digit_handler(0)}>
                    <span>"0"</span>
                </button>
                <button style={BUTTON_STYLE} on={on_equals}>
                    <span>"="</span>
                </button>
                <button style={BUTTON_STYLE} on={on_clear}>
                    <span>"C"</span>
                </button>
                <button style={BUTTON_STYLE} on={make_op_handler(Op::Sub)}>
                    <span>"-"</span>
                </button>
            </view>
        </window>
    }]
}

fn main() {
    let runtime = moxie_native::Runtime::new(|| app!());
    runtime.start();
}
