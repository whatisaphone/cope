use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

#[derive(Clone)]
pub struct Atom<T> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

impl<T> Atom<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial)),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get(&self) -> MutexGuard<'_, T> {
        let mut current_reaction = CURRENT_REACTION.lock();
        if let Some(current_reaction) = &mut *current_reaction {
            current_reaction.sources.push(self.subscribers.clone());
        }

        self.value.lock()
    }

    pub fn set(&self, value: T) {
        *self.value.lock() = value;

        for subscriber in &*self.subscribers.lock() {
            subscriber.lock()();
        }
    }
}

type Subscriber = Arc<Mutex<dyn Fn() + Send>>;

pub fn react(f: impl Fn() + Send + 'static) {
    let mut current_reaction = CURRENT_REACTION.lock();
    assert!(current_reaction.is_none());
    *current_reaction = Some(Reaction::new());
    drop(current_reaction);

    f();

    let mut current_reaction = CURRENT_REACTION.lock();
    let reaction = current_reaction.take().unwrap();
    let sources = reaction.sources.clone();
    drop(current_reaction);

    let f = Arc::new(Mutex::new(f));
    for source in sources {
        source.lock().push(f.clone());
    }
}

struct Reaction {
    sources: Vec<Arc<Mutex<Vec<Subscriber>>>>,
}

impl Reaction {
    pub fn new() -> Self {
        Reaction {
            sources: Vec::new(),
        }
    }
}

static CURRENT_REACTION: Lazy<Mutex<Option<Reaction>>> = Lazy::new(|| Mutex::new(None));
