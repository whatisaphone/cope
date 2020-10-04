use crate::reactive::{ListMutation, TrackingVec};
use cope::singleton::react;
use std::convert::TryInto;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Element, Node};

pub fn tracked_map<T: 'static>(
    xs: TrackingVec<T>,
    parent: Element,
    mut f: impl FnMut(&T) -> Node + 'static,
) {
    react(move || {
        // Re-run whenever `xs` changes
        xs.get(0);

        for mutation in xs.mutations.borrow_mut().drain(..) {
            // NOTE: This is an incomplete implementation but good enough for proof of
            // concept.
            match mutation {
                ListMutation::Insert(index) => {
                    let item = xs.get(index).unwrap();
                    let reference = parent.children().item(index.try_into().unwrap());
                    parent
                        .insert_before(&f(&*item), reference.map(<_>::unchecked_into).as_ref())
                        .unwrap_throw();
                }
                ListMutation::Remove(index) => {
                    parent
                        .children()
                        .item(index.try_into().unwrap())
                        .unwrap_throw()
                        .remove();
                }
            }
        }
    });
}
