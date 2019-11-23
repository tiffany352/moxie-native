#![recursion_limit = "256"]

use moxie_native::dom::*;
use moxie_native::moxie::*;
use moxie_native::*;

#[topo::nested]
fn foo() -> Vec<Node<Window>> {
    let click_count: Key<u32> = state!(|| 0);

    let click_count2 = click_count.clone();
    let on_click = move |_: &ClickEvent| {
        click_count2.update(|count| Some(count + 1));
    };

    vec![mox! {
        <window>
            <view padding=10.0>
                <span text_size=20.0>
                    "Bigger Te" "xt"
                </span>
                <span>
                    "foo bar baz"
                    " the quick brown fox jumps over the lazy dog"
                </span>
                <button on={on_click} color={Color::new(238,238,238,255)}>
                    <span>
                        "Clicked " {% "{}", click_count} " times)"
                    </span>
                </button>
                <view color={Color::new(255,0,0,255)} width=200.0 height=200.0></view>
                <view color={Color::new(0,255,0,255)} width=250.0 height=150.0></view>
            </view>
        </window>
    }]
}

fn main() {
    let runtime = moxie_native::Runtime::new(|| foo!());
    runtime.start();
}
