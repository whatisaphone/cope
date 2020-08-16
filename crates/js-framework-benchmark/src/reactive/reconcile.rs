use crate::reactive::{react, Atom};

// TODO: get rid of bounds
pub fn track<T: Clone + Eq + 'static>(xs: Atom<Vec<T>>, mut sink: impl Sink<T> + 'static) {
    let mut cache = Vec::new();

    react(move || {
        let xs = xs.get();

        // This is the dumbest reconciler ever created
        let mut index = 0;
        while index < xs.len().max(cache.len()) {
            match (cache.get(index), xs.get(index)) {
                (None, None) => unreachable!(),
                (Some(_), None) => {
                    sink.remove(index);
                    cache.remove(index);
                }
                (None, Some(_)) => {
                    sink.insert(index, &xs[index]);
                    cache.insert(index, xs[index].clone());
                    index += 1;
                }
                (Some(prev), Some(next)) if prev != next => {
                    sink.remove(index);
                    cache.remove(index);

                    sink.insert(index, &xs[index]);
                    cache.insert(index, xs[index].clone());
                    index += 1;
                }
                (Some(_), Some(_)) /* prev == next */ => {
                    index += 1;
                }
            }
        }
    });
}

pub trait Sink<T> {
    fn remove(&mut self, index: usize);
    fn insert(&mut self, index: usize, item: &T);
}
