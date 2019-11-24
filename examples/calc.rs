#![recursion_limit = "512"]

use moxie_native::dom::*;
use moxie_native::moxie::*;
use moxie_native::*;

const STYLES: &'static [&'static Style] = &[
    style! {
        (class_name == "row") => {
            width: Value { pixels: 0.0, ems: 0.0, view_width: 1.0, view_height: 0.0, },
            height: Value { pixels: 0.0, ems: 0.0, view_width: 0.0, view_height: 0.2, },
            direction: Direction::Horizontal,
            background_color: Color { red: 0, green: 0, blue: 0, alpha: 0 },
            text_size: Value { pixels: 25.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
            padding: Value { pixels: 4.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
        }
    },
    style! {
        (class_name == "button") => {
            width: Value { pixels: -8.0, ems: 0.0, view_width: 0.25, view_height: 0.0, },
            height: Value { pixels: -8.0, ems: 0.0, view_width: 0.0, view_height: 0.2, },
            background_color: Color { red: 200, green: 200, blue: 200, alpha: 255 },
            padding: Value { pixels: 10.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
            margin: Value { pixels: 4.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
        }
    },
];

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
        <window styles={STYLES} title="Moxie-Native Calculator">
            <view class_name="row">
                <span>{% "{}", state.display()}</span>
            </view>
            <view class_name="row">
                <button class_name="button" on={make_digit_handler(7)}>
                    <span>"7"</span>
                </button>
                <button class_name="button" on={make_digit_handler(8)}>
                    <span>"8"</span>
                </button>
                <button class_name="button" on={make_digit_handler(9)}>
                    <span>"9"</span>
                </button>
                <button class_name="button" on={make_op_handler(Op::Mul)}>
                    <span>"*"</span>
                </button>
            </view>
            <view class_name="row">
                <button class_name="button" on={make_digit_handler(4)}>
                    <span>"4"</span>
                </button>
                <button class_name="button" on={make_digit_handler(5)}>
                    <span>"5"</span>
                </button>
                <button class_name="button" on={make_digit_handler(6)}>
                    <span>"6"</span>
                </button>
                <button class_name="button" on={make_op_handler(Op::Div)}>
                    <span>"/"</span>
                </button>
            </view>
            <view class_name="row">
                <button class_name="button" on={make_digit_handler(1)}>
                    <span>"1"</span>
                </button>
                <button class_name="button" on={make_digit_handler(2)}>
                    <span>"2"</span>
                </button>
                <button class_name="button" on={make_digit_handler(3)}>
                    <span>"3"</span>
                </button>
                <button class_name="button" on={make_op_handler(Op::Add)}>
                    <span>"+"</span>
                </button>
            </view>
            <view class_name="row">
                <button class_name="button" on={make_digit_handler(0)}>
                    <span>"0"</span>
                </button>
                <button class_name="button" on={on_equals}>
                    <span>"="</span>
                </button>
                <button class_name="button" on={on_clear}>
                    <span>"C"</span>
                </button>
                <button class_name="button" on={make_op_handler(Op::Sub)}>
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
