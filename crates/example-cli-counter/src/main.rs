#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![cfg_attr(feature = "strict", deny(warnings))]

use crate::reactive::{react, Atom};
use std::io;

mod reactive;

fn main() {
    let count = Atom::new(0);

    react({
        let count = count.clone();
        move || {
            println!("{}", count.get());
        }
    });

    for _ in 0..5 {
        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();

        let new_count = *count.get() + 1;
        count.set(new_count);
    }
}
