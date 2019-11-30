#![recursion_limit = "256"]

use moxie_native::prelude::*;

const CONTAINER_STYLE: Style = define_style! {
    padding: 10 px,
};

const H1_STYLE: Style = define_style! {
    text_size: 20 px,
};

const BUTTON_STYLE: Style = define_style! {
    background_color: rgb(238, 238, 238),
};

const VIEW1_STYLE: Style = define_style! {
    background_color: rgb(255, 0, 0),
    display: block,
    width: 200 px,
    height: 200 px,
};

const VIEW2_STYLE: Style = define_style! {
    background_color: rgb(0, 255, 0),
    display: block,
    width: 250 px,
    height: 150 px,
};

const SQUARE_STYLE: Style = define_style! {
    background_color: rgb(0, 0, 0),
    display: block,
    width: 10 px,
    height: 10 px,
};

#[topo::nested]
fn foo() -> Vec<Node<Window>> {
    let click_count: Key<u32> = state!(|| 0);

    let click_count2 = click_count.clone();
    let on_click = move |_: &ClickEvent| {
        click_count2.update(|count| Some(count + 1));
    };

    vec![mox! {
        <window>
            <view style={CONTAINER_STYLE}>
                <span style={H1_STYLE}>
                    "Bigger Te" "xt"
                </span>
                <span>
                    "foo bar baz"
                    " the quick brown fox "<span style={H1_STYLE}>"jumps"</span><view style={SQUARE_STYLE}></view>" over the lazy dog"
                </span>
                <button on={on_click} style={BUTTON_STYLE}>
                    <span>
                        "Clicked " {% "{}", click_count} " times)"
                    </span>
                </button>
                <view style={VIEW1_STYLE}></view>
                <view style={VIEW2_STYLE}></view>
            </view>
        </window>
    }]
}

fn main() {
    let runtime = moxie_native::Runtime::new(|| foo!());
    runtime.start();
}
