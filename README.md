# polars-formula

[![Crates.io](https://img.shields.io/crates/v/polars-formula.svg)](https://crates.io/crates/polars-formula)
[![Documentation](https://docs.rs/polars-formula/badge.svg)](https://docs.rs/polars-formula)
[![License](https://img.shields.io/crates/l/polars-formula.svg)](LICENSE)

A high-performance formula parsing and materialization library for Rust that brings R-style/Patsy/Formulaic formula syntax to the Polars DataFrame ecosystem.

## 🚀 Features

- **🔍 Formula Parsing**: Parse R-style formulas like `y ~ x1 + x2 + x1:x2`
- **🔢 Polynomial Terms**: Support for polynomial expansions with `poly(x, degree)`
- **🔗 Interactions**: Automatic handling of interaction terms using `:`
- **🎯 Intercept Control**: Flexible intercept inclusion/exclusion
- **🧹 Clean Column Names**: Automatic cleaning of complex column names for better usability
- **🧮 Linear Algebra Ready**: Direct conversion to [faer](https://github.com/sarah-quinones/faer-rs) matrices (optional feature)
- **📚 Rich Documentation**: Comprehensive examples and API documentation

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
polars-formula = "0.1"
polars = { version = "0.50", features = ["lazy"] }
```

To enable linear algebra conversions to `faer`, turn on the optional feature:

```toml
[dependencies]
polars-formula = { version = "0.1", features = ["faer"] }
```

## 🏃‍♂️ Quick Start

### Run Exmple with Cargo

```bash
git clone https://github.com/alexh/polars-formula.git
cd polars-formula
cargo run --example clean_names_demo
```


## 🎯 Supported Formula Syntax

### Basic Operations

- **Variables**: `x`, `income`, `age`
- **Addition**: `x1 + x2` (include both terms)
- **Interactions**: `x1:x2` (product of x1 and x2)
- **Intercept**: Automatically included (use options to control)

### Functions

- **Polynomials**: `poly(x, 3)` expands to x, x², x³
- **Constants**: Numeric literals like `1`, `0` for intercept control

### Grouping

- **Parentheses**: `(x1 + x2):z` for grouped interactions

## 📖 Examples

### Linear Regression

```rust
use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions};

let df = df!(
    "price" => [100.0, 150.0, 200.0, 250.0],
    "size" => [1000.0, 1500.0, 2000.0, 2500.0],
    "age" => [5.0, 10.0, 15.0, 20.0]
)?;

let formula = Formula::parse("price ~ size + age")?;
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;

// X now contains: [intercept, size, age] (names are cleaned by default)
println!("Design matrix shape: {}x{}", X.height(), X.width());
```

### Polynomial Regression

```rust
use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions};

let df = df!(
    "y" => [1.0, 4.0, 9.0, 16.0, 25.0],
    "x" => [1.0, 2.0, 3.0, 4.0, 5.0]
)?;

// Fit a cubic polynomial
let formula = Formula::parse("y ~ poly(x, 3)")?;
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;

// X contains: [intercept, x, x², x³] (names are cleaned by default)
println!("Polynomial features: {:?}", X.get_column_names());
```

### Interaction Terms

```rust
use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions};

let df = df!(
    "outcome" => [10.0, 20.0, 30.0, 40.0],
    "treatment" => [1.0, 0.0, 1.0, 0.0],
    "dose" => [5.0, 5.0, 10.0, 10.0]
)?;

// Model with interaction
let formula = Formula::parse("outcome ~ treatment + dose + treatment:dose")?;
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;

// X contains: [intercept, treatment, dose, treatment_dose] (names are cleaned by default)
```

### Integration with Linear Algebra (optional `faer` feature)

```rust
use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions, polars_to_faer, series_to_faer_col};

let df = df!(
    "y" => [1.0, 2.0, 3.0, 4.0],
    "x1" => [1.0, 2.0, 3.0, 4.0],
    "x2" => [2.0, 4.0, 6.0, 8.0]
)?;

let formula = Formula::parse("y ~ x1 + x2")?;
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;

// Convert to faer matrices for linear algebra
let X_matrix = polars_to_faer(&X)?;
let y_vector = series_to_faer_col(&y)?;

println!("Matrix dimensions: {}x{}", X_matrix.nrows(), X_matrix.ncols());
println!("Vector length: {}", y_vector.nrows());

// Now you can perform linear algebra operations
// let coefficients = (X_matrix.transpose() * &X_matrix).inverse()? * X_matrix.transpose() * &y_vector;
```

## ⚙️ Configuration

### Controlling Intercepts

```rust
use polars_formula::MaterializeOptions;

// With intercept (default)
let opts_with = MaterializeOptions::default();

// Without intercept
let opts_without = MaterializeOptions {
    rhs_intercept: false,
    ..Default::default()
};

// Custom intercept name
let opts_custom = MaterializeOptions {
    intercept_name: "Constant",
    ..Default::default()
};
```

### Clean Column Names

Column names are automatically cleaned by default for better usability:

```rust
use polars_formula::{Formula, MaterializeOptions, make_clean_names};

let df = df!(
    "y" => [1.0, 2.0, 3.0],
    "x" => [1.0, 2.0, 3.0]
)?;

let formula = Formula::parse("y ~ poly(x,2)")?;

// With clean names (default)
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
println!("Cleaned names: {:?}", X.get_column_names());
// Output: ["intercept", "poly_x_2_1", "poly_x_2_2"]

// Without clean names
let opts = MaterializeOptions {
    clean_names: false,
    ..Default::default()
};
let (y, X) = formula.materialize(&df, opts)?;
println!("Original names: {:?}", X.get_column_names());
// Output: ["Intercept", "poly(x,2)^1", "poly(x,2)^2"]

// Manual cleaning
let cleaned = make_clean_names("poly(x,2)^1");
assert_eq!(cleaned, "poly_x_2_1");
```

## 🎛️ Advanced Usage

- **Custom Intercept Name**: Control `intercept_name` in `MaterializeOptions`
- **Disable Cleaning**: Set `clean_names: false` to preserve original expressions

## 🧪 Performance

polars-formula is built on top of Polars, one of the fastest DataFrame libraries available. Key performance characteristics:

- **Zero-copy when possible**: Leverages Polars' memory efficiency
- **Parallel computation**: Benefits from Polars' multi-threading
- **Lazy evaluation**: Supports Polars' lazy API patterns
- **SIMD optimizations**: Inherits Polars' vectorized operations

## 📦 Dependencies

- **Polars**: For providing the foundational DataFrame library
- **faer**: For high-performance linear algebra capabilities (optional via feature)

## 📜 License

MIT
