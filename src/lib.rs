//! # polars-formula
//!
//! A parsing and materialization library for Rust that brings
//! R-style and Python Patsy/Formulaic formula syntax to the Polars DataFrame ecosystem.
//!
//! ## Overview
//!
//! This library provides a simple, focused API for parsing statistical model formulas
//! and materializing them into design matrices. It seamlessly integrates with Polars
//! DataFrames and provides efficient conversion to faer matrices for linear algebra operations.
//!
//! ## Simple API
//!
//! The library exposes just 4 main functions for a clean, focused experience:
//!
//! ```rust
//! use polars::prelude::*;
//! use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};
//!
//! // 1. Parse and canonicalize a formula
//! let spec = canonicalize("y ~ x1 + x2")?;
//!
//! // 2. Print the canonical formula with colors
//! print_formula(&spec);
//!
//! // 3. Materialize against your data
//! let (y, x, z) = materialize(&spec, &df)?;
//!
//! // 4. Inspect the full model specification
//! print_modelspec(&spec);
//! ```
//!
//! ## Supported Syntax
//!
//! | Syntax | Description |
//! |--------|-------------|
//! | `y ~ x1 + x2` | Linear regression |
//! | `y ~ x1 * x2` | Product terms (expands to x1 + x2 + x1:x2) |
//! | `y ~ x1:x2` | Interaction terms |
//! | `y ~ poly(x1, 2)` | Polynomial terms (x, x², x³, ...) |
//! | `y ~ (1\|group)` | Random intercepts |
//! | `y ~ (x\|group)` | Random slopes |
//! | `y ~ (x\|\|group)` | Uncorrelated random effects |
//! | `y ~ I(x)` | Identity function (literal interpretation) |
//! | `y ~ x^2` | Power terms |
//! | `y ~ (a+b)^3` | Polynomial expansion |
//! | `y ~ a/b` | Nesting (a + a:b) |
//! | `y ~ b %in% a` | Nesting (b within a) |
//! | `y \| weights(w) ~ x` | Auxiliary terms (weights, se, trials, etc.) |
//! | `Surv(time, event) ~ x` | Survival analysis |
//! | `cbind(success, failure) ~ x` | Multivariate responses |
//! | `s(x, k=10, bs="tp")` | Smooth terms (s, t2, te, ti) |
//! | `family=gaussian() y ~ x` | Distribution families |
//! | `y ~ x + sigma ~ z` | Distributional parameters |
//! | `y ~ x + ar(p=1)` | Autocorrelation terms |

#![forbid(unsafe_code)]

use chumsky::Parser as ChumskyParser;
use polars::prelude::*;
use thiserror::Error;

// Internal implementation modules - not exposed to users
mod internal;

// Re-export the error type for users
#[derive(Debug, Error)]
pub enum Error {
    /// Parse error during syntax analysis.
    ///
    /// Occurs when the formula string doesn't conform to the expected
    /// formula grammar.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars_formula::{canonicalize, Error};
    ///
    /// // Incomplete expression
    /// match canonicalize("y ~ x +") {
    ///     Err(Error::Parse { pos, msg }) => {
    ///         assert!(msg.contains("unexpected end"));
    ///     }
    ///     _ => panic!("Expected parse error"),
    /// }
    /// ```
    #[error("parse error at token {pos:?}: {msg}")]
    Parse {
        /// Token position where the error occurred (None if at end)
        pos: Option<usize>,
        /// Description of the parsing error
        msg: String,
    },

    /// Semantic error during formula materialization.
    ///
    /// Occurs when the formula is syntactically correct but cannot be
    /// materialized against the provided data due to missing columns,
    /// type conversion failures, or other data-related issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars::prelude::*;
    /// use polars_formula::{canonicalize, materialize, Error};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let df = df!("x" => [1, 2, 3])?; // Note: integers, not floats
    /// let spec = canonicalize("missing_column ~ x")?;
    ///
    /// match materialize(&spec, &df) {
    ///     Err(Error::Semantic(msg)) => {
    ///         assert!(msg.contains("unknown column 'missing_column'"));
    ///     }
    ///     _ => panic!("Expected semantic error"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[error("semantic error: {0}")]
    Semantic(
        /// Description of the semantic error
        String,
    ),
}

// --- Top-level API Functions -------------------------------------------------------

/// Parse and canonicalize a formula string into a ModelSpec.
///
/// This function takes a formula string, parses it, and returns the canonicalized
/// ModelSpec. This is the primary entry point for formula processing.
///
/// # Arguments
///
/// * `formula` - A formula string to parse (e.g., `"y ~ x1 + x2"`)
///
/// # Returns
///
/// Returns a `Result<ModelSpec, Error>` containing the parsed and canonicalized
/// formula or an error if the formula syntax is invalid.
///
/// # Examples
///
/// ```rust
/// use polars_formula::canonicalize;
///
/// let spec = canonicalize("y ~ x1 + x2")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn canonicalize(formula: &str) -> Result<internal::dsl::ModelSpec, Error> {
    let model_spec = internal::dsl::parser::parser()
        .parse(formula.chars().collect::<Vec<_>>())
        .map_err(|e| Error::Parse {
            pos: None,
            msg: format!("Parse error: {:?}", e),
        })?;

    Ok(internal::dsl::canon::canonicalize(&model_spec))
}

/// Materialize a ModelSpec against a DataFrame to produce design matrices.
///
/// This function takes a ModelSpec and materializes it into concrete numeric
/// matrices suitable for statistical modeling.
///
/// # Arguments
///
/// * `spec` - The ModelSpec to materialize
/// * `df` - The DataFrame containing the data to materialize against
///
/// # Returns
///
/// Returns `Result<(DataFrame, DataFrame, DataFrame), Error>` where:
/// - First DataFrame: The response variable(s) (y)
/// - Second DataFrame: The fixed effects design matrix (X)
/// - Third DataFrame: The random effects design matrix (Z)
///
/// # Examples
///
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{canonicalize, materialize};
///
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let spec = canonicalize("y ~ x")?;
/// let (y, x, z) = materialize(&spec, &df)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn materialize(
    spec: &internal::dsl::ModelSpec,
    df: &DataFrame,
) -> Result<(DataFrame, DataFrame, DataFrame), Error> {
    // Create default materialization options
    let opts = internal::dsl::MaterializeOptions::default();
    internal::dsl::materialize::materialize(df, spec, opts)
}

/// Print the canonical formula with syntax highlighting.
///
/// This function takes a ModelSpec and prints its canonical form with
/// colored syntax highlighting for better readability.
///
/// # Arguments
///
/// * `spec` - The ModelSpec to print
///
/// # Examples
///
/// ```rust
/// use polars_formula::{canonicalize, print_formula};
///
/// let spec = canonicalize("y ~ x1 * x2")?;
/// print_formula(&spec);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn print_formula(spec: &internal::dsl::ModelSpec) {
    let canonicalized_str = internal::dsl::pretty::pretty(spec);
    let color_pretty = internal::color::Color::default();
    println!("{}", color_pretty.formula(&canonicalized_str));
}

/// Pretty print the full ModelSpec structure.
///
/// This function takes a ModelSpec and prints its complete structure
/// in a human-readable format.
///
/// # Arguments
///
/// * `spec` - The ModelSpec to print
///
/// # Examples
///
/// ```rust
/// use polars_formula::{canonicalize, print_modelspec};
///
/// let spec = canonicalize("y ~ x1 + x2")?;
/// print_modelspec(&spec);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn print_modelspec(spec: &internal::dsl::ModelSpec) {
    println!("ModelSpec:");
    println!("  Family: {:?}", spec.family);
    println!("  Link: {:?}", spec.link);
    println!("  Formula: {}", internal::dsl::pretty::pretty(spec));
    println!("  Distributional Parameters: {:?}", spec.dpars);
    println!("  Autocorrelation: {:?}", spec.autocor);
}

// --- Utility functions ----------------------------------------------------

/// Clean column names for use in statistical modeling.
///
/// This function converts column names to a clean format suitable for
/// statistical modeling by:
/// Internal function to clean column names by converting to lowercase,
/// replacing special characters with underscores, and handling edge cases.
fn make_clean_names(name: &str) -> String {
    if name.trim().is_empty() {
        return "column".to_string();
    }

    let mut result = String::new();
    let mut prev_was_underscore = false;

    for ch in name.chars() {
        let is_special = ch.is_whitespace()
            || matches!(
                ch,
                '!' | '@'
                    | '#'
                    | '$'
                    | '%'
                    | '^'
                    | '&'
                    | '*'
                    | '('
                    | ')'
                    | '-'
                    | '+'
                    | '='
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '|'
                    | '\\'
                    | '`'
                    | '~'
                    | ':'
                    | ','
            );

        if is_special {
            if !prev_was_underscore {
                result.push('_');
                prev_was_underscore = true;
            }
        } else if ch.is_alphanumeric() || ch == '_' {
            result.push(ch.to_ascii_lowercase());
            prev_was_underscore = false;
        }
        // Skip other characters entirely
    }

    // Remove leading and trailing underscores
    result = result.trim_matches('_').to_string();

    // Handle empty result
    if result.is_empty() {
        result = "column".to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_basic() {
        let spec = canonicalize("y ~ x1 + x2").expect("Failed to parse formula");
        assert!(matches!(spec.formula.lhs, internal::dsl::Response::Var(_)));
    }

    #[test]
    fn test_materialize_basic() {
        // Create test data
        let df = df!(
            "y" => [1.0, 2.0, 3.0],
            "x1" => [1.0, 2.0, 3.0],
            "x2" => [1.0, 2.0, 3.0]
        )
        .expect("Failed to create test DataFrame");

        let spec = canonicalize("y ~ x1 + x2").expect("Failed to parse formula");

        // Materialize the formula
        let (y, x, z) = materialize(&spec, &df).expect("Failed to materialize formula");

        // Test LHS (y)
        assert_eq!(y.width(), 1);
        assert_eq!(y.height(), 3);

        // Test RHS (X matrix)
        assert_eq!(x.width(), 3); // Intercept + x1 + x2
        assert_eq!(x.height(), 3);

        // Test Z matrix (should be empty for this model)
        assert_eq!(z.width(), 0);
        assert_eq!(z.height(), 0);
    }

    #[test]
    fn test_make_clean_names() {
        // Basic cleaning
        assert_eq!(make_clean_names("My Column"), "my_column");
        assert_eq!(make_clean_names("x1:x2"), "x1_x2");
        assert_eq!(make_clean_names("poly(x,2)^1"), "poly_x_2_1");

        // Polynomial terms
        assert_eq!(make_clean_names("poly(income,3)^2"), "poly_income_3_2");
        assert_eq!(make_clean_names("poly(age,2)^1"), "poly_age_2_1");

        // Interaction terms
        assert_eq!(make_clean_names("treatment:dose"), "treatment_dose");
        assert_eq!(make_clean_names("(x1+x2):z"), "x1_x2_z");

        // Special characters
        assert_eq!(make_clean_names("Column Name!"), "column_name");
        assert_eq!(make_clean_names("x@#$%"), "x");

        // Edge cases
        assert_eq!(make_clean_names(""), "column");
        assert_eq!(make_clean_names("___"), "column");
        assert_eq!(make_clean_names("   "), "column");
    }
}
