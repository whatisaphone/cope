#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use std::{cell::Cell, panic, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn __start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let count = Rc::new(Cell::new(0));

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();

    let current = document.create_element("div")?;
    current.set_text_content(Some(&count.get().to_string()));
    body.append_with_node_1(&current)?;

    let decrement = document.create_element("button")?;
    decrement.add_event_listener_with_callback(
        "click",
        Closure::wrap(Box::new({
            let count = count.clone();
            let current = current.clone();
            move || {
                count.set(count.get() - 1);
                current.set_text_content(Some(&count.get().to_string()));
            }
        }) as Box<dyn Fn()>)
        .into_js_value()
        .unchecked_ref(),
    )?;
    decrement.set_text_content(Some("-"));
    body.append_with_node_1(&decrement)?;

    let increment = document.create_element("button")?;
    increment.add_event_listener_with_callback(
        "click",
        Closure::wrap(Box::new(move || {
            count.set(count.get() + 1);
            current.set_text_content(Some(&count.get().to_string()));
        }) as Box<dyn Fn()>)
        .into_js_value()
        .unchecked_ref(),
    )?;
    increment.set_text_content(Some("+"));
    body.append_with_node_1(&increment)?;
    Ok(())
}
