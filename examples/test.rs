#![recursion_limit = "256"]

use moxie_native::dom::*;
use moxie_native::moxie::*;
use moxie_native::*;

const STYLES: &'static [&'static Style] = &[
    style! {
        (class_name == "container") => {
            padding: Value { pixels: 10.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
        }
    },
    style! {
        (class_name == "h1") => {
            text_size: Value { pixels: 20.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
        }
    },
    style! {
        (class_name == "button") => {
            background_color: Color { red: 238, green: 238, blue: 238, alpha: 255 },
        }
    },
    style! {
        (class_name == "view1") => {
            background_color: Color { red: 255, green: 0, blue: 0, alpha: 255},
            display: Display::Block,
            width: Value { pixels: 200.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
            height: Value { pixels: 200.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
        }
    },
    style! {
        (class_name == "view2") => {
            background_color: Color { red: 0, green: 255, blue: 0, alpha: 255},
            display: Display::Block,
            width: Value { pixels: 250.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
            height: Value { pixels: 150.0, ems: 0.0, view_width: 0.0, view_height: 0.0, },
        }
    },
];

#[topo::nested]
fn foo() -> Vec<Node<Window>> {
    let click_count: Key<u32> = state!(|| 0);

    let click_count2 = click_count.clone();
    let on_click = move |_: &ClickEvent| {
        click_count2.update(|count| Some(count + 1));
    };

    vec![mox! {
        <window styles={STYLES}>
            <view class_name="container">
                <span class_name="h1">
                    "Bigger Te" "xt"
                </span>
                <span>
                    "foo bar baz"
                    " the quick brown fox jumps over the lazy dog"
                </span>
                <button on={on_click} class_name="button">
                    <span>
                        "Clicked " {% "{}", click_count} " times)"
                    </span>
                </button>
                <view class_name="view1"></view>
                <view class_name="view2"></view>
            </view>
        </window>
    }]
}

fn main() {
    let runtime = moxie_native::Runtime::new(|| foo!());
    runtime.start();
}
