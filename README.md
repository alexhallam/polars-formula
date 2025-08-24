
[![Crates.io](https://img.shields.io/crates/v/polars-formula.svg)](https://crates.io/crates/polars-formula)
[![Documentation](https://docs.rs/polars-formula/badge.svg)](https://docs.rs/polars-formula)
[![License](https://img.shields.io/crates/l/polars-formula.svg)](LICENSE)

<h1 align="center">polars-formula</h1>

<p align="center">A formula parsing and materialization library for Rust that brings R-style/Formulaic/Patsy formula syntax to the Polars DataFrame ecosystem.</p>

<p align="center">
  <img src="img/mango_pixel.png" alt="logo" width="120">
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
cargo run --example simple_formula
```

The code in the `simple_formula.rs` example is a simple example of how to use the `polars-formula` library to parse a formula and materialize it into a `DataFrame`.

```rust
use polars::prelude::*; // DataFrame andCsvReader
use polars_formula::{Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple dataset
    let df: DataFrame =
        CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;

    // Original formula
    let formula_str = "mpg ~ wt + hp + cyl + wt:hp + poly(disp, 2) - 1";
    println!("Original: {}", formula_str);

    // Materialize the formula
    let formula = Formula::parse(formula_str)?;
    let (y, x) = formula.materialize(&df, MaterializeOptions::default())?;

    // Print the results
    println!("y: {}", y);
    println!("X: {}", x);

    Ok(())
}
```
Original: mpg ~ wt + hp + cyl + wt:hp + poly(disp, 2) - 1
y: shape: (32, 1)
┌──────┐
│ mpg  │
│ ---  │
│ f64  │
╞══════╡
│ 21.0 │
│ 21.0 │
│ 22.8 │
│ 21.4 │
│ 18.7 │
│ …    │
│ 30.4 │
│ 15.8 │
│ 19.7 │
│ 15.0 │
│ 21.4 │
└──────┘
X: shape: (32, 6)
┌───────┬───────┬─────┬─────────┬───────────────┬───────────────┐
│ wt    ┆ hp    ┆ cyl ┆ wt_hp   ┆ poly_disp_2_1 ┆ poly_disp_2_2 │
│ ---   ┆ ---   ┆ --- ┆ ---     ┆ ---           ┆ ---           │
│ f64   ┆ f64   ┆ f64 ┆ f64     ┆ f64           ┆ f64           │
╞═══════╪═══════╪═════╪═════════╪═══════════════╪═══════════════╡
│ 2.62  ┆ 110.0 ┆ 6.0 ┆ 288.2   ┆ 160.0         ┆ 25600.0       │
│ 2.875 ┆ 110.0 ┆ 6.0 ┆ 316.25  ┆ 160.0         ┆ 25600.0       │
│ 2.32  ┆ 93.0  ┆ 4.0 ┆ 215.76  ┆ 108.0         ┆ 11664.0       │
│ 3.215 ┆ 110.0 ┆ 6.0 ┆ 353.65  ┆ 258.0         ┆ 66564.0       │
│ 3.44  ┆ 175.0 ┆ 8.0 ┆ 602.0   ┆ 360.0         ┆ 129600.0      │
│ …     ┆ …     ┆ …   ┆ …       ┆ …             ┆ …             │
│ 1.513 ┆ 113.0 ┆ 4.0 ┆ 170.969 ┆ 95.1          ┆ 9044.01       │
│ 3.17  ┆ 264.0 ┆ 8.0 ┆ 836.88  ┆ 351.0         ┆ 123201.0      │
│ 2.77  ┆ 175.0 ┆ 6.0 ┆ 484.75  ┆ 145.0         ┆ 21025.0       │
│ 3.57  ┆ 335.0 ┆ 8.0 ┆ 1195.95 ┆ 301.0         ┆ 90601.0       │
│ 2.78  ┆ 109.0 ┆ 4.0 ┆ 303.02  ┆ 121.0         ┆ 14641.0       │
└───────┴───────┴─────┴─────────┴───────────────┴───────────────┘
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

##### Categorical encoding (default heuristic):

- Treat as categorical if `Utf8` or low-cardinality integer (≤ 32 distinct).

- Treatment coding with base = `"1"` if present (configurable).

- Names like `var_level` (e.g., `period_2`, `period_3`, ...).

### C. Random Effects (Grouping)

| DSL                | Call              | Parse                                                   | Canonical → Materialize (Z)                                                  | Status          |           |   |
| ------------------ | ----------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------- | --------------- | --------- | - |
| `(1 \| g)`         | parse/materialize | `Group{inner=Intercept(true), spec=g, kind=Correlated}` | Z: one-hot per level `ri_g_*`                                                | ✅               |           |   |
| `(x \| g)`         |                   | `Group{inner=Var(x), ...}`                              | **canonical** `(1 \| g) + (0 + x \| g)`; Z has intercept block + slope block | 🟡 (slopes mat) |           |   |
| `(x \|\| g)`       |                   | `Group{..., kind=Uncorrelated}`                         | same blocks; block-diagonal G                                                | 🟡              |           |   |
| `(1 \| g1:g2)`     |                   | `Group{spec=g1:g2}`                                     | Z levels are interaction levels                                              | 🟡              |           |   |
| `(1 \| g1/g2)`     |                   | `Group{spec=g1/g2}`                                     | **canonical** \`(1                                                           | g1) + (1        | g1\:g2)\` | ✅ |
| `(1 \| mm(g1,g2))` |                   | `Group{spec=Func("mm",...)}`                            | multi-membership Z                                                           | 🗺️             |           |   |



### D. Response, Family, and Canonicalization

| DSL                                                                            | Call                                                       | Parse (AST sketch)                                                                                                                                                 | Materialize (y / X / Z)                                                                                                                                                                                               | Status                                                        |
| ------------------------------------------------------------------------------ | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| `y ~ x1 + x2`                                                                  | `Formula::parse("y ~ x1 + x2")` → `.materialize(df, opts)` | `lhs=Var("y"); rhs=Sum([Var("x1"), Var("x2")])`                                                                                                                    | `y`: numeric `Series` • `X`: `[intercept, x1, x2]` • `Z`: none                                                                                                                                                        | ✅                                                             |
| `y ~ 0 + x1 + x2` *(no intercept)* & `y ~ x1 + x2 - 1` *(no intercept)*                                               | same                                                       | `lhs=Var("y"); rhs=Sum([Intercept(false), ...])`                                                                                                                   | `X`: `[x1, x2]` (no intercept)                                                                                                                                                                                        | ✅                                                             |
| `incidence \| trials(size) ~ period + (1 \| herd), family = binomial("logit")` | same                                                       | `lhs=Var("incidence"); aterms=[Trials(Var("size"))]; family=binomial("logit"); rhs=Sum([Var("period"), Group(inner=Intercept(true), spec=herd, kind=Correlated)])` | **y**: `BinomialTrials { successes=incidence, trials=size }` • **X**: treatment-coded `period` → `[intercept, period_2, period_3, period_4]` • **Z**: random-intercept one-hot per `herd` → `[ri_herd_1..ri_herd_15]` | ✅ (family header parsing: 🟡 if trailing parse not yet wired) |
| `Surv(time, event) ~ x + (1 \| id)`                                            | same                                                       | `lhs=Surv{time,event}`; rhs includes group                                                                                                                         | **y**: `(time,event)`; **X**/**Z** depend on backend (Cox PH)                                                                                                                                                         | 🗺️                                                           |
| `mvbind(y1,y2) ~ x`                                                            | same                                                       | `lhs=Multi(["y1","y2"])`                                                                                                                                           | **y**: multivariate bundle • **X** as usual • **Z** optional                                                                                                                                                          | 🗺️                                                           |


Canonicalization rules (always applied):

- `a*b → a + b + a:b`

- `(x \| g) → (1 \| g) + (0 + x \| g)`

- `g1/g2 → g1 + g1:g2`

- `a:b:c` is left-associative (R semantics): `(a:b):c`

- Intercept control: `1` adds, `0` removes (on each side where applicable)


### E. LHS Addition Terms (a-terms)

| DSL (LHS)                  | Call | Parse                       | Materialize                                   | Status |
| -------------------------- | ---- | --------------------------- | --------------------------------------------- | ------ |
| `y \| trials(n)`           | same | `aterms=[Trials(Var("n"))]` | **y**: `BinomialTrials{successes=y,trials=n}` | ✅      |
| `y \| weights(w)`          | same | `aterms=[Weights(expr)]`    | sidecar weights for loss/likelihood           | 🟡     |
| `y \| se(se_y)`            | same | `aterms=[Se(expr)]`         | sidecar known SEs (meta-analysis)             | 🟡     |
| `y \| cens(c)`             | same | `aterms=[Cens(expr)]`       | sidecar censoring indicator                   | 🟡     |
| `y \| trunc(lb=0, ub=100)` | same | `aterms=[Trunc{lb,ub}]`     | bounds for truncated likelihood               | 🟡     |
| `y \| subset(cond)`        | same | `aterms=[Subset(expr)]`     | row filter (semantic stage)                   | 🟡     |
| `y \| rate(exposure)`      | same | `aterms=[Rate(expr)]`       | exposure offset semantics                     | 🟡     |



a-terms chain with `|`: `y | trials(n) | weights(w) | se(se_y) | cens(c) | trunc(lb=0, ub=100) | subset(cond) | rate(exposure)`

### F. Distributional / Multi-parameter RHS (parsing hooks)

| DSL                                 | Parse                              | Materialize                                     | Status |
| ----------------------------------- | ---------------------------------- | ----------------------------------------------- | ------ |
| `sigma ~ z`, `zi ~ z`, `phi ~ z`, … | `Dpar{name="sigma", rhs=...}` etc. | built as extra fixed effects blocks per d-param | 🟡     |


### G. Materialization Contracts (at a glance)

| Scenario                                                         | y (response bundle)                     | X (fixed effects)                           | Z (random effects)                  | Notes                                  |
| ---------------------------------------------------------------- | --------------------------------------- | ------------------------------------------- | ----------------------------------- | -------------------------------------- |
| **Gaussian** `y ~ x + z`                                         | numeric `Series<f64>`                   | `[intercept, x, z]`                         | —                                   | —                                      |
| **Binomial with trials** `y \| trials(n) ~ period + (1 \| herd)` | `BinomialTrials{successes=y, trials=n}` | `[intercept, period_2, period_3, period_4]` | `[ri_herd_1..ri_herd_15]`           | failures = `n - y` (derived if needed) |
| **Random slope** `y ~ x + (x \| g)`                              | numeric                                 | `[intercept, x]`                            | blocks for `ri_g_*` and `rs_x__g_*` | slopes materialization 🟡              |
| **Dot** `y ~ .`                                                  | numeric                                 | all RHS cols except LHS/groups              | optional                            | 🗺️                                    |
