#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::reactive::TrackingVec;
use cope::singleton::Atom;
use cope_dom::elements::{a, button, div, h1, span, table, td, tr, ElementBuilder};
use js_sys::Math;
use std::{
    cell::{Cell, RefCell},
    panic,
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use web_sys::{window, Element};
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
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let state = Rc::new(RefCell::new(State::default()));

    let document = window().unwrap_throw().document().unwrap_throw();
    let body = document.body().unwrap_throw();
    body.append_with_node_1(&app(&state).build()).unwrap_throw();
}

fn app(state: &Rc<RefCell<State>>) -> ElementBuilder<Element> {
    div()
        .class_name("container")
        .child(jumbotron(state.clone()))
        .child(
            table()
                .class_name("table table-hover table-striped test-data")
                // TODO: .child(tbody().children(map(state.data, |item| row(state, item)))),
        )
        .child(span().class_name("preloadicon glyphicon glyphicon-remove"))
}

fn jumbotron(state: Rc<RefCell<State>>) -> ElementBuilder<Element> {
    let run = {
        let state = state.clone();
        move || {
            state.borrow().data.clear();
            append_rows(&state.borrow(), 1000);
        }
    };

    let runlots = {
        let state = state.clone();
        move || {
            state.borrow().data.clear();
            append_rows(&state.borrow(), 10000);
        }
    };

    let add = {
        let state = state.clone();
        move || {
            append_rows(&state.borrow(), 1000);
        }
    };

    let update = {
        let state = state.clone();
        move || {
            let state = state.borrow();
            for item in state.data.as_slice().iter().step_by(10) {
                *item.label.get_mut() += " !!!";
            }
        }
    };

    let clear = {
        let state = state.clone();
        move || {
            state.borrow().data.clear();
        }
    };

    let swaprows = {
        let state = state.clone();
        move || {
            let state = state.borrow();
            if state.data.len() > 998 {
                state.data.swap(1, 998);
            }
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

fn row(
    state: &Rc<RefCell<State>>,
    item: Rc<Item>,
    selected_id: Atom<usize>,
) -> ElementBuilder<Element> {
    let handle_select = {
        let item_id = item.id;
        let selected_id = selected_id.clone();
        move || {
            selected_id.set(item_id);
        }
    };

    let handle_remove = {
        let item_id = item.id;
        let state = state.clone();
        move || {
            let state = state.borrow();
            let index = state
                .data
                .as_slice()
                .iter()
                .position(|i| i.id == item_id)
                .unwrap();
            state.data.remove(index);
        }
    };

    tr().class_name(if *selected_id.get() == item.id {
        "danger"
    } else {
        ""
    })
    .child(td().class_name("col-md-1").child(item.id.to_string()))
    .child(
        td().class_name("col-md-4")
            .child(a().on_click(handle_select).child(&*item.label.get())),
    )
    .child(
        td().child(
            a().child(
                span()
                    .class_name("glyphicon glyphicon-remove")
                    .on_click(handle_remove),
            ),
        )
        .child(td().class_name("col-md-1")),
    )
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
