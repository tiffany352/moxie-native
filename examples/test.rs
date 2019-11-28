#![recursion_limit = "256"]

use moxie_native::prelude::*;

const STYLES: &'static [&'static Style] = &[
    define_style! {
        class: container {
            padding: 10 px,
        }
    },
    define_style! {
        class: h1 {
            text_size: 20 px,
        }
    },
    define_style! {
        class: button {
            background_color: rgb(238, 238, 238),
        }
    },
    define_style! {
        class: view1 {
            background_color: rgb(255, 0, 0),
            display: block,
            width: 200 px,
            height: 200 px,
        }
    },
    define_style! {
        class: view2 {
            background_color: rgb(0, 255, 0),
            display: block,
            width: 250 px,
            height: 150 px,
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
