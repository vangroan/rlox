//! Scan implementations for `std` types.

use crate::context::Context;
use crate::scan::Scan;

impl<T: Scan> Scan for Vec<T> {
    fn scan(&self, ctx: &mut Context<'_>) {
        for item in self {
            item.scan(ctx);
        }
    }

    unsafe fn root(&self) {
        todo!()
    }

    unsafe fn unroot(&self) {
        todo!()
    }
}
