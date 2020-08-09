#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::reactive::{react, Atom};
use std::panic;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Element, HtmlInputElement};
use wee_alloc::WeeAlloc;

mod reactive;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

struct Todo {
    title: String,
    completed: bool,
}

#[wasm_bindgen(start)]
pub fn __start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let new_title = Atom::new(String::new());
    let todos = Atom::new(Vec::new());

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();

    let input: HtmlInputElement = document.create_element("input")?.unchecked_into();
    input.add_event_listener_with_callback(
        "input",
        Closure::wrap(Box::new({
            let new_title = new_title.clone();
            let input = input.clone();
            move || {
                new_title.set(input.value());
            }
        }) as Box<dyn Fn()>)
        .into_js_value()
        .unchecked_ref(),
    )?;
    body.append_with_node_1(&input)?;
    react({
        let new_title = new_title.clone();
        move || {
            input.set_value(&new_title.get());
        }
    });

    let add_button = document.create_element("button")?;
    add_button.add_event_listener_with_callback(
        "click",
        Closure::wrap(Box::new({
            let items = todos.clone();
            move || {
                items.get_mut().push(Todo {
                    title: new_title.get().clone(),
                    completed: false,
                });
                new_title.get_mut().clear();
            }
        }) as Box<dyn FnMut()>)
        .into_js_value()
        .unchecked_ref(),
    )?;
    add_button.set_text_content(Some("Add"));
    body.append_with_node_1(&add_button)?;

    let list = document.create_element("ul")?;
    body.append_with_node_1(&list)?;

    map(list, todos, move |todo| {
        let li = document.create_element("li").unwrap_throw();
        let label = document.create_element("label").unwrap_throw();
        let check: HtmlInputElement = document
            .create_element("input")
            .unwrap_throw()
            .unchecked_into();
        check.set_type("checkbox");
        check.set_checked(todo.completed);
        label.append_with_node_1(&check).unwrap_throw();
        label.append_with_str_2(" ", &todo.title).unwrap_throw();
        li.append_with_node_1(&label).unwrap_throw();
        li
    });
    Ok(())
}

fn map<T: 'static>(parent: Element, xs: Atom<Vec<T>>, f: impl Fn(&T) -> Element + 'static) {
    react(move || {
        while let Some(last_child) = parent.last_child() {
            parent.remove_child(&last_child).unwrap_throw();
        }

        for x in xs.get().iter() {
            parent.append_with_node_1(&f(x)).unwrap_throw();
        }
    });
}
