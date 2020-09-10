use crate::reactive::Atom;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub struct TrackingVec<T> {
    inner: Atom<Vec<T>>,
    pub(crate) mutations: Rc<RefCell<Vec<ListMutation>>>,
}

impl<T> TrackingVec<T> {
    pub fn new() -> Self {
        Self {
            inner: Atom::new(Vec::new()),
            mutations: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn as_slice(&self) -> Ref<'_, [T]> {
        Ref::map(self.inner.get(), |i| i.as_slice())
    }

    pub fn len(&self) -> usize {
        self.inner.get().len()
    }

    pub fn get(&self, index: usize) -> Option<Ref<'_, T>> {
        let inner = self.inner.get();
        if index >= inner.len() {
            return None;
        }
        Some(Ref::map(inner, |inner| inner.get(index).unwrap()))
    }

    pub fn reserve(&self, additional: usize) {
        // TODO: this doesn't mutate the vec, can we "cheat" somehow and not run
        // reactions?
        self.inner.get_mut().reserve(additional);
    }

    pub fn swap(&self, a: usize, b: usize) {
        let mut mutations = self.mutations.borrow_mut();
        mutations.push(ListMutation::Remove(a));
        mutations.push(ListMutation::Insert(a));
        mutations.push(ListMutation::Remove(b));
        mutations.push(ListMutation::Insert(b));
        drop(mutations);

        self.inner.get_mut().swap(a, b);
    }

    pub fn push(&self, value: T) {
        let index = self.inner.get().len();
        self.mutations
            .borrow_mut()
            .push(ListMutation::Insert(index));

        self.inner.get_mut().push(value);
    }

    pub fn clear(&self) {
        let mut mutations = self.mutations.borrow_mut();
        for index in (0..self.inner.get().len()).rev() {
            mutations.push(ListMutation::Remove(index));
        }
        drop(mutations);

        self.inner.get_mut().clear();
    }

    pub fn remove(&self, index: usize) {
        self.mutations
            .borrow_mut()
            .push(ListMutation::Remove(index));
        self.inner.get_mut().remove(index);
    }
}

impl<T> Default for TrackingVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for TrackingVec<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            mutations: self.mutations.clone(),
        }
    }
}

pub enum ListMutation {
    Insert(usize),
    Remove(usize),
}
