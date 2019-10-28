#[doc(hidden)]
pub use moxie::*;

use super::Node;
use {moxie, std::cell::Cell};

pub mod elements;

/// Create and mount a [DOM text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// This is normally called by the [`moxie::mox!`] macro.
#[topo::nested]
#[topo::from_env(parent: &MemoElement)]
pub fn text(s: impl ToString) {
    // TODO consider a ToOwned-based memoization API that's lower level?
    // memo_ref<Ref, Arg, Output>(reference: Ref, init: impl FnOnce(Arg) -> Output)
    // where Ref: ToOwned<Owned=Arg> + PartialEq, etcetcetc
    let text_node = memo!(s.to_string(), |s| parent.node.create_text_node(s));
    parent.ensure_child_attached(&text_node);
}

/// Create and mount an [HTML element](https://developer.mozilla.org/en-US/docs/Web/API/Element).
/// Called from the individual element macros, which in turn are normally called by the output of
/// the [`moxie::mox!`] macro.
///
/// The created DOM node is memoized at the bound callsite, allowing for subsequent re-executions to
/// be very cheap.
///
/// Mutation of the created element is performed during the `with_elem` closure via the provided
/// [`moxie_dom::MemoElement`] wrapper.
#[topo::nested]
#[topo::from_env(parent: &MemoElement)]
pub fn element<ChildRet>(
    ty: &'static str,
    with_elem: impl FnOnce(&MemoElement) -> ChildRet,
) -> ChildRet {
    let elem = memo!(ty, |ty| parent.node.create_element(ty));
    parent.ensure_child_attached(&elem);
    let elem = MemoElement::new(elem.into());
    with_elem(&elem)
}

/// A topologically-nested "incremental smart pointer" for an HTML element.
///
/// Created during execution of the (element) macro and the element-specific wrappers. Offers a
/// "stringly-typed" API for mutating the contained DOM nodes, adhering fairly closely to the
/// upstream web specs.
pub struct MemoElement {
    curr: Cell<Option<Node>>,
    node: Node,
}

impl MemoElement {
    pub fn new(node: Node) -> Self {
        Self {
            curr: Cell::new(None),
            node,
        }
    }

    /// Retrieves access to the raw HTML element underlying the (MemoElement).
    ///
    /// Because this offers an escape hatch around the memoized mutations, it should be used with
    /// caution. Also because of this, it has a silly name intended to loudly announce that
    /// care must be taken.
    ///
    /// Code called by the root function of your application will be run quite frequently and
    /// so the tools for memoization are important for keeping your application responsive. If you
    /// have legitimate needs for this API, please consider filing an issue with your use case so
    /// the maintainers of this crate can consider "official" ways to support it.
    pub fn raw_node_that_has_sharp_edges_please_be_careful(&self) -> Node {
        self.node.clone()
    }

    // FIXME this should be topo-nested
    // TODO and it should be able to express its slot as an annotation
    /// Declare an attribute of the element, mutating the actual element's attribute when the passed
    /// value changes.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and removes the attribute
    /// when `drop`ped, to ensure that the attribute is removed when this declaration is no longer
    /// referenced in the most recent (`moxie::Revision`).
    pub fn attr(&self, name: &'static str, value: impl ToString) -> &Self {
        topo::call!(slot: name, {
            memo_with!(
                value.to_string(),
                |v| {
                    self.node.set_attribute(name, v);
                    scopeguard::guard(self.node.clone(), move |elem| elem.remove_attribute(name))
                },
                |_| {}
            )
        });
        self
    }

    // FIXME this should be topo-nested
    /// Declare an event handler on the element.
    ///
    /// A guard value is stored as a resulting "effect" of the mutation, and removes the attribute
    /// when `drop`ped, to ensure that the attribute is removed when this declaration is no longer
    /// referenced in the most recent (`moxie::Revision`).
    ///
    /// Currently this is performed on every Revision, as changes to event handlers don't typically
    /// affect the debugging experience and have not yet shown up in performance profiles.
    /*pub fn on<Ev>(&self, callback: impl FnMut(Ev) + 'static) -> &Self
    where
        Ev: 'static + Event,
    {
        topo::call!(slot: Ev::NAME, {
            memo_with!(
                moxie::embed::Revision::current(),
                |_| EventHandle::new(&self.node, callback),
                |_| {}
            );
        });
        self
    }*/

    fn ensure_child_attached(&self, new_child: &Node) {
        let prev_sibling = self.curr.replace(Some(new_child.clone()));

        let existing = if prev_sibling.is_none() {
            self.node.first_child()
        } else {
            prev_sibling.and_then(|p| p.next_sibling())
        };

        if let Some(ref existing) = existing {
            if existing != new_child {
                self.node.replace_child(new_child, existing);
            }
        } else {
            self.node.append_child(new_child);
        }
    }

    /// Declare the inner contents of the element, usually declaring children within the inner
    /// scope. After any children have been run and their nodes attached, this clears any trailing
    /// child nodes to ensure the element's children are correct per the latest declaration.
    // FIXME this should be topo-nested
    pub fn inner<Ret>(&self, children: impl FnOnce() -> Ret) -> Ret {
        let elem = self.node.clone();
        let last_desired_child;
        let ret;
        topo::call!(
            {
                ret = children();

                // before this melement is dropped when the environment goes out of scope,
                // we need to get the last recorded child from this revision
                last_desired_child = topo::Env::expect::<MemoElement>().curr.replace(None);
            },
            env! {
                MemoElement => MemoElement::new(self.node.clone()),
            }
        );

        // if there weren't any children declared this revision, we need to make sure we clean up
        // any from the last revision
        if let Some(c) = last_desired_child {
            let mut next_to_remove = c.next_sibling();

            while let Some(to_remove) = next_to_remove {
                next_to_remove = to_remove.next_sibling();
                elem.remove_child(&to_remove);
            }
        } else {
            elem.clear_children();
        }

        ret
    }
}
