//! # polars-formula
//!
//! A high-performance formula parsing and materialization library for Rust that brings
//! R-style and Python Patsy/Formulaic formula syntax to the Polars DataFrame ecosystem.
//!
//! ## Overview
//!
//! This library provides a comprehensive solution for parsing statistical model formulas
//! and materializing them into design matrices. It seamlessly integrates with Polars
//! DataFrames and provides efficient conversion to faer matrices for linear algebra operations.
//!
//! ## Features
//!
//! - **Formula Parsing**: Parse R-style formulas like `y ~ x1 + x2 + x1:x2`
//! - **Polynomial Terms**: Support for polynomial expansions with `poly(x, degree)`
//! - **Interactions**: Automatic handling of interaction terms using `:`
//! - **Intercept Control**: Flexible intercept inclusion/exclusion
//! - **High Performance**: Built on Polars
//! - **Linear Algebra Ready**: Direct conversion to faer matrices
//!
//! ## Quick Start
//!
//! ```rust
//! use polars::prelude::*;
//! use polars_formula::{Formula, MaterializeOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create sample data
//! let df = df!(
//!     "y" => [1.0, 2.0, 3.0, 4.0, 5.0],
//!     "x1" => [1.0, 2.0, 3.0, 4.0, 5.0],
//!     "x2" => [2.0, 3.0, 4.0, 5.0, 6.0]
//! )?;
//!
//! // Parse a formula
//! let formula = Formula::parse("y ~ x1 + x2")?;
//!
//! // Materialize the formula into design matrices
//! let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
//!
//! println!("Response variable: {:?}", y);
//! println!("Design matrix: {:?}", X);
//! # Ok(())
//! # }
//! ```
//!
//! ## Supported Formula Syntax
//!
//! ### Basic Operations
//!
//! - **Variables**: `x`, `income`, `age`
//! - **Addition**: `x1 + x2` (include both terms)
//! - **Interactions**: `x1:x2` (product of x1 and x2)
//! - **Intercept**: Automatically included (use options to control)
//!
//! ### Functions
//!
//! - **Polynomials**: `poly(x, 3)` expands to x, x², x³
//! - **Constants**: Numeric literals like `1`, `0` for intercept control
//!
//! ### Grouping
//!
//! - **Parentheses**: `(x1 + x2):z` for grouped interactions
//!
//! ## Examples
//!
//! ### Linear Regression
//!
//! ```rust
//! use polars::prelude::*;
//! use polars_formula::{Formula, MaterializeOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let df = df!(
//!     "price" => [100.0, 150.0, 200.0, 250.0],
//!     "size" => [1000.0, 1500.0, 2000.0, 2500.0],
//!     "age" => [5.0, 10.0, 15.0, 20.0]
//! )?;
//!
//! let formula = Formula::parse("price ~ size + age")?;
//! let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
//!
//! // X now contains: [Intercept, size, age]
//! println!("Design matrix shape: {}x{}", X.height(), X.width());
//! # Ok(())
//! # }
//! ```
//!
//! ### Polynomial Regression
//!
//! ```rust
//! use polars::prelude::*;
//! use polars_formula::{Formula, MaterializeOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let df = df!(
//!     "y" => [1.0, 4.0, 9.0, 16.0, 25.0],
//!     "x" => [1.0, 2.0, 3.0, 4.0, 5.0]
//! )?;
//!
//! // Fit a cubic polynomial
//! let formula = Formula::parse("y ~ poly(x, 3)")?;
//! let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
//!
//! // X contains: [Intercept, x, x², x³]
//! println!("Polynomial features: {:?}", X.get_column_names());
//! # Ok(())
//! # }
//! ```
//!
//! ### Interaction Terms
//!
//! ```rust
//! use polars::prelude::*;
//! use polars_formula::{Formula, MaterializeOptions};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let df = df!(
//!     "outcome" => [10.0, 20.0, 30.0, 40.0],
//!     "treatment" => [1.0, 0.0, 1.0, 0.0],
//!     "dose" => [5.0, 5.0, 10.0, 10.0]
//! )?;
//!
//! // Model with interaction
//! let formula = Formula::parse("outcome ~ treatment + dose + treatment:dose")?;
//! let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
//!
//! // X contains: [Intercept, treatment, dose, treatment:dose]
//! # Ok(())
//! # }
//! ```
//!
//! ### Converting to Linear Algebra Matrices
//!
//! Note: the following example requires the `faer` feature.
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # #[cfg(feature = "faer")]
//! # {
//! # use polars::prelude::*;
//! # use polars_formula::{Formula, MaterializeOptions, polars_to_faer, series_to_faer_col};
//!
//! let df = df!(
//!     "y" => [1.0, 2.0, 3.0, 4.0],
//!     "x1" => [1.0, 2.0, 3.0, 4.0],
//!     "x2" => [2.0, 4.0, 6.0, 8.0]
//! )?;
//!
//! let formula = Formula::parse("y ~ x1 + x2")?;
//! let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
//!
//! // Convert to faer matrices for linear algebra
//! let X_matrix = polars_to_faer(&X)?;
//! let y_vector = series_to_faer_col(&y)?;
//!
//! println!("Matrix dimensions: {}x{}", X_matrix.nrows(), X_matrix.ncols());
//! println!("Vector length: {}", y_vector.nrows());
//!
//! // Now you can perform linear algebra operations
//! // let coefficients = (X_matrix.transpose() * &X_matrix).inverse()? * X_matrix.transpose() * &y_vector;
//! # }
//! # Ok(())
//! # }
//! ```
//!
//! ## Planned Features
//!
//! Future versions will include:
//! - **Categorical Variables**: `C(category)` for factor encoding
//! - **Term Removal**: `-x` to remove terms from expansions
//! - **Nested Effects**: `x/y` for nested structures
//! - **Dot Expansion**: `.` to include all available variables
//! - **Spline Functions**: Smooth function approximations
//! - **Lag Operations**: Time series support
//! - **Custom Functions**: User-defined transformations

#![forbid(unsafe_code)]

use std::fmt::{self, Display};
use std::str::FromStr;

use polars::prelude::*;
use thiserror::Error;

// --- Public surface -------------------------------------------------------

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
    /// The parsed abstract syntax tree of the formula.
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
    /// println!("LHS terms: {}", formula.ast.lhs.len());
    /// println!("RHS terms: {}", formula.ast.rhs.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub ast: Ast,
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
        let tokens = Lexer::new(src).lex_all()?;
        let mut p = Parser::new(tokens);
        let ast = p.parse_formula()?;
        Ok(Self { ast })
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
    ) -> Result<(Series, DataFrame), Error> {
        materialize_dataframe(df, &self.ast, opts)
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
/// ## Linear Regression Example
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, polars_to_faer, series_to_faer_col};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "price" => [100.0, 150.0, 200.0, 250.0],
///     "size" => [1000.0, 1500.0, 2000.0, 2500.0]
/// )?;
///
/// let formula = Formula::parse("price ~ size")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // Convert to matrices
/// let X_mat = polars_to_faer(&X)?;
/// let y_vec = series_to_faer_col(&y)?;
///
/// // Ready for linear algebra: β = (X^T X)^{-1} X^T y
/// println!("Ready for regression with X: {}×{}, y: {}",
///          X_mat.nrows(), X_mat.ncols(), y_vec.nrows());
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
/// ## Complete Linear Regression Setup
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, polars_to_faer, series_to_faer_col};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "sales" => [100.0, 200.0, 300.0, 400.0, 500.0],
///     "advertising" => [10.0, 20.0, 30.0, 40.0, 50.0],
///     "price" => [5.0, 4.5, 4.0, 3.5, 3.0]
/// )?;
///
/// let formula = Formula::parse("sales ~ advertising + price")?;
/// let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
///
/// // Convert to faer matrices
/// let X_matrix = polars_to_faer(&X)?;
/// let y_vector = series_to_faer_col(&y)?;
///
/// // Now ready for linear algebra operations
/// assert_eq!(X_matrix.nrows(), y_vector.nrows()); // Same number of observations
///
/// // Example: compute X^T * y
/// let xty = X_matrix.transpose() * &y_vector;
/// println!("X^T * y computed, shape: {}×{}", xty.nrows(), xty.ncols());
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

/// Low-level function to materialize a parsed AST against a DataFrame.
///
/// This function is used internally by `Formula::materialize()` but is exposed
/// for advanced use cases where you need direct control over the AST.
///
/// # Arguments
///
/// * `df` - The DataFrame containing the data
/// * `ast` - The parsed abstract syntax tree of the formula
/// * `opts` - Materialization options
///
/// # Returns
///
/// Returns `(Series, DataFrame)` representing the response variable and design matrix.
///
/// # Examples
///
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::{Formula, MaterializeOptions, materialize_dataframe};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0],
///     "x" => [1.0, 2.0, 3.0]
/// )?;
///
/// let formula = Formula::parse("y ~ x")?;
/// // Access the internal AST for advanced processing
/// let (y, X) = materialize_dataframe(&df, &formula.ast, MaterializeOptions::default())?;
///
/// assert_eq!(y.len(), 3);
/// assert_eq!(X.width(), 2); // Intercept + x
/// # Ok(())
/// # }
/// ```
///
/// # Note
///
/// Most users should use `Formula::materialize()` instead of this function.
/// This is primarily for library authors and advanced use cases.
pub fn materialize_dataframe(
    df: &DataFrame,
    ast: &Ast,
    opts: MaterializeOptions,
) -> Result<(Series, DataFrame), Error> {
    let (lhs_terms, rhs_terms) = (&ast.lhs, &ast.rhs);

    // --- LHS: expect exactly one term that yields exactly one column
    let y_cols = eval_terms(df, lhs_terms)?;
    let y = match y_cols.as_slice() {
        [(name, s)] => {
            // cast to f64 for downstream LA
            let s = cast_to_f64_named(name, s)?;
            s
        }
        [] => return Err(Error::Semantic("lhs is empty; expected a target".into())),
        _ => return Err(Error::Semantic("lhs must materialize to one column".into())),
    };

    // --- RHS
    let mut cols = Vec::<(String, Series)>::new();
    if opts.rhs_intercept {
        let n = df.height();
        let ones =
            Float64Chunked::from_slice(opts.intercept_name.into(), &vec![1.0; n]).into_series();
        cols.push((opts.intercept_name.to_string(), ones));
    }

    cols.extend(eval_terms(df, rhs_terms)?);

    // Build X DataFrame preserving column order
    let (names, series): (Vec<_>, Vec<_>) = cols.into_iter().unzip();
    // Ensure unique column names by adding suffixes if needed
    let mut unique_series = Vec::new();
    let mut name_counts = std::collections::HashMap::new();

    for (name, s) in names.into_iter().zip(series.into_iter()) {
        let count = name_counts.entry(name.clone()).or_insert(0);
        *count += 1;
        let unique_name = if *count > 1 {
            format!("{}_{}", name, *count - 1)
        } else {
            name
        };

        // Apply name cleaning if requested
        let final_name = if opts.clean_names {
            make_clean_names(&unique_name)
        } else {
            unique_name
        };

        let mut new_series = s.clone();
        new_series.rename(final_name.into());
        unique_series.push(new_series.into());
    }

    let x = DataFrame::new(unique_series).map_err(|e| Error::Semantic(e.to_string()))?;

    Ok((y, x))
}

// --- AST ------------------------------------------------------------------

/// Abstract Syntax Tree representation of a parsed formula.
///
/// An `Ast` represents the parsed structure of a statistical formula, containing
/// the left-hand side (response variables) and right-hand side (predictor terms).
///
/// # Structure
///
/// - `lhs`: Terms on the left side of `~` (typically the response variable)
/// - `rhs`: Terms on the right side of `~` (predictor variables and transformations)
///
/// # Examples
///
/// ```rust
/// use polars_formula::Formula;
///
/// // Parse a formula to get its AST
/// let formula = Formula::parse("sales ~ advertising + price + advertising:price")?;
/// let ast = &formula.ast;
///
/// // Inspect the structure
/// println!("Response terms: {}", ast.lhs.len());  // 1 (sales)
/// println!("Predictor terms: {}", ast.rhs.len()); // 3 (advertising, price, interaction)
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// For formulas without `~`, the entire expression is treated as RHS:
/// ```rust
/// use polars_formula::Formula;
///
/// let formula = Formula::parse("x1 + x2 + x1:x2")?;
/// let ast = &formula.ast;
///
/// assert_eq!(ast.lhs.len(), 0); // No response variable specified
/// assert_eq!(ast.rhs.len(), 3); // Three predictor terms
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct Ast {
    /// Terms representing the response variable(s) (left side of `~`)
    pub lhs: Vec<Term>,
    /// Terms representing the predictor variables and transformations (right side of `~`)
    pub rhs: Vec<Term>,
}

/// A term in a formula expression.
///
/// Terms represent the building blocks of formulas, including variables,
/// function calls, interactions, and grouped expressions.
#[derive(Debug, Clone)]
pub enum Term {
    Var(String),
    Func(FuncCall),
    Interaction(Vec<Term>),
    Group(Box<Vec<Term>>),
}

/// A function call in a formula expression.
///
/// Represents functions like `poly(x, 3)` or `log(income)`.
#[derive(Debug, Clone)]
pub struct FuncCall {
    /// The name of the function
    pub name: String,
    /// Arguments passed to the function
    pub args: Vec<Arg>,
}

/// An argument to a function call.
///
/// Arguments can be either terms (expressions) or numeric literals.
#[derive(Debug, Clone)]
pub enum Arg {
    /// A term argument (like a variable or expression)
    Term(Term),
    /// A numeric literal argument
    Number(f64),
}

// --- Lexer ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum TokKind {
    Ident(String),
    Number(f64),
    Tilde,
    Plus,
    Colon,
    Comma,
    LParen,
    RParen,
    Dot,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokKind,
    pos: usize,
}

#[derive(Debug)]
struct Lexer<'a> {
    s: &'a str,
    it: std::str::CharIndices<'a>,
    peek: Option<(usize, char)>,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Self {
        let mut it = s.char_indices();
        let peek = it.next();
        Self { s, it, peek }
    }

    fn bump(&mut self) -> Option<(usize, char)> {
        let cur = self.peek;
        self.peek = self.it.next();
        cur
    }

    fn lex_all(mut self) -> Result<Vec<Token>, Error> {
        let mut out = Vec::new();
        while let Some((i, ch)) = self.peek {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.bump();
                }
                '~' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::Tilde,
                        pos: i,
                    });
                }
                '+' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::Plus,
                        pos: i,
                    });
                }
                ':' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::Colon,
                        pos: i,
                    });
                }
                ',' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::Comma,
                        pos: i,
                    });
                }
                '(' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::LParen,
                        pos: i,
                    });
                }
                ')' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::RParen,
                        pos: i,
                    });
                }
                '.' => {
                    self.bump();
                    out.push(Token {
                        kind: TokKind::Dot,
                        pos: i,
                    });
                }
                '0'..='9' => out.push(self.lex_number()?),
                'a'..='z' | 'A'..='Z' | '_' => out.push(self.lex_ident()?),
                _ => {
                    return Err(Error::Lex {
                        pos: i,
                        msg: format!("unexpected char '{ch}'"),
                    });
                }
            }
        }
        Ok(out)
    }

    fn lex_ident(&mut self) -> Result<Token, Error> {
        let (start, _) = self.bump().unwrap();
        let mut end = start + 1;
        while let Some((i, ch)) = self.peek {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.bump();
                end = i + ch.len_utf8();
            } else {
                break;
            }
        }
        let text = &self.s[start..end];
        Ok(Token {
            kind: TokKind::Ident(text.to_string()),
            pos: start,
        })
    }

    fn lex_number(&mut self) -> Result<Token, Error> {
        let (start, c0) = self.bump().unwrap();
        let mut end = start + c0.len_utf8();
        let mut seen_dot = false;
        while let Some((i, ch)) = self.peek {
            match ch {
                '0'..='9' => {
                    self.bump();
                    end = i + 1;
                }
                '.' if !seen_dot => {
                    self.bump();
                    end = i + 1;
                    seen_dot = true;
                }
                _ => break,
            }
        }
        let text = &self.s[start..end];
        let val = f64::from_str(text).map_err(|_| Error::Lex {
            pos: start,
            msg: format!("bad number '{text}'"),
        })?;
        Ok(Token {
            kind: TokKind::Number(val),
            pos: start,
        })
    }
}

// --- Parser (Pratt-ish) ---------------------------------------------------

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, idx: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.idx)
    }
    fn bump(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.idx);
        if t.is_some() {
            self.idx += 1;
        }
        t
    }

    fn expect(&mut self, expect: &TokKind) -> Result<(), Error> {
        match self.bump() {
            Some(t) if &t.kind == expect => Ok(()),
            Some(t) => Err(Error::Parse {
                pos: Some(t.pos),
                msg: format!("expected {:?}, got {:?}", expect, t.kind),
            }),
            None => Err(Error::Parse {
                pos: None,
                msg: "unexpected end".into(),
            }),
        }
    }

    fn parse_formula(&mut self) -> Result<Ast, Error> {
        let lhs_terms = self.parse_sum_terms()?;
        if let Some(t) = self.peek() {
            if matches!(t.kind, TokKind::Tilde) {
                self.bump();
                // Handle empty RHS after tilde
                let rhs_terms = if self.peek().is_some() {
                    self.parse_sum_terms()?
                } else {
                    Vec::new()
                };
                return Ok(Ast {
                    lhs: lhs_terms,
                    rhs: rhs_terms,
                });
            }
        }
        // No '~' -> rhs-only formula; lhs empty
        Ok(Ast {
            lhs: Vec::new(),
            rhs: lhs_terms,
        })
    }

    // sum := prod ( "+" prod )*
    fn parse_sum_terms(&mut self) -> Result<Vec<Term>, Error> {
        let mut terms = Vec::new();
        let mut first = true;
        loop {
            if !first {
                match self.peek() {
                    Some(t) if matches!(t.kind, TokKind::Plus) => {
                        self.bump();
                    }
                    _ => break,
                }
            }
            first = false;
            let term = self.parse_prod()?;
            terms.push(term);
        }
        Ok(terms)
    }

    // prod := atom ( ":" atom )*
    fn parse_prod(&mut self) -> Result<Term, Error> {
        let mut atoms = vec![self.parse_atom()?];
        while let Some(t) = self.peek() {
            if matches!(t.kind, TokKind::Colon) {
                self.bump();
                atoms.push(self.parse_atom()?);
            } else {
                break;
            }
        }
        if atoms.len() == 1 {
            Ok(atoms.pop().unwrap())
        } else {
            Ok(Term::Interaction(atoms))
        }
    }

    // atom := IDENT | IDENT '(' args ')' | '(' sum ')' | '.' | NUMBER
    fn parse_atom(&mut self) -> Result<Term, Error> {
        let t = self
            .bump()
            .ok_or_else(|| Error::Parse {
                pos: None,
                msg: "unexpected end".into(),
            })?
            .clone();
        match t.kind {
            TokKind::Ident(name) => {
                // function or variable
                if let Some(Token {
                    kind: TokKind::LParen,
                    ..
                }) = self.peek()
                {
                    // lookahead
                    self.bump(); // consume '('
                    let args = self.parse_args()?;
                    self.expect(&TokKind::RParen)?;
                    Ok(Term::Func(FuncCall { name, args }))
                } else {
                    Ok(Term::Var(name))
                }
            }
            TokKind::LParen => {
                let inner = self.parse_sum_terms()?;
                self.expect(&TokKind::RParen)?;
                Ok(Term::Group(Box::new(inner)))
            }
            TokKind::Number(v) => {
                // treat 1/0 as special constants later (e.g., intercept control)
                Ok(Term::Func(FuncCall {
                    name: "const".into(),
                    args: vec![Arg::Number(v)],
                }))
            }
            TokKind::Dot => Ok(Term::Func(FuncCall {
                name: "dot".into(),
                args: vec![],
            })),
            other => Err(Error::Parse {
                pos: Some(t.pos),
                msg: format!("unexpected token in atom: {:?}", other),
            }),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Arg>, Error> {
        let mut args = Vec::new();
        // empty arg list
        if let Some(Token {
            kind: TokKind::RParen,
            ..
        }) = self.peek()
        {
            return Ok(args);
        }
        loop {
            // number or term
            match self.peek().cloned() {
                Some(Token {
                    kind: TokKind::Number(v),
                    ..
                }) => {
                    self.bump();
                    args.push(Arg::Number(v));
                }
                _ => {
                    let term = self.parse_prod()?;
                    args.push(Arg::Term(term));
                }
            }
            match self.peek() {
                Some(Token {
                    kind: TokKind::Comma,
                    ..
                }) => {
                    self.bump();
                }
                _ => break,
            }
        }
        Ok(args)
    }
}

// --- Materialization ------------------------------------------------------

fn eval_terms(df: &DataFrame, terms: &[Term]) -> Result<Vec<(String, Series)>, Error> {
    let mut out = Vec::new();
    for t in terms {
        eval_term(df, t, &mut out, None)?;
    }
    Ok(out)
}

fn eval_term(
    df: &DataFrame,
    term: &Term,
    out: &mut Vec<(String, Series)>,
    prefix: Option<String>,
) -> Result<(), Error> {
    match term {
        Term::Var(name) => {
            let s = df
                .column(name)
                .map_err(|_| Error::Semantic(format!("unknown column '{name}'")))?;
            let series = s.as_series().unwrap();
            let s = cast_to_f64_named(name, series)?;
            let n = prefix_name(prefix.as_deref(), name);
            out.push((n, s));
        }
        Term::Group(inner) => {
            for t in inner.as_ref() {
                eval_term(df, t, out, prefix.clone())?;
            }
        }
        Term::Interaction(parts) => {
            // For each part, expand into its own columns, then take cartesian product and multiply
            let mut groups: Vec<Vec<(String, Series)>> = Vec::new();
            for p in parts {
                let mut g = Vec::new();
                eval_term(df, p, &mut g, None)?;
                groups.push(g);
            }
            let acc: Vec<(String, Series)> = groups.into_iter().fold(Vec::new(), |acc, g| {
                if acc.is_empty() {
                    return g;
                }
                let mut new_acc = Vec::new();
                for (n1, s1) in acc.iter() {
                    let s1 = cast_to_f64_named(n1, s1).expect("cast");
                    let ca1 = s1.f64().unwrap().clone();
                    for (n2, s2) in g.iter() {
                        let s2 = cast_to_f64_named(n2, s2).expect("cast");
                        let ca2 = s2.f64().unwrap();
                        let prod = &ca1 * ca2;
                        let name = format!("{n1}:{n2}");
                        new_acc.push((name, prod.into_series()));
                    }
                }
                new_acc
            });
            if !acc.is_empty() {
                out.extend(acc.into_iter());
            }
        }
        Term::Func(FuncCall { name, args }) => {
            match name.as_str() {
                // poly(x, d)
                "poly" => {
                    if args.len() != 2 {
                        return Err(Error::Semantic(
                            "poly expects 2 args: (column, degree)".into(),
                        ));
                    }
                    let col_name = match &args[0] {
                        Arg::Term(Term::Var(s)) => s.clone(),
                        _ => {
                            return Err(Error::Semantic(
                                "poly: first arg must be a column name".into(),
                            ));
                        }
                    };
                    let degree = match args[1] {
                        Arg::Number(d) => d as usize,
                        _ => return Err(Error::Semantic("poly: degree must be numeric".into())),
                    };

                    let base = df
                        .column(&col_name)
                        .map_err(|_| Error::Semantic(format!("unknown column '{col_name}'")))?;
                    let series = base.as_series().unwrap();
                    let base = cast_to_f64_named(&col_name, series)?;
                    let mut pow = base.f64().unwrap().clone();
                    for k in 1..=degree {
                        if k > 1 {
                            pow = &pow * base.f64().unwrap();
                        }
                        let nm = match &prefix {
                            Some(p) => format!("{p}:poly({col_name},{degree})^{k}"),
                            None => format!("poly({col_name},{degree})^{k}"),
                        };
                        out.push((nm, pow.clone().into_series()));
                    }
                }
                // constants 0/1 etc.
                "const" => {
                    if let [Arg::Number(v)] = args.as_slice() {
                        // Commonly used to toggle intercept; here we just emit a literal column
                        let n = df.height();
                        let s =
                            Float64Chunked::from_slice("const".into(), &vec![*v; n]).into_series();
                        let nm = match &prefix {
                            Some(p) => format!("{p}:{v}"),
                            None => format!("{v}"),
                        };
                        out.push((nm, s));
                    } else {
                        return Err(Error::Semantic("const expects one numeric arg".into()));
                    }
                }
                // dot placeholder: not implemented yet
                "dot" => {
                    return Err(Error::Semantic(
                        "'.' expansion not implemented in this starter".into(),
                    ));
                }
                _ => return Err(Error::Semantic(format!("unknown function '{name}'"))),
            }
        }
    }
    Ok(())
}

fn prefix_name(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(p) => format!("{p}:{name}"),
        None => name.to_string(),
    }
}

fn cast_to_f64_named(name: &str, s: &Series) -> Result<Series, Error> {
    let s = s
        .cast(&DataType::Float64)
        .map_err(|e| Error::Semantic(format!("cast '{name}' to f64 failed: {e}")))?;
    Ok(s)
}

fn cast_to_f64(s: &Series) -> Result<Float64Chunked, Error> {
    let s = s
        .cast(&DataType::Float64)
        .map_err(|e| Error::Semantic(format!("cast to f64 failed: {e}")))?;
    Ok(s.f64().unwrap().clone())
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
/// // This will produce a Lex error
/// match Formula::parse("y ~~ x") {
///     Err(Error::Lex { pos, msg }) => {
///         println!("Lexical error at position {}: {}", pos, msg);
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
    /// Lexical analysis error during tokenization.
    ///
    /// Occurs when the formula string contains invalid characters or
    /// malformed tokens that cannot be recognized by the lexer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use polars_formula::{Formula, Error};
    ///
    /// // Invalid character in formula
    /// match Formula::parse("y ~ x$invalid") {
    ///     Err(Error::Lex { pos, msg }) => {
    ///         assert_eq!(pos, 5); // Position of '$'
    ///         assert!(msg.contains("unexpected char"));
    ///     }
    ///     _ => panic!("Expected lex error"),
    /// }
    /// ```
    #[error("lex error at {pos}: {msg}")]
    Lex {
        /// Character position where the error occurred
        pos: usize,
        /// Description of the lexical error
        msg: String,
    },

    /// Parse error during syntax analysis.
    ///
    /// Occurs when the token sequence doesn't conform to the expected
    /// formula grammar, even if individual tokens are valid.
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

// --- Display helpers (debugging) -----------------------------------------

impl Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Var(s) => write!(f, "{s}"),
            Term::Func(FuncCall { name, args }) => {
                let as_ = args
                    .iter()
                    .map(|a| match a {
                        Arg::Term(t) => format!("{t}"),
                        Arg::Number(v) => v.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{name}({as_})")
            }
            Term::Interaction(ts) => {
                let s = ts
                    .iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<_>>()
                    .join(":");
                write!(f, "{s}")
            }
            Term::Group(inner) => {
                let s = inner
                    .iter()
                    .map(|t| format!("{t}"))
                    .collect::<Vec<_>>()
                    .join(" + ");
                write!(f, "({s})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_parsing() {
        let formula_str = "y ~ x1 + poly(x2, 2) + x1:x2";
        let f = Formula::parse(formula_str).expect("Failed to parse formula");

        // Test that parsing succeeds
        assert!(matches!(f.ast.lhs.len(), 1)); // Should have one LHS term (y)
        assert!(matches!(f.ast.rhs.len(), 3)); // Should have three RHS terms
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

        let formula_str = "y ~ x1 + poly(x2, 2) + x1:x2";
        let f = Formula::parse(formula_str).expect("Failed to parse formula");

        // Materialize the formula
        let (y, x) = f
            .materialize(&df, MaterializeOptions::default())
            .expect("Failed to materialize formula");

        // Test LHS (y)
        assert_eq!(y.name(), "y");
        assert_eq!(y.len(), 3);

        // Test RHS (X matrix)
        // Should have: Intercept, x1, poly(x2,2)^1, poly(x2,2)^2, x1:x2
        assert_eq!(x.width(), 5);
        assert_eq!(x.height(), 3);

        // Check column names (cleaned by default)
        let expected_columns = vec!["intercept", "x1", "poly_x2_2_1", "poly_x2_2_2", "x1_x2"];

        for (i, expected_name) in expected_columns.iter().enumerate() {
            assert_eq!(x.get_columns()[i].name(), *expected_name);
        }

        // Test specific values
        let intercept_col = x.column("intercept").expect("intercept column not found");
        assert_eq!(intercept_col.f64().unwrap().get(0), Some(1.0));
        assert_eq!(intercept_col.f64().unwrap().get(2), Some(1.0));

        let x1_col = x.column("x1").expect("x1 column not found");
        assert_eq!(x1_col.f64().unwrap().get(0), Some(1.0));
        assert_eq!(x1_col.f64().unwrap().get(2), Some(3.0));

        let poly1_col = x
            .column("poly_x2_2_1")
            .expect("poly_x2_2_1 column not found");
        assert_eq!(poly1_col.f64().unwrap().get(0), Some(1.0));
        assert_eq!(poly1_col.f64().unwrap().get(2), Some(3.0));

        let poly2_col = x
            .column("poly_x2_2_2")
            .expect("poly_x2_2_2 column not found");
        assert_eq!(poly2_col.f64().unwrap().get(0), Some(1.0)); // 1.0^2 = 1.0
        assert_eq!(poly2_col.f64().unwrap().get(2), Some(9.0)); // 3.0^2 = 9.0

        let interaction_col = x.column("x1_x2").expect("x1_x2 column not found");
        assert_eq!(interaction_col.f64().unwrap().get(0), Some(1.0)); // 1.0 * 1.0 = 1.0
        assert_eq!(interaction_col.f64().unwrap().get(2), Some(9.0)); // 3.0 * 3.0 = 9.0
    }

    #[test]
    fn test_formula_without_intercept() {
        let mut df: DataFrame = df!(
            "y" => [1.0, 2.0, 3.0],
            "x1" => [1.0, 2.0, 3.0],
            "x2" => [1.0, 2.0, 3.0]
        )
        .expect("Failed to create test DataFrame");

        let formula_str = "y ~ x1 + poly(x2, 2) + x1:x2";
        let f = Formula::parse(formula_str).expect("Failed to parse formula");

        let opts = MaterializeOptions {
            rhs_intercept: false,
            intercept_name: "Intercept",
            clean_names: false,
        };

        let (y, x) = f
            .materialize(&df, opts)
            .expect("Failed to materialize formula");

        // Should have 4 columns: x1, poly(x2,2)^1, poly(x2,2)^2, x1:x2 (no intercept)
        assert_eq!(x.width(), 4);

        // Check that Intercept column is not present
        assert!(x.column("Intercept").is_err());
    }

    #[test]
    #[cfg(feature = "faer")]
    fn test_faer_conversion() {
        let df = df!(
            "y" => [1.0, 2.0, 3.0],
            "x1" => [1.0, 2.0, 3.0],
            "x2" => [1.0, 2.0, 3.0]
        )
        .expect("Failed to create test DataFrame");

        let formula_str = "y ~ x1 + poly(x2, 2) + x1:x2";
        let f = Formula::parse(formula_str).expect("Failed to parse formula");

        let (y, x) = f
            .materialize(&df, MaterializeOptions::default())
            .expect("Failed to materialize formula");

        // Convert to faer matrices
        let x_matrix = polars_to_faer(&x).expect("Failed to convert X to faer matrix");
        let y_vector = series_to_faer_col(&y).expect("Failed to convert y to faer column");

        // Test matrix dimensions
        assert_eq!(x_matrix.nrows(), 3); // 3 rows
        assert_eq!(x_matrix.ncols(), 5); // 5 columns (Intercept, x1, poly^1, poly^2, x1:x2)
        assert_eq!(y_vector.nrows(), 3); // 3 rows
        assert_eq!(y_vector.ncols(), 1); // 1 column

        // Test some specific values
        assert_eq!(x_matrix[(0, 0)], 1.0); // Intercept
        assert_eq!(x_matrix[(0, 1)], 1.0); // x1
        assert_eq!(x_matrix[(0, 2)], 1.0); // poly(x2,2)^1
        assert_eq!(x_matrix[(0, 3)], 1.0); // poly(x2,2)^2
        assert_eq!(x_matrix[(0, 4)], 1.0); // x1:x2

        assert_eq!(y_vector[0], 1.0);
        assert_eq!(y_vector[1], 2.0);
        assert_eq!(y_vector[2], 3.0);
    }

    #[test]
    fn test_formula_parsing_edge_cases() {
        // Test valid formula without RHS
        let f = Formula::parse("y ~").expect("Failed to parse formula with empty RHS");
        assert_eq!(f.ast.lhs.len(), 1);
        assert_eq!(f.ast.rhs.len(), 0);

        // Test formula without tilde (RHS only)
        let f = Formula::parse("x1 + x2").expect("Failed to parse RHS-only formula");
        assert_eq!(f.ast.lhs.len(), 0);
        assert_eq!(f.ast.rhs.len(), 2);

        // Test invalid formula
        assert!(Formula::parse("y ~ x1 +").is_err()); // Incomplete expression
        assert!(Formula::parse("y ~ poly(x1,)").is_err()); // Incomplete function call
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

    #[test]
    fn test_formula_with_clean_names() -> Result<(), Box<dyn std::error::Error>> {
        let df = df!(
            "y" => [1.0, 2.0, 3.0],
            "x" => [1.0, 2.0, 3.0]
        )
        .expect("Failed to create test DataFrame");

        let formula = Formula::parse("y ~ poly(x,2)")?;
        let opts = MaterializeOptions {
            clean_names: true,
            ..Default::default()
        };
        let (y, X) = formula.materialize(&df, opts)?;

        // Check that names are cleaned
        let column_names: Vec<&str> = X.get_column_names().iter().map(|s| s.as_str()).collect();
        assert!(column_names.contains(&"intercept"));
        assert!(column_names.contains(&"poly_x_2_1"));
        assert!(column_names.contains(&"poly_x_2_2"));

        // Original names should not be present
        assert!(!column_names.contains(&"Intercept"));
        assert!(!column_names.contains(&"poly(x,2)^1"));
        assert!(!column_names.contains(&"poly(x,2)^2"));

        Ok(())
    }
}
