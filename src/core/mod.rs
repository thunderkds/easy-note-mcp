//! The core owns the file format, the store, context capture, the guard and queries.
//! Both surfaces call only the core.

pub mod ai;
pub mod ai_view;
pub mod config;
pub mod context;
pub mod format;
pub(crate) mod fsutil;
pub mod guard;
pub mod overrides;
pub mod query;
pub mod store;
pub mod tags;
