
[![Crates.io](https://img.shields.io/crates/v/polars-formula.svg)](https://crates.io/crates/polars-formula)
[![Documentation](https://docs.rs/polars-formula/badge.svg)](https://docs.rs/polars-formula)
[![License](https://img.shields.io/crates/l/polars-formula.svg)](LICENSE)

<h1 align="center">polars-formula</h1>

<p align="center">A formula parsing and materialization library for Rust that brings R-style/Formulaic/Patsy formula syntax to the Polars DataFrame ecosystem.</p>

![logo](img/mango_pixel.png)

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


## 🎯 Supported Formula Syntax

### Basic Operations

- **Variables**: `x`, `income`, `age`
- **Addition**: `x1 + x2` (include both terms)
- **Interactions**: `x1:x2` (product of x1 and x2)
- **Intercept**: Automatically included (use `-1` or options to control)

### Functions

- **Polynomials**: `poly(x, 3)` expands to x, x², x³
- **Constants**: Numeric literals like `1`, `0` for intercept control
- **Intercept Removal**: Use `-1` to remove the intercept term

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

### Linear Regression without Intercept

```rust
use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions};

let df = df!(
    "price" => [100.0, 150.0, 200.0, 250.0],
    "size" => [1000.0, 1500.0, 2000.0, 2500.0],
    "age" => [5.0, 10.0, 15.0, 20.0]
)?;

let formula = Formula::parse("price ~ size + age - 1")?;
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;

// X now contains: [size, age] (no intercept)
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
use polars_formula::{Formula, MaterializeOptions};

// With intercept (default)
let formula = Formula::parse("y ~ x1 + x2")?;
let opts_with = MaterializeOptions::default();

// Without intercept using -1 syntax
let formula_no_intercept = Formula::parse("y ~ x1 + x2 - 1")?;

// Without intercept using options
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


# Formula Operators


| Operator | Arity | Description | Formulaic | Patsy | R |
|---------:|:-----:|:------------|:---------:|:-----:|:-:|
| `"..."`[^1] | 1 | String literal. | ✓ | ✓ | ✗ |
| `[0-9]+\.[0-9]+`[^1] | 1 | Numerical literal. | ✓ | ✗ | ✗ |
| `` `...` ``[^1] | 1 | Quotes fieldnames within the incoming dataframe, allowing the use of special characters, e.g. `` `my|special$column!` `` | ✓ | ✗ | ✓ |
| `{...}`[^1] | 1 | Quotes python operations, as a more convenient way to do Python operations than `I(...)`, e.g. `` {`my|col`**2} `` | ✓ | ✗ | ✗ |
| `<function>(...)`[^1] | 1 | Python transform on column, e.g. `my_func(x)` which is equivalent to `{my_func(x)}` | ✓[^2] | ✓ | ✗ |
|-----|
| `(...)` | 1 | Groups operations, overriding normal precedence rules. All operations with the parentheses are performed before the result of these operations is permitted to be operated upon by its peers. | ✓ | ✗ | ✓ |
|-----|
| `.`[^9] | 0 | Stands in as a wild-card for the sum of variables in the data not used on the left-hand side of a formula. | ✓ | ✗ | ✓ |
|-----|
| `**` | 2 | Includes all n-th order interactions of the terms in the left operand, where n is the (integral) value of the right operand, e.g. `(a+b+c)**2` is equivalent to `a + b + c + a:b + a:c + b:c`. | ✓ | ✓ | ✓ |
| `^` | 2 | Alias for `**`. | ✓ | ✗[^3] | ✓ |
|-----|
| `:` | 2 | Adds a new term that corresponds to the interaction of its operands (i.e. their elementwise product). | ✓[^4] | ✓ | ✓ |
|-----|
| `*` | 2 | Includes terms for each of the additive and interactive effects of the left and right operands, e.g. `a * b` is equivalent to `a + b + a:b`. | ✓ | ✓ | ✓ |
| `/` | 2 | Adds terms describing nested effects. It expands to the addition of a new term for the left operand and the interaction of all left operand terms with the right operand, i.e `a / b` is equivalent to `a + a:b`, `(a + b) / c` is equivalent to `a + b + a:b:c`, and `a/(b+c)` is equivalent to `a + a:b + a:c`.[^5] | ✓ | ✓ | ✓ |
| `%in%` | 2 | As above, but with arguments inverted: e.g. `b %in% a` is equivalent to `a / b`. | ✓ | ✗ | ✓ |
|-----|
| `+` | 2 | Adds a new term to the set of features. | ✓ | ✓ | ✓ |
| `-` | 2 | Removes a term from the set of features (if present). | ✓ | ✓ | ✓ |
| `+` | 1 | Returns the current term unmodified (not very useful). | ✓ | ✓ | ✓ |
| `-` | 1 | Negates a term (only implemented for 0, in which case it is replaced with `1`). | ✓ | ✓ | ✓ |
|-----|
| `\|` | 2 | Splits a formula into multiple parts, allowing the simultaneous generation of multiple model matrices. When on the right-hand-side of the `~` operator, all parts will attract an additional intercept term by default. | ✓ | ✗ | ✓[^6] |
|-----|
| `~` | 1,2 | Separates the target features from the input features. If absent, it is assumed that we are considering only the the input features. Unless otherwise indicated, it is assumed that the input features implicitly include an intercept. | ✓ | ✓ | ✓ |
| `[ . ~ . ]` | 2 | [Experimental] Multi stage formula notation, which is useful in (e.g.) IV contexts. Requires the `MULTISTAGE` feature flag to be passed to the parser. | ✓ | ✗ | ✗ |


## Transforms

Formulaic supports arbitrary transforms, any of which can also preserve state so
that new data can undergo the same transformation as that used during modelling.
The currently implemented transforms are shown below. Commonly used transforms
that have *not* been implemented by `formulaic` are explicitly noted also.

| Transform | Description | Formulaic | Patsy | R |
|----------:|:------------|:---------:|:-----:|:-:|
| `I(...)` | Identity transform, allowing arbitrary Python/R operations, e.g. `I(x+y)`. Note that in `formulaic`, it is more idiomatic to use `{x+y}`. | ✓ | ✓ | ✓ |
| `Q('<column_name>')` | Look up feature by potentially exotic name, e.g. `Q('wacky name!')`. Note that in `formulaic`, it is more idiomatic to use ``` `wacky name!` ```. | ✓ | ✓ | ✗ |
| `C(...)` | Categorically encode a column, e.g. `C(x)` | ✓ | ✓ | ✓ |
| `center(...)` | Shift column data so mean is zero. | ✓ | ✓ | ✗ |
| `scale(...)` | Shift column so mean is zero and variance is 1. | ✓ | ✓[^7] | ✓ |
| `standardize(...)` | Alias of `scale`. | ✓[^8] | ✓ | ✗ |
| `lag(...[, <k>])` | Generate lagging or leading columns (useful for datasets collected at regular intervals). | ✓ | ✗ | ✓ |
| `poly(...)` | Generates a polynomial basis, allowing non-linear fits. | ✓ | ✗ | ✓ |
| `bs(...)` | Generates a B-Spline basis, allowing non-linear fits. | ✓ | ✓ | ✓ |
| `cs(...)` | Generates a natural cubic spline basis, allowing non-linear fits. | ✓ | ✓ | ✓ |
| `cr(...)` | Alias for `cs` above. | ✓ | ✗ | ✓ |
| `cc(...)` | Generates a cyclic cubic spline basis, allowing non-linear fits. | ✓ | ✓ | ✓ |
| `te(...)` | Generates a tensor product smooth. | ✗ | ✓ | ✓ |
| `hashed(...)` | Categorically encode a deterministic hash of a column. | ✓ | ✗ | ✗ |
| ...       | Others? Contributions welcome!     | ? | ? | ? |

!!! tip
    Any function available in the `context` dictionary will also be available
    as transform, along with some commonly used functions imported from
    numpy: `log`, `log10`, `log2`, `exp`, `exp10`, and `exp2`. In addition
    the `numpy` module is always available as `np`. Thus, formulas like:
    `log(y) ~ x + 10` will always do the right thing, even when these functions
    have not been made available in the user namespace.

!!! note
    Formulaic does not (yet) support including extra terms in the formula that
    will not result in additions to the dataframe, for example model annotations
    like R's `offset(...)`.

## Behaviours and Conventions

Beyond the formula operator grammar itself there are some differing behaviours
and conventions of which you should be aware.

  - Formulaic follows Patsy and then enhanced `Formula` R package in that both
    sides of the `~` operator are treated considered to be using the formula
    grammar, with the only difference being that the right hand side attracts an
    intercept by default. In vanilla R, the left hand side is treated as R code
    (and so `x + y ~ z` would result in a single column on the left-hand-side).
    You can recover vanilla R's behaviour by nesting the operations in a Python
    operator block (as described in the operator table): `{y1 + y2} ~ a + b`.
  - Formula terms in Formulaic are always sorted first by the order of the
    interaction, and then alphabetically. In R and patsy, this second ordering
    is done in the order that columns were introduced to the formula (patsy
    additionally sorts by which fields are involved in the interactions). As a
    result formulas generated by `formulaic` with the same set of fields will
    always generate the same model matrix.
  - Formulaic follows patsy's more rigourous handling of whether or not to
    include an intercept term. In R, `b-1` and `(b-1)` both do not have an
    intercept, whereas in Formulaic and Patsy the parentheses are resolved
    first, and so the first does not have an intercept and the second does
    (because '1 +' is implicitly prepended to the right hand side of the formula).
  - Formulaic borrows a clever algorithm introduced by Patsy to carefully choose
    where to reduce the rank of the model matrix in order to ensure that the
    matrix is structurally full rank. This avoids producing over-specified model
    matrices in contexts that R would (since it only considers local full-rank
    structure, rather than global structure). You can read more about this in
    [Patsy's documentation](https://patsy.readthedocs.io/en/latest/formulas.html).


[^1]: This "operator" is actually part of the tokenisation process.
[^2]: Formulaic additionally supports quoted fields with special characters, e.g. `` my_func(`my|special+column`) ``.
[^3]: The caret operator is not supported, but will not cause an error. It is ignored by the patsy formula parser, and treated as XOR Python operation on column.
[^4]: Note that Formulaic also allows you to use this to scale columns, for example: `2.5:a` (this scaling happens after factor coding).
[^5]: This somewhat confusing operator is useful when you want to include hierachical features in your data, and where certain interaction terms do not make sense (particularly in ANOVA contexts). For example, if `a` represents countries, and `b` represents cities, then the full product of terms from `a * b === a + b + a:b` does not make sense, because any value of `b` is guaranteed to coincide with a value in `a`, and does not independently add value. Thus, the operation `a / b === a + a:b` results in more sensible dataset. As a result, the `/` operator is right-distributive, since if `b` and `c` were both nested in `a`, you would want `a/(b+c) === a + a:b + a:c`. Likewise, the operator is not left-distributive, since if `c` is nested under both `a` and `b` separately, then you want `(a + b)/c === a + b + a:b:c`. Lastly, if `c` is nested in `b`, and `b` is nested in `a`, then you would want `a/b/c === a + a:(b/c) === a + a:b + a:b:c`.
[^6]: Implemented by an R package called [Formula](https://cran.r-project.org/web/packages/Formula/index.html) that extends the default formula syntax.
[^7]: Patsy uses the `rescale` keyword rather than `scale`, but provides the same functionality.
[^8]: For increased compatibility with patsy, we use patsy's signature for `standardize`.
[^9]: Requires additional context to be passed in when directly using the `Formula` constructor.

# Inspiration

1. [Formulaic](https://github.com/pydata/formulaic)
2. [Patsy](https://github.com/pydata/patsy)
3. [R](https://www.r-project.org/)