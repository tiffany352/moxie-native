use ui_lib::{start, App};

struct MyApp;

impl App for MyApp {}

fn main() {
    start(MyApp);
}
