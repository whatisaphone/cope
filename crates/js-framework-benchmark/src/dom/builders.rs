use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;

macro_rules! define_element_builders {
    ($($name:ident => $type:ident),* $(,)?) => {
        $(
            #[allow(dead_code)]
            pub fn $name() -> ::web_sys::$type {
                let document = window().unwrap_throw().document().unwrap_throw();
                let element = document
                    .create_element(stringify!($name))
                    .unwrap_throw();
                element.unchecked_into()
            }
        )*
    };
}

define_element_builders!(
    a => Element,
    button => HtmlButtonElement,
    div => Element,
    h1 => Element,
    span => Element,
    table => Element,
    tbody => Element,
    tr => Element,
    td => Element,
);
