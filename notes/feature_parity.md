# Feature Parity Table

This table shows the implementation status of polars-formula features compared to R's `model.matrix()` and Python's Formulaic/Patsy libraries.

## Core Formula Features

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Basic formulas** | ✅ | ✅ | ✅ | ✅ | `y ~ x1 + x2` |
| **Intercept control** | ✅ | ✅ | ✅ | ✅ | `y ~ x - 1` or `y ~ 0 + x` |
| **Variable references** | ✅ | ✅ | ✅ | ✅ | Direct column names |
| **Numeric variables** | ✅ | ✅ | ✅ | ✅ | Automatic conversion |

## Transformations & Functions

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Identity function** | ✅ | ✅ | ✅ | ✅ | `I(x)` |
| **Polynomials** | 🟡 | ✅ | ✅ | ✅ | `poly(x, 2)` - parsing only |
| **Power terms** | ❌ | ✅ | ✅ | ✅ | `x^2` |
| **Log/exp functions** | ❌ | ✅ | ✅ | ✅ | `log(x)`, `exp(x)` |
| **Spline bases** | ❌ | ✅ | ✅ | ✅ | `s(x)`, `ns(x)`, `bs(x)` |

## Interactions & Products

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Binary interactions** | 🟡 | ✅ | ✅ | ✅ | `x1:x2` - parsing only |
| **Product expansion** | 🟡 | ✅ | ✅ | ✅ | `x1*x2` - parsing only |
| **Multi-way interactions** | ❌ | ✅ | ✅ | ✅ | `x1:x2:x3` |
| **Nesting** | ❌ | ✅ | ✅ | ✅ | `a/b` or `b %in% a` |

## Categorical Variables

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Categorical detection** | ❌ | ✅ | ✅ | ✅ | Auto-detect string columns |
| **Treatment contrasts** | ❌ | ✅ | ✅ | ✅ | Default in R |
| **Sum contrasts** | ❌ | ✅ | ✅ | ✅ | `contr.sum` |
| **Helmert contrasts** | ❌ | ✅ | ✅ | ✅ | `contr.helmert` |
| **Polynomial contrasts** | ❌ | ✅ | ✅ | ✅ | `contr.poly` |
| **Custom contrasts** | ❌ | ✅ | ✅ | ✅ | User-defined |

## Random Effects

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Random intercepts** | 🟡 | ✅ | ✅ | ❌ | `(1\|group)` - dense only |
| **Random slopes** | 🟡 | ✅ | ✅ | ❌ | `(x\|group)` - dense only |
| **Uncorrelated effects** | ❌ | ✅ | ✅ | ❌ | `(x\|\|group)` |
| **Crossed effects** | ❌ | ✅ | ✅ | ❌ | `(1\|g1) + (1\|g2)` |
| **Nested effects** | ❌ | ✅ | ✅ | ❌ | `(1\|g1/g2)` |

## Response Types

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Single response** | ✅ | ✅ | ✅ | ✅ | `y ~ x` |
| **Multi-response** | 🟡 | ✅ | ✅ | ✅ | `cbind(y1, y2) ~ x` - basic |
| **Binomial trials** | 🟡 | ✅ | ✅ | ✅ | `cbind(success, failure) ~ x` |
| **Survival** | ❌ | ✅ | ✅ | ✅ | `Surv(time, event) ~ x` |

## Advanced Features

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Distributional parameters** | ❌ | ✅ | ✅ | ❌ | `y ~ x + sigma ~ z` |
| **Autocorrelation** | ❌ | ✅ | ✅ | ❌ | `y ~ x + ar(p=1)` |
| **Weights** | ❌ | ✅ | ✅ | ✅ | `y \| weights(w) ~ x` |
| **Offset** | ❌ | ✅ | ✅ | ✅ | `y ~ x + offset(log(n))` |

## Polars Integration

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **DataFrame input** | ✅ | ✅ | ✅ | ✅ | Direct Polars support |
| **LazyFrame integration** | ❌ | ❌ | ❌ | ❌ | `formula.to_exprs()` |
| **Sparse outputs** | ❌ | ❌ | ❌ | ❌ | Sparse Z matrices |
| **faer conversion** | 🟡 | ❌ | ❌ | ❌ | Basic support |

## Performance & Quality

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Memory efficiency** | 🟡 | ✅ | ✅ | ✅ | Random effects need sparse |
| **Numerical stability** | 🟡 | ✅ | ✅ | ✅ | Orthogonal polynomials |
| **Large dataset support** | 🟡 | ✅ | ✅ | ✅ | Needs optimization |
| **Parallel processing** | ❌ | ✅ | ✅ | ❌ | Not implemented |

## Developer Experience

| Feature | Status | R | Python Formulaic | Python Patsy | Notes |
|---------|--------|---|------------------|--------------|-------|
| **Error messages** | 🟡 | ✅ | ✅ | ✅ | Basic, needs improvement |
| **Documentation** | 🟡 | ✅ | ✅ | ✅ | ~21% documented |
| **Examples** | 🟡 | ✅ | ✅ | ✅ | Basic examples only |
| **Testing** | 🟡 | ✅ | ✅ | ✅ | Basic test coverage |

## Legend

- ✅ **Implemented**: Feature is fully functional
- 🟡 **Partial**: Feature is parsed but not fully materialized
- ❌ **Not implemented**: Feature is not yet available
- 🔄 **In progress**: Feature is being actively developed

## Priority Implementation Order

### Phase 1: Critical Gaps (Weeks 1-2)
1. **Categorical contrasts** - Core statistical functionality
2. **Polynomial expansion** - Fix `poly()` function
3. **Interactions** - Fix `:` and `*` operators
4. **Sparse random effects** - Performance improvement

### Phase 2: Advanced Features (Weeks 3-4)
1. **Spline bases** - `s()`, `ns()`, `bs()` functions
2. **Multi-response handling** - Proper `cbind()` support
3. **LazyFrame integration** - `formula.to_exprs()`
4. **Distributional parameters** - `y ~ x + sigma ~ z`

### Phase 3: Polish & Testing (Weeks 5-6)
1. **Error diagnostics** - Better error messages
2. **Documentation** - Expand coverage to >80%
3. **Golden tests** - vs R and Python implementations
4. **Performance benchmarks** - Optimization

## Contributing

If you'd like to help implement any of these features, please check the [roadmap](notes/roadmap.md) for detailed implementation plans and the [contribution guidelines](CONTRIBUTING.md) for development setup.

## References

- **R**: `model.matrix()` function in base R
- **Python Formulaic**: [formulaic](https://github.com/matthewwardrop/formulaic) library
- **Python Patsy**: [patsy](https://github.com/pydata/patsy) library (deprecated but reference)
