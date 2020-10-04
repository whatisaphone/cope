use crate::instance::engine::Engine;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

type Subscription = Rc<dyn Fn()>;

#[derive(Clone)]
pub struct Atom<T> {
    engine: Rc<Engine>,
    value: Rc<RefCell<T>>,
    next: Rc<RefCell<Option<Box<dyn FnOnce(&mut T)>>>>,
    subscriptions: Rc<RefCell<Vec<Rc<RefCell<Vec<Subscription>>>>>>,
}

impl<T: 'static> Atom<T> {
    pub fn new(engine: Rc<Engine>, initial_value: T) -> Self {
        Self {
            engine,
            value: Rc::new(RefCell::new(initial_value)),
            next: Rc::new(RefCell::new(None)),
            subscriptions: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn get(&self) -> Ref<'_, T> {
        self.engine.track(self.subscriptions.clone());
        self.value.borrow()
    }

    pub fn mutate(&self, f: impl FnOnce(&mut T) + 'static) {
        let mut next = self.next.borrow_mut();
        assert!(next.is_none());
        // TODO: avoid heap allocation?
        *next = Some(Box::new(f));
        drop(next);

        let batch = self.engine.batch();
        self.engine.enqueue({
            let value = self.value.clone();
            let next = self.next.clone();
            move || {
                next.borrow_mut().take().unwrap()(&mut value.borrow_mut());
            }
        });
        drop(batch);

        for subscriptions in self.subscriptions.borrow().iter() {
            for subscription in subscriptions.borrow().iter() {
                subscription();
            }
        }
    }

    pub fn set(&self, value: T) {
        self.mutate(|dest| {
            *dest = value;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::engine::Engine;

    #[test]
    fn get_initial_value() {
        let engine = Rc::new(Engine::new());
        let atom = Atom::new(engine, 123);
        assert_eq!(*atom.get(), 123);
    }

    #[test]
    fn set() {
        let engine = Rc::new(Engine::new());
        let atom = Atom::new(engine, 0);
        atom.set(42);
        assert_eq!(*atom.get(), 42);
    }

    #[test]
    fn mutate() {
        let engine = Rc::new(Engine::new());
        let atom = Atom::new(engine, 10);
        atom.mutate(|x| {
            *x += 1;
        });
        assert_eq!(*atom.get(), 11);
    }
}
