use crate::{
    context::Context,
    gc::{Gc, GcBox, GcColor},
    scan::Scan,
};
use std::{cell::Cell, collections::LinkedList, ptr::NonNull};

pub struct Collector {
    // TODO: Packed arena
    // arena: typed_arena::Arena<NonNull<GcBox<dyn Scan>>>,
    /// For development we're using a linked list.
    head: Cell<Option<NonNull<GcBox<dyn Scan>>>>,

    /// Queue of gray objects that need to be scanned.
    gray: Vec<NonNull<GcBox<dyn Scan>>>,
    gray_new: Vec<NonNull<GcBox<dyn Scan>>>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            // arena: ...,
            head: Cell::new(None),
            gray: vec![],
            gray_new: vec![],
        }
    }

    pub fn alloc<T: 'static + Scan>(&mut self, value: T) -> Gc<T> {
        // Because we return a `Gc<T>` that we lose track of, we must consider it part of the root set.
        let sized = GcBox {
            root: Cell::new(1),
            color: Cell::new(GcColor::Gray),
            next: Cell::new(self.head.get()),
            value,
        };
        let ptr = unsafe { NonNull::new_unchecked(Box::leak(Box::new(sized))) };
        self.head.set(Some(ptr));

        // Initial state is that object immediately reachable from the root set are marked gray.
        self.gray.push(ptr);

        // SAFETY: We trust the arena won't give us a bad reference, so we assume it's not null.
        //         By converting a pointer we're detaching the reference from the arena's lifetime, but it will be
        //         kept in the reference counted `Gc<T>` pointer.  The arena is only dropped when all `Gc<T>`
        //         pointers are collected.
        Gc::from_inner(ptr)
    }

    /// Run a garbage collection cycle.
    pub fn collect(&mut self) {
        let Self { gray, gray_new, .. } = self;
        let mut ctx = Context { gray: gray_new };
        while !gray.is_empty() {
            if let Some(ptr) = gray.pop() {
                let gc_box = unsafe { ptr.as_ref() };
                println!("Popped {:?}", ptr);
                gc_box.value.scan(&mut ctx);
                gc_box.color.set(GcColor::Black);
            }
        }
    }

    pub(crate) fn scan_ptr(ctx: &mut Context, ptr: NonNull<GcBox<dyn Scan>>) {
        let gc_box = unsafe { ptr.as_ref() };
        match gc_box.color.get() {
            GcColor::White => {
                // We now know that this object can be reached from a gray object
                // and should not be collected.
                gc_box.color.set(GcColor::Gray);
                ctx.gray.push(ptr);
                // We don't recurse further with the scan. A pointer forms a boundary for work.
            }
            GcColor::Gray | GcColor::Black => {}
        }
    }
}
