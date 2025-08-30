
[![Crates.io](https://img.shields.io/crates/v/polars-formula.svg)](https://crates.io/crates/polars-formula)
[![Documentation](https://docs.rs/polars-formula/badge.svg)](https://docs.rs/polars-formula)
[![License](https://img.shields.io/crates/l/polars-formula.svg)](LICENSE)

<h1 align="center">polars-formula</h1>


<p align="center">
  <img src="img/mango_pixel.png" alt="logo" width="120">
</p>

---
<p align="center">formula in model-matrix out</p>

---

<p align="center">A formula parsing and materialization library for Rust that brings R-style/Formulaic/Patsy formula syntax to the Polars DataFrame ecosystem.</p>


## ⚠️ In development ⚠️

This library is in development and is not yet ready for production use.

## Motivation

I wanted to work with formulas in Rust, using formula syntax. This library aims to fill that gap by providing a way to parse and materialize formulas directly into Polars DataFrames.

## 🚀 Features

- **🔍 Formula Parsing (Canonicalization)**: Parse formulas like `y ~ x1 + x2 + x1:x2 + poly(x1, 3) - 1`
- **🧹 Clean Column Names**: Automatic cleaning of complex column names for better usability
- **🎨 Colored Output**: Beautiful syntax highlighting for formulas

## API

Only four functions are exposed:

`canonicalize()` - Convert a formula string into its canonical form
`materialize()` - Convert a formula and DataFrame into response vector and design matrix
`print_formula()` - Print a formula with syntax highlighting
`print_modelspec()` - Print a model specification


## 📦 Installation

Run `cargo add polars-formula` or add this to your `Cargo.toml`:

```toml
[dependencies]
polars-formula = "0.3.8"
polars = { version = "0.50", features = ["lazy"] }
```

## 🏃‍♂️ Quick Start

### Basic Formula Parsing, coloring, and materialization


```rust
// Example 01
// ==========
//  If you git clone this repo, you can run this example with:
// git clone https://github.com/alexhallam/polars-formula.git
// cd polars-formula
// cargo run --example 01
use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple API Demo ===\n");

    // Load data
    let df = CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;

    // Formula string
    let formula_str = "mpg ~ cyl + wt*hp + poly(disp, 4) - 1";

    // Step 1: Canonicalize (parse and canonicalize)
    println!("\n1. Parse and canonicalize formula");
    let spec = canonicalize(formula_str)?;
    print_formula(&spec);

    // Step 2: Print the full model spec
    println!("\n2. Full model specification:");
    print_modelspec(&spec);

    // Step 3: Materialize the formula
    println!("\n3. Materializing formula");
    let (y, x, _z) = materialize(&spec, &df)?;
    println!("   Results: y={}\n X={}\n", y, x);
    Ok(())
}
```

<p align="center">
  <img src="img/output.png" alt="output">
</p>

## 📚 Run More Examples

Run the examples to see the library in action:

```bash
cargo run --example 01
cargo run --example 02
cargo run --example 03

```

## 🔧 Supported Formula Syntax

### Basic Operations
- **Variables**: `x`, `income`, `age`
- **Addition**: `x1 + x2` (include both terms)
- **Interactions**: `x1:x2` (product of x1 and x2)
- **Products**: `x1 * x2` (expands to `x1 + x2 + x1:x2`)
- **Intercept**: Automatically included (use `-1` to remove)

### Functions
- **Polynomials**: `poly(x, 3)` expands to x, x², x³
- **Identity**: `I(x)` for literal interpretation
- **Constants**: Numeric literals like `1`, `0` for intercept control

### Random Effects
- **Random Intercepts**: `(1|group)` - one random effect per group level
- **Random Slopes**: `(x|group)` - random slope for variable x per group
- **Uncorrelated**: `(x||group)` - uncorrelated random effects

### Advanced Features
- **Family Specification**: `y ~ x, family=gaussian()`
- **Distributional Parameters**: `y ~ x + sigma ~ z`
- **Autocorrelation**: `y ~ x + ar(p=1)`

## 🎯 Key Benefits

1. **R-like Syntax**: Familiar formula syntax for statisticians
2. **Type Safety**: Rust's type system ensures correctness
3. **Performance**: Built on Polars for high-performance data manipulation
4. **Extensibility**: Easy to add new formula features
5. **Integration**: Seamless integration with the Rust data science ecosystem

## 🔮 Future Features

- **Categorical Variables**: `C(category)` for factor encoding
- **Spline Functions**: `s(x, k=10)` for smooth function approximations
- **Time Series**: Lag operations and ARIMA support
- **Custom Functions**: User-defined transformations
- **Model Fitting**: Direct integration with statistical modeling libraries

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
