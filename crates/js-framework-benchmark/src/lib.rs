#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::{
    dom::{
        list::{map_children, ElementBuilderChildren},
        misc::toggle_class,
    },
    reactive::TrackingVec,
};
use cope::singleton::{react, Atom};
use cope_dom::elements::{a, button, div, h1, span, table, tbody, td, tr, ElementBuilder};
use js_sys::Math;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{window, Element, Event};
use wee_alloc::WeeAlloc;

mod dom;
mod reactive;

#[global_allocator]
static ALLOC: WeeAlloc<'_> = WeeAlloc::INIT;

#[derive(Default)]
struct State {
    next_id: Cell<usize>,
    data: TrackingVec<Rc<Item>>,
    selected_id: Atom<usize>,
}

struct Item {
    id: usize,
    label: Atom<String>,
}

#[wasm_bindgen(start)]
pub fn __start() {
    #[cfg(debug_assertions)]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let state = Rc::new(State::default());

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();
    body.append_with_node_1(&app(&state).build()).unwrap_throw();
}

fn app(state: &Rc<State>) -> ElementBuilder<Element> {
    let handle_select = {
        let selected_id = state.selected_id.clone();
        move |item_id: usize| {
            selected_id.set(item_id);
        }
    };

    let handle_remove = {
        let state = state.clone();
        move |item_id: usize| {
            let index = state
                .data
                .as_slice()
                .iter()
                .position(|i| i.id == item_id)
                .unwrap();
            state.data.remove(index);
        }
    };

    let handle_delegated_click = move |event: Event| {
        let target = event.target().unwrap_throw();
        let target: &Element = target.unchecked_ref();
        let tr = target.closest("tr").unwrap_throw().unwrap_throw();
        let td = tr.query_selector("td").unwrap_throw().unwrap_throw();
        let item_id = td.text_content().unwrap_throw().parse().unwrap();

        if target.node_name() == "A" {
            handle_select(item_id);
        } else if target.node_name() == "SPAN" {
            handle_remove(item_id);
        }
    };

    div()
        .class_name("container")
        .child(jumbotron(state.clone()))
        .child(
            table()
                .class_name("table table-hover table-striped test-data")
                .add_event_listener_with_callback("click", handle_delegated_click)
                .child(tbody().children(map_children(state.data.clone(), {
                    let state = state.clone();
                    move |item| row(&state, item)
                }))),
        )
        .child(span().class_name("preloadicon glyphicon glyphicon-remove"))
}

fn jumbotron(state: Rc<State>) -> ElementBuilder<Element> {
    let run = {
        let state = state.clone();
        move || {
            state.data.clear();
            append_rows(&state, 1000);
        }
    };

    let runlots = {
        let state = state.clone();
        move || {
            state.data.clear();
            append_rows(&state, 10000);
        }
    };

    let add = {
        let state = state.clone();
        move || {
            append_rows(&state, 1000);
        }
    };

    let update = {
        let state = state.clone();
        move || {
            for item in state.data.as_slice().iter().step_by(10) {
                *item.label.get_mut() += " !!!";
            }
        }
    };

    let clear = {
        let state = state.clone();
        move || {
            state.data.clear();
        }
    };

    let swaprows = move || {
        if state.data.len() > 998 {
            state.data.swap(1, 998);
        }
    };

    div().class_name("jumbotron").child(
        div()
            .class_name("row")
            .child(div().class_name("col-md-6").child(h1().child("cope")))
            .child(
                div()
                    .class_name("col-md-6")
                    .child(header_button("run", "Create 1,000 rows", run))
                    .child(header_button("runlots", "Create 10,000 rows", runlots))
                    .child(header_button("add", "Append 1,000 rows", add))
                    .child(header_button("update", "Update every 10th row", update))
                    .child(header_button("clear", "Clear", clear))
                    .child(header_button("swaprows", "Swap Rows", swaprows)),
            ),
    )
}

fn header_button(id: &str, text: &str, on_click: impl Fn() + 'static) -> ElementBuilder<Element> {
    div().class_name("col-sm-6 smallpad").child(
        button()
            .id(id)
            .class_name("btn btn-primary btn-block")
            .type_("button")
            .on_click(on_click)
            .child(text),
    )
}

fn row(state: &Rc<State>, item: &Rc<Item>) -> ElementBuilder<Element> {
    thread_local! {
        static TEMPLATE: Element = tr()
            .child(td().class_name("col-md-1"))
            .child(td().class_name("col-md-4").child(a()))
            .child(
                td().child(a().child(span().class_name("glyphicon glyphicon-remove")))
                    .child(td().class_name("col-md-1")),
            )
            .build();
    }

    let tr = TEMPLATE
        .with(|t| t.clone_node_with_deep(true))
        .unwrap_throw()
        .unchecked_into::<Element>();

    toggle_class(tr.clone(), "danger", {
        let state = state.clone();
        let item_id = item.id;
        move || *state.selected_id.get() == item_id
    });

    let id_cell = tr.first_child().unwrap_throw();
    id_cell.set_text_content(Some(&item.id.to_string()));

    let label_cell = id_cell.next_sibling().unwrap_throw();
    let label_link = label_cell.first_child().unwrap_throw();
    react({
        let item = item.clone();
        move || {
            label_link.set_text_content(Some(&item.label.get()));
        }
    });

    ElementBuilder::new(tr)
}

fn append_rows(state: &State, count: usize) {
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
