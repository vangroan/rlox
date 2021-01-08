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
