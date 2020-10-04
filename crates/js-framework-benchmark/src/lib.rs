#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::{
    dom::{
        builders::{a, button, div, h1, span, table, tbody, td, tr},
        list::tracked_map,
        misc::toggle_class,
    },
    reactive::{batch, react, Atom, TrackingVec},
};
use js_sys::Math;
use std::{
    cell::{Cell, RefCell},
    panic,
    rc::Rc,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Element, Event, Node};
use wee_alloc::WeeAlloc;

mod dom;
mod reactive;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

#[derive(Default)]
struct State {
    next_id: Cell<usize>,
    data: TrackingVec<Rc<Item>>,
}

struct Item {
    id: usize,
    label: Atom<String>,
}

#[wasm_bindgen(start)]
pub fn __start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let state = Rc::new(RefCell::new(State::default()));

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();
    body.append_with_node_1(&app(&state)).unwrap_throw();
}

fn app(state: &Rc<RefCell<State>>) -> Element {
    let selected_id = Atom::new(0);

    let handle_select = {
        let selected_id = selected_id.clone();
        move |id: usize| {
            selected_id.set(id);
        }
    };

    let handle_remove = {
        let state = state.clone();
        move |id: usize| {
            let state = state.borrow();
            let index = state
                .data
                .as_slice()
                .iter()
                .position(|item| item.id == id)
                .unwrap();
            state.data.remove(index);
        }
    };

    let container = div();
    container.class_list().add_1("container").unwrap_throw();

    let jumbotron = jumbotron(state.clone());
    container.append_with_node_1(&jumbotron).unwrap_throw();

    let table = table();
    table
        .class_list()
        .add_4("table", "table-hover", "table-striped", "test-data")
        .unwrap_throw();
    let tbody = tbody();
    table.append_with_node_1(&tbody).unwrap_throw();
    container.append_with_node_1(&table).unwrap_throw();

    table
        .add_event_listener_with_callback(
            "click",
            Closure::wrap(Box::new({
                move |event: Event| {
                    let target = event.target().unwrap_throw();
                    let target: &Element = target.unchecked_ref();
                    let item_id = target
                        .closest("tr")
                        .unwrap_throw()
                        .unwrap_throw()
                        .query_selector("td")
                        .unwrap_throw()
                        .unwrap_throw()
                        .text_content()
                        .unwrap_throw()
                        .parse()
                        .unwrap();

                    if target.node_name() == "A" {
                        handle_select(item_id);
                    } else if target.node_name() == "SPAN" {
                        handle_remove(item_id);
                    }
                }
            }) as Box<dyn Fn(_)>)
            .into_js_value()
            .unchecked_ref(),
        )
        .unwrap_throw();

    let data = state.borrow().data.clone();
    tracked_map(data, tbody, move |item| {
        row(item.clone(), selected_id.clone())
    });

    let preload_icon = span();
    preload_icon
        .class_list()
        .add_3("preloadicon", "glyphicon", "glyphicon-remove")
        .unwrap_throw();
    container.append_with_node_1(&preload_icon).unwrap_throw();

    container
}

fn jumbotron(state: Rc<RefCell<State>>) -> Element {
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

    row.append_with_node_1(&header_button("run", "Create 1,000 rows", {
        let state = state.clone();
        move || {
            state.borrow().data.clear();
            append_rows(&state.borrow(), 1000);
        }
    }))
    .unwrap_throw();
    row.append_with_node_1(&header_button("runlots", "Create 10,000 rows", {
        let state = state.clone();
        move || {
            state.borrow().data.clear();
            append_rows(&state.borrow(), 10000);
        }
    }))
    .unwrap_throw();
    row.append_with_node_1(&header_button("add", "Append 1,000 rows", {
        let state = state.clone();
        move || {
            append_rows(&state.borrow(), 1000);
        }
    }))
    .unwrap_throw();
    row.append_with_node_1(&header_button("update", "Update every 10th row", {
        let state = state.clone();
        move || {
            let state = state.borrow();
            for item in state.data.as_slice().iter().step_by(10) {
                *item.label.get_mut() += " !!!";
            }
        }
    }))
    .unwrap_throw();
    row.append_with_node_1(&header_button("clear", "Clear", {
        let state = state.clone();
        move || {
            state.borrow().data.clear();
        }
    }))
    .unwrap_throw();
    row.append_with_node_1(&header_button("swaprows", "Swap Rows", move || {
        let state = state.borrow();
        if state.data.len() > 998 {
            state.data.swap(1, 998);
        }
    }))
    .unwrap_throw();

    jumbotron
}

fn header_button(id: &str, text: &str, on_click: impl Fn() + 'static) -> Element {
    let col = div();
    col.class_list()
        .add_2("col-sm-6", "smallpad")
        .unwrap_throw();

    let button = button();
    button.set_id(id);
    button
        .class_list()
        .add_3("btn", "btn-primary", "btn-block")
        .unwrap_throw();
    button.set_type("button");
    button.set_text_content(Some(text));
    button
        .add_event_listener_with_callback(
            "click",
            Closure::wrap(Box::new(on_click) as Box<dyn Fn()>)
                .into_js_value()
                .unchecked_ref(),
        )
        .unwrap_throw();
    col.append_with_node_1(&button).unwrap_throw();

    col
}

fn row(item: Rc<Item>, selected_id: Atom<usize>) -> Node {
    thread_local! {
        static TEMPLATE: Element = {
            let tr = tr();

            let id_cell = td();
            id_cell.class_list().add_1("col-md-1").unwrap_throw();
            tr.append_with_node_1(&id_cell).unwrap_throw();

            let label_cell = td();
            label_cell.class_list().add_1("col-md-4").unwrap_throw();
            let label_link = a();
            label_cell.append_with_node_1(&label_link).unwrap_throw();
            tr.append_with_node_1(&label_cell).unwrap_throw();

            let remove_cell = td();
            let remove_link = a();
            let remove_icon = span();
            remove_icon
                .class_list()
                .add_2("glyphicon", "glyphicon-remove")
                .unwrap_throw();
            remove_link.append_with_node_1(&remove_icon).unwrap_throw();
            remove_cell.append_with_node_1(&remove_link).unwrap_throw();
            tr.append_with_node_1(&remove_cell).unwrap_throw();

            let pad_cell = td();
            pad_cell.class_list().add_1("col-md-1").unwrap_throw();
            tr.append_with_node_1(&pad_cell).unwrap_throw();

            tr
        };
    }

    let tr = TEMPLATE.with(|t| t.clone_node_with_deep(true).unwrap_throw());
    toggle_class(tr.clone().unchecked_into(), "danger", {
        let item = item.clone();
        move || *selected_id.get() == item.id
    });

    let id_cell = tr.first_child().unwrap_throw();
    id_cell.set_text_content(Some(&item.id.to_string()));

    let label_cell = id_cell.next_sibling().unwrap_throw();
    let label_link = label_cell.first_child().unwrap_throw();
    react(move || {
        label_link.set_text_content(Some(&item.label.get()));
    });

    tr
}

fn append_rows(state: &State, count: usize) {
    let _batch = batch();

    let data = state.data.batch();
    data.reserve(count);
    for _ in 0..count {
        state.next_id.set(state.next_id.get() + 1);
        let label = format!(
            "{} {} {}",
            random_choice(ADJECTIVES),
            random_choice(COLORS),
            random_choice(NOUNS),
        );
        data.push(Rc::new(Item {
            id: state.next_id.get(),
            label: Atom::new(label),
        }));
    }
}

fn random_choice<'a>(xs: &[&'a str]) -> &'a str {
    xs[random_int(xs.len())]
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn random_int(max: usize) -> usize {
    (Math::random() * max as f64) as usize
}

const ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];
const COLORS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black",
    "orange",
];
const NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger",
    "pizza", "mouse", "keyboard",
];
