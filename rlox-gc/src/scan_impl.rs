//! Scan implementations for `std` types.

use crate::context::Context;
use crate::scan::Scan;

impl<T: Scan> Scan for Option<T> {
    fn scan(&self, ctx: &mut Context<'_>) {
        if let Some(val) = self {
            val.scan(ctx);
        }
    }

    unsafe fn root(&self) {
        if let Some(val) = self {
            val.root();
        }
    }

    unsafe fn unroot(&self) {
        if let Some(val) = self {
            val.unroot();
        }
    }
}

impl<T: Scan> Scan for Vec<T> {
    fn scan(&self, ctx: &mut Context<'_>) {
        println!("Scan Vec<T>");
        for item in self {
            println!("  Scan Vec<T> item");
            item.scan(ctx);
        }
    }

    unsafe fn root(&self) {
        todo!()
    }

    unsafe fn unroot(&self) {
        for item in self {
            item.unroot();
        }
    }
}
