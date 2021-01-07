//! Macro tests cannot live in the special proc macro package.
use rlox_derive::array_init;

/// Importantly, this type does not implement `Clone` or `Copy`.
#[derive(Debug, PartialEq, Eq)]
struct Foo {
    value: u32,
}

#[test]
fn test_array_init() {
    let arr1 = array_init!(Foo, [Foo { value: 169_093_200 }; 16]);
    for el in &arr1 {
        assert_eq!(el, &Foo { value: 169_093_200 });
    }
}
