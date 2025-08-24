
[![Crates.io](https://img.shields.io/crates/v/polars-formula.svg)](https://crates.io/crates/polars-formula)
[![Documentation](https://docs.rs/polars-formula/badge.svg)](https://docs.rs/polars-formula)
[![License](https://img.shields.io/crates/l/polars-formula.svg)](LICENSE)

<h1 align="center">polars-formula</h1>

<p align="center">A formula parsing and materialization library for Rust that brings R-style/Formulaic/Patsy formula syntax to the Polars DataFrame ecosystem.</p>

<p align="center">
<img src="img/mango_pixel.png" alt="logo" width="100">
</p>

## 🚀 Features

- **🔍 Formula Parsing**: Parse formulas like `y ~ x1 + x2 + x1:x2 + poly(x1, 3) - 1`
- **🧹 Clean Column Names**: Automatic cleaning of complex column names for better usability
- **🧮 Linear Algebra Ready**: Direct conversion to [faer](https://github.com/sarah-quinones/faer-rs) matrices (optional feature)

## 📦 Installation

Run `cargo add polars-formula` or add this to your `Cargo.toml`:

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

### Formula Parsing

```bash
git clone https://github.com/alexh/polars-formula.git
cd polars-formula
cargo run --example clean_names_demo
```

## Capability Tables (DSL ↔ call ↔ parse ↔ materialize)

### A. Basic Operators & Sugar

| DSL           | Call              | Parse                                             | Materialize                                 | Status |
| ------------- | ----------------- | ------------------------------------------------- | ------------------------------------------- | ------ |
| `x1 + x2`     | parse/materialize | `Sum([x1,x2])`                                    | add both features                           | ✅      |
| `x1:x2`       |                   | `Interaction([x1,x2])`                            | product (incl. proper categorical crossing) | ✅      |
| `x1*x2`       |                   | `Prod([x1,x2])` → **canonical** `x1 + x2 + x1:x2` | expand and add all                          | ✅      |
| `(x1 + x2):z` |                   | `Interaction([Group(..), z])` after grouping      | `x1:z` and `x2:z` only                      | ✅      |
| `(x1 + x2)^2` |                   | `Pow{base=Sum(..), exp=2}`                        | expanded via canon rules                    | 🟡     |
| `a/b`         |                   | `Nest{outer=a, inner=b, kind=Slash}`              | **canonical** `a + a:b`                     | ✅      |
| `a %in% b`    |                   | `Nest{…, kind=In}`                                | semantics at validation/materialize         | 🟡     |
| `0` / `-1`    |                   | `Intercept(false)`                                | drop intercept                              | ✅      |
| `.`           |                   | `Dot`                                             | expand “all RHS cols except LHS & groups”   | 🗺️    |

### B. Transformations & Functions

| DSL                             | Call              | Parse                                     | Materialize                        | Status |
| ------------------------------- | ----------------- | ----------------------------------------- | ---------------------------------- | ------ |
| `poly(x, 3)`                    | parse/materialize | `Func{name="poly", args=[x,3]}`           | columns: `poly(x,3)^1`, `^2`, `^3` | ✅      |
| `log(x)`, `exp(x)`, `sqrt(x)`   |                   | `Func{..., [x]}`                          | numeric transform of column(s)     | ✅      |
| `scale(x)`                      |                   | `Func{..., [x]}`                          | center/scale                       | 🟡     |
| `C(var)` *(force categorical)*  |                   | `Func{"C",[var]}`                         | one-hot (treatment coded)          | 🟡     |
| `s(x, k=10, bs="tp")`           |                   | `Smooth{kind=S, vars=["x"], args={k,bs}}` | smooth backend hook                | 🟡     |
| `te(x,z)`, `ti(...)`, `t2(...)` |                   | `Smooth{kind=TE/TI/T2,...}`               | tensor smooths (backend)           | 🟡     |