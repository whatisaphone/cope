use crate::reactive::{reconcile, Atom};
use std::convert::TryInto;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::Element;

// TODO: get rid of bounds
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
