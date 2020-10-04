use cope::singleton::Atom;
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub struct TrackingVec<T> {
    inner: Atom<Vec<T>>,
    pub(crate) mutations: Rc<RefCell<Vec<ListMutation>>>,
}

impl<T: 'static> TrackingVec<T> {
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

    pub fn batch(&self) -> TrackingVecMut<'_, T> {
        TrackingVecMut { inner: self }
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

impl<T: 'static> Default for TrackingVec<T> {
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

#[allow(clippy::module_name_repetitions)]
pub struct TrackingVecMut<'a, T: 'static> {
    inner: &'a TrackingVec<T>,
}

impl<'a, T: 'static> TrackingVecMut<'a, T> {
    pub fn reserve(&self, additional: usize) {
        self.inner.inner.sample_mut().reserve(additional);
    }

    pub fn push(&self, value: T) {
        let index = self.inner.inner.get().len();
        self.inner
            .mutations
            .borrow_mut()
            .push(ListMutation::Insert(index));

        self.inner.inner.sample_mut().push(value);
    }
}

impl<'a, T: 'static> Drop for TrackingVecMut<'a, T> {
    fn drop(&mut self) {
        // Flush updates
        drop(self.inner.inner.get_mut());
    }
}

pub enum ListMutation {
    Insert(usize),
    Remove(usize),
}
