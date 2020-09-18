use crate::reactive::{Batch, Subscriber, CURRENT_REACTION, CURRENT_UPDATE};
use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

#[derive(Default)]
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
        let batch = Batch::new();

        CURRENT_UPDATE.with(|current_update| {
            let mut current_update = current_update.borrow_mut();
            let current_update = current_update.as_mut().unwrap();
            current_update.extend_subscribers_dedup(self.subscribers.borrow().iter());
        });

        AtomMut::new(self.value.borrow_mut(), batch)
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

pub struct AtomMut<'a, T> {
    // Option dance (TODO: now unnecessary?)
    reff: Option<RefMut<'a, T>>,
    // `batch` is dropped after `reff`. Its `Drop` runs reactions if needed.
    #[allow(dead_code)]
    batch: Batch,
}

impl<'a, T> AtomMut<'a, T> {
    pub fn new(reff: RefMut<'a, T>, batch: Batch) -> Self {
        AtomMut {
            reff: Some(reff),
            batch,
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
