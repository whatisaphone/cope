#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::dom::{button, div, h1};
use std::panic;
use wasm_bindgen::prelude::*;
use web_sys::{window, Element};
use wee_alloc::WeeAlloc;

mod dom;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn __start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();

    body.append_with_node_1(&app()).unwrap_throw();
}

fn app() -> Element {
    let container = div();
    container.class_list().add_1("container").unwrap_throw();

    let jumbotron = jumbotron();
    container.append_with_node_1(&jumbotron).unwrap_throw();

    container
}

fn jumbotron() -> Element {
    let jumbotron = div();
    jumbotron.class_list().add_1("jumbotron").unwrap_throw();

    let row = div();
    row.class_list().add_1("row").unwrap_throw();
    jumbotron.append_with_node_1(&row).unwrap_throw();

    let left_col = div();
    left_col.class_list().add_1("col-md-6").unwrap_throw();
    row.append_with_node_1(&left_col).unwrap_throw();

    let h1 = h1();
    h1.set_text_content(Some("cope"));
    left_col.append_with_node_1(&h1).unwrap_throw();

    let right_col = div();
    right_col.class_list().add_1("col-md-6").unwrap_throw();
    row.append_with_node_1(&right_col).unwrap_throw();

    let row = div();
    row.class_list().add_1("row").unwrap_throw();
    right_col.append_with_node_1(&row).unwrap_throw();

    let create_1000_rows = header_button("Create 1,000 rows");
    row.append_with_node_1(&create_1000_rows).unwrap_throw();

    jumbotron
}

fn header_button(text: &str) -> Element {
    let col = div();
    col.class_list()
        .add_2("col-sm-6", "smallpad")
        .unwrap_throw();

    let button = button();
    button
        .class_list()
        .add_3("btn", "btn-primary", "btn-block")
        .unwrap_throw();
    button.set_type("button");
    button.set_text_content(Some(text));
    col.append_with_node_1(&button).unwrap_throw();

    col
}
