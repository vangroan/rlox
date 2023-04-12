use crate::{
    context::Context,
    gc::{Gc, GcBox, GcColor},
    scan::Scan,
};
use std::{cell::Cell, ptr::NonNull};

pub struct Collector {
    // TODO: Packed arena
    // arena: typed_arena::Arena<NonNull<GcBox<dyn Scan>>>,
    /// For development we're using a linked list.
    head: Option<NonNull<GcBox<dyn Scan>>>,
    state: CollectState,

    /// Temporary head used for sweep pahse.
    sweep: Option<NonNull<GcBox<dyn Scan>>>,
    sweep_prev: Option<NonNull<GcBox<dyn Scan>>>,

    /// Temporary head used for wake phase.
    wake: Option<NonNull<GcBox<dyn Scan>>>,

    /// Queue of gray objects that need to be scanned.
    gray: Vec<NonNull<GcBox<dyn Scan>>>,
    gray_new: Vec<NonNull<GcBox<dyn Scan>>>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            // arena: ...,
            head: None,
            state: CollectState::Sleep,

            sweep: None,
            sweep_prev: None,

            wake: None,

            gray: vec![],
            gray_new: vec![],
        }
    }

    pub fn alloc<T: 'static + Scan>(&mut self, value: T) -> Gc<T> {
        // When a value containing a `Gc<T>` moves into another `Gc<T>`, we
        // need to unroot the child pointer and all its contents.
        //
        // It won't be de-allocated if `Scan` is properly implemented on the value,
        // as it will be reachable from the new root.
        value.unroot();

        let sized = GcBox {
            // Because we return a `Gc<T>` that we lose track
            // of, we must consider it part of the root set.
            root: Cell::new(1),
            // We color the new box white, but a sweep may be in progress!
            // However this is safe because we're inserting into the head
            // of the linked list. The sweep phase would have started at
            // the old head, which is now the next pointer.
            //
            // Important consideration when moving to a packed arena, we may
            // be sweeping the arena and deallocating items colored white. The
            // future solution depends on how the packing will be implemented.
            color: Cell::new(GcColor::White),
            next: Cell::new(self.head),
            value,
        };
        let ptr = unsafe { NonNull::new_unchecked(Box::leak(Box::new(sized))) };
        self.head = Some(ptr);
        // println!("Alloc {:?}", ptr);

        // Allocation can occur while the sweep phase is in progress.
        //
        // If the previously swept pointer is `None`, it means the sweep phase
        // is currently looking at the head node of the sweep list, which we just changed to
        // a newly allocated node.
        //
        // To enforce the invariant and let the assumptions of the sweep algorithm hold true, we
        // need to let the sweep know the head is now in the previous position.
        if self.state == CollectState::Sweep && self.sweep_prev.is_none() {
            self.sweep_prev = self.head;
        }

        // Initial state is that object immediately reachable from the root set are marked gray.
        // self.gray.push(ptr);

        // SAFETY: We trust the arena won't give us a bad reference, so we assume it's not null.
        //         By converting a pointer we're detaching the reference from the arena's lifetime, but it will be
        //         kept in the reference counted `Gc<T>` pointer.  The arena is only dropped when all `Gc<T>`
        //         pointers are collected.
        Gc::from_inner(ptr)
    }

    /// Returns the number of objects that have been allocated.
    ///
    /// This is an expensive call that needs to iterate over the entire contents of the inner storage.
    #[inline]
    pub fn len(&self) -> usize {
        let mut head = self.head;
        let mut count = 0;
        while let Some(ptr) = head {
            let gc_box = unsafe { ptr.as_ref() };
            count += 1;
            head = gc_box.next.get();
        }
        count
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Run a garbage collection cycle.
    pub fn collect(&mut self) {
        // println!("Collect");
        // TODO: Wake on allocate
        self.state = CollectState::Wake;
        self.wake = self.head;

        loop {
            match self.state {
                CollectState::Wake => {
                    // All roots must be set to gray.
                    if let Some(ptr) = self.wake {
                        let gc_box = unsafe { ptr.as_ref() };
                        self.wake = gc_box.next.get();

                        // A `GcBox` is considered part of the root set if
                        // its reference count is not zero.
                        if gc_box.is_root() {
                            // println!("Root discovered {:?}", ptr);
                            gc_box.color.set(GcColor::Gray);
                            self.gray.push(ptr);
                        }
                    } else {
                        // All roots have been considered.
                        self.state = CollectState::Mark;
                    }
                }
                CollectState::Mark => {
                    // println!("gray {:?}", self.gray);

                    let mut ctx = Context {
                        gray: &mut self.gray_new,
                    };
                    if let Some(ptr) = self.gray.pop() {
                        let gc_box = unsafe { ptr.as_ref() };
                        // println!("Mark {:?}", ptr);
                        gc_box.value.scan(&mut ctx);
                        gc_box.color.set(GcColor::Black);

                        // Reachable items have been set from white to gray.
                        self.gray.extend(ctx.gray.drain(..));
                    } else {
                        // println!("Preparing for sweep");
                        self.state = CollectState::Sweep;
                        // Prepare for sweep phase.
                        self.sweep = self.head;
                    }
                }
                CollectState::Sweep => {
                    if let Some(sweep_ptr) = self.sweep {
                        // SAFETY: We need to be careful in the `Sweep` phase not to take
                        //         the `GcBox` as a reference/borrow and keep it on the stack.
                        //         The raw pointer will soon be deallocated turning the
                        //         would be reference invalid and violating Rust's invariants.
                        //         If we use that hypothetical reference after the `Box` drop
                        //         then we're in for a bad time.
                        let next_ptr = unsafe { sweep_ptr.as_ref().next.get() };
                        self.sweep = next_ptr;

                        let color = unsafe { sweep_ptr.as_ref().color.get() };
                        match color {
                            GcColor::White => {
                                // println!("Deallocate {:?}", sweep_ptr);

                                match self.sweep_prev {
                                    Some(sweep_prev) => {
                                        // println!("De-link {:?}", sweep_ptr);
                                        // If the previously swept pointer is `Some` then
                                        // we are in the middle of the linked list, and the current
                                        // node needs to be delinked.
                                        let gc_box_prev = unsafe { sweep_prev.as_ref() };
                                        gc_box_prev.next.set(next_ptr);
                                    }
                                    None => {
                                        // println!("De-link Head {:?}", sweep_ptr);
                                        // If the previously swept pointer is `None` then
                                        // we are looking at the head of the linked list.
                                        // The head pointer needs to be advanced to the next
                                        // node in the list.
                                        assert_eq!(self.head, Some(sweep_ptr));
                                        self.head = next_ptr;
                                        // println!("New Head {:?}", self.head);
                                    }
                                }
                                // Cast the pointer to a box and let it drop.
                                // SAFETY: If all the invariants of the collector hold true, we can safely
                                //         drop this `GcBox`. Any pointer remaining in a Gc<T>, Vec<_> or
                                //         linked list, is a bug in the collector.
                                debug_assert_eq!(
                                    unsafe { sweep_ptr.as_ref().root.get() },
                                    0,
                                    "GcBox deallocated but still rooted."
                                );
                                unsafe {
                                    drop(Box::from_raw(sweep_ptr.as_ptr()));
                                }
                            }
                            GcColor::Black => {
                                // println!("Survive {:?}", sweep_ptr);
                                self.sweep_prev = Some(sweep_ptr);

                                // Reachable from root set.
                                // We change it back to white in preparation for the next mark-and-sweep.
                                unsafe {
                                    sweep_ptr.as_ref().color.set(GcColor::White);
                                }
                            }
                            GcColor::Gray => unreachable!("Something was placed in the gray set during sweep phase."),
                        }
                    } else {
                        // Done sweeping.
                        self.sweep_prev = None;
                        self.state = CollectState::Sleep;
                    }
                }
                CollectState::Sleep => break,
            }
        }
    }

    #[inline]
    pub(crate) fn scan_ptr(ctx: &mut Context, ptr: NonNull<GcBox<dyn Scan>>) {
        let gc_box = unsafe { ptr.as_ref() };
        match gc_box.color.get() {
            GcColor::White => {
                // We now know that this object can be reached from a gray object
                // and should not be collected.
                gc_box.color.set(GcColor::Gray);
                ctx.gray.push(ptr);
                // println!("Push {:?}", ptr);
                // We don't recurse further with the scan. A pointer forms a boundary for work.
            }
            GcColor::Gray | GcColor::Black => {}
        }
    }

    #[inline]
    pub(crate) fn unroot_ptr(ptr: NonNull<GcBox<dyn Scan>>) {
        let gc_box = unsafe { ptr.as_ref() };
        gc_box.dec();
    }

    fn can_drop(&self) -> bool {
        // TODO: Panic if there are still roots
        true
    }
}

impl Drop for Collector {
    fn drop(&mut self) {
        if self.can_drop() {
            // println!("Collector Drop");
            // Deallocate owned pointers.
            self.collect();
            assert_eq!(self.len(), 0, "Collector dropped but some items are still reachable");
        }
    }
}

impl Default for Collector {
    fn default() -> Self {
        Collector::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CollectState {
    Wake,
    Mark,
    Sweep,
    Sleep,
}
