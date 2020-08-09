#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::reactive::{react, Atom};
use std::{convert::TryInto, panic};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Element, HtmlInputElement};
use wee_alloc::WeeAlloc;

mod reactive;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

// TODO: get rid of derives if possible
#[derive(Clone, Eq, PartialEq)]
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

// TODO: get rid of bounds if possible
fn map<T: Clone + Eq + 'static>(
    parent: Element,
    xs: Atom<Vec<T>>,
    mut f: impl FnMut(&T) -> Element + 'static,
) {
    let mut cache = Vec::new();

    react(move || {
        let xs = xs.get();

        // This is the dumbest reconciler ever created
        let mut mutations = Vec::new();
        for index in 0..xs.len().max(cache.len()) {
            match (cache.get(index), xs.get(index)) {
                (None, None) => unreachable!(),
                (Some(_), None) => {
                    mutations.push(ListMutation::Remove(index));
                }
                (None, Some(_)) => {
                    mutations.push(ListMutation::Insert(index));
                }
                (Some(prev), Some(next)) if prev != next => {
                    mutations.push(ListMutation::Remove(index));
                    mutations.push(ListMutation::Insert(index));
                }
                (Some(_), Some(_)) => {}
            }
        }

        for mutation in &mutations {
            match mutation {
                &ListMutation::Remove(index) => {
                    cache.remove(index);
                }
                &ListMutation::Insert(index) => {
                    cache.insert(index, xs[index].clone());
                }
            }
        }

        for mutation in mutations {
            match mutation {
                ListMutation::Remove(index) => {
                    parent
                        .children()
                        .item(index.try_into().unwrap())
                        .unwrap_throw()
                        .remove();
                }
                ListMutation::Insert(index) => {
                    let reference = parent.children().item(index.try_into().unwrap());
                    parent
                        .insert_before(&f(&xs[index]), reference.map(<_>::unchecked_into).as_ref())
                        .unwrap_throw();
                }
            }
        }
    });
}

enum ListMutation {
    Remove(usize),
    Insert(usize),
}
