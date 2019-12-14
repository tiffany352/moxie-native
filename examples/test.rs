#![recursion_limit = "256"]

use moxie_native::prelude::*;

define_style! {
    static CONTAINER_STYLE = {
        padding: 10 px,
    };

    static H1_STYLE = {
        text_size: 20 px,
    };

    static BUTTON_STYLE = {
        background_color: rgb(238, 238, 238),
    };

    static VIEW1_STYLE = {
        background_color: rgb(255, 0, 0),
        display: block,
        width: 200 px,
        height: 200 px,
    };

    static VIEW2_STYLE = {
        background_color: rgb(0, 255, 0),
        display: block,
        width: 250 px,
        height: 150 px,
    };

    static SQUARE_STYLE = {
        background_color: rgb(0, 0, 0),
        display: block,
        width: 10 px,
        height: 10 px,
    };

    static TEXTFIELD_STYLE = {
        border_color: rgb(160, 160, 160),
        background_color: rgb(255, 255, 255),
        border_radius: 4 px,
        border_thickness: 1 px,
        height: 30 px,
        width: 300 px,

        if state: focus {
            border_color: rgb(160, 160, 255),
        }
    };
}

#[topo::nested]
fn foo() -> Node<App> {
    let click_count: Key<u32> = state!(|| 0);

    let click_count2 = click_count.clone();
    let on_click = move |_: &ClickEvent| {
        click_count2.update(|count| Some(count + 1));
    };

    mox! {
        <app>
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
                    <textfield style={TEXTFIELD_STYLE} />
                    <view style={VIEW1_STYLE}></view>
                    <view style={VIEW2_STYLE}></view>
                </view>
            </window>
        </app>
    }
}

fn main() {
    let runtime = moxie_native::Runtime::new(|| foo!());
    runtime.start();
}
