use wasm_bindgen::prelude::*;
use web_sys::{window, Element};

pub fn div() -> ElementBuilder {
    let document = window().unwrap_throw().document().unwrap_throw();
    let element = document.create_element("div").unwrap_throw();
    ElementBuilder { element }
}

#[must_use]
pub struct ElementBuilder {
    element: Element,
}

impl ElementBuilder {
    #[must_use]
    pub fn build(self) -> Element {
        self.element
    }
}

#[cfg(test)]
mod tests {
    use crate::elements::div;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn build() {
        let div = div().build();
        assert_eq!(div.node_name(), "DIV");
    }
}
