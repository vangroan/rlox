//! Scan implementations for `std` types.
use crate::{context::Context, scan::Scan};
use std::collections::HashMap;

macro_rules! static_scan {
    ($v:ty) => {
        unsafe impl Scan for $v {
            #[inline(always)]
            fn scan(&self, _: &mut Context) {}

            #[inline(always)]
            fn root(&self) {}

            #[inline(always)]
            fn unroot(&self) {}
        }
    };
}

static_scan!(());
static_scan!(bool);
static_scan!(u8);
static_scan!(u16);
static_scan!(u32);
static_scan!(u64);
static_scan!(u128);
static_scan!(i8);
static_scan!(i16);
static_scan!(i32);
static_scan!(i64);
static_scan!(i128);
static_scan!(f32);
static_scan!(f64);
static_scan!(String);

macro_rules! array_scan {
    ($n:literal) => {
        unsafe impl<T: Scan> Scan for [T; $n] {
            #[inline(always)]
            fn scan(&self, ctx: &mut Context) {
                for elem in self {
                    elem.scan(ctx)
                }
            }

            #[inline(always)]
            fn root(&self) {
                for elem in self {
                    elem.root()
                }
            }

            #[inline(always)]
            fn unroot(&self) {
                for elem in self {
                    elem.unroot()
                }
            }
        }
    };
}

array_scan!(0);
array_scan!(1);
array_scan!(2);
array_scan!(3);
array_scan!(4);
array_scan!(5);
array_scan!(6);
array_scan!(7);
array_scan!(8);
array_scan!(9);
array_scan!(10);
array_scan!(11);
array_scan!(12);
array_scan!(13);
array_scan!(14);
array_scan!(15);
array_scan!(16);
array_scan!(17);
array_scan!(18);
array_scan!(19);
array_scan!(20);
array_scan!(21);
array_scan!(22);
array_scan!(23);
array_scan!(24);
array_scan!(25);
array_scan!(26);
array_scan!(27);
array_scan!(28);
array_scan!(29);
array_scan!(30);
array_scan!(31);
array_scan!(32);

unsafe impl<T: Scan> Scan for Option<T> {
    #[inline]
    fn scan(&self, ctx: &mut Context<'_>) {
        if let Some(val) = self {
            val.scan(ctx);
        }
    }

    #[inline]
    fn root(&self) {
        if let Some(val) = self {
            val.root();
        }
    }

    #[inline]
    fn unroot(&self) {
        if let Some(val) = self {
            val.unroot();
        }
    }
}

unsafe impl<T: Scan> Scan for Vec<T> {
    #[inline]
    fn scan(&self, ctx: &mut Context<'_>) {
        println!("Scan Vec<T>");
        for item in self {
            println!("  Scan Vec<T> item");
            item.scan(ctx);
        }
    }

    #[inline]
    fn root(&self) {
        for item in self {
            item.root();
        }
    }

    #[inline]
    fn unroot(&self) {
        for item in self {
            item.unroot();
        }
    }
}

unsafe impl<K, V> Scan for HashMap<K, V>
where
    K: Scan,
    V: Scan,
{
    #[inline]
    fn scan(&self, ctx: &mut Context<'_>) {
        println!("Scan HashMap<K, V>");
        for (k, v) in self {
            k.scan(ctx);
            v.scan(ctx);
        }
    }

    #[inline]
    fn root(&self) {
        for (k, v) in self {
            k.root();
            v.root();
        }
    }

    #[inline]
    fn unroot(&self) {
        for (k, v) in self {
            k.unroot();
            v.unroot();
        }
    }
}
