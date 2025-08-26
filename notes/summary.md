# polars-formula: Feedback Summary & Action Plan

## 📋 Executive Summary

Your feedback on polars-formula was incredibly thorough and insightful. The library has **strong foundations** with a rich AST and clean architecture, but there's a significant gap between what the **parser promises** and what the **materializer delivers**. The core issue is that many advanced features are parsed but not properly materialized.

## 🎯 Key Strengths (Keep Building On These)

1. **Ambitious, useful scope** - Targeting Patsy/Formulaic/R-style DSL with clear `(y, X, Z)` output
2. **Solid foundations** - Chumsky parsing, rich AST, clean API design
3. **Good design choices** - Three DataFrame return format, optional `faer` feature
4. **Comprehensive DSL** - AST supports families, smooths, distributional params, groups

## 🚨 Critical Gaps (Address First)

### 1. **Materialization vs Parsing Mismatch**
- **Problem**: Parser supports advanced syntax but materializer has explicit TODOs
- **Impact**: Users get confused when `poly(x, 2)` parses but returns just `x`
- **Solution**: Implement materialization for parsed features

### 2. **Core Transforms Not Finished**
- `poly()` returns original series instead of polynomial columns
- Interactions (`:`) and products (`*`) are stubbed with TODOs
- Categorical variables not handled (no contrasts)

### 3. **Random Effects Inefficient**
- Dense one-hot encoding (memory heavy, slow)
- Needs sparse structures for scalability

### 4. **Documentation Gap**
- ~21% documented in rustdoc
- README promises more than code delivers

## 📊 Implementation Priority

### **Phase 1: Quick Wins (Weeks 1-2)**
1. **Fix `poly()` function** - Return actual polynomial columns
2. **Implement `:` interactions** - Proper column multiplication  
3. **Add categorical contrasts** - Treatment, sum, Helmert
4. **Sparse random effects** - Replace dense one-hot

### **Phase 2: Advanced Features (Weeks 3-4)**
1. **Spline bases** - `s()`, `ns()`, `bs()` functions
2. **Multi-response handling** - Proper `cbind()` support
3. **LazyFrame integration** - `formula.to_exprs()`
4. **Distributional parameters** - `y ~ x + sigma ~ z`

### **Phase 3: Polish & Testing (Weeks 5-6)**
1. **Error diagnostics** - Better error messages
2. **Documentation** - Expand to >80% coverage
3. **Golden tests** - vs R and Python implementations
4. **Performance benchmarks** - Optimization

## 🧪 Current State Verification

The `quick_wins_demo.rs` example confirms the issues:

```rust
// Current broken behaviors:
poly(x1, 2)     → 1 column (should be 2: x1, x1²)
x1:x2           → 1 column (should be 1: x1 * x2)  
species         → 1 column (should be 2: treatment contrasts)
(1|species)     → 3 dense columns (should be 2 sparse)
```

## 🎯 Success Metrics

- [ ] `poly(x, 2)` returns 2 columns (x, x²)
- [ ] `x1:x2` returns 1 column (x1 * x2)
- [ ] `species` (categorical) returns 2 columns with treatment contrasts
- [ ] `(1|group)` returns sparse Z matrix
- [ ] `formula.to_exprs()` returns Vec<Expr> for LazyFrame
- [ ] All tests pass
- [ ] Performance within 2x of R/Python equivalents

## 📁 Files Created

1. **`notes/roadmap.md`** - Comprehensive implementation roadmap
2. **`notes/implementation_plan.md`** - Detailed technical implementation plan
3. **`notes/feature_parity.md`** - Feature parity table for README
4. **`examples/quick_wins_demo.rs`** - Demonstrates current issues
5. **`tests/quick_wins_test.rs`** - Test template for fixes

## 🚀 Next Steps

1. **Start with `poly()` function** - Highest impact, relatively simple
2. **Add categorical contrasts** - Core statistical functionality
3. **Implement interactions** - Fix `:` and `*` operators
4. **Add sparse random effects** - Performance improvement
5. **Update documentation** - Reflect actual capabilities

## 💡 Recommendations

1. **Be explicit about parsing vs materialization** in docs
2. **Add feature parity table** to README to set expectations
3. **Implement golden tests** vs R/Python for validation
4. **Consider LazyFrame integration** as a differentiator
5. **Focus on core statistical features** before advanced ones

## 🔗 Resources

- **Roadmap**: `notes/roadmap.md`
- **Implementation Plan**: `notes/implementation_plan.md`  
- **Feature Parity**: `notes/feature_parity.md`
- **Demo**: `examples/quick_wins_demo.rs`
- **Tests**: `tests/quick_wins_test.rs`

The library has excellent potential - with these fixes, it could become the go-to formula parsing library for the Rust/Polars ecosystem!
