use crate::{instance, instance::AtomMut};
use std::{
    cell::{Ref, RefMut},
    sync::Arc,
};

thread_local! {
    static ENGINE: Arc<instance::Engine> = Arc::new(instance::Engine::new());
}

pub fn batch() -> Batch {
    ENGINE.with(|engine| Batch::new(engine.batch()))
}

pub fn react(f: impl FnMut() + 'static) {
    ENGINE.with(|engine| engine.react(f))
}

#[must_use]
pub struct Batch {
    #[allow(dead_code)] // This is only here to be dropped
    inner: instance::Batch,
}

impl Batch {
    pub fn new(inner: instance::Batch) -> Self {
        Batch { inner }
    }
}

pub struct Atom<T> {
    inner: instance::Atom<T>,
}

impl<T: 'static> Atom<T> {
    pub fn new(initial_value: T) -> Self {
        let engine = ENGINE.with(<_>::clone);
        Self {
            inner: instance::Atom::new(engine, initial_value),
        }
    }

    #[must_use]
    pub fn get(&self) -> Ref<'_, T> {
        self.inner.get()
    }

    #[must_use]
    pub fn get_mut(&self) -> AtomMut<'_, T> {
        self.inner.get_mut()
    }

    #[must_use]
    pub fn sample_mut(&self) -> RefMut<'_, T> {
        self.inner.sample_mut()
    }

    pub fn set(&self, value: T) {
        self.inner.set(value);
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
