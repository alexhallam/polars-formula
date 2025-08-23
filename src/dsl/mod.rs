//! DSL module for polars-formula
//!
//! This module provides a comprehensive formula DSL implementation with:
//! - Rich AST representation
//! - Chumsky-based parser
//! - Canonicalization passes
//! - Pretty-printing
//! - Property-based testing

pub mod ast;
pub mod canon;
pub mod materialize;
pub mod parser;
pub mod pretty;

pub use ast::*;
pub use canon::*;
pub use materialize::*;
pub use parser::*;
pub use pretty::*;
