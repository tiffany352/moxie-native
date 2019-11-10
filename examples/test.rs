#[macro_use]
extern crate ui_lib;

use ui_lib::dom::view::TestEvent;
use ui_lib::moxie::*;

#[topo::nested]
fn foo() {
    mox! {
        <window>
            <view padding="10" on={|_:TestEvent| ()}>
                <view color="255,0,0,255" width="200" height="200"></view>
                <view color="0,255,0,255" width="250" height="150"></view>
            </view>
        </window>
    }
}

fn main() {
    let runtime = ui_lib::Runtime::new(|| foo!());
    runtime.start();
}
