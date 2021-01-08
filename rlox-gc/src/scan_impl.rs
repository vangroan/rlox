//! Scan implementations for `std` types.
use crate::{context::Context, scan::Scan};

macro_rules! static_scan {
    ($v:ty) => {
        unsafe impl Scan for $v {
            fn scan(&self, _: &mut Context) {}

            fn root(&self) {}

            fn unroot(&self) {}
        }
    };
}

static_scan!(());

unsafe impl<T: Scan> Scan for Option<T> {
    fn scan(&self, ctx: &mut Context<'_>) {
        if let Some(val) = self {
            val.scan(ctx);
        }
    }

    fn root(&self) {
        if let Some(val) = self {
            val.root();
        }
    }

    fn unroot(&self) {
        if let Some(val) = self {
            val.unroot();
        }
    }
}

unsafe impl<T: Scan> Scan for Vec<T> {
    fn scan(&self, ctx: &mut Context<'_>) {
        println!("Scan Vec<T>");
        for item in self {
            println!("  Scan Vec<T> item");
            item.scan(ctx);
        }
    }

    fn root(&self) {
        todo!()
    }

    fn unroot(&self) {
        for item in self {
            item.unroot();
        }
    }
}
