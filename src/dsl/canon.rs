use super::ast::*;
use std::collections::HashSet;

/// Canonicalize a ModelSpec by expanding syntactic sugar and normalizing expressions.
///
/// This function transforms a parsed ModelSpec into its canonical form by:
/// - Expanding product terms (`*`) into main effects and interactions
/// - Expanding nested terms (`/`) into main effects and interactions  
/// - Flattening nested sums and interactions
/// - Normalizing group expressions and function calls
/// - Hoisting autocorrelation terms
///
/// # Arguments
///
/// * `spec` - The parsed ModelSpec to canonicalize
///
/// # Returns
///
/// Returns a new ModelSpec with all expressions in canonical form.
///
/// # Examples
///
/// ## Expand Product Terms
/// ```rust
/// use polars_formula::dsl::{parser::parser, canon::canonicalize};
///
/// let p = parser();
/// let spec = p.parse("y ~ x1*x2".chars().collect::<Vec<_>>()).unwrap();
/// let canonicalized = canonicalize(&spec);
///
/// // x1*x2 expands to x1 + x2 + x1:x2
/// assert!(canonicalized.formula.rhs.to_string().contains("x1"));
/// assert!(canonicalized.formula.rhs.to_string().contains("x2"));
/// assert!(canonicalized.formula.rhs.to_string().contains("x1:x2"));
/// ```
///
/// ## Expand Nested Terms
/// ```rust
/// use polars_formula::dsl::{parser::parser, canon::canonicalize};
///
/// let p = parser();
/// let spec = p.parse("y ~ x1/x2".chars().collect::<Vec<_>>()).unwrap();
/// let canonicalized = canonicalize(&spec);
///
/// // x1/x2 expands to x1 + x1:x2
/// assert!(canonicalized.formula.rhs.to_string().contains("x1"));
/// assert!(canonicalized.formula.rhs.to_string().contains("x1:x2"));
/// ```
///
/// ## Complex Formula
/// ```rust
/// use polars_formula::dsl::{parser::parser, canon::canonicalize};
///
/// let p = parser();
/// let spec = p.parse("y ~ (x1 + x2)*z + poly(w, 3)".chars().collect::<Vec<_>>()).unwrap();
/// let canonicalized = canonicalize(&spec);
///
/// // Expands to: x1 + x2 + z + x1:z + x2:z + poly(w, 3)
/// // All product terms are expanded into main effects and interactions
/// ```
pub fn canonicalize(spec: &ModelSpec) -> ModelSpec {
    let mut canonicalized = spec.clone();

    // Canonicalize the main formula RHS
    canonicalized.formula.rhs = canonicalize_expr(canonicalized.formula.rhs.clone());

    // Canonicalize distributional parameter formulas
    for dpar in &mut canonicalized.dpars {
        dpar.rhs = canonicalize_expr(dpar.rhs.clone());
    }

    // Flatten and normalize the response aterms
    canonicalized.formula.aterms = canonicalize_aterms(canonicalized.formula.aterms.clone());

    // Hoist inline autocorrelation terms
    let (rhs, hoisted_autocor) = hoist_autocor(canonicalized.formula.rhs.clone());
    canonicalized.formula.rhs = rhs;
    canonicalized.autocor.extend(hoisted_autocor);

    canonicalized
}

/// Canonicalize a single expression by expanding syntactic sugar and normalizing structure.
///
/// This function transforms an expression into its canonical form by:
/// - Expanding product terms (`*`) into main effects and interactions
/// - Expanding nested terms (`/`) into main effects and interactions
/// - Flattening nested sums and interactions
/// - Normalizing group expressions and function calls
///
/// # Arguments
///
/// * `expr` - The expression to canonicalize
///
/// # Returns
///
/// Returns a new expression in canonical form.
///
/// # Examples
///
/// ## Expand Product Expression
/// ```rust
/// use polars_formula::dsl::{ast::Expr, canon::canonicalize_expr};
///
/// let prod_expr = Expr::Prod(vec![
///     Expr::Var("x1".to_string()),
///     Expr::Var("x2".to_string())
/// ]);
/// let canonicalized = canonicalize_expr(prod_expr);
///
/// // x1*x2 expands to x1 + x2 + x1:x2
/// match canonicalized {
///     Expr::Sum(terms) => {
///         assert_eq!(terms.len(), 3); // x1, x2, x1:x2
///     }
///     _ => panic!("Expected Sum expression")
/// }
/// ```
///
/// ## Expand Nested Expression
/// ```rust
/// use polars_formula::dsl::{ast::{Expr, NestKind}, canon::canonicalize_expr};
///
/// let nested_expr = Expr::Nest {
///     outer: Box::new(Expr::Var("x1".to_string())),
///     inner: Box::new(Expr::Var("x2".to_string())),
///     kind: NestKind::Slash,
/// };
/// let canonicalized = canonicalize_expr(nested_expr);
///
/// // x1/x2 expands to x1 + x1:x2
/// match canonicalized {
///     Expr::Sum(terms) => {
///         assert_eq!(terms.len(), 2); // x1, x1:x2
///     }
///     _ => panic!("Expected Sum expression")
/// }
/// ```
///
/// ## Flatten Nested Sums
/// ```rust
/// use polars_formula::dsl::{ast::Expr, canon::canonicalize_expr};
///
/// let nested_sum = Expr::Sum(vec![
///     Expr::Var("x1".to_string()),
///     Expr::Sum(vec![
///         Expr::Var("x2".to_string()),
///         Expr::Var("x3".to_string())
///     ])
/// ]);
/// let canonicalized = canonicalize_expr(nested_sum);
///
/// // Flattens to x1 + x2 + x3
/// match canonicalized {
///     Expr::Sum(terms) => {
///         assert_eq!(terms.len(), 3);
///     }
///     _ => panic!("Expected Sum expression")
/// }
/// ```
pub fn canonicalize_expr(expr: Expr) -> Expr {
    match expr {
        // Expand * sugar: a*b -> a + b + a:b
        Expr::Prod(terms) => {
            let expanded = expand_star_terms(terms);
            canonicalize_expr(expanded)
        }
        // Expand / sugar: a/b -> a + a:b
        Expr::Nest {
            outer,
            inner,
            kind: NestKind::Slash,
        } => {
            let main = canonicalize_expr(*outer.clone());
            let interaction = canonicalize_expr(Expr::Interaction(vec![*outer, *inner]));
            canonicalize_expr(Expr::Sum(vec![main, interaction]))
        }
        // Keep %in% as is but canonicalize children
        Expr::Nest { outer, inner, kind } => Expr::Nest {
            outer: Box::new(canonicalize_expr(*outer)),
            inner: Box::new(canonicalize_expr(*inner)),
            kind,
        },
        // Flatten nested sums
        Expr::Sum(terms) => {
            let flattened = flatten_sum(terms);
            let canonicalized: Vec<Expr> = flattened.into_iter().map(canonicalize_expr).collect();
            flatten_sum_result(canonicalized)
        }
        // Flatten nested interactions
        Expr::Interaction(terms) => {
            let flattened = flatten_interaction(terms);
            let canonicalized: Vec<Expr> = flattened.into_iter().map(canonicalize_expr).collect();
            flatten_interaction_result(canonicalized)
        }
        // Canonicalize power expressions
        Expr::Pow { base, exp } => Expr::Pow {
            base: Box::new(canonicalize_expr(*base)),
            exp: Box::new(canonicalize_expr(*exp)),
        },
        // Canonicalize group expressions
        Expr::Group {
            inner,
            spec,
            kind,
            id,
        } => canonicalize_group_expr(*inner, spec, kind, id),
        // Canonicalize identity expressions
        Expr::Identity(inner) => Expr::Identity(Box::new(canonicalize_expr(*inner))),
        // Canonicalize function calls
        Expr::Func { name, args } => Expr::Func {
            name,
            args: args.into_iter().map(canonicalize_expr).collect(),
        },
        // Canonicalize smooth expressions
        Expr::Smooth { kind, vars, args } => {
            let canonicalized_args = args
                .into_iter()
                .map(|(k, v)| (k, canonicalize_expr(v)))
                .collect();
            Expr::Smooth {
                kind,
                vars,
                args: canonicalized_args,
            }
        }
        // Leave atoms as-is
        expr => expr,
    }
}

/// Expand star terms: [a, b, c] -> a + b + c + a:b + a:c + b:c + a:b:c
fn expand_star_terms(terms: Vec<Expr>) -> Expr {
    if terms.len() < 2 {
        return if terms.is_empty() {
            Expr::Intercept(true)
        } else {
            terms.into_iter().next().unwrap()
        };
    }

    let mut result = Vec::new();

    // Add main effects
    for term in &terms {
        result.push(term.clone());
    }

    // Add all possible interactions (2-way, 3-way, etc.)
    for k in 2..=terms.len() {
        let combinations = get_combinations(&terms, k);
        for combo in combinations {
            if combo.len() > 1 {
                result.push(Expr::Interaction(combo));
            }
        }
    }

    Expr::Sum(result)
}

/// Generate all k-combinations of a vector
fn get_combinations<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 || k > items.len() {
        return vec![];
    }
    if k == 1 {
        return items.iter().map(|x| vec![x.clone()]).collect();
    }

    let mut result = Vec::new();
    for i in 0..items.len() - k + 1 {
        let rest_combos = get_combinations(&items[i + 1..], k - 1);
        for mut combo in rest_combos {
            combo.insert(0, items[i].clone());
            result.push(combo);
        }
    }
    result
}

/// Flatten sum expressions
fn flatten_sum(terms: Vec<Expr>) -> Vec<Expr> {
    let mut result = Vec::new();
    for term in terms {
        match term {
            Expr::Sum(nested) => result.extend(flatten_sum(nested)),
            _ => result.push(term),
        }
    }
    result
}

/// Create a flattened sum result
fn flatten_sum_result(terms: Vec<Expr>) -> Expr {
    match terms.len() {
        0 => Expr::Intercept(false), // Empty sum = 0
        1 => terms.into_iter().next().unwrap(),
        _ => Expr::Sum(terms),
    }
}

/// Flatten interaction expressions  
fn flatten_interaction(terms: Vec<Expr>) -> Vec<Expr> {
    let mut result = Vec::new();
    for term in terms {
        match term {
            Expr::Interaction(nested) => result.extend(flatten_interaction(nested)),
            _ => result.push(term),
        }
    }
    result
}

/// Create a flattened interaction result
fn flatten_interaction_result(terms: Vec<Expr>) -> Expr {
    match terms.len() {
        0 => Expr::Intercept(true), // Empty interaction = 1
        1 => terms.into_iter().next().unwrap(),
        _ => Expr::Interaction(terms),
    }
}

/// Canonicalize group expressions by expanding them into standard mixed-effects notation
fn canonicalize_group_expr(
    inner: Expr,
    spec: GroupSpec,
    kind: GroupKind,
    id: Option<String>,
) -> Expr {
    // Canonicalize the inner expression first
    let canonicalized_inner = canonicalize_expr(inner);
    let canonicalized_spec = canonicalize_group_spec(spec);

    match canonicalized_inner {
        // If inner is just an intercept (1), keep as is
        Expr::Intercept(true) => Expr::Group {
            inner: Box::new(Expr::Intercept(true)),
            spec: canonicalized_spec,
            kind,
            id,
        },
        // If inner is a variable (e.g., Days), expand to (1|group) + (0 + var|group)
        Expr::Var(var_name) => {
            let random_intercept = Expr::Group {
                inner: Box::new(Expr::Intercept(true)),
                spec: canonicalized_spec.clone(),
                kind: kind.clone(),
                id: id.clone(),
            };

            let random_slope = Expr::Group {
                inner: Box::new(Expr::Sum(vec![
                    Expr::Intercept(false), // 0 +
                    Expr::Var(var_name.clone()),
                ])),
                spec: canonicalized_spec,
                kind,
                id,
            };

            Expr::Sum(vec![random_intercept, random_slope])
        }
        // For other expressions, canonicalize and keep as group
        other => Expr::Group {
            inner: Box::new(other),
            spec: canonicalized_spec,
            kind,
            id,
        },
    }
}

/// Canonicalize group specifications
fn canonicalize_group_spec(spec: GroupSpec) -> GroupSpec {
    match spec {
        GroupSpec::Func { name, args } => GroupSpec::Func {
            name,
            args: args.into_iter().map(canonicalize_expr).collect(),
        },
        spec => spec,
    }
}

/// Canonicalize aterms
fn canonicalize_aterms(aterms: Vec<Aterm>) -> Vec<Aterm> {
    // Remove duplicates and canonicalize expressions within aterms
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for aterm in aterms {
        let canonicalized = match aterm {
            Aterm::Se(expr) => Aterm::Se(canonicalize_expr(expr)),
            Aterm::Weights(expr) => Aterm::Weights(canonicalize_expr(expr)),
            Aterm::Trials(expr) => Aterm::Trials(canonicalize_expr(expr)),
            Aterm::Cens(expr) => Aterm::Cens(canonicalize_expr(expr)),
            Aterm::Trunc { lb, ub } => Aterm::Trunc {
                lb: lb.map(canonicalize_expr),
                ub: ub.map(canonicalize_expr),
            },
            Aterm::Subset(expr) => Aterm::Subset(canonicalize_expr(expr)),
            Aterm::Rate(expr) => Aterm::Rate(canonicalize_expr(expr)),
            Aterm::Thres { gr } => Aterm::Thres {
                gr: gr.map(canonicalize_expr),
            },
            Aterm::Dec(expr) => Aterm::Dec(canonicalize_expr(expr)),
            Aterm::Cat(expr) => Aterm::Cat(canonicalize_expr(expr)),
            Aterm::Index(expr) => Aterm::Index(canonicalize_expr(expr)),
            Aterm::VReal(exprs) => Aterm::VReal(exprs.into_iter().map(canonicalize_expr).collect()),
            Aterm::VInt(exprs) => Aterm::VInt(exprs.into_iter().map(canonicalize_expr).collect()),
            Aterm::Mi => Aterm::Mi,
        };

        // Simple deduplication based on discriminant
        let key = std::mem::discriminant(&canonicalized);
        if !seen.contains(&key) {
            seen.insert(key);
            result.push(canonicalized);
        }
    }

    result
}

/// Hoist inline autocorrelation terms from the expression
fn hoist_autocor(expr: Expr) -> (Expr, Vec<Autocor>) {
    let mut hoisted = Vec::new();
    let cleaned = hoist_autocor_recursive(expr, &mut hoisted);
    (cleaned, hoisted)
}

/// Recursively hoist autocorrelation terms
fn hoist_autocor_recursive(expr: Expr, hoisted: &mut Vec<Autocor>) -> Expr {
    match expr {
        // Detect autocorrelation function calls and hoist them
        Expr::Func { name, args } if is_autocor_function(&name) => {
            // Convert to Autocor and add to hoisted list
            let mut autocor_args = std::collections::HashMap::new();
            for (i, arg) in args.into_iter().enumerate() {
                autocor_args.insert(format!("arg{}", i), arg);
            }
            hoisted.push(Autocor {
                name: name.clone(),
                args: autocor_args,
            });

            // Replace with intercept (effectively remove from expression)
            Expr::Intercept(true)
        }
        Expr::Sum(terms) => {
            let processed: Vec<Expr> = terms
                .into_iter()
                .map(|t| hoist_autocor_recursive(t, hoisted))
                .filter(|t| !matches!(t, Expr::Intercept(true))) // Remove hoisted terms
                .collect();
            flatten_sum_result(processed)
        }
        Expr::Interaction(terms) => {
            let processed: Vec<Expr> = terms
                .into_iter()
                .map(|t| hoist_autocor_recursive(t, hoisted))
                .collect();
            flatten_interaction_result(processed)
        }
        Expr::Group {
            inner,
            spec,
            kind,
            id,
        } => Expr::Group {
            inner: Box::new(hoist_autocor_recursive(*inner, hoisted)),
            spec,
            kind,
            id,
        },
        expr => expr,
    }
}

/// Check if a function name is an autocorrelation function
fn is_autocor_function(name: &str) -> bool {
    matches!(
        name,
        "ar" | "ma" | "arma" | "cosy" | "unstr" | "sar" | "car" | "fcor"
    )
}
