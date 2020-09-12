use crate::reactive::{react, reconcile, Atom, ListMutation, TrackingVec};
use std::convert::TryInto;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Element, Node};

// TODO: get rid of bounds
#[allow(dead_code)]
pub fn map<T: Clone + Eq + 'static>(
    xs: Atom<Vec<T>>,
    parent: Element,
    f: impl FnMut(&T) -> Element + 'static,
) {
    struct Sink<F> {
        parent: Element,
        f: F,
    }

    impl<T, F: FnMut(&T) -> Element> reconcile::Sink<T> for Sink<F> {
        fn remove(&mut self, index: usize) {
            self.parent
                .children()
                .item(index.try_into().unwrap())
                .unwrap_throw()
                .remove();
        }

        fn insert(&mut self, index: usize, item: &T) {
            let reference = self.parent.children().item(index.try_into().unwrap());
            self.parent
                .insert_before(&(self.f)(item), reference.map(<_>::unchecked_into).as_ref())
                .unwrap_throw();
        }
    }

    reconcile::track(xs, Sink { parent, f });
}

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
