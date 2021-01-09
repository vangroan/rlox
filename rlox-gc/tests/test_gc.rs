#![allow(clippy::blacklisted_name)]
use rlox_gc::{context::Context, scan::Scan, Collector, Gc};
use rlox_gc_derive::Scan;
use std::cell::Cell;

#[derive(Debug, Scan)]
struct Foo {
    value: u32,
    other: Gc<Bar>,
    items: Vec<Bar>,
}

// unsafe impl Scan for Foo {
//     fn scan(&self, ctx: &mut Context) {
//         println!("Scan Foo");
//         self.other.scan(ctx);
//         self.items.scan(ctx);
//     }
//
//     fn root(&self) {
//         self.other.root();
//         self.items.root();
//     }
//
//     fn unroot(&self) {
//         self.other.unroot();
//         self.items.unroot();
//     }
// }

#[derive(Debug)]
struct Bar {
    value: Cell<u32>,
    other: Option<Gc<Foo>>,
}

unsafe impl Scan for Bar {
    fn scan(&self, ctx: &mut Context) {
        println!("Scan Bar");
        self.value.set(self.value.get() + 1);
        self.other.scan(ctx);
    }

    fn root(&self) {
        self.other.root();
    }

    fn unroot(&self) {
        self.other.unroot();
    }
}

/// Test for rooting and unrooting rules.
#[test]
fn test_gc_rooting() {
    let mut gc = Collector::new();

    let bar = Bar {
        value: Cell::new(1000),
        other: None,
    };
    let bar_gc = gc.alloc(bar);
    let foo_2 = Foo {
        value: 20000,
        other: bar_gc.clone(),
        items: vec![],
    };
    let foo = Foo {
        value: 10000,
        other: bar_gc.clone(),
        items: vec![
            Bar {
                value: Cell::new(2000),
                other: None,
            },
            Bar {
                value: Cell::new(3000),
                other: Some(gc.alloc(foo_2)),
            },
        ],
    };
    drop(bar_gc); // Decrement reference count
    println!("Gc len {}", gc.len()); //> 2

    // By moving the foo into a `Gc<T>`, it becomes a root, and its contents are no longer considered roots.
    println!("Alloc foo");
    let foo_gc = gc.alloc(foo);
    println!("Gc len {}", gc.len()); //> 3

    assert!(Gc::is_root(&foo_gc), "GC pointer outside of arena must be a root.");
    // println!("size of Gc on stack {}", std::mem::size_of::<Gc<Foo>>());
    println!("{:#?}", foo_gc);

    gc.collect();
    println!("Gc len {}", gc.len()); //> 3

    println!("{:#?}", foo_gc);

    let foo_3 = {
        let bar = gc.alloc(Bar {
            value: Cell::new(4000),
            other: None,
        });
        gc.alloc(Foo {
            value: 30000,
            other: bar,
            items: vec![],
        })
    };

    // Foo is no longer reachable.
    drop(foo_gc);
    println!("Gc len {}", gc.len()); //> 3
    assert_eq!(gc.len(), 5);

    gc.collect();
    println!("Gc len {}", gc.len()); //> 0
    assert_eq!(gc.len(), 2);

    // println!("{:?}", foo_3);
    drop(foo_3);
}
