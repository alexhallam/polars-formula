# Release v0.1.3 - Major Refactoring and Improvements

## 🎉 What's New

This release brings significant improvements to code quality, user experience, and maintainability while adding beautiful custom RGB color support.

## ✨ Key Features

### 🎨 Custom RGB Color Support
- **Response variables**: `rgb(191, 97, 106)` - Beautiful coral red
- **Terms/predictors**: `rgb(180, 142, 173)` - Elegant lavender  
- **Operators**: `rgb(235, 203, 139)` - Warm golden yellow
- **Fallback**: All other elements use the term color for consistency

### 🔧 Major Code Improvements
- **99% reduction** in duplicate color module code
- **Eliminated code duplication** between legacy and DSL parsers
- **Fixed all test failures** and compilation warnings
- **Enhanced error handling** with clear deprecation notices

### 📚 Documentation Overhaul
- **Completely rewritten README** with clear, actionable examples
- **Comprehensive DSL example** showcasing all major features
- **Migration guide** from legacy to modern API
- **Better code organization** and maintainability

## 🚀 New Examples

### Basic Formula Parsing
```rust
use polars_formula::dsl::{parser::parser, materialize::materialize_dsl_spec};
use chumsky::Parser;

let p = parser();
let spec = p.parse("y ~ x1 + x2".chars().collect::<Vec<_>>())?;
let (y, X, Z) = materialize_dsl_spec(&df, &spec, MaterializeOptions::default())?;
```

### Advanced Features
```rust
// Complex formula with interactions and random effects
let spec = p.parse("y ~ x1 * x2 + poly(x1, 2) + (1|group)".chars().collect::<Vec<_>>())?;

// Canonicalization
let canonicalized = canonicalize(&spec);
let canonical_str = pretty(&canonicalized);
// Output: y ~ x1 + x2 + x1:x2 + poly(x1, 2) + (1|group)
```

### Beautiful Colored Output
```rust
use polars_formula::SimpleColoredPretty;

let color_pretty = SimpleColoredPretty::default();
let formula = "y ~ x1 + x2 + x1:x2 + poly(x1, 2)";
println!("{}", color_pretty.formula(formula));
// Outputs beautifully colored formula with custom RGB colors
```

## 🔄 Migration Guide

### From Legacy API to DSL
**Before (deprecated):**
```rust
use polars_formula::{Formula, MaterializeOptions};
let formula = Formula::parse("y ~ x1 + x2")?;
let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;
```

**After (recommended):**
```rust
use polars_formula::dsl::{parser::parser, materialize::materialize_dsl_spec};
use chumsky::Parser;

let p = parser();
let spec = p.parse("y ~ x1 + x2".chars().collect::<Vec<_>>())?;
let (y, X, Z) = materialize_dsl_spec(&df, &spec, MaterializeOptions::default())?;
```

## 🧪 Test Results

- ✅ **All core tests passing**
- ✅ **All examples compiling and running**
- ✅ **Comprehensive DSL example working**
- ✅ **Custom colors displaying correctly**

## 📦 Installation

```bash
cargo add polars-formula@0.1.3
```

## 🎯 What's Working

### ✅ Fully Supported
- Basic formula parsing (`y ~ x1 + x2`)
- Interaction terms (`x1:x2`, `x1 * x2`)
- Polynomial terms (`poly(x, 2)`)
- Random effects (`(1|group)`)
- Family specifications (`family=gaussian()`)
- Canonicalization and pretty-printing
- Custom RGB color output
- Materialization to Polars DataFrames

### 🟡 Partially Supported
- Complex survival analysis (parser works, materialization in progress)
- Advanced aterms (parser works, materialization in progress)
- Distributional parameters (parser works, materialization in progress)

## 🔮 Future Plans

- Complete survival analysis materialization
- Enhanced categorical variable handling
- Spline function support
- Time series operations
- Direct model fitting integration

## 🙏 Thanks

Thank you to all contributors and users who provided feedback and helped improve the codebase!

---

**Full changelog and technical details available in [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md)**
