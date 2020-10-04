use std::{cell::RefCell, rc::Rc};

pub struct Engine {
    current_reaction: RefCell<Option<Reaction>>,
    pub(crate) current_update: RefCell<Option<Update>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            current_reaction: RefCell::new(None),
            current_update: RefCell::new(None),
        }
    }

    pub(crate) fn track(&self, subscriptions: Rc<RefCell<Vec<Rc<RefCell<Vec<Subscription>>>>>>) {
        let mut reaction = self.current_reaction.borrow_mut();
        let reaction = match reaction.as_mut() {
            Some(reaction) => reaction,
            None => return,
        };
        subscriptions
            .borrow_mut()
            .push(reaction.subscriptions.clone());
    }

    pub fn batch(self: &Rc<Self>) -> Batch {
        let mut current_update = self.current_update.borrow_mut();
        let root = current_update.is_none();
        if !root {
            return Batch { engine: None };
        }

        *current_update = Some(Update::new());
        Batch {
            engine: Some(self.clone()),
        }
    }

    pub(crate) fn enqueue(&self, f: impl FnOnce() + 'static) {
        let mut update = self.current_update.borrow_mut();
        let update = update.as_mut().unwrap();
        update.updates.push(Box::new(f));
    }

    pub fn react(self: Rc<Self>, f: impl Fn() + 'static) {
        let mut current_reaction = self.current_reaction.borrow_mut();
        assert!(current_reaction.is_none());
        *current_reaction = Some(Reaction::new());
        drop(current_reaction);

        f();

        let mut current_reaction = self.current_reaction.borrow_mut();
        let reaction = current_reaction.take().unwrap();
        reaction.subscriptions.borrow_mut().push(Rc::new(f));
    }
}

fn slow_pop_front<T>(xs: &mut Vec<T>) -> Option<T> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.remove(0))
    }
}

type Subscription = Rc<dyn Fn()>;

struct Reaction {
    subscriptions: Rc<RefCell<Vec<Subscription>>>,
}

impl Reaction {
    pub fn new() -> Self {
        Reaction {
            subscriptions: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

pub(crate) struct Update {
    updates: Vec<Box<dyn FnOnce()>>,
}

impl Update {
    pub fn new() -> Self {
        Update {
            updates: Vec::new(),
        }
    }
}

pub struct Batch {
    engine: Option<Rc<Engine>>,
}

impl Drop for Batch {
    fn drop(&mut self) {
        let engine = match &mut self.engine {
            Some(x) => x,
            None => return,
        };

        loop {
            let head = {
                let mut update = engine.current_update.borrow_mut();
                slow_pop_front(&mut update.as_mut().unwrap().updates)
            };
            let head = match head {
                Some(x) => x,
                None => break,
            };
            head();
        }

        engine.current_update.borrow_mut().take().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::atom::Atom;
    use std::rc::Rc;

    #[test]
    fn react_simple() {
        let engine = Rc::new(Engine::new());
        let atom = Atom::new(engine.clone(), 1);
        let sink = Rc::new(RefCell::new(Vec::new()));
        engine.react({
            let atom = atom.clone();
            let sink = sink.clone();
            move || {
                sink.borrow_mut().push(*atom.get());
            }
        });
        atom.set(2);
        assert_eq!(*sink.borrow(), [1, 2]);
    }

    #[test]
    fn commit_changes_after_reaction() {
        let engine = Rc::new(Engine::new());
        let atom = Atom::new(engine.clone(), 1);

        let batch = engine.batch();
        assert_eq!(*atom.get(), 1);
        atom.set(2);
        assert_eq!(*atom.get(), 1);
        drop(batch);
        assert_eq!(*atom.get(), 2);
    }
}
