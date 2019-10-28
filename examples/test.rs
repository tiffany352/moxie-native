#[macro_use]
extern crate ui_lib;

use ui_lib::moxie::*;
use ui_lib::{start, App};

#[topo::nested]
fn foo() {
    mox! {
        <window>
            <view>
                "asdf"
            </view>
        </window>
    }
}

struct MyApp;

impl App for MyApp {}

fn main() {
    let mut runtime = ui_lib::UIRuntime::new(|| foo!());
    runtime.run_once();
    //start(MyApp);
}
