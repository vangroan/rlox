use rlox_gc::{scan::Scan, Gc};
use std::cell::Cell;

#[derive(Debug)]
struct Foo {
    value: u32,
    other: Gc<Bar>,
}

impl Scan for Foo {
    fn scan(&self) {
        self.other.scan();
    }

    unsafe fn root(&self) {
        todo!()
    }

    unsafe fn unroot(&self) {
        todo!()
    }
}

#[derive(Debug)]
struct Bar {
    value: Cell<u32>,
    other: Option<Gc<Foo>>,
}

impl Scan for Bar {
    fn scan(&self) {
        self.value.set(self.value.get() + 1);
    }

    unsafe fn root(&self) {
        todo!()
    }

    unsafe fn unroot(&self) {
        todo!()
    }
}

/// Test for rooting and unrooting rules.
#[test]
fn test_gc_rooting() {
    let bar = Bar {
        value: Cell::new(0),
        other: None,
    };
    let foo = Foo {
        value: 2,
        other: Gc::new(bar),
    };

    assert!(Gc::is_root(&foo.other), "GC pointer outside of arena must be a root.");
    println!("size of Gc on stack {}", std::mem::size_of::<Gc<Foo>>());
    println!("{:#?}", foo);
}
