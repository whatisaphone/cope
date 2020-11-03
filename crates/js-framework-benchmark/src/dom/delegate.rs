use js_sys::Reflect;
use wasm_bindgen::{prelude::*, JsCast, UnwrapThrowExt};
use web_sys::{window, Event, EventTarget};

pub fn setup_delegated_events(name: &str) {
    let window = window().unwrap_throw();
    let document = window.document().unwrap_throw();
    let body = document.body().unwrap_throw();

    body.add_event_listener_with_callback(
        name,
        Closure::wrap(Box::new(handle_delegated_event) as Box<dyn Fn(&Event)>)
            .into_js_value()
            .unchecked_ref(),
    )
    .unwrap_throw();
}

fn decorate_event_name(name: &str) -> String {
    format!("__cope_event_{}", name)
}

#[allow(clippy::module_name_repetitions)]
pub fn delegate_event(target: &EventTarget, name: &str, listener: impl FnMut() + 'static) {
    let decorated_name = decorate_event_name(name);
    Reflect::set(
        target,
        &decorated_name.into(),
        &EventDelegation::new(Box::new(listener)).into(),
    )
    .unwrap_throw();
}

fn handle_delegated_event(event: &Event) {
    let decorated_name = decorate_event_name(&event.type_());
    let target = match event.target() {
        Some(target) => target,
        None => return,
    };
    let delegation = Reflect::get(&target, &decorated_name.into()).unwrap_throw();
    if delegation.is_falsy() {
        return;
    }
    let EventDelegation { mut f } = EventDelegation::from(&delegation);
    f();
}

#[wasm_bindgen]
struct EventDelegation {
    f: Box<dyn FnMut()>,
}

impl EventDelegation {
    pub fn new(f: Box<dyn FnMut()>) -> Self {
        EventDelegation { f }
    }
}

// https://github.com/rustwasm/wasm-bindgen/issues/1642
#[allow(clippy::needless_pass_by_value)]
mod convert {
    use crate::dom::delegate::EventDelegation;
    use js_sys::Object;
    use wasm_bindgen::{prelude::*, JsValue};

    #[wasm_bindgen]
    extern "C" {
        type Wrapper;

        #[wasm_bindgen(method, setter)]
        fn set_inner(this: &Wrapper, value: &JsValue);

        #[wasm_bindgen(method, getter)]
        fn inner(this: &Wrapper) -> EventDelegation;
    }

    impl From<&JsValue> for EventDelegation {
        fn from(value: &JsValue) -> Self {
            let wrapper = Wrapper::from(JsValue::from(Object::new()));
            wrapper.set_inner(value);
            wrapper.inner()
        }
    }
}
