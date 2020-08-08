use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

#[derive(Clone)]
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

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;

        for subscriber in &*self.subscribers.borrow() {
            subscriber.borrow()();
        }
    }
}

type Subscriber = Rc<RefCell<dyn Fn()>>;

pub fn react(f: impl Fn() + 'static) {
    CURRENT_REACTION.with(|current_reaction| {
        let mut current_reaction = current_reaction.borrow_mut();
        assert!(current_reaction.is_none());
        *current_reaction = Some(Reaction::new());
    });

    f();

    let sources = CURRENT_REACTION.with(|current_reaction| {
        let mut current_reaction = current_reaction.borrow_mut();
        let reaction = current_reaction.take().unwrap();
        reaction.sources
    });

    let f = Rc::new(RefCell::new(f));
    for source in sources {
        source.borrow_mut().push(f.clone());
    }
}

struct Reaction {
    sources: Vec<Rc<RefCell<Vec<Subscriber>>>>,
}

impl Reaction {
    pub fn new() -> Self {
        Reaction {
            sources: Vec::new(),
        }
    }
}

thread_local! {
    static CURRENT_REACTION: RefCell<Option<Reaction>> = RefCell::new(None);
}
