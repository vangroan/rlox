//! Trait for a value that can live in the garbage collector.
use crate::context::Context;

pub trait Scan {
    fn scan(&self, ctx: &mut Context<'_>);

    /// Mark all the `Gc<T>` pointers contained in this value as root.
    ///
    /// This will prevent them from being garbage collected.
    unsafe fn root(&self);

    /// Mark all the `Gc<T>` pointers contained in this value as no longer root.
    ///
    /// Unsafe because this will allow them to be garbage collected, even if you hold on
    /// to this value.
    unsafe fn unroot(&self);
}
