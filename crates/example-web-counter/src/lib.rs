#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::reactive::{react, Atom};
use std::panic;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use wee_alloc::WeeAlloc;

mod reactive;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn __start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let count = Atom::new(0);

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();

    let current = document.create_element("div")?;
    body.append_with_node_1(&current)?;

    react({
        let count = count.clone();
        move || {
            current.set_text_content(Some(&count.get().to_string()));
        }
    });

    let decrement = document.create_element("button")?;
    decrement.add_event_listener_with_callback(
        "click",
        Closure::wrap(Box::new({
            let count = count.clone();
            move || {
                let new_count = *count.get() - 1;
                count.set(new_count);
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
            let new_count = *count.get() + 1;
            count.set(new_count);
        }) as Box<dyn Fn()>)
        .into_js_value()
        .unchecked_ref(),
    )?;
    increment.set_text_content(Some("+"));
    body.append_with_node_1(&increment)?;
    Ok(())
}
