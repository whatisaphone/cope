use crate::reactive::{ListMutation, TrackingVec};
use cope::singleton::react;
use cope_dom::elements::ElementBuilder;
use std::convert::TryInto;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::Element;

// TODO: generalize this
pub trait ElementBuilderChildren {
    fn children<T, F>(self, list: MapChildren<T, F>) -> Self
    where
        T: 'static,
        F: Fn(&T) -> ElementBuilder<Element> + 'static;
}

impl<E: AsRef<Element>> ElementBuilderChildren for ElementBuilder<E> {
    fn children<T, F>(self, list: MapChildren<T, F>) -> Self
    where
        T: 'static,
        F: Fn(&T) -> ElementBuilder<Element> + 'static,
    {
        list.begin(self.as_ref().as_ref().clone());
        self
    }
}

pub fn map_children<T, F>(xs: TrackingVec<T>, f: F) -> MapChildren<T, F>
where
    F: Fn(&T) -> ElementBuilder<Element> + 'static,
{
    MapChildren { xs, f }
}

pub struct MapChildren<T, F> {
    xs: TrackingVec<T>,
    f: F,
}

impl<T, F> MapChildren<T, F>
where
    T: 'static,
    F: Fn(&T) -> ElementBuilder<Element> + 'static,
{
    fn begin(self, parent: Element) {
        let Self { xs, f } = self;

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
                            .insert_before(&f(&*item).build(), reference.map(<_>::into).as_ref())
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
}
