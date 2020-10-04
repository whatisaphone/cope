use crate::instance::engine::Engine;
use std::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
    sync::Arc,
};

type Subscription = Arc<RefCell<dyn FnMut()>>;

pub struct Atom<T> {
    engine: Arc<Engine>,
    value: Arc<RefCell<T>>,
    subscriptions: Arc<RefCell<Vec<Arc<RefCell<Vec<Subscription>>>>>>,
}

impl<T: 'static> Atom<T> {
    pub fn new(engine: Arc<Engine>, initial_value: T) -> Self {
        Self {
            engine,
            value: Arc::new(RefCell::new(initial_value)),
            subscriptions: Arc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        self.engine.track(self.subscriptions.clone());
        self.value.borrow()
    }

    pub fn get_mut(&self) -> AtomMut<'_, T> {
        AtomMut {
            value: Some(self.value.borrow_mut()),
            subscriptions: self.subscriptions.clone(),
        }
    }

    pub fn set(&self, value: T) {
        *self.get_mut() = value;
    }
}

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            value: self.value.clone(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}

pub struct AtomMut<'a, T> {
    // Option dance
    value: Option<RefMut<'a, T>>,
    subscriptions: Arc<RefCell<Vec<Arc<RefCell<Vec<Subscription>>>>>>,
}

impl<T> Deref for AtomMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value.as_ref().unwrap()
    }
}

impl<T> DerefMut for AtomMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.value.as_mut().unwrap()
    }
}

impl<T> Drop for AtomMut<'_, T> {
    fn drop(&mut self) {
        drop(self.value.take());

        for subscriptions in self.subscriptions.borrow().iter() {
            for subscription in subscriptions.borrow_mut().iter_mut() {
                let mut func = subscription.borrow_mut();
                // https://github.com/rust-lang/rust/issues/51886
                (&mut *func)();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::engine::Engine;

    #[test]
    fn get_initial_value() {
        let engine = Arc::new(Engine::new());
        let atom = Atom::new(engine, 123);
        assert_eq!(*atom.get(), 123);
    }

    #[test]
    fn set() {
        let engine = Arc::new(Engine::new());
        let atom = Atom::new(engine, 0);
        atom.set(42);
        assert_eq!(*atom.get(), 42);
    }

    #[test]
    fn mutate() {
        let engine = Arc::new(Engine::new());
        let atom = Atom::new(engine, 10);
        *atom.get_mut() += 1;
        assert_eq!(*atom.get(), 11);
    }
}
