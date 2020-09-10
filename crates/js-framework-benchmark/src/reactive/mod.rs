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
