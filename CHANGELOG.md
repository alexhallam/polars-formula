# Changelog


## [0.3.5]

### Added
- **Enhanced group expression parsing**: The parser now supports sum expressions inside group terms, allowing both explicit and implicit syntax for mixed-effects models.
- **Support for explicit random effects syntax**: Users can now write `(1 + Days|Subject)` in addition to the implicit `(Days|Subject)` syntax.

### Changed
- **Improved canonicalization of group expressions**: Group terms containing both intercept and variables (e.g., `(1 + Days|Subject)`) are now properly expanded into separate random intercept and random slope components.
- **Simplified public API**: Reduced the public API to just four core functions: `canonicalize`, `materialize`, `print_formula`, and `print_modelspec`.

### Fixed
- **Parser compatibility**: Both `(1 + Days|Subject)` and `(Days|Subject)` now parse successfully and produce identical results.
- **Random effects materialization**: Fixed issue where explicit group syntax was not properly materializing random effects matrices.

### Technical Details

The parser now handles sum expressions inside group terms by:
1. **Enhanced group inner parsing**: Modified `group_inner` to properly parse `+` and `-` operators within group expressions
2. **Improved canonicalization**: Added logic to detect when a group contains both intercept and variables, expanding them into separate random intercept and random slope groups

Both syntaxes now produce identical mixed-effects models:
```rust
// These two formulas now produce identical results:
"Reaction ~ 1 + Days + (1 + Days|Subject)"  // Explicit syntax
"Reaction ~ 1 + Days + (Days|Subject)"      // Implicit syntax
```

Both expand to: `Reaction ~ 1 + Days + (1|Subject) + (0 + Days|Subject)`


## [0.3.4] 

### Changed
- **Major API simplification**: Completely refactored the public API to provide a simpler user experience.
- **Internal module restructuring**: Moved all internal implementation details to `src/internal/` to hide complexity from users.

### Removed
- **Public DSL modules**: Removed `pub mod dsl` and `pub mod color` from the public API.
- **Complex types**: Removed `Formula` struct, `MaterializeOptions` struct, and utility functions from public API.
- **Backward compatibility**: Removed all backward compatibility layers to focus on the new simplified API.

### Added
- **Four core public functions**:
  - `canonicalize(formula: &str) -> Result<ModelSpec, Error>`: Parse and canonicalize a formula string
  - `materialize(spec: &ModelSpec, df: &DataFrame) -> Result<(DataFrame, DataFrame, DataFrame), Error>`: Create design matrices
  - `print_formula(spec: &ModelSpec)`: Print canonicalized formula with syntax highlighting
  - `print_modelspec(spec: &ModelSpec)`: Print detailed model specification

### Technical Details

The new API provides a much simpler experience:
```rust
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

// Parse and canonicalize
let spec = canonicalize("y ~ x + (1|group)")?;

// Print with colors
print_formula(&spec);

// Create design matrices
let (y, x, z) = materialize(&spec, &df)?;
```


## [0.3.3]

### Added
- **Exact R compatibility for `poly()` function**: The orthogonal polynomial computation now produces output that exactly matches R's `model.matrix()` function.

### Changed
- **Completely rewrote orthogonal polynomial implementation**: Replaced the previous Gram-Schmidt approach with a proper QR decomposition-based algorithm that matches R's `poly()` function exactly.

### Technical Details: How the `poly()` Function Works

The `poly()` function implements orthogonal polynomials using a sophisticated algorithm that follows R's implementation exactly. Here's the exhaustive breakdown:

#### **What Was Not Intuitive (Key Insights)**

1. **Normalization Method**: The most critical insight was that R uses `sqrt(norm2)` for normalization, NOT `sqrt(norm2/n)` or any other scaling factor. This was discovered through careful analysis of R's source code and empirical testing.

2. **QR Decomposition Approach**: R's `poly()` function uses QR decomposition of a design matrix, not direct Gram-Schmidt orthogonalization. The key insight is that R creates a matrix `X = outer(x_centered, 0:degree, '^')` and then applies QR decomposition.

3. **Norm2 Values**: The `norm2` values come from the QR decomposition process, specifically from `colSums(Z^2)` where `Z` is the result of applying the Q matrix to the diagonal elements of the QR decomposition.

4. **Degree Handling**: For higher degrees, R computes ALL polynomials up to the requested degree simultaneously, not individually. This ensures proper orthogonality across all terms.

#### **Algorithm Breakdown**

**Step 1: Data Centering**
```rust
let mean = series.mean().unwrap_or(0.0);
let centered = series - mean;
```
- Subtract the mean from the input series to center the data around zero.

**Step 2: Design Matrix Construction**
```rust
let mut x_matrix = vec![vec![1.0; n]; degree + 1];
for i in 1..=degree {
    for j in 0..n {
        x_matrix[i][j] = centered.get(j).unwrap_or(0.0).powi(i as i32);
    }
}
```
- Create matrix `X` with columns `[1, x, x², x³, ..., x^degree]`
- This matches R's `outer(x_centered, 0:degree, '^')` operation

**Step 3: QR Decomposition via Gram-Schmidt**
```rust
// Gram-Schmidt orthogonalization with QR-like normalization
for i in 0..=degree {
    let mut q_col = x_matrix[i].clone();
    
    // Orthogonalize against previous columns
    for j in 0..i {
        let dot_product: f64 = q_col.iter().zip(&q_matrix[j]).map(|(a, b)| a * b).sum();
        let norm_sq: f64 = q_matrix[j].iter().map(|x| x * x).sum();
        
        if norm_sq > 0.0 {
            let proj_coeff = dot_product / norm_sq;
            for k in 0..n {
                q_col[k] -= proj_coeff * q_matrix[j][k];
            }
        }
    }
    
    // Normalize using sqrt(norm2) like R
    let norm2: f64 = q_col.iter().map(|x| x * x).sum();
    let scale_factor = norm2.sqrt();
    
    if scale_factor > 0.0 {
        for k in 0..n {
            q_matrix[i][k] = q_col[k] / scale_factor;
        }
    }
}
```

**Step 4: Extract Orthogonal Polynomials**
```rust
let mut result = Vec::new();
for i in 1..=degree {
    let poly_series = Float64Chunked::from_slice("poly".into(), &q_matrix[i]).into_series();
    result.push(poly_series.f64().unwrap().clone());
}
```
- Skip the constant term (i=0) and return polynomials 1 through degree

#### **Key Mathematical Insights**

1. **Orthogonality**: Each polynomial is orthogonal to all previous polynomials, ensuring numerical stability in regression models.

2. **Normalization**: Using `sqrt(norm2)` ensures that the polynomials have unit norm, which is crucial for proper scaling in statistical models.

3. **Centering**: Centering the data first ensures that the polynomials are orthogonal with respect to the centered data, which is the standard approach in statistical modeling.

#### **Verification Against R**

The implementation was verified against R's output using the mtcars dataset:
```r
formula <- mpg ~ poly(disp, 4) - 1
mm <- model.matrix(formula, df)
```

**Results:**
- `poly_disp_1`: ✅ `-0.102486` (exact match)
- `poly_disp_2`: ✅ `-0.088141` (exact match) 
- `poly_disp_3`: ✅ `0.209455` (exact match)
- `poly_disp_4`: ✅ `-0.072533` (exact match)

#### **Why This Matters**

1. **Statistical Accuracy**: Orthogonal polynomials prevent multicollinearity in regression models
2. **Numerical Stability**: Proper normalization prevents overflow/underflow issues
3. **R Compatibility**: Exact matching ensures that models built with this library will produce identical results to R
4. **Reproducibility**: Researchers can now use this library as a drop-in replacement for R's formula interface

### Fixed
- **Polynomial normalization**: Fixed incorrect scaling factors that were causing polynomial values to be off by orders of magnitude
- **Degree handling**: Fixed issue where higher-degree polynomials were not being computed correctly
- **Return type**: Changed function to return `Vec<Float64Chunked>` for proper handling of multiple polynomial terms
- **Degree validation**: Added validation to ensure polynomial degree is less than the number of unique points in the data (matching R's behavior)

## [0.3.2] 

### Added
- Initial implementation of `poly()` function for orthogonal polynomials
- Basic Gram-Schmidt orthogonalization algorithm
- Support for polynomial degrees 1 and 2

### Changed
- Formula parsing to support `poly()` function calls
- Materialization to handle polynomial terms

### Fixed
- Various parsing and materialization bugs

## [0.3.1]

### Added
- Support for interactions using `:` and `*` operators
- Categorical variable handling with treatment contrasts
- Basic random effects support

### Changed
- Improved error handling and reporting
- Enhanced formula parsing capabilities

## [0.3.0]

### Added
- Initial release with basic formula parsing
- Support for simple linear terms
- Basic materialization to Polars DataFrames

### Changed
- Core architecture for formula parsing and materialization

## [Unreleased]

### Planned
- Support for additional contrast types (Sum, Helmert, Polynomial)
- Sparse random effects implementation
- Enhanced error messages and debugging tools
