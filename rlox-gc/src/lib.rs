mod arena;
mod cell;
mod collect;
pub mod context;
mod gc;
pub mod scan;
mod scan_impl;

pub use collect::Collector;
pub use gc::Gc;

// TODO: Allocator
// TODO: Trace trait
// TODO: Derive Codegen
// TODO: Gc<T>
// TODO: GcBox<T> - Pointer to heap header info.
// TODO: Collector - Tri-color mark and sweep
// TODO: Mark-and-Copy - Generational semi-spaces
