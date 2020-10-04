pub use self::{
    atom::Atom,
    tracking_vec::{ListMutation, TrackingVec},
};
use std::{cell::RefCell, rc::Rc};

mod atom;
pub mod reconcile;
mod tracking_vec;

type Subscriber = Rc<RefCell<dyn FnMut()>>;

pub fn react(mut f: impl FnMut() + 'static) {
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

pub fn batch() -> Batch {
    Batch::new()
}

#[non_exhaustive]
pub struct Batch;

impl Batch {
    fn new() -> Self {
        CURRENT_UPDATE.with(|current_update| {
            let mut current_update = current_update.borrow_mut();
            let current_update = current_update.get_or_insert_with(Update::new);
            current_update.ref_count += 1;
        });

        Self
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        CURRENT_UPDATE.with(|current_update| {
            let mut current_update = current_update.borrow_mut();
            if let Some(update) = &mut *current_update {
                update.ref_count -= 1;
                if update.ref_count == 0 {
                    update.commit();
                    *current_update = None;
                }
            }
        })
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

struct Update {
    subscribers: Vec<Subscriber>,
    ref_count: usize,
}

impl Update {
    pub fn new() -> Self {
        Update {
            subscribers: Vec::new(),
            ref_count: 0,
        }
    }

    pub fn extend_subscribers_dedup<'a>(
        &mut self,
        subscribers: impl IntoIterator<Item = &'a Subscriber>,
    ) {
        for subscriber in subscribers {
            let already_exists = self.subscribers.iter().any(|s| Rc::ptr_eq(s, &subscriber));
            if !already_exists {
                self.subscribers.push(subscriber.clone());
            }
        }
    }

    pub fn commit(&self) {
        for subscriber in &self.subscribers {
            let mut func = subscriber.borrow_mut();
            // https://github.com/rust-lang/rust/issues/51886
            (&mut *func)();
        }
    }
}

thread_local! {
    static CURRENT_REACTION: RefCell<Option<Reaction>> = RefCell::new(None);
    static CURRENT_UPDATE: RefCell<Option<Update>> = RefCell::new(None);
}
