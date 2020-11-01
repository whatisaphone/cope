use crate::sealed::Sealed;
use wasm_bindgen::prelude::*;
use web_sys::{window, Element};

#[must_use]
pub struct ElementBuilder {
    element: Element,
}

impl ElementBuilder {
    #[must_use]
    pub fn build(self) -> Element {
        self.element
    }

    pub fn class_name(self, value: &str) -> Self {
        self.element.set_class_name(value);
        self
    }

    pub fn child(self, value: impl ElementBuilderChild) -> Self {
        value.append(&self);
        self
    }
}

pub trait ElementBuilderChild: Sealed {
    fn append(self, parent: &ElementBuilder);
}

impl<T: ElementBuilderChild> Sealed for T {}

impl ElementBuilderChild for ElementBuilder {
    fn append(self, parent: &ElementBuilder) {
        let node = self.build();
        parent.element.append_with_node_1(&node).unwrap_throw();
    }
}

impl ElementBuilderChild for &str {
    fn append(self, parent: &ElementBuilder) {
        parent.element.append_with_str_1(self).unwrap_throw();
    }
}

macro_rules! define_builder {
    ($name:ident) => {
        pub fn $name() -> ElementBuilder {
            let document = window().unwrap_throw().document().unwrap_throw();
            let element = document.create_element(stringify!($name)).unwrap_throw();
            ElementBuilder { element }
        }
    };
}

define_builder!(a);
define_builder!(button);
define_builder!(div);
define_builder!(h1);
define_builder!(span);
define_builder!(table);
define_builder!(tbody);
define_builder!(td);
define_builder!(tr);

#[cfg(test)]
mod tests {
    use crate::elements::{div, span};
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn build() {
        let div = div().build();
        assert_eq!(div.node_name(), "DIV");
    }

    #[wasm_bindgen_test]
    fn child_element() {
        let div = div().child(span()).build();
        assert_eq!(div.outer_html(), "<div><span></span></div>");
    }

    #[wasm_bindgen_test]
    fn child_str() {
        let div = div().child("str").build();
        assert_eq!(div.outer_html(), "<div>str</div>");
    }
}
