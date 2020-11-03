use cope::singleton::react;
use cope_dom::elements::ElementBuilder;
use std::cell::Cell;
use web_sys::Element;

pub trait ElementBuilderClass {
    fn class(self, name: &'static str, f: impl Fn() -> bool + 'static) -> Self;
}

impl<E: AsRef<Element>> ElementBuilderClass for ElementBuilder<E> {
    fn class(self, name: &'static str, f: impl Fn() -> bool + 'static) -> Self {
        let element = self.as_ref().as_ref().clone();
        toggle_class(element, name, f);
        self
    }
}

pub fn toggle_class(element: Element, class: &'static str, f: impl Fn() -> bool + 'static) {
    let previous = Cell::new(false);

    react(move || {
        let next = f();
        match (previous.get(), next) {
            (false, true) => {
                element.class_list().add_1(class).unwrap();
                previous.set(true);
            }
            (true, false) => {
                element.class_list().remove_1(class).unwrap();
                previous.set(false);
            }
            _ => {}
        }
    });
}
