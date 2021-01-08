#![cfg(feature = "derive")]

use rlox_gc::derive::Scan;

#[test]
fn test_basic_derive() {
    #[derive(Scan)]
    struct Foo {
        a: u32,
        b: u64,
        c: u128,
        d: i32,
        e: i64,
        f: i128,
        g: String,
    }

    #[derive(Scan)]
    struct Bar(u16, u32, u128, i32, i64, i128, String);

    // Zero sized
    #[derive(Scan)]
    struct Baz;
}
