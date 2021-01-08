use rlox_gc::{context::Context, scan::Scan, Collector, Gc};
use std::cell::Cell;

#[derive(Debug)]
struct Foo {
    value: u32,
    other: Gc<Bar>,
    items: Vec<Bar>,
}

impl Scan for Foo {
    fn scan(&self, ctx: &mut Context) {
        self.other.scan(ctx);
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
    fn scan(&self, ctx: &mut Context) {
        self.value.set(self.value.get() + 1);
        // TODO: Walk deeper. Fix cycles first.
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
    let mut gc = Collector::new();

    let bar = Bar {
        value: Cell::new(1),
        other: None,
    };
    let foo_2 = Foo {
        value: 3,
        other: gc.alloc(Bar {
            value: Cell::new(4),
            other: None,
        }),
        items: vec![],
    };
    let foo = Foo {
        value: 2,
        other: gc.alloc(bar),
        items: vec![
            Bar {
                value: Cell::new(2),
                other: None,
            },
            Bar {
                value: Cell::new(3),
                other: Some(gc.alloc(foo_2)),
            },
        ],
    };

    assert!(Gc::is_root(&foo.other), "GC pointer outside of arena must be a root.");
    println!("size of Gc on stack {}", std::mem::size_of::<Gc<Foo>>());
    println!("{:#?}", foo);

    gc.collect();

    println!("{:#?}", foo);
}
