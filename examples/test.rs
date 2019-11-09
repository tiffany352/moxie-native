#[macro_use]
extern crate ui_lib;

use ui_lib::moxie::*;

#[topo::nested]
fn foo() {
    mox! {
        <window>
            <view foo="bar">
                "asdf"
            </view>
        </window>
    }
}

fn main() {
    let runtime = ui_lib::Runtime::new(|| foo!());
    runtime.start();
}
