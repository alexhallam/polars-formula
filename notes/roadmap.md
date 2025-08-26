# polars-formula Roadmap

## 🎯 Priority 1: Core Materialization (Table Stakes)

### 1.1 Categorical Variables & Contrasts
**Status**: ❌ Not implemented  
**Priority**: Critical

```rust
// Target API
let opts = MaterializeOptions::default()
    .with_contrast("species", Contrast::Treatment { baseline: "setosa" })
    .with_contrast_default(Contrast::Sum);

// Implementation needed:
// - Detect categorical columns (string, categorical, or explicit)
// - Implement contrast types: Treatment, Sum, Helmert, Polynomial
// - Stable level ordering and deterministic column naming
// - NA policy handling
```

**Files to modify**:
- `src/dsl/materialize.rs` - Add categorical detection and contrast handling
- `src/lib.rs` - Add `Contrast` enum and `MaterializeOptions` methods
- `src/dsl/ast.rs` - Add contrast specifications to AST

### 1.2 Interactions & Polynomial Bases
**Status**: 🟡 Partially implemented (parsing only)  
**Priority**: Critical

```rust
// Current: poly() returns original series
// Target: poly(x, 2) → [x, x²] with orthogonal option
// Target: x1:x2 → interaction columns
// Target: x1*x2 → x1 + x2 + x1:x2
```

**Implementation needed**:
- `poly()`: Orthogonal vs raw polynomials, centering/standardization
- `:` interactions: Proper column multiplication
- `*` products: Full expansion to sum of terms
- `^` powers: Polynomial expansion

### 1.3 Spline Bases
**Status**: 🟡 Parsing only  
**Priority**: High

```rust
// Target API
s(x, k=10, bs="tp")  // Thin plate splines
ns(x, df=5)          // Natural splines
bs(x, degree=3)      // B-splines
```

**Implementation needed**:
- Basis construction for different spline types
- Knot placement algorithms
- Boundary condition handling

### 1.4 Robust Random Effects
**Status**: 🟡 Basic implementation (dense one-hot)  
**Priority**: High

```rust
// Current: Dense one-hot encoding (memory heavy)
// Target: Sparse representation with efficient Z construction
// Support: (1|g), (x|g), (x||g) with uncorrelated structure
```

**Implementation needed**:
- Sparse matrix representation (CSR/CSC)
- Efficient group encoding
- Support for complex random effect structures
- Integration with `faer-sparse` feature

## 🎯 Priority 2: Advanced Features

### 2.1 Multi-Response & Special Responses
**Status**: 🟡 Basic placeholders  
**Priority**: Medium

```rust
// Current: Multi-response takes first column, binomial ignores trials
// Target: Proper handling of:
// - cbind(success, failure) ~ x
// - Surv(time, event) ~ x
// - Binomial with trials parameter
```

### 2.2 Distributional Parameters & Families
**Status**: ❌ Not implemented  
**Priority**: Medium

```rust
// Target API
y ~ x + sigma ~ z  // Distributional parameters
family=gaussian()  // Distribution families
link=logit()       // Link functions
```

### 2.3 Autocorrelation Terms
**Status**: ❌ Not implemented  
**Priority**: Low

```rust
// Target API
y ~ x + ar(p=1)    // AR(1) terms
y ~ x + ma(q=1)    // MA(1) terms
```

## 🎯 Priority 3: Polars-Native Features

### 3.1 LazyFrame Integration
**Status**: ❌ Not implemented  
**Priority**: High

```rust
// Target API
let terms = formula.to_exprs()?;  // Vec<Expr>
let lazy_df = df.lazy()
    .with_columns(terms)
    .collect()?;
```

**Implementation needed**:
- Formula → `Vec<Expr>` conversion
- Integration with Polars optimizer
- Lazy materialization

### 3.2 Sparse Outputs
**Status**: ❌ Not implemented  
**Priority**: Medium

```rust
// Target API
let (y, X_dense, Z_sparse) = materialize_sparse(&df, &spec, opts)?;
let matrices = (X, Z, y).to_faer()?;  // Behind faer feature flag
```

## 🎯 Priority 4: Developer Experience

### 4.1 Error Handling & Diagnostics
**Status**: 🟡 Basic  
**Priority**: High

```rust
// Target: Colored, helpful error messages
// - Parse error highlighting
// - Semantic error suggestions
// - "Did you mean?" for unknown variables
// - Lint pass for ambiguous expressions
```

### 4.2 Documentation & Examples
**Status**: 🟡 ~21% documented  
**Priority**: Medium

**Needed**:
- Expand rustdoc coverage to >80%
- Cookbook examples (categoricals, interactions, random effects, splines)
- Feature parity table in README
- Quickstart with pinned Polars versions

### 4.3 Testing & Benchmarks
**Status**: 🟡 Basic tests  
**Priority**: Medium

**Needed**:
- Golden tests vs R `model.matrix()` / brms
- Parity tests vs Python Formulaic/Patsy
- Benchmarks vs existing solutions
- CI: lint, test, docs.rs, MSRV check

## 🎯 Priority 5: Production Polish

### 5.1 Term Map & Provenance
**Status**: ❌ Not implemented  
**Priority**: Low

```rust
// Target API
let (y, X, Z, info) = materialize_with_info(&df, &spec, opts)?;
// info: DesignInfo { terms, columns, levels, contrasts, transforms }
```

### 5.2 Performance Optimizations
**Status**: 🟡 Basic  
**Priority**: Low

**Needed**:
- Memory-efficient materialization
- Parallel processing for large datasets
- Caching of basis matrices

## 📋 Implementation Checklist

### Phase 1: Core Materialization (Weeks 1-2)
- [ ] Implement categorical detection and contrasts
- [ ] Fix `poly()` to return proper polynomial columns
- [ ] Implement `:` interactions
- [ ] Implement `*` product expansion
- [ ] Add sparse random effects with `faer-sparse`

### Phase 2: Advanced Features (Weeks 3-4)
- [ ] Implement spline bases (`s()`, `ns()`, `bs()`)
- [ ] Fix multi-response handling
- [ ] Add distributional parameters support
- [ ] Implement LazyFrame integration

### Phase 3: Polish & Testing (Weeks 5-6)
- [ ] Expand documentation coverage
- [ ] Add comprehensive test suite
- [ ] Implement error diagnostics
- [ ] Add CI pipeline
- [ ] Create feature parity table

### Phase 4: Production Ready (Weeks 7-8)
- [ ] Performance optimizations
- [ ] Term map and provenance
- [ ] Benchmarks and profiling
- [ ] Final documentation polish

## 🧪 Golden Tests to Implement

```rust
// Test vs R model.matrix()
#[test]
fn test_vs_r_model_matrix() {
    // Categorical with contrasts
    // Interactions and polynomials
    // Random effects
    // Splines
}

// Test vs Python Formulaic
#[test]
fn test_vs_formulaic() {
    // Feature parity comparison
    // Performance benchmarks
}
```

## 📊 Feature Parity Table

| Feature | Status | R | Python Formulaic | Notes |
|---------|--------|---|------------------|-------|
| Basic formulas | ✅ | ✅ | ✅ | `y ~ x1 + x2` |
| Interactions | 🟡 | ✅ | ✅ | Parsing only |
| Polynomials | 🟡 | ✅ | ✅ | Basic implementation |
| Categoricals | ❌ | ✅ | ✅ | Critical gap |
| Random effects | 🟡 | ✅ | ✅ | Dense only |
| Splines | ❌ | ✅ | ✅ | Not implemented |
| Multi-response | 🟡 | ✅ | ✅ | Basic only |
| Survival | ❌ | ✅ | ✅ | Not implemented |
| Distributional | ❌ | ✅ | ✅ | Not implemented |

## 🚀 Quick Wins

1. **Fix `poly()` function** - Return actual polynomial columns instead of original series
2. **Implement `:` interactions** - Proper column multiplication
3. **Add categorical contrasts** - Treatment, sum, Helmert contrasts
4. **Sparse random effects** - Replace dense one-hot with sparse matrices
5. **LazyFrame integration** - Formula → `Vec<Expr>` conversion

These changes will immediately improve the library's usefulness and bring it closer to production readiness.
