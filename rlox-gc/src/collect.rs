use crate::gc::{Gc, GcBox};

pub struct Collector<A> {
    arena: A,
}
