//! TODO: Scan is not implemented for `RefCell` or `Cell` on purpose.
//!       Interior mutability breaks the invariants of the garbage
//!       collector if we can move a `Gc<T>` out of another `Gc<T>`
//!       without marking it as a root.
//!       We must thus implement our own `GcCell` if we want to offer
//!       interior mutability to users.
//!       See: https://manishearth.github.io/blog/2015/09/01/designing-a-gc-in-rust/
