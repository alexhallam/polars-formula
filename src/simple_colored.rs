//! # Simple Colored Pretty Printer (DEPRECATED)
//!
//! ⚠️ **This module is deprecated.** Use the main `color` module instead.
//!
//! The functionality has been consolidated into the main `color` module for better
//! maintainability and consistency.

#[deprecated(since = "0.1.2", note = "Use the main color module instead")]
pub use crate::color::{ColorConfig, ColoredPretty as SimpleColoredPretty};
