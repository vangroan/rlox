//! `Gc<T>` smart pointer.
use crate::{context::Context, scan::Scan, Collector};
use std::{
    cell::Cell,
    fmt::{self, Debug},
    ops::Deref,
    ptr::NonNull,
};

pub struct Gc<T: Scan + ?Sized> {
    ptr: NonNull<GcBox<T>>,
}

impl<T: Scan + ?Sized> Gc<T> {
    pub(crate) fn from_inner(ptr: NonNull<GcBox<T>>) -> Self {
        Gc { ptr }
    }

    #[inline(always)]
    fn inner(&self) -> &GcBox<T> {
        // Safe because a GcBox must be alive and non-null for a `Gc` to exist.
        unsafe { self.ptr.as_ref() }
    }

    /// Indicates that this `Gc` is considered part of the root set.
    pub fn is_root(gc: &Gc<T>) -> bool {
        gc.inner().root.get() > 0
    }

    /// Returns the size in bytes of the data being pointed to.
    ///
    /// The returned value is the sum of the size of `T` and the header metadata used
    /// by the mark-and-sweep algorithm.
    pub fn inner_size(gc: &Gc<T>) -> usize {
        ::std::mem::size_of_val(gc.inner())
    }
}

impl<T: Scan + Sized> Gc<T> {}

impl<T: Debug + Scan + ?Sized> Debug for Gc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gc")
            .field("ptr", &self.ptr)
            .field("_box", &self.inner())
            .finish()
    }
}

impl<T: Scan + ?Sized> Drop for Gc<T> {
    fn drop(&mut self) {
        self.inner().dec();
    }
}

impl<T: Scan + ?Sized> Deref for Gc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner().value()
    }
}

impl<T: Scan + ?Sized> Clone for Gc<T> {
    fn clone(&self) -> Self {
        // Important: We must increment the reference count of the backing box.
        unsafe { self.ptr.as_ref().incr() };

        // SAFETY: We're going to share the same pointer between multiple `Gc<T>` now.
        //         It's reference counted, so we trust the shared value will be garbage collected
        //         when the count reaches 0 AND it's no longer reachable.
        Gc::from_inner(self.ptr)
    }
}

unsafe impl<T: 'static + Scan> Scan for Gc<T> {
    fn scan(&self, ctx: &mut Context) {
        Collector::scan_ptr(ctx, self.ptr);
    }

    fn root(&self) {
        todo!("Re-root the Gc when it leaves a `GcCell`")
    }

    fn unroot(&self) {
        Collector::unroot_ptr(self.ptr);
    }
}

/// Internal pointer type to garbage collected space.
#[derive(Debug)]
#[doc(hidden)]
pub(crate) struct GcBox<T: Scan + ?Sized> {
    pub(crate) root: Cell<u32>,
    pub(crate) color: Cell<GcColor>,
    pub(crate) next: Cell<Option<NonNull<GcBox<dyn Scan>>>>,
    pub(crate) value: T,
}

impl<T: Scan + ?Sized> GcBox<T> {
    pub(crate) fn dec(&self) {
        // Unlike an `Rc` we can decrement the reference count even though
        // it's already 0. Decrement can happen when an owning `Gc<T>` is
        // dropped, and when it is moved into another `Gc<T>` via `Collector::alloc_gc`.
        if self.root.get() > 0 {
            self.root.set(self.root.get() - 1);
        }
    }

    pub(crate) fn incr(&self) {
        self.root.set(self.root.get() + 1);
    }

    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    #[inline(always)]
    pub(crate) fn is_root(&self) -> bool {
        self.root.get() > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum GcColor {
    /// `GcBox` is condemned, and can be collected.
    White = 1u8,
    /// `GcBox` is reachable from a root, and must not be collected.
    /// Child `Scan` instances have not been visited yet.
    Gray = 2u8,
    /// `GcBox` is reachable from a root, and must not be collected.
    /// Immediate Child `Scan` instances have been marked as `Gray`.
    Black = 4u8,
}

impl From<u8> for GcColor {
    fn from(val: u8) -> GcColor {
        match val {
            1 => GcColor::White,
            2 => GcColor::Gray,
            4 => GcColor::Black,
            _ => panic!("Invalid GcColor conversion from {}", val),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(dead_code)]
    struct Foo {
        a: u32,
        b: u8,
    }

    #[test]
    fn test_gcbox_size() {
        println!("size of Gc<()> {}", ::std::mem::size_of::<Gc<()>>());
        println!("size of GcBox<()> {}", ::std::mem::size_of::<GcBox<()>>());
        println!("size of Foo {}", ::std::mem::size_of::<Foo>());
    }

    #[test]
    fn test_gcbox_color_pack() {
        for color in &[GcColor::White, GcColor::Gray, GcColor::Black] {
            let gcbox = GcBox {
                root: Cell::new(1),
                color: Cell::new(*color),
                next: Cell::new(None),
                value: (),
            };

            assert_eq!(gcbox.color.get(), *color);
        }
    }
}
