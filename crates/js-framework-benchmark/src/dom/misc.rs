use crate::reactive::react;
use std::cell::Cell;
use web_sys::Element;

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
