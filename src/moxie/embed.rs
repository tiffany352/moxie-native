//! Embedding APIs offering finer-grained control over execution of the runtime.

use {
    super::MemoElement,
    crate::Node,
    moxie::{embed::Runtime, topo},
};

/// Wrapper around `moxie::embed::Runtime` which provides an `Env` for building trees of DOM nodes.
#[must_use]
pub struct UIRuntime(Runtime<Box<dyn FnMut()>, ()>);

impl UIRuntime {
    /// Construct a new `UIRuntime` which will maintain the children of the provided `parent`.
    ///
    /// On its own, a `UIRuntime` is inert and must either have its `run_once` method called when
    /// a re-render is needed, or be scheduled with [`UIRuntime::animation_frame_scheduler`].
    pub fn new(mut root: impl FnMut() + 'static) -> Self {
        let parent = Node::create_root();
        UIRuntime(Runtime::new(Box::new(move || {
            topo::call!(
                { root() },
                env! {
                    MemoElement => MemoElement::new(parent.clone()),
                }
            )
        })))
    }

    /// Run the root function in a fresh [moxie::Revision]. See [moxie::embed::Runtime::run_once]
    /// for details.
    pub fn run_once(&mut self) {
        self.0.run_once();
    }
}
