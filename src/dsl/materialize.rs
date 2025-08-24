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
/// use polars_formula::dsl::{parser::parser, materialize_dsl_spec, MaterializeOptions};
/// use chumsky::Parser;
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
/// let (y, X, Z) = materialize_dsl_spec(&df, &spec, MaterializeOptions::default())?;
///
/// println!("Response: {:?}", y);
/// println!("Fixed effects: {:?}", X);
/// println!("Random effects: {:?}", Z);
/// # Ok(())
/// # }
/// ```
pub fn materialize_dsl_spec(
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

/// Materialize a DSL Formula against a DataFrame.
fn materialize_formula(
    df: &DataFrame,
    formula: &Formula,
    opts: MaterializeOptions,
) -> Result<(DataFrame, DataFrame, DataFrame), Error> {
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
        Expr::Var(name) => df
            .column(name)
            .map_err(|_| Error::Semantic(format!("Column '{}' not found in DataFrame", name)))
            .and_then(|s| {
                s.as_series()
                    .ok_or_else(|| Error::Semantic("Failed to convert column to series".into()))
            })
            .map(|s| s.clone()),
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
            // TODO: Implement proper interaction materialization
            if let Some(first_term) = terms.first() {
                materialize_expr(df, first_term)
            } else {
                Err(Error::Semantic("Empty interaction expression".into()))
            }
        }
        Expr::Func { name, args } => {
            // Handle special functions
            match name.as_str() {
                "poly" => materialize_poly(df, args),
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
            // For interactions, create a product of the terms
            // TODO: Implement proper interaction materialization
            if let Some(first_term) = terms.first() {
                let (fixed_cols, random_cols) =
                    materialize_expr_to_columns_with_random(df, first_term)?;
                Ok((fixed_cols, random_cols))
            } else {
                Ok((Vec::new(), Vec::new()))
            }
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

/// Materialize a polynomial function.
fn materialize_poly(df: &DataFrame, args: &[Expr]) -> Result<Series, Error> {
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
    let _degree = match degree_expr {
        Expr::Num(n) => *n as usize,
        _ => {
            return Err(Error::Semantic(
                "Second argument to poly() must be a number".into(),
            ))
        }
    };

    // Get the variable column
    let var_series = df
        .column(var_name)
        .map_err(|_| Error::Semantic(format!("Column '{}' not found in DataFrame", var_name)))?
        .as_series()
        .ok_or_else(|| Error::Semantic("Failed to convert column to series".into()))?
        .clone();

    // For now, just return the original series
    // TODO: Implement proper polynomial expansion
    Ok(var_series)
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
