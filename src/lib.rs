//! # polars-formula
//!
//! A parsing and materialization library for Rust that brings
//! R-style and Python Patsy/Formulaic formula syntax to the Polars DataFrame ecosystem.
//!
//! ## Overview
//!
//! This library provides a comprehensive solution for parsing statistical model formulas
//! and materializing them into design matrices. It seamlessly integrates with Polars
//! DataFrames and provides efficient conversion to faer matrices for linear algebra operations.
//!
//! The DSL parser supports:
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
//!
//! ## Features
//!
//! - **Formula Parsing**: Parse R-style formulas like `y ~ x1 + x2 + x1:x2`
//! - **Polynomial Terms**: Support for polynomial expansions with `poly(x, degree)`
//! - **Interactions**: Automatic handling of interaction terms using `:`
//! - **Intercept Control**: Flexible intercept inclusion/exclusion

#![forbid(unsafe_code)]

use chumsky::Parser as ChumskyParser;
use polars::prelude::*;
use thiserror::Error;

// DSL module for the comprehensive formula parser
pub mod color;
pub mod dsl;

// --- Public surface -------------------------------------------------------

// Re-export colored pretty-printing
pub use color::{Color, ColorConfig};

/// A parsed statistical formula that can be materialized into design matrices.
///
/// `Formula` represents a statistical model formula (like `y ~ x1 + x2 + x1:x2`) that has been
/// parsed into an internal abstract syntax tree. It can be materialized against a DataFrame
/// to produce response variables and design matrices suitable for statistical modeling.
///
/// # Examples
///
/// ## Basic Linear Model
///
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "sales" => [100.0, 200.0, 300.0, 400.0],
///     "advertising" => [10.0, 20.0, 30.0, 40.0],
///     "price" => [9.99, 8.99, 7.99, 6.99]
/// )?;
///
/// let formula = Formula::parse("sales ~ advertising + price")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// println!("Response: {}", y.name()); // "sales"
/// println!("Predictors: {:?}", X.get_column_names()); // ["Intercept", "advertising", "price"]
/// # Ok(())
/// # }
/// ```
///
/// ## Quadratic Regression
///
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 4.0, 9.0, 16.0, 25.0],
///     "x" => [1.0, 2.0, 3.0, 4.0, 5.0]
/// )?;
///
/// let formula = Formula::parse("y ~ poly(x, 2)")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // X contains [Intercept, x, x²]
/// assert_eq!(X.width(), 3);
/// # Ok(())
/// # }
/// ```
///
/// ## Model with Interactions
///
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "outcome" => [10.0, 20.0, 30.0, 40.0],
///     "treatment" => [1.0, 0.0, 1.0, 0.0],
///     "dose" => [5.0, 5.0, 10.0, 10.0]
/// )?;
///
/// let formula = Formula::parse("outcome ~ treatment + dose + treatment:dose")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // X contains: [Intercept, treatment, dose, treatment:dose]
/// # Ok(())
/// # }
/// ```
pub struct Formula {
    /// The parsed DSL ModelSpec of the formula.
    ///
    /// This field is public to allow advanced users to inspect or modify
    /// the parsed formula structure before materialization.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars_formula::Formula;
    ///
    /// let formula = Formula::parse("y ~ x1 + x2")?;
    ///
    /// // Inspect the parsed structure
    /// println!("Response: {:?}", formula.spec.formula.lhs);
    /// println!("Predictors: {:?}", formula.spec.formula.rhs);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub spec: dsl::ModelSpec,
}

impl Formula {
    /// Parse a formula string into a `Formula` object.
    ///
    /// This method parses a statistical formula string using R-style syntax and creates
    /// a `Formula` object that can be materialized against DataFrames.
    ///
    /// # Arguments
    ///
    /// * `src` - A formula string to parse (e.g., `"y ~ x1 + x2"`)
    ///
    /// # Returns
    ///
    /// Returns a `Result<Formula, Error>` containing the parsed formula or an error
    /// if the formula syntax is invalid.
    ///
    /// # Supported Syntax
    ///
    /// - **Variables**: `x`, `income`, `age`
    /// - **Response**: `y ~ ...` (left side of `~`)
    /// - **Predictors**: `x1 + x2` (right side of `~`)
    /// - **Interactions**: `x1:x2` (product terms)
    /// - **Polynomials**: `poly(x, degree)`
    /// - **Grouping**: `(x1 + x2):z`
    /// - **Constants**: `1`, `0`
    ///
    /// # Examples
    ///
    /// ## Simple Linear Regression
    /// ```rust
    /// use polars_formula::Formula;
    ///
    /// let formula = Formula::parse("price ~ size")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Multiple Linear Regression
    /// ```rust
    /// use polars_formula::Formula;
    ///
    /// let formula = Formula::parse("sales ~ advertising + price + season")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Polynomial Terms
    /// ```rust
    /// use polars_formula::Formula;
    ///
    /// let formula = Formula::parse("y ~ poly(x, 3)")?; // x + x² + x³
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Interaction Effects
    /// ```rust
    /// use polars_formula::Formula;
    ///
    /// let formula = Formula::parse("outcome ~ drug + dose + drug:dose")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Complex Formulas
    /// ```rust
    /// use polars_formula::Formula;
    ///
    /// let formula = Formula::parse("y ~ poly(x1, 2) + x2 + (x1 + x2):x3")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error for:
    /// - Invalid syntax: `Formula::parse("y ~~ x")`
    /// - Incomplete expressions: `Formula::parse("y ~ x +")`
    /// - Invalid function calls: `Formula::parse("y ~ poly(x,)")`
    pub fn parse(src: &str) -> Result<Self, Error> {
        let p = dsl::parser();
        let spec = p
            .parse(src.chars().collect::<Vec<_>>())
            .map_err(|e| Error::Parse {
                pos: None,
                msg: format!("DSL parse error: {:?}", e),
            })?;
        Ok(Self { spec })
    }

    /// Materialize the formula against a DataFrame to produce design matrices.
    ///
    /// This method takes a DataFrame and materializes the parsed formula into concrete
    /// numeric matrices suitable for statistical modeling. It returns a tuple of
    /// `(response_variable, design_matrix)`.
    ///
    /// # Arguments
    ///
    /// * `df` - The DataFrame containing the data to materialize against
    /// * `opts` - Options controlling materialization behavior (intercept, naming, etc.)
    ///
    /// # Returns
    ///
    /// Returns `(Series, DataFrame)` where:
    /// - `Series`: The response variable (left side of `~`)
    /// - `DataFrame`: The design matrix (right side of `~` with transformations applied)
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    /// ```rust
    /// use polars::prelude::*;
    /// use polars_formula::{Formula, MaterializeOptions};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let df = df!(
    ///     "mpg" => [25.0, 30.0, 20.0, 35.0],
    ///     "weight" => [3000.0, 2500.0, 4000.0, 2000.0],
    ///     "horsepower" => [150.0, 120.0, 200.0, 100.0]
    /// )?;
    ///
    /// let formula = Formula::parse("mpg ~ weight + horsepower")?;
    /// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
    ///
    /// println!("Response variable: {}", y.name()); // "mpg"
    /// println!("Design matrix columns: {:?}", X.get_column_names());
    /// // ["Intercept", "weight", "horsepower"]
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Polynomial Features
    /// ```rust
    /// use polars::prelude::*;
    /// use polars_formula::{Formula, MaterializeOptions};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let df = df!(
    ///     "y" => [1.0, 8.0, 27.0, 64.0],
    ///     "x" => [1.0, 2.0, 3.0, 4.0]
    /// )?;
    ///
    /// let formula = Formula::parse("y ~ poly(x, 3)")?;
    /// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
    ///
    /// // X contains: [Intercept, x, x², x³]
    /// assert_eq!(X.width(), 4);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Without Intercept
    /// ```rust
    /// use polars::prelude::*;
    /// use polars_formula::{Formula, MaterializeOptions};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let df = df!(
    ///     "y" => [1.0, 2.0, 3.0],
    ///     "x" => [1.0, 2.0, 3.0]
    /// )?;
    ///
    /// let formula = Formula::parse("y ~ x")?;
    /// let opts = MaterializeOptions {
    ///     rhs_intercept: false,
    ///     ..Default::default()
    /// };
    /// let (y, X) = formula.materialize(&df, opts)?;
    ///
    /// // X contains only: [x] (no intercept column)
    /// assert_eq!(X.width(), 1);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Type Conversions
    ///
    /// All columns are automatically cast to `f64` for numerical computation.
    /// The materialization process handles:
    /// - Integer to float conversion
    /// - Boolean to float conversion (false=0.0, true=1.0)
    /// - Polynomial expansion
    /// - Interaction term computation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Referenced columns don't exist in the DataFrame
    /// - Type conversion to f64 fails
    /// - The left-hand side produces multiple columns
    /// - Memory allocation fails for large matrices
    pub fn materialize(
        &self,
        df: &DataFrame,
        opts: MaterializeOptions,
    ) -> Result<(DataFrame, DataFrame), Error> {
        let (y, x, _z) = dsl::materialize(df, &self.spec, opts)?;
        // For backward compatibility, return (y, x) and ignore random effects for now
        Ok((y, x))
    }
}

/// Configuration options for formula materialization.
///
/// `MaterializeOptions` controls how formulas are materialized into design matrices.
/// This includes intercept handling, column naming, and other transformation behaviors.
///
/// # Examples
///
/// ## Default Options (with intercept and clean names)
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let formula = Formula::parse("y ~ x")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // X contains: [intercept, x] (names are cleaned by default)
/// assert_eq!(X.width(), 2);
/// assert!(X.get_column_names().iter().any(|s| s.as_str() == "intercept"));
/// # Ok(())
/// # }
/// ```
///
/// ## Without Intercept
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let formula = Formula::parse("y ~ x")?;
/// let opts = MaterializeOptions {
///     rhs_intercept: false,
///     ..Default::default()
/// };
/// let (y, X) = formula.materialize(&df, opts)?;
///
/// // X contains only: [x] (names are cleaned by default)
/// assert_eq!(X.width(), 1);
/// assert!(!X.get_column_names().iter().any(|s| s.as_str() == "intercept"));
/// # Ok(())
/// # }
/// ```
///
/// ## Without Clean Column Names
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let formula = Formula::parse("y ~ poly(x,2)")?;
/// let opts = MaterializeOptions {
///     clean_names: false,
///     ..Default::default()
/// };
/// let (y, X) = formula.materialize(&df, opts)?;
///
/// // X contains: [Intercept, poly(x,2)^1, poly(x,2)^2] (original names)
/// assert_eq!(X.width(), 3);
/// assert!(X.get_column_names().iter().any(|s| s.as_str() == "Intercept"));
/// assert!(X.get_column_names().iter().any(|s| s.as_str() == "poly(x,2)^1"));
/// # Ok(())
/// # }
/// ```
///
/// ## Custom Intercept Name
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let formula = Formula::parse("y ~ x")?;
/// let opts = MaterializeOptions {
///     rhs_intercept: true,
///     intercept_name: "Constant",
///     clean_names: true,
/// };
/// let (y, X) = formula.materialize(&df, opts)?;
///
/// // X contains: [constant, x] (names are cleaned by default)
/// assert!(X.get_column_names().iter().any(|s| s.as_str() == "constant"));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MaterializeOptions {
    /// Whether to include an intercept term in the design matrix.
    ///
    /// When `true` (default), adds a column of 1s to the design matrix.
    /// This is standard for most regression models. Set to `false` for
    /// models that should pass through the origin.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars_formula::MaterializeOptions;
    ///
    /// // With intercept (default)
    /// let opts_with = MaterializeOptions::default();
    /// assert_eq!(opts_with.rhs_intercept, true);
    ///
    /// // Without intercept
    /// let opts_without = MaterializeOptions {
    ///     rhs_intercept: false,
    ///     ..Default::default()
    /// };
    /// assert_eq!(opts_without.rhs_intercept, false);
    /// ```
    pub rhs_intercept: bool,

    /// Name to use for the intercept column when `rhs_intercept` is `true`.
    ///
    /// This determines the column name in the resulting design matrix.
    /// Common choices include "Intercept", "Constant", or "(Intercept)".
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars_formula::MaterializeOptions;
    ///
    /// // Default name
    /// let opts_default = MaterializeOptions::default();
    /// assert_eq!(opts_default.intercept_name, "Intercept");
    ///
    /// // Custom name
    /// let opts_custom = MaterializeOptions {
    ///     intercept_name: "Constant",
    ///     ..Default::default()
    /// };
    /// assert_eq!(opts_custom.intercept_name, "Constant");
    /// ```
    pub intercept_name: &'static str,

    /// Whether to clean column names using `make_clean_names()`.
    ///
    /// When `true` (default), applies the `make_clean_names()` function to all column names
    /// in the resulting design matrix. This makes column names more user-friendly
    /// by converting special characters to underscores and using lowercase.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars_formula::MaterializeOptions;
    ///
    /// // With cleaning (default)
    /// let opts_default = MaterializeOptions::default();
    /// assert_eq!(opts_default.clean_names, true);
    ///
    /// // Without cleaning
    /// let opts_no_clean = MaterializeOptions {
    ///     clean_names: false,
    ///     ..Default::default()
    /// };
    /// assert_eq!(opts_no_clean.clean_names, false);
    /// ```
    pub clean_names: bool,
}

impl Default for MaterializeOptions {
    fn default() -> Self {
        Self {
            rhs_intercept: true,
            intercept_name: "Intercept",
            clean_names: true,
        }
    }
}

// --- Errors ---------------------------------------------------------------

/// Errors that can occur during formula parsing and materialization.
///
/// This enum represents all possible errors that can arise when working with
/// polars-formula, from parsing syntax errors to data materialization issues.
///
/// # Examples
///
/// ## Handling Parse Errors
/// ```rust
/// use polars_formula::{Formula, Error};
///
/// // This will produce a parse error
/// match Formula::parse("y ~~ x") {
///     Err(Error::Parse { pos, msg }) => {
///         println!("Parse error: {}", msg);
///     }
///     Err(e) => println!("Other error: {}", e),
///     Ok(_) => unreachable!(),
/// }
/// ```
///
/// ## Handling Semantic Errors
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, Error};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!("x" => [1.0, 2.0, 3.0])?;
/// let formula = Formula::parse("y ~ x")?; // y doesn't exist
///
/// match formula.materialize(&df, MaterializeOptions::default()) {
///     Err(Error::Semantic(msg)) => {
///         println!("Data error: {}", msg); // "unknown column 'y'"
///     }
///     Err(e) => println!("Other error: {}", e),
///     Ok(_) => unreachable!(),
/// }
/// # Ok(())
/// # }
/// ```
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
    /// use polars_formula::{Formula, Error};
    ///
    /// // Incomplete expression
    /// match Formula::parse("y ~ x +") {
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
    /// use polars_formula::{Formula, MaterializeOptions, Error};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let df = df!("x" => [1, 2, 3])?; // Note: integers, not floats
    /// let formula = Formula::parse("missing_column ~ x")?;
    ///
    /// match formula.materialize(&df, MaterializeOptions::default()) {
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

// --- Utility functions ----------------------------------------------------

/// Convert a Polars DataFrame to a dense faer matrix for linear algebra operations.
///
/// This function converts a numeric DataFrame (typically a design matrix) into a dense
/// faer matrix suitable for high-performance linear algebra computations. All columns
/// must be convertible to `f64`.
///
/// # Arguments
///
/// * `df` - A reference to a DataFrame with numeric columns
///
/// # Returns
///
/// Returns a `Result<faer::Mat<f64>, Error>` containing a dense matrix with dimensions
/// `n × p` where `n` is the number of rows and `p` is the number of columns.
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::polars_to_faer;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "x1" => [1.0, 2.0, 3.0],
///     "x2" => [4.0, 5.0, 6.0]
/// )?;
///
/// let matrix = polars_to_faer(&df)?;
///
/// // Matrix dimensions: 3 rows × 2 columns
/// assert_eq!(matrix.nrows(), 3);
/// assert_eq!(matrix.ncols(), 2);
///
/// // Access elements: matrix[(row, col)]
/// assert_eq!(matrix[(0, 0)], 1.0);
/// assert_eq!(matrix[(1, 1)], 5.0);
/// # Ok(())
/// # }
/// ```
///
/// ## With Formula Materialization
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, polars_to_faer};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0, 4.0],
///     "x" => [1.0, 2.0, 3.0, 4.0]
/// )?;
///
/// let formula = Formula::parse("y ~ x")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // Convert design matrix to faer for linear algebra
/// let X_matrix = polars_to_faer(&X)?;
///
/// println!("Design matrix shape: {}×{}", X_matrix.nrows(), X_matrix.ncols());
/// // Can now perform matrix operations like X^T * X
/// # Ok(())
/// # }
/// ```
///
/// # Performance Notes
///
/// - The conversion creates a dense matrix in column-major format
/// - All data is copied from Polars to faer representation
/// - For large matrices, consider chunking operations if memory is limited
/// - faer matrices are optimized for BLAS/LAPACK operations
///
/// # Errors
///
/// Returns an error if:
/// - Any column cannot be converted to `f64`
/// - The DataFrame contains non-contiguous data (call `.rechunk()` first)
/// - Memory allocation fails for large matrices
#[cfg(feature = "faer")]
pub fn polars_to_faer(df: &DataFrame) -> Result<faer::Mat<f64>, Error> {
    use faer::Mat;

    let n = df.height();
    let p = df.width();
    let mut mat = Mat::<f64>::zeros(n, p);

    for (j, col_name) in df.get_column_names().iter().enumerate() {
        let s = df.column(col_name).unwrap();
        let series = s.as_series().unwrap();
        let ca = cast_to_f64(series)?;
        let v = ca.rechunk();
        let slice = v
            .cont_slice()
            .map_err(|_| Error::Semantic("column not contiguous; rechunk the input".into()))?;
        for i in 0..n {
            // faer is column-major: (row, col)
            mat[(i, j)] = slice[i];
        }
    }
    Ok(mat)
}

/// Convert a Polars Series to a faer column vector for linear algebra operations.
///
/// This function converts a numeric Series (typically a response variable) into a dense
/// faer column vector suitable for high-performance linear algebra computations.
///
/// # Arguments
///
/// * `y` - A reference to a Series containing numeric data
///
/// # Returns
///
/// Returns a `Result<faer::Col<f64>, Error>` containing a column vector with `n` elements
/// where `n` is the length of the input Series.
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::series_to_faer_col;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let series = Series::new("values".into(), &[1.0, 2.0, 3.0, 4.0]);
/// let vector = series_to_faer_col(&series)?;
///
/// // Vector length
/// assert_eq!(vector.nrows(), 4);
/// assert_eq!(vector.ncols(), 1);
///
/// // Access elements: vector[row]
/// assert_eq!(vector[0], 1.0);
/// assert_eq!(vector[3], 4.0);
/// # Ok(())
/// # }
/// ```
///
/// ## With Formula Response Variable
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, series_to_faer_col};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "temperature" => [20.0, 25.0, 30.0, 35.0],
///     "pressure" => [1.0, 1.1, 1.2, 1.3]
/// )?;
///
/// let formula = Formula::parse("temperature ~ pressure")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // Convert response variable to column vector
/// let y_vector = series_to_faer_col(&y)?;
///
/// println!("Response vector length: {}", y_vector.nrows());
/// # Ok(())
/// # }
/// ```
///
/// # Performance Notes
///
/// - Creates a dense column vector suitable for BLAS operations
/// - Data is copied from Polars to faer representation
/// - Memory layout is optimized for mathematical operations
/// - Integrates seamlessly with faer's linear algebra ecosystem
///
/// # Errors
///
/// Returns an error if:
/// - The Series cannot be converted to `f64`
/// - The Series contains non-contiguous data (call `.rechunk()` first)
/// - Memory allocation fails for large vectors
#[cfg(feature = "faer")]
pub fn series_to_faer_col(y: &Series) -> Result<faer::Col<f64>, Error> {
    use faer::Col;

    let ca = cast_to_f64(y)?;
    let v = ca.rechunk();
    let slice = v
        .cont_slice()
        .map_err(|_| Error::Semantic("series not contiguous; rechunk the input".into()))?;
    let mut col = Col::<f64>::zeros(slice.len());
    for (i, &val) in slice.iter().enumerate() {
        col[i] = val;
    }
    Ok(col)
}

/// Clean column names to be more user-friendly and consistent.
///
/// This function transforms column names to follow consistent naming conventions,
/// making them easier to work with in code. It handles special characters,
/// spaces, and creates predictable names for complex expressions.
///
/// # Transformations Applied
///
/// - **Special characters**: `^`, `(`, `)`, `,`, `:`, `+`, `-`, `*`, `/` → `_`
/// - **Spaces and tabs**: → `_`
/// - **Multiple underscores**: Collapsed to single `_`
/// - **Leading/trailing underscores**: Removed
/// - **Case**: Converted to lowercase
/// - **Polynomial terms**: `poly(x,2)^1` → `poly_x_2_1`
/// - **Interaction terms**: `x1:x2` → `x1_x2`
///
/// # Arguments
///
/// * `name` - The original column name to clean
///
/// # Returns
///
/// Returns a cleaned version of the column name that is safe for use in code.
///
/// # Examples
///
/// ## Basic Cleaning
/// ```rust
/// use polars_formula::make_clean_names;
///
/// assert_eq!(make_clean_names("My Column"), "my_column");
/// assert_eq!(make_clean_names("x1:x2"), "x1_x2");
/// assert_eq!(make_clean_names("poly(x,2)^1"), "poly_x_2_1");
/// ```
///
/// ## Complex Expressions
/// ```rust
/// use polars_formula::make_clean_names;
///
/// // Polynomial terms
/// assert_eq!(make_clean_names("poly(income,3)^2"), "poly_income_3_2");
/// assert_eq!(make_clean_names("poly(age,2)^1"), "poly_age_2_1");
///
/// // Interaction terms
/// assert_eq!(make_clean_names("treatment:dose"), "treatment_dose");
/// assert_eq!(make_clean_names("(x1+x2):z"), "x1_x2_z");
///
/// // Special characters
/// assert_eq!(make_clean_names("Column Name!"), "column_name");
/// assert_eq!(make_clean_names("x@#$%"), "x");
/// ```
///
/// ## Formula Materialization Integration
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, make_clean_names};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let formula = Formula::parse("y ~ poly(x,2)")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // Clean the column names
/// let cleaned_names: Vec<String> = X.get_column_names()
///     .iter()
///     .map(|name| make_clean_names(name.as_str()))
///     .collect();
///
/// println!("Original: {:?}", X.get_column_names());
/// println!("Cleaned: {:?}", cleaned_names);
/// // Original: ["Intercept", "poly(x,2)^1", "poly(x,2)^2"]
/// // Cleaned: ["intercept", "poly_x_2_1", "poly_x_2_2"]
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// - This function is inspired by the R janitor package's `make_clean_names`
/// - Cleaned names are safe for use in most programming contexts
/// - The function preserves the semantic meaning while improving usability
/// - For DataFrames with many columns, consider applying this to all column names
pub fn make_clean_names(name: &str) -> String {
    let mut result = String::new();
    let mut prev_was_underscore = false;

    for ch in name.chars() {
        let is_special = matches!(
            ch,
            '^' | '('
                | ')'
                | ','
                | ':'
                | '+'
                | '-'
                | '*'
                | '/'
                | ' '
                | '\t'
                | '\n'
                | '\r'
                | '!'
                | '@'
                | '#'
                | '$'
                | '%'
                | '&'
                | '='
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | '\\'
                | '`'
                | '~'
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

// --- Internal helper functions --------------------------------------------

fn cast_to_f64(s: &Series) -> Result<Float64Chunked, Error> {
    let s = s
        .cast(&DataType::Float64)
        .map_err(|e| Error::Semantic(format!("cast to f64 failed: {e}")))?;
    Ok(s.f64().unwrap().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_parsing() {
        let formula_str = "y ~ x1 + poly(x2, 2) + x1:x2";
        let f = Formula::parse(formula_str).expect("Failed to parse formula");

        // Test that parsing succeeds
        assert!(matches!(f.spec.formula.lhs, dsl::Response::Var(_)));
    }

    #[test]
    fn test_formula_materialization() {
        // Create test data
        let df = df!(
            "y" => [1.0, 2.0, 3.0],
            "x1" => [1.0, 2.0, 3.0],
            "x2" => [1.0, 2.0, 3.0]
        )
        .expect("Failed to create test DataFrame");

        let formula_str = "y ~ x1 + x2";
        let f = Formula::parse(formula_str).expect("Failed to parse formula");

        // Materialize the formula
        let (_y, x) = f
            .materialize(&df, MaterializeOptions::default())
            .expect("Failed to materialize formula");

        // Test LHS (y)
        assert_eq!(_y.width(), 1);
        assert_eq!(_y.height(), 3);

        // Test RHS (X matrix)
        assert_eq!(x.width(), 3); // Intercept + x1 + x2
        assert_eq!(x.height(), 3);
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
