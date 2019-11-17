#[macro_use]
extern crate ui_lib;

use moxie::*;
use ui_lib::dom::view::TestEvent;
use ui_lib::dom::*;

#[topo::nested]
fn foo() -> Vec<Node<Window>> {
    vec![mox! {
        <window>
            <view padding="10" on={|_:&TestEvent| ()}>
                <span>
                    "as" "df"
                    " foo bar baz"
                    " the quick brown fox jumps over the lazy dog"
                </span>
                <view color="255,0,0,255" width="200" height="200"></view>
                <view color="0,255,0,255" width="250" height="150"></view>
            </view>
        </window>
    }]
}

fn main() {
    let runtime = ui_lib::Runtime::new(|| foo!());
    runtime.start();
}
