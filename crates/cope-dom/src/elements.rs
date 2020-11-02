use crate::sealed::Sealed;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Element, HtmlButtonElement};

#[must_use]
pub struct ElementBuilder<E> {
    element: E,
}

impl<E> ElementBuilder<E> {
    pub fn build(self) -> E {
        self.element
    }
}

impl<E> AsRef<E> for ElementBuilder<E> {
    fn as_ref(&self) -> &E {
        &self.element
    }
}

impl<E: AsRef<Element>> ElementBuilder<E> {
    pub fn child(self, value: impl ElementChild) -> Self {
        value.append(&self);
        self
    }

    pub fn class_name(self, value: &str) -> Self {
        self.element.as_ref().set_class_name(value);
        self
    }

    pub fn id(self, value: &str) -> Self {
        self.element.as_ref().set_id(value);
        self
    }

    pub fn on_click(self, f: impl FnMut() + 'static) -> Self {
        self.element
            .as_ref()
            .add_event_listener_with_callback(
                "click",
                Closure::wrap(Box::new(f) as Box<dyn FnMut()>)
                    .into_js_value()
                    .unchecked_ref(),
            )
            .unwrap_throw();
        self
    }
}

pub trait ElementChild: Sealed {
    fn append<P: AsRef<Element>>(self, parent: &ElementBuilder<P>);
}

impl<T: ElementChild> Sealed for T {}

impl<E: AsRef<Element>> ElementChild for ElementBuilder<E> {
    fn append<P: AsRef<Element>>(self, parent: &ElementBuilder<P>) {
        let parent = parent.element.as_ref();
        let node = self.build();
        parent.append_with_node_1(node.as_ref()).unwrap_throw();
    }
}

impl<T: AsRef<str>> ElementChild for T {
    fn append<P: AsRef<Element>>(self, parent: &ElementBuilder<P>) {
        parent
            .element
            .as_ref()
            .append_with_str_1(self.as_ref())
            .unwrap_throw();
    }
}

impl<E: AsRef<HtmlButtonElement>> ElementBuilder<E> {
    pub fn type_(self, value: &str) -> Self {
        self.element.as_ref().set_type(value);
        self
    }
}

macro_rules! define_builder {
    ($name:ident => $type:ident) => {
        pub fn $name() -> ElementBuilder<::web_sys::$type> {
            let document = window().unwrap_throw().document().unwrap_throw();
            let element = document.create_element(stringify!($name)).unwrap_throw();
            ElementBuilder {
                element: element.unchecked_into(),
            }
        }
    };
}

define_builder!(a => Element);
define_builder!(button => HtmlButtonElement);
define_builder!(div => Element);
define_builder!(h1 => Element);
define_builder!(span => Element);
define_builder!(table => Element);
define_builder!(tbody => Element);
define_builder!(td => Element);
define_builder!(tr => Element);

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
