use std::{cell::RefCell, sync::Arc};

#[derive(Default)]
pub struct Engine {
    current_reaction: RefCell<Option<Reaction>>,
    pub(crate) current_update: RefCell<Option<Update>>,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn track(&self, subscriptions: &Arc<RefCell<SubscriptionList>>) {
        let mut reaction = self.current_reaction.borrow_mut();
        let reaction = match reaction.as_mut() {
            Some(reaction) => reaction,
            None => return,
        };
        subscriptions
            .borrow_mut()
            .push(reaction.subscriptions.clone());
    }

    pub fn batch(self: &Arc<Self>) -> Batch {
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

    pub fn react(&self, mut f: impl FnMut() + 'static) {
        let mut current_reaction = self.current_reaction.borrow_mut();
        assert!(current_reaction.is_none());
        *current_reaction = Some(Reaction::new());
        drop(current_reaction);

        f();

        let mut current_reaction = self.current_reaction.borrow_mut();
        let reaction = current_reaction.take().unwrap();
        reaction
            .subscriptions
            .borrow_mut()
            .push(Arc::new(RefCell::new(f)));
    }
}

fn slow_pop_front<T>(xs: &mut Vec<T>) -> Option<T> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.remove(0))
    }
}

type SubscriptionList = Vec<Arc<RefCell<Vec<Subscription>>>>;
type Subscription = Arc<RefCell<dyn FnMut()>>;

struct Reaction {
    subscriptions: Arc<RefCell<Vec<Subscription>>>,
}

impl Reaction {
    pub fn new() -> Self {
        Reaction {
            subscriptions: Arc::new(RefCell::new(Vec::new())),
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

#[must_use]
pub struct Batch {
    engine: Option<Arc<Engine>>,
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
    use crate::instance::{Atom, Engine};
    use std::{cell::RefCell, sync::Arc};

    #[test]
    fn react_simple() {
        let engine = Arc::new(Engine::new());
        let atom = Atom::new(engine.clone(), 1);
        let sink = Arc::new(RefCell::new(Vec::new()));
        engine.react({
            let atom = atom.clone();
            let sink = sink.clone();
            move || {
                sink.borrow_mut().push(*atom.get());
            }
        });
        assert_eq!(*sink.borrow(), [1]);
        atom.set(2);
        assert_eq!(*sink.borrow(), [1, 2]);
    }
}
