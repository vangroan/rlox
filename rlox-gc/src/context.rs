use crate::{gc::GcBox, scan::Scan};
use std::ptr::NonNull;

/// Ephemeral context used during scan.
pub struct Context<'ctx> {
    /// Queue of objects that have been marked gray during a scan.
    pub gray: &'ctx mut Vec<NonNull<GcBox<dyn Scan>>>,
}
