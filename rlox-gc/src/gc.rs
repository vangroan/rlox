//! `Gc<T>` smart pointer.
use crate::{context::Context, scan::Scan, Collector};
use std::borrow::BorrowMut;
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

impl<T: 'static + Scan> Scan for Gc<T> {
    fn scan(&self, ctx: &mut Context) {
        Collector::scan_ptr(ctx, self.ptr);
    }

    unsafe fn root(&self) {
        todo!()
    }

    unsafe fn unroot(&self) {
        todo!()
    }
}

/// Internal pointer type to garbage collected space.
///
/// The `flag` field is information pertinent to the garbage collection algorithm, packed into
/// a 32-bit fields to reduce the overall size overhead of the struct.
#[derive(Debug)]
#[doc(hidden)]
pub struct GcBox<T: Scan + ?Sized> {
    pub(crate) root: Cell<u32>,
    pub(crate) color: Cell<GcColor>,
    pub(crate) next: Cell<Option<NonNull<GcBox<dyn Scan>>>>,
    pub(crate) value: T,
}

impl<T: Scan + ?Sized> GcBox<T> {
    fn dec(&self) {
        self.root.set(self.root.get() - 1);
    }

    fn incr(&self) {
        self.root.set(self.root.get() + 1);
    }

    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    fn color(&self) -> GcColor {
        self.color.get()
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

    impl Scan for () {
        fn scan(&self, _: &mut Context) {}

        unsafe fn root(&self) {}

        unsafe fn unroot(&self) {}
    }

    struct Foo {
        a: u32,
        b: u8,
    }

    #[test]
    fn test_gcbox_size() {
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

            assert_eq!(gcbox.color(), *color);
        }
    }
}
