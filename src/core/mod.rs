//! The core owns the file format, the store, context capture, the guard and queries.
//! Both surfaces call only the core.

pub mod config;
pub mod context;
pub mod format;
mod fsutil;
pub mod store;
