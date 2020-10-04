use crate::reactive::{Subscriber, CURRENT_REACTION};
use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub struct Atom<T> {
    value: Rc<RefCell<T>>,
    subscribers: Rc<RefCell<Vec<Subscriber>>>,
}

impl<T> Atom<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(initial)),
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        CURRENT_REACTION.with(|current_reaction| {
            if let Some(current_reaction) = &mut *current_reaction.borrow_mut() {
                current_reaction.sources.push(self.subscribers.clone());
            }
        });

        self.value.borrow()
    }

    pub fn get_mut(&self) -> AtomMut<'_, T> {
        AtomMut::new(self.value.borrow_mut(), self.subscribers.clone())
    }

    pub fn set(&self, value: T) {
        *self.get_mut() = value;
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct AtomMut<'a, T> {
    // Option dance
    reff: Option<RefMut<'a, T>>,
    subscribers: Rc<RefCell<Vec<Subscriber>>>,
}

impl<'a, T> AtomMut<'a, T> {
    pub fn new(reff: RefMut<'a, T>, subscribers: Rc<RefCell<Vec<Subscriber>>>) -> Self {
        AtomMut {
            reff: Some(reff),
            subscribers,
        }
    }
}

impl<'a, T> Drop for AtomMut<'a, T> {
    fn drop(&mut self) {
        // Drop our borrow first so the subscribers are able to borrow
        drop(self.reff.take());

        for subscriber in &mut *self.subscribers.borrow_mut() {
            let mut func = subscriber.borrow_mut();
            // https://github.com/rust-lang/rust/issues/51886
            (&mut *func)();
        }
    }
}

impl<'a, T> Deref for AtomMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.reff.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for AtomMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.reff.as_mut().unwrap()
    }
}
