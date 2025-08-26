# Implementation Plan: Quick Wins

## 1. Fix `poly()` Function

### Current Problem
```rust
// In materialize.rs line ~491
// TODO: Implement proper polynomial expansion
Ok(var_series)  // Just returns original series!
```

### Target Implementation
```rust
fn materialize_poly(df: &DataFrame, args: &[Expr]) -> Result<Series, Error> {
    // Parse arguments: poly(var, degree, raw?, normalize?)
    let var_name = extract_var_name(&args[0])?;
    let degree = extract_degree(&args[1])?;
    let raw = args.get(2).and_then(|arg| extract_bool(arg)).unwrap_or(false);
    let normalize = args.get(3).and_then(|arg| extract_bool(arg)).unwrap_or(true);
    
    let var_series = df.column(var_name)?.as_series()?.clone();
    
    if raw {
        // Raw polynomials: [x, x², x³, ...]
        let mut poly_series = Vec::new();
        for d in 1..=degree {
            let powered = var_series.pow(d as f64)?;
            poly_series.push(powered);
        }
        // Return as multiple columns or concatenated
        Ok(concat_series(poly_series)?)
    } else {
        // Orthogonal polynomials (numerically stable)
        let orthogonal = compute_orthogonal_polynomials(&var_series, degree, normalize)?;
        Ok(orthogonal)
    }
}
```

### Files to Modify
1. `src/dsl/materialize.rs` - Replace the TODO with actual implementation
2. `src/dsl/ast.rs` - Add polynomial options to AST if needed
3. `tests/poly_test.rs` - Add comprehensive tests

### Test Cases
```rust
#[test]
fn test_poly_raw() {
    // poly(x, 2, raw=true) → [x, x²]
}

#[test]
fn test_poly_orthogonal() {
    // poly(x, 2, raw=false) → [orthogonal_1, orthogonal_2]
}

#[test]
fn test_poly_normalize() {
    // poly(x, 2, raw=false, normalize=true) → normalized orthogonal
}
```

## 2. Fix Interactions (`:`)

### Current Problem
```rust
// In materialize.rs line ~284
// TODO: Implement proper interaction materialization
if let Some(first_term) = terms.first() {
    materialize_expr(df, first_term)  // Just returns first term!
}
```

### Target Implementation
```rust
fn materialize_interaction(df: &DataFrame, terms: &[Expr]) -> Result<Series, Error> {
    let mut result = None;
    
    for term in terms {
        let term_series = materialize_expr(df, term)?;
        
        if let Some(ref current) = result {
            // Element-wise multiplication
            result = Some(current.mul(&term_series)?);
        } else {
            result = Some(term_series);
        }
    }
    
    result.ok_or_else(|| Error::Semantic("Empty interaction".into()))
}
```

### Test Cases
```rust
#[test]
fn test_binary_interaction() {
    // x1:x2 → x1 * x2
}

#[test]
fn test_multiway_interaction() {
    // x1:x2:x3 → x1 * x2 * x3
}
```

## 3. Add Categorical Contrasts

### New Types to Add
```rust
// In src/lib.rs
#[derive(Debug, Clone)]
pub enum Contrast {
    Treatment { baseline: Option<String> },
    Sum,
    Helmert,
    Polynomial,
}

// Extend MaterializeOptions
impl MaterializeOptions {
    pub fn with_contrast(mut self, var: &str, contrast: Contrast) -> Self {
        self.contrasts.insert(var.to_string(), contrast);
        self
    }
    
    pub fn with_contrast_default(mut self, contrast: Contrast) -> Self {
        self.default_contrast = Some(contrast);
        self
    }
}
```

### Implementation
```rust
fn materialize_categorical(
    df: &DataFrame, 
    var_name: &str, 
    contrast: &Contrast
) -> Result<Vec<(String, Series)>, Error> {
    let series = df.column(var_name)?.as_series()?.clone();
    let levels = get_unique_levels(&series)?;
    
    match contrast {
        Contrast::Treatment { baseline } => {
            let baseline = baseline.as_deref().unwrap_or(&levels[0]);
            create_treatment_contrasts(&series, &levels, baseline)
        }
        Contrast::Sum => create_sum_contrasts(&series, &levels),
        Contrast::Helmert => create_helmert_contrasts(&series, &levels),
        Contrast::Polynomial => create_polynomial_contrasts(&series, &levels),
    }
}
```

## 4. Sparse Random Effects

### Current Problem
```rust
// In materialize.rs around line ~691
// TODO: Implement more complex random effects
// Currently creates dense one-hot matrices
```

### Target Implementation
```rust
#[cfg(feature = "faer")]
fn materialize_sparse_random_effects(
    df: &DataFrame,
    group_var: &str,
    inner_expr: &Expr,
) -> Result<faer_sparse::CsMat<f64>, Error> {
    // Build sparse matrix directly instead of dense one-hot
    let n_rows = df.height();
    let group_levels = get_unique_levels(df.column(group_var)?)?;
    let n_groups = group_levels.len();
    
    // Determine Z matrix dimensions based on inner expression
    let (n_cols, indices, values) = build_sparse_indices(
        df, group_var, inner_expr, &group_levels
    )?;
    
    // Create sparse matrix
    let z_sparse = faer_sparse::CsMat::new(
        (n_rows, n_cols),
        indices,
        values,
    );
    
    Ok(z_sparse)
}
```

## 5. LazyFrame Integration

### New API
```rust
// In src/lib.rs
impl ModelSpec {
    pub fn to_exprs(&self) -> Result<Vec<Expr>, Error> {
        // Convert formula to Polars expressions
        // This allows integration with LazyFrame
        self.formula.to_polars_exprs()
    }
}

// Usage:
let terms = spec.to_exprs()?;
let lazy_df = df.lazy()
    .with_columns(terms)
    .collect()?;
```

## Implementation Order

1. **Week 1**: Fix `poly()` and interactions
   - Implement polynomial expansion (raw and orthogonal)
   - Implement proper interaction materialization
   - Add comprehensive tests

2. **Week 2**: Add categorical contrasts
   - Implement contrast types (Treatment, Sum, Helmert)
   - Add to MaterializeOptions API
   - Add tests with real categorical data

3. **Week 3**: Sparse random effects
   - Implement sparse matrix construction
   - Add faer-sparse feature integration
   - Performance benchmarks

4. **Week 4**: LazyFrame integration
   - Formula → Vec<Expr> conversion
   - Integration tests with LazyFrame
   - Documentation updates

## Testing Strategy

### Golden Tests
```rust
// tests/golden_tests.rs
#[test]
fn test_vs_r_poly() {
    // Compare poly(x, 2) output with R's poly()
}

#[test]
fn test_vs_r_contrasts() {
    // Compare categorical contrasts with R's model.matrix()
}

#[test]
fn test_vs_formulaic() {
    // Compare with Python Formulaic output
}
```

### Performance Tests
```rust
// benches/materialization.rs
#[bench]
fn bench_poly_materialization(b: &mut Bencher) {
    // Benchmark polynomial materialization
}

#[bench]
fn bench_categorical_contrasts(b: &mut Bencher) {
    // Benchmark categorical processing
}
```

## Success Metrics

- [ ] `poly(x, 2)` returns 2 columns (x, x²)
- [ ] `x1:x2` returns 1 column (x1 * x2)
- [ ] `species` (categorical) returns 2 columns with treatment contrasts
- [ ] `(1|group)` returns sparse Z matrix
- [ ] `formula.to_exprs()` returns Vec<Expr> for LazyFrame
- [ ] All tests pass
- [ ] Performance within 2x of R/Python equivalents
