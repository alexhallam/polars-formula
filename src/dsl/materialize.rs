use super::ast::*;
use crate::{Error, MaterializeOptions};
use polars::prelude::*;

/// Materialize a DSL ModelSpec against a DataFrame to produce design matrices.
///
/// This function takes a DSL ModelSpec and materializes it into concrete
/// numeric matrices suitable for statistical modeling.
///
/// # Arguments
///
/// * `df` - The DataFrame containing the data to materialize against
/// * `spec` - The DSL ModelSpec to materialize
/// * `opts` - Options controlling materialization behavior
///
/// # Returns
///
/// Returns `(DataFrame, DataFrame, DataFrame)` where:
/// - `DataFrame`: The response variable(s) (left side of `~`)
/// - `DataFrame`: The fixed effects design matrix (X)
/// - `DataFrame`: The random effects design matrix (Z)
///
/// # Examples
///
/// ```rust
/// use polars::prelude::*;
/// use polars_formula::dsl::{parser::parser, materialize, MaterializeOptions};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let df = df!(
///     "y" => [1.0, 2.0, 3.0, 4.0],
///     "x1" => [1.0, 2.0, 3.0, 4.0],
///     "x2" => [2.0, 3.0, 4.0, 5.0]
/// )?;
///
/// let p = parser();
/// let spec = p.parse("y ~ x1 + x2")?;
/// let (y, X, Z) = materialize(&df, &spec, MaterializeOptions::default())?;
///
/// println!("Response: {:?}", y);
/// println!("Fixed effects: {:?}", X);
/// println!("Random effects: {:?}", Z);
/// # Ok(())
/// # }
/// ```
pub fn materialize(
    df: &DataFrame,
    spec: &ModelSpec,
    opts: MaterializeOptions,
) -> Result<(DataFrame, DataFrame, DataFrame), Error> {
    // Materialize the main formula
    let (y, x, z) = materialize_formula(df, &spec.formula, opts)?;

    // TODO: Handle distributional parameters (dpars)
    // TODO: Handle autocorrelation terms (autocor)
    // TODO: Handle family/link specifications

    Ok((y, x, z))
}

/// Check if an expression contains a -1 term (intercept removal)
fn has_intercept_removal(expr: &Expr) -> bool {
    match expr {
        Expr::Sum(terms) => terms.iter().any(|term| {
            matches!(term, Expr::Func { name, args } if name == "NEG" && args.len() == 1 && matches!(&args[0], Expr::Num(1.0)))
        }),
        Expr::Func { name, args } if name == "NEG" && args.len() == 1 => {
            matches!(&args[0], Expr::Num(1.0))
        }
        _ => false,
    }
}

/// Materialize a DSL Formula against a DataFrame.
fn materialize_formula(
    df: &DataFrame,
    formula: &Formula,
    mut opts: MaterializeOptions,
) -> Result<(DataFrame, DataFrame, DataFrame), Error> {
    // Check if formula has -1 term (intercept removal)
    if has_intercept_removal(&formula.rhs) {
        opts.rhs_intercept = false;
    }

    // Materialize LHS (response)
    let y = materialize_response(df, &formula.lhs)?;

    // Materialize RHS (predictors) - separate fixed and random effects
    let (x, z) = materialize_rhs_with_random(df, &formula.rhs, opts)?;

    // TODO: Handle aterms (weights, se, etc.)

    Ok((y, x, z))
}

/// Materialize a response expression.
fn materialize_response(df: &DataFrame, response: &Response) -> Result<DataFrame, Error> {
    match response {
        Response::Var(name) => {
            let series = df
                .column(name)
                .map_err(|_| Error::Semantic(format!("Column '{}' not found in DataFrame", name)))
                .and_then(|s| {
                    s.as_series()
                        .ok_or_else(|| Error::Semantic("Failed to convert column to series".into()))
                })?;

            // Convert Series to DataFrame
            Ok(series.clone().into_frame())
        }
        Response::BinomialTrials {
            successes,
            trials: _,
        } => {
            // For binomial trials, we return the successes series
            // The trials series is available for validation and downstream processing
            if let Expr::Var(successes_name) = successes {
                let series = df
                    .column(successes_name)
                    .map_err(|_| {
                        Error::Semantic(format!(
                            "Column '{}' not found in DataFrame",
                            successes_name
                        ))
                    })
                    .and_then(|s| {
                        s.as_series().ok_or_else(|| {
                            Error::Semantic("Failed to convert column to series".into())
                        })
                    })?;

                // Convert Series to DataFrame
                Ok(series.clone().into_frame())
            } else {
                Err(Error::Semantic(
                    "Binomial successes must be a variable name".into(),
                ))
            }
        }
        Response::Multi(names) => {
            // For now, just take the first variable
            // TODO: Implement proper multi-response handling
            if let Some(name) = names.first() {
                let series = df
                    .column(name)
                    .map_err(|_| {
                        Error::Semantic(format!("Column '{}' not found in DataFrame", name))
                    })
                    .and_then(|s| {
                        s.as_series().ok_or_else(|| {
                            Error::Semantic("Failed to convert column to series".into())
                        })
                    })?;

                // Convert Series to DataFrame
                Ok(series.clone().into_frame())
            } else {
                Err(Error::Semantic("Empty multi-response specification".into()))
            }
        }
        Response::Surv {
            time,
            event: _event,
            time2: _,
        } => {
            // For survival analysis, we'll use the time variable as response for now
            // TODO: Implement proper survival response handling
            let series = materialize_expr(df, time)?;
            // Convert Series to DataFrame
            Ok(series.clone().into_frame())
        }
        Response::Func { name: _name, args } => {
            // For now, treat function responses as the first argument
            // TODO: Implement proper function response handling
            if let Some(first_arg) = args.first() {
                let series = materialize_expr(df, first_arg)?;
                // Convert Series to DataFrame
                Ok(series.clone().into_frame())
            } else {
                Err(Error::Semantic(
                    "Function response with no arguments".into(),
                ))
            }
        }
    }
}

/// Materialize the RHS expression into fixed and random effects design matrices.
fn materialize_rhs_with_random(
    df: &DataFrame,
    rhs: &Expr,
    opts: MaterializeOptions,
) -> Result<(DataFrame, DataFrame), Error> {
    let mut fixed_cols = Vec::new();
    let mut random_cols = Vec::new();

    // Add intercept if requested
    if opts.rhs_intercept {
        let n = df.height();
        let ones =
            Float64Chunked::from_slice(opts.intercept_name.into(), &vec![1.0; n]).into_series();
        fixed_cols.push((opts.intercept_name.to_string(), ones));
    }

    // Materialize the RHS expression, separating fixed and random effects
    let (fixed_rhs_cols, random_rhs_cols) = materialize_expr_to_columns_with_random(df, rhs)?;
    fixed_cols.extend(fixed_rhs_cols);
    random_cols.extend(random_rhs_cols);

    // Build fixed effects DataFrame
    let fixed_df = build_dataframe_from_cols(fixed_cols, &opts)?;

    // Build random effects DataFrame
    let random_df = build_dataframe_from_cols(random_cols, &opts)?;

    Ok((fixed_df, random_df))
}

/// Materialize the RHS expression into a design matrix (for backward compatibility).
#[allow(dead_code)]
fn materialize_rhs(
    df: &DataFrame,
    rhs: &Expr,
    opts: MaterializeOptions,
) -> Result<DataFrame, Error> {
    let (fixed_df, _random_df) = materialize_rhs_with_random(df, rhs, opts)?;
    Ok(fixed_df)
}

/// Build a DataFrame from a list of columns with proper naming.
fn build_dataframe_from_cols(
    cols: Vec<(String, Series)>,
    opts: &MaterializeOptions,
) -> Result<DataFrame, Error> {
    if cols.is_empty() {
        // Return empty DataFrame with correct number of rows
        return Ok(DataFrame::new(vec![]).map_err(|e| Error::Semantic(e.to_string()))?);
    }

    let (names, series): (Vec<_>, Vec<_>) = cols.into_iter().unzip();
    let mut unique_series = Vec::new();
    let mut name_counts = std::collections::HashMap::new();

    for (name, s) in names.into_iter().zip(series.into_iter()) {
        let count = name_counts.entry(name.clone()).or_insert(0);
        *count += 1;
        let unique_name = if *count > 1 {
            format!("{}_{}", name, *count - 1)
        } else {
            name
        };

        // Apply name cleaning if requested
        let final_name = if opts.clean_names {
            crate::make_clean_names(&unique_name)
        } else {
            unique_name
        };

        let mut new_series = s.clone();
        new_series.rename(final_name.into());
        unique_series.push(new_series.into());
    }

    DataFrame::new(unique_series).map_err(|e| Error::Semantic(e.to_string()))
}

/// Materialize an expression to a single Series.
fn materialize_expr(df: &DataFrame, expr: &Expr) -> Result<Series, Error> {
    match expr {
        Expr::Var(name) => {
            // Check if this is a categorical variable
            let series = df
                .column(name)
                .map_err(|_| Error::Semantic(format!("Column '{}' not found in DataFrame", name)))?
                .as_series()
                .ok_or_else(|| Error::Semantic("Failed to convert column to series".into()))?
                .clone();

            // Check if this is a categorical variable (string type)
            if let Ok(str_series) = series.str() {
                // Convert categorical variable to treatment contrasts
                materialize_categorical_to_contrasts(str_series, name)
            } else {
                // Numeric variable, return as-is
                Ok(series)
            }
        }
        Expr::Num(n) => {
            let n_rows = df.height();
            Ok(Float64Chunked::from_slice("constant".into(), &vec![*n; n_rows]).into_series())
        }
        Expr::Intercept(true) => {
            let n_rows = df.height();
            Ok(Float64Chunked::from_slice("intercept".into(), &vec![1.0; n_rows]).into_series())
        }
        Expr::Intercept(false) => {
            let n_rows = df.height();
            Ok(Float64Chunked::from_slice("zero".into(), &vec![0.0; n_rows]).into_series())
        }
        Expr::Sum(terms) => {
            // For sums, we'll materialize each term and add them
            // TODO: Implement proper sum materialization
            if let Some(first_term) = terms.first() {
                materialize_expr(df, first_term)
            } else {
                Err(Error::Semantic("Empty sum expression".into()))
            }
        }
        Expr::Interaction(terms) => {
            // For interactions, multiply the terms
            if terms.is_empty() {
                return Err(Error::Semantic("Empty interaction expression".into()));
            }

            let mut result: Option<Series> = None;

            for term in terms {
                let term_series = materialize_expr(df, term)?;

                if let Some(ref current) = result {
                    // Element-wise multiplication
                    result = Some((&*current * &term_series).map_err(|e| {
                        Error::Semantic(format!("Failed to multiply interaction terms: {}", e))
                    })?);
                } else {
                    result = Some(term_series);
                }
            }

            result.ok_or_else(|| Error::Semantic("Failed to materialize interaction".into()))
        }
        Expr::Func { name, args } => {
            // Handle special functions
            match name.as_str() {
                "poly" => {
                    // For poly() in materialize_expr, just return the first polynomial term
                    // The full expansion is handled in materialize_expr_to_columns_with_random
                    let poly_cols = materialize_poly_to_columns(df, args)?;
                    if let Some((_, first_series)) = poly_cols.first() {
                        Ok(first_series.clone())
                    } else {
                        Err(Error::Semantic("poly() returned no columns".into()))
                    }
                }
                "NEG" => {
                    // Negation function - negate the inner expression
                    if let Some(inner) = args.first() {
                        let inner_series = materialize_expr(df, inner)?;
                        // For now, just return the inner series
                        // TODO: Implement proper negation
                        Ok(inner_series)
                    } else {
                        Err(Error::Semantic("NEG function with no arguments".into()))
                    }
                }
                "I" => {
                    // Identity function - materialize the inner expression
                    if let Some(inner) = args.first() {
                        materialize_expr(df, inner)
                    } else {
                        Err(Error::Semantic(
                            "Identity function with no arguments".into(),
                        ))
                    }
                }
                _ => {
                    // For unknown functions, try to materialize the first argument
                    // TODO: Implement proper function handling
                    if let Some(first_arg) = args.first() {
                        materialize_expr(df, first_arg)
                    } else {
                        Err(Error::Semantic(format!(
                            "Unknown function '{}' with no arguments",
                            name
                        )))
                    }
                }
            }
        }
        Expr::Smooth {
            kind: _kind,
            vars,
            args: _,
        } => {
            // For smooths, we'll use the first variable for now
            // TODO: Implement proper smooth materialization
            if let Some(first_var) = vars.first() {
                df.column(first_var)
                    .map_err(|_| {
                        Error::Semantic(format!("Column '{}' not found in DataFrame", first_var))
                    })
                    .and_then(|s| {
                        s.as_series().ok_or_else(|| {
                            Error::Semantic("Failed to convert column to series".into())
                        })
                    })
                    .map(|s| s.clone())
            } else {
                Err(Error::Semantic("Smooth with no variables".into()))
            }
        }
        Expr::Group {
            inner,
            spec: _,
            kind: _,
            id: _,
        } => {
            // For groups, materialize the inner expression
            // TODO: Implement proper group materialization
            materialize_expr(df, inner)
        }
        _ => {
            // For other expressions, return an error for now
            // TODO: Implement remaining expression types
            Err(Error::Semantic(format!(
                "Unsupported expression type: {:?}",
                expr
            )))
        }
    }
}

/// Materialize an expression to multiple columns, separating fixed and random effects.
fn materialize_expr_to_columns_with_random(
    df: &DataFrame,
    expr: &Expr,
) -> Result<(Vec<(String, Series)>, Vec<(String, Series)>), Error> {
    match expr {
        Expr::Sum(terms) => {
            let mut fixed_cols = Vec::new();
            let mut random_cols = Vec::new();
            for term in terms {
                let (term_fixed, term_random) = materialize_expr_to_columns_with_random(df, term)?;
                fixed_cols.extend(term_fixed);
                random_cols.extend(term_random);
            }
            Ok((fixed_cols, random_cols))
        }
        Expr::Group {
            inner,
            spec,
            kind,
            id,
        } => {
            // Handle random effects
            let random_cols = materialize_group_to_columns(df, inner, spec, kind, id)?;
            Ok((Vec::new(), random_cols))
        }
        Expr::Interaction(terms) => {
            // For interactions, handle categorical variables properly
            if terms.is_empty() {
                return Ok((Vec::new(), Vec::new()));
            }

            // Materialize each term to get columns (may be multiple for categorical)
            let mut term_columns = Vec::new();
            for term in terms {
                let (fixed_cols, _) = materialize_expr_to_columns_with_random(df, term)?;
                term_columns.push(fixed_cols);
            }

            // Create interactions between all combinations
            let mut interaction_cols = Vec::new();

            if term_columns.len() == 2 {
                // Binary interaction - most common case
                let (cols1, cols2) = (&term_columns[0], &term_columns[1]);

                for (name1, series1) in cols1 {
                    for (name2, series2) in cols2 {
                        let interaction_name = format!("{}_x_{}", name1, name2);
                        let interaction_series = (&series1 * &series2).map_err(|e| {
                            Error::Semantic(format!("Failed to multiply interaction terms: {}", e))
                        })?;
                        interaction_cols.push((interaction_name, interaction_series));
                    }
                }
            } else {
                // Multi-way interaction - for now, just multiply the first columns
                let mut result_series = None;
                let mut interaction_name = String::new();

                for (i, cols) in term_columns.iter().enumerate() {
                    if let Some((name, series)) = cols.first() {
                        if result_series.is_none() {
                            result_series = Some(series.clone());
                            interaction_name = name.clone();
                        } else {
                            let current = result_series.as_mut().unwrap();
                            *current = (&*current * series).map_err(|e| {
                                Error::Semantic(format!(
                                    "Failed to multiply interaction terms: {}",
                                    e
                                ))
                            })?;
                            interaction_name = format!("{}_x_{}", interaction_name, name);
                        }
                    }
                }

                if let Some(series) = result_series {
                    interaction_cols.push((interaction_name, series));
                }
            }

            Ok((interaction_cols, Vec::new()))
        }
        Expr::Prod(terms) => {
            // For products, expand into main effects and interactions
            // TODO: Implement proper product expansion
            let mut fixed_cols = Vec::new();
            let mut random_cols = Vec::new();
            for term in terms {
                let (term_fixed, term_random) = materialize_expr_to_columns_with_random(df, term)?;
                fixed_cols.extend(term_fixed);
                random_cols.extend(term_random);
            }
            Ok((fixed_cols, random_cols))
        }
        Expr::Func { name, args } if name == "NEG" => {
            // Handle negation - for -1, this should remove intercept, not create a column
            if args.len() == 1 {
                if let Expr::Num(1.0) = &args[0] {
                    // This is -1, which should remove intercept, not create a column
                    Ok((Vec::new(), Vec::new()))
                } else {
                    // This is -something_else, materialize the inner expression
                    let (fixed_cols, random_cols) =
                        materialize_expr_to_columns_with_random(df, &args[0])?;
                    Ok((fixed_cols, random_cols))
                }
            } else {
                Err(Error::Semantic(
                    "NEG function with wrong number of arguments".into(),
                ))
            }
        }
        Expr::Func { name, args } if name == "poly" => {
            // Handle polynomial expansion - return multiple columns
            let poly_cols = materialize_poly_to_columns(df, args)?;
            Ok((poly_cols, Vec::new()))
        }
        Expr::Var(name) => {
            // Handle categorical variables
            let series = df
                .column(name)
                .map_err(|_| Error::Semantic(format!("Column '{}' not found in DataFrame", name)))?
                .as_series()
                .ok_or_else(|| Error::Semantic("Failed to convert column to series".into()))?
                .clone();

            // Check if this is a categorical variable (string type)
            if let Ok(str_series) = series.str() {
                // This is a categorical variable - create contrast columns
                let contrast_cols = create_categorical_contrasts(&str_series, name)?;
                Ok((contrast_cols, Vec::new()))
            } else {
                // This is a numeric variable - return as single column
                Ok((vec![(name.clone(), series)], Vec::new()))
            }
        }
        _ => {
            // For single expressions, materialize to one column (fixed effect)
            let series = materialize_expr(df, expr)?;
            let name = match expr {
                Expr::Var(name) => name.clone(),
                Expr::Num(n) => format!("constant_{}", n),
                Expr::Intercept(true) => "intercept".to_string(),
                Expr::Intercept(false) => "zero".to_string(),
                Expr::Func { name, .. } => name.clone(),
                Expr::Smooth { kind, vars, .. } => {
                    format!(
                        "{}_{}",
                        match kind {
                            SmoothKind::S => "s",
                            SmoothKind::T2 => "t2",
                            SmoothKind::TE => "te",
                            SmoothKind::TI => "ti",
                        },
                        vars.join("_")
                    )
                }
                _ => "expr".to_string(),
            };
            Ok((vec![(name, series)], Vec::new()))
        }
    }
}

/// Materialize an expression to multiple columns (for backward compatibility).
#[allow(dead_code)]
fn materialize_expr_to_columns(
    df: &DataFrame,
    expr: &Expr,
) -> Result<Vec<(String, Series)>, Error> {
    let (fixed_cols, random_cols) = materialize_expr_to_columns_with_random(df, expr)?;
    let mut all_cols = fixed_cols;
    all_cols.extend(random_cols);
    Ok(all_cols)
}

/// Materialize a polynomial function to multiple columns.
fn materialize_poly_to_columns(
    df: &DataFrame,
    args: &[Expr],
) -> Result<Vec<(String, Series)>, Error> {
    if args.len() != 2 {
        return Err(Error::Semantic(
            "poly() requires exactly 2 arguments".into(),
        ));
    }

    let var_expr = &args[0];
    let degree_expr = &args[1];

    // Get the variable name
    let var_name = match var_expr {
        Expr::Var(name) => name,
        _ => {
            return Err(Error::Semantic(
                "First argument to poly() must be a variable name".into(),
            ))
        }
    };

    // Get the degree
    let degree = match degree_expr {
        Expr::Num(n) => *n as usize,
        _ => {
            return Err(Error::Semantic(
                "Second argument to poly() must be a number".into(),
            ))
        }
    };

    // Extract optional arguments
    let raw = args
        .get(2)
        .and_then(|arg| {
            if let Expr::Bool(b) = arg {
                Some(*b)
            } else {
                None
            }
        })
        .unwrap_or(false);

    let _normalize = args
        .get(3)
        .and_then(|arg| {
            if let Expr::Bool(b) = arg {
                Some(*b)
            } else {
                None
            }
        })
        .unwrap_or(true);

    // Get the variable column
    let var_series = df
        .column(var_name)
        .map_err(|_| Error::Semantic(format!("Column '{}' not found in DataFrame", var_name)))?
        .as_series()
        .ok_or_else(|| Error::Semantic("Failed to convert column to series".into()))?
        .clone();

    // Convert to f64 for polynomial operations
    let f64_series = var_series.f64().map_err(|_| {
        Error::Semantic(format!(
            "Column '{}' cannot be converted to numeric for polynomial expansion",
            var_name
        ))
    })?;

    if raw {
        // Raw polynomials: [x, x², x³, ...]
        let mut poly_cols = Vec::new();

        for d in 1..=degree {
            let col_name = if d == 1 {
                var_name.clone()
            } else {
                format!("{}_{}", var_name, d)
            };

            // Compute the actual polynomial term using Polars power operations
            let poly_series = if d == 1 {
                f64_series.clone().into_series()
            } else {
                // Use Polars power operation: apply_values with pow
                let power_series = f64_series.apply_values(|x| x.powi(d as i32));
                power_series.into_series()
            };

            poly_cols.push((col_name, poly_series));
        }

        Ok(poly_cols)
    } else {
        // Orthogonal polynomials (numerically stable)
        // For degree > 1, return multiple columns
        let orthogonal_polys = compute_orthogonal_polynomials(&f64_series, degree)?;
        let mut poly_cols = Vec::new();
        for (i, poly) in orthogonal_polys.into_iter().enumerate() {
            let col_name = format!("poly_{}_{}", var_name, i + 1);
            poly_cols.push((col_name, poly.into_series()));
        }
        Ok(poly_cols)
    }
}

/// Compute orthogonal polynomials using QR decomposition like R's poly() function
fn compute_orthogonal_polynomials(
    series: &Float64Chunked,
    degree: usize,
) -> Result<Vec<Float64Chunked>, Error> {
    if degree == 0 {
        return Ok(Vec::new());
    }

    let n = series.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    // Check that degree is less than number of unique points (R's constraint)
    let unique_count = series
        .unique()
        .map_err(|e| Error::Semantic(format!("Failed to get unique values: {}", e)))?
        .len();

    if degree >= unique_count {
        return Err(Error::Semantic(format!(
            "'degree' must be less than number of unique points. Got degree={}, unique points={}",
            degree, unique_count
        )));
    }

    // Step 1: Center the data (subtract mean)
    let mean = series.mean().unwrap_or(0.0);
    let centered = series - mean;

    if degree == 1 {
        // For degree 1, return the centered and normalized series
        // R's poly() normalizes by sqrt(norm2) where norm2 is from QR decomposition
        // For degree 1, norm2 is the sum of squares of the centered values
        let norm2 = (&centered * &centered).sum().unwrap_or(0.0);
        let scale_factor = norm2.sqrt(); // Use sqrt(norm2) to match R's scaling
        if scale_factor > 0.0 {
            return Ok(vec![(&centered / scale_factor).into()]);
        } else {
            return Ok(vec![centered.into()]);
        }
    }

    // Step 2: Create the design matrix X with powers 0 to degree
    // X = [1, x, x^2, ..., x^degree]
    let mut x_matrix = vec![vec![1.0; n]; degree + 1];
    for i in 1..=degree {
        for j in 0..n {
            x_matrix[i][j] = centered.get(j).unwrap_or(0.0).powi(i as i32);
        }
    }

    // Step 3: Perform QR decomposition manually (simplified version)
    // This is a simplified QR decomposition that approximates R's behavior
    let mut q_matrix = vec![vec![0.0; n]; degree + 1];
    let mut r_matrix = vec![vec![0.0; degree + 1]; degree + 1];

    // Gram-Schmidt orthogonalization with QR-like normalization
    for i in 0..=degree {
        // Start with the i-th column of X
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
            r_matrix[i][i] = scale_factor;
        } else {
            q_matrix[i] = q_col;
            r_matrix[i][i] = 0.0;
        }
    }

    // Step 4: Extract the orthogonal polynomials (skip the constant term)
    let mut result = Vec::new();
    for i in 1..=degree {
        let poly_series = Float64Chunked::from_slice("poly".into(), &q_matrix[i]).into_series();
        result.push(poly_series.f64().unwrap().clone());
    }

    Ok(result)
}

/// Create contrast columns for categorical variables
fn create_categorical_contrasts(
    series: &StringChunked,
    var_name: &str,
) -> Result<Vec<(String, Series)>, Error> {
    // Get unique levels and sort them
    let mut levels: Vec<String> = series
        .into_iter()
        .filter_map(|s| s.map(|s| s.to_string()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    levels.sort();

    if levels.len() <= 1 {
        // Single level or empty - return empty
        return Ok(Vec::new());
    }

    // For now, implement treatment contrasts (default)
    // TODO: Support different contrast types from MaterializeOptions
    let mut contrast_cols = Vec::new();

    // Create treatment contrasts (skip first level as baseline)
    for level in &levels[1..] {
        let col_name = format!("{}_{}", var_name, level);
        let mut col_data = vec![0.0; series.len()];

        // Set to 1.0 for rows where series == level
        for (i, val) in series.into_iter().enumerate() {
            if let Some(val_str) = val {
                if val_str.to_string() == *level {
                    col_data[i] = 1.0;
                }
            }
        }

        let contrast_series =
            Float64Chunked::from_slice((&col_name).into(), &col_data).into_series();
        contrast_cols.push((col_name, contrast_series));
    }

    Ok(contrast_cols)
}

/// Materialize a group expression to random effects columns.
fn materialize_group_to_columns(
    df: &DataFrame,
    inner: &Expr,
    spec: &GroupSpec,
    _kind: &GroupKind,
    _id: &Option<String>,
) -> Result<Vec<(String, Series)>, Error> {
    let n_rows = df.height();

    match spec {
        GroupSpec::Expr(GroupExpr(terms)) => {
            // Get the first grouping variable (for now, handle simple cases)
            if let Some((group_var, _)) = terms.first() {
                // Simple grouping variable
                let group_series = df
                    .column(group_var)
                    .map_err(|_| {
                        Error::Semantic(format!("Group variable '{}' not found", group_var))
                    })?
                    .as_series()
                    .ok_or_else(|| {
                        Error::Semantic("Failed to convert group column to series".into())
                    })?
                    .clone();

                // Get unique group levels
                let unique_groups = group_series
                    .unique()
                    .map_err(|e| Error::Semantic(format!("Failed to get unique groups: {}", e)))?;
                let sorted_groups = unique_groups
                    .sort(SortOptions::default())
                    .map_err(|e| Error::Semantic(format!("Failed to sort groups: {}", e)))?;

                // Convert to strings, handling both string and numeric types
                let group_levels: Vec<String> = if let Ok(str_series) = sorted_groups.str() {
                    str_series
                        .into_iter()
                        .map(|s| s.unwrap_or("").to_string())
                        .collect()
                } else {
                    // Try as numeric and convert to string
                    if let Ok(i64_series) = sorted_groups.i64() {
                        i64_series
                            .into_iter()
                            .map(|n| n.map(|v| v.to_string()).unwrap_or("".to_string()))
                            .collect()
                    } else if let Ok(f64_series) = sorted_groups.f64() {
                        f64_series
                            .into_iter()
                            .map(|n| n.map(|v| v.to_string()).unwrap_or("".to_string()))
                            .collect()
                    } else {
                        return Err(Error::Semantic(
                            "Group variable must be string or numeric".into(),
                        ));
                    }
                };

                let mut random_cols = Vec::new();

                match inner {
                    Expr::Intercept(true) => {
                        // Random intercept: ri(Subject=<level>) for each group level
                        for level in &group_levels {
                            let col_name = format!("ri({}={})", group_var, level);
                            let mut col_data = vec![0.0; n_rows];

                            // Set to 1.0 for rows where group == level
                            if let Ok(str_series) = group_series.str() {
                                for (i, group_val) in str_series.into_iter().enumerate() {
                                    if let Some(val) = group_val {
                                        if val == level {
                                            col_data[i] = 1.0;
                                        }
                                    }
                                }
                            } else if let Ok(i64_series) = group_series.i64() {
                                for (i, group_val) in i64_series.into_iter().enumerate() {
                                    if let Some(val) = group_val {
                                        if val.to_string() == *level {
                                            col_data[i] = 1.0;
                                        }
                                    }
                                }
                            } else if let Ok(f64_series) = group_series.f64() {
                                for (i, group_val) in f64_series.into_iter().enumerate() {
                                    if let Some(val) = group_val {
                                        if val.to_string() == *level {
                                            col_data[i] = 1.0;
                                        }
                                    }
                                }
                            }

                            let series =
                                Float64Chunked::from_slice(col_name.clone().into(), &col_data)
                                    .into_series();
                            random_cols.push((col_name, series));
                        }
                    }
                    Expr::Sum(terms) => {
                        // Handle (0 + var|group) case
                        if terms.len() == 2 {
                            if let (Expr::Intercept(false), Expr::Var(var_name)) =
                                (&terms[0], &terms[1])
                            {
                                // Random slope: rs(var|group=<level>) for each group level
                                let var_series = df
                                    .column(var_name)
                                    .map_err(|_| {
                                        Error::Semantic(format!(
                                            "Variable '{}' not found",
                                            var_name
                                        ))
                                    })?
                                    .as_series()
                                    .ok_or_else(|| {
                                        Error::Semantic(
                                            "Failed to convert variable column to series".into(),
                                        )
                                    })?
                                    .clone();

                                for level in &group_levels {
                                    let col_name =
                                        format!("rs({}|{}={})", var_name, group_var, level);
                                    let mut col_data = vec![0.0; n_rows];

                                    // Set to var value for rows where group == level
                                    // Handle both f64 and i64 variable types
                                    let var_values: Vec<Option<f64>> = if let Ok(f64_series) =
                                        var_series.f64()
                                    {
                                        f64_series.into_iter().collect()
                                    } else if let Ok(i64_series) = var_series.i64() {
                                        i64_series
                                            .into_iter()
                                            .map(|v| v.map(|x| x as f64))
                                            .collect()
                                    } else {
                                        return Err(Error::Semantic(
                                            "Variable column must be numeric (i64 or f64)".into(),
                                        ));
                                    };

                                    if let Ok(str_series) = group_series.str() {
                                        for (i, (group_val, var_val)) in str_series
                                            .into_iter()
                                            .zip(var_values.iter())
                                            .enumerate()
                                        {
                                            if let (Some(g), Some(v)) = (group_val, var_val) {
                                                if g == level {
                                                    col_data[i] = *v;
                                                }
                                            }
                                        }
                                    } else if let Ok(i64_series) = group_series.i64() {
                                        for (i, (group_val, var_val)) in i64_series
                                            .into_iter()
                                            .zip(var_values.iter())
                                            .enumerate()
                                        {
                                            if let (Some(g), Some(v)) = (group_val, var_val) {
                                                if g.to_string() == *level {
                                                    col_data[i] = *v;
                                                }
                                            }
                                        }
                                    } else if let Ok(f64_series) = group_series.f64() {
                                        for (i, (group_val, var_val)) in f64_series
                                            .into_iter()
                                            .zip(var_values.iter())
                                            .enumerate()
                                        {
                                            if let (Some(g), Some(v)) = (group_val, var_val) {
                                                if g.to_string() == *level {
                                                    col_data[i] = *v;
                                                }
                                            }
                                        }
                                    }

                                    let series = Float64Chunked::from_slice(
                                        col_name.clone().into(),
                                        &col_data,
                                    )
                                    .into_series();
                                    random_cols.push((col_name, series));
                                }
                            }
                        }
                    }
                    _ => {
                        // For other expressions, treat as fixed effect for now
                        // TODO: Implement more complex random effects
                    }
                }

                Ok(random_cols)
            } else {
                Ok(Vec::new())
            }
        }
        _ => {
            // For other group specifications, return empty for now
            // TODO: Implement other group types
            Ok(Vec::new())
        }
    }
}

/// Convert a categorical variable to treatment contrasts.
///
/// This function takes a string series representing a categorical variable
/// and converts it to treatment contrasts (dummy variables) where the first
/// level is the reference level.
///
/// # Arguments
///
/// * `str_series` - The string series representing the categorical variable
/// * `var_name` - The name of the variable
///
/// # Returns
///
/// Returns a Series with the first contrast (second level vs first level).
/// For now, we return just the first contrast to avoid complexity in interactions.
///
/// # Examples
///
/// For a variable with levels ["A", "B", "C"], this creates a contrast
/// where B=1, A=0, C=0 (B vs A contrast).
fn materialize_categorical_to_contrasts(
    str_series: &StringChunked,
    var_name: &str,
) -> Result<Series, Error> {
    // Get unique levels
    let unique_levels: Vec<String> = str_series
        .into_iter()
        .filter_map(|x| x.map(|s| s.to_string()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if unique_levels.len() < 2 {
        return Err(Error::Semantic(format!(
            "Categorical variable '{}' must have at least 2 levels, found {}",
            var_name,
            unique_levels.len()
        )));
    }

    // Sort levels to ensure consistent ordering
    let mut sorted_levels = unique_levels;
    sorted_levels.sort();

    // Create treatment contrast: second level vs first level
    let reference_level = &sorted_levels[0];
    let contrast_level = &sorted_levels[1];

    let contrast_data: Vec<f64> = str_series
        .into_iter()
        .map(|x| match x {
            Some(level) if level == contrast_level => 1.0,
            _ => 0.0,
        })
        .collect();

    let contrast_name = format!("{}_{}_vs_{}", var_name, contrast_level, reference_level);
    Ok(Float64Chunked::from_slice(contrast_name.into(), &contrast_data).into_series())
}
