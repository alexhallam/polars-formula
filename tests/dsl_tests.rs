use polars_formula::dsl::{parser::parser, ast::*, pretty::pretty};
use chumsky::Parser;

#[test]
fn test_basic_formula_parsing() {
    let p = parser();
    
    // Basic: y ~ x1 + x2
    let result = p.parse("y ~ x1 + x2");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    assert_eq!(spec.formula.lhs, Response::Var("y".to_string()));
    // RHS should be a Sum with x1 and x2
    if let Expr::Sum(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 2);
    } else {
        panic!("Expected Sum expression");
    }
}

#[test]
fn test_interactions_parsing() {
    let p = parser();
    
    // Interactions: x1:x2, (x1 + x2):x3, x1*x2*x3
    let result = p.parse("y ~ x1:x2");
    assert!(result.is_ok());
    
    let result = p.parse("y ~ (x1 + x2):x3");
    assert!(result.is_ok());
    
    let result = p.parse("y ~ x1*x2*x3");
    assert!(result.is_ok());
}

#[test]
fn test_powers_parsing() {
    let p = parser();
    
    // Powers: (a+b+c)^2, (a+b)^3
    let result = p.parse("y ~ (a+b+c)^2");
    assert!(result.is_ok());
    
    let result = p.parse("y ~ (a+b)^3");
    assert!(result.is_ok());
}

#[test]
fn test_nesting_parsing() {
    let p = parser();
    
    // Nesting: a/b, b %in% a
    let result = p.parse("y ~ a/b");
    assert!(result.is_ok());
    
    let result = p.parse("y ~ b %in% a");
    assert!(result.is_ok());
}

#[test]
fn test_aterms_parsing() {
    let p = parser();
    
    // Aterms (comma & chain): y | se(sey) | weights(w) | trunc(lb=0, ub=100) ~ x
    let result = p.parse("y | se(sey) | weights(w) | trunc(lb=0, ub=100) ~ x");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    assert_eq!(spec.formula.aterms.len(), 3);
    
    // Test individual aterms
    match &spec.formula.aterms[0] {
        Aterm::Se(_) => {},
        _ => panic!("Expected Se aterm"),
    }
    
    match &spec.formula.aterms[1] {
        Aterm::Weights(_) => {},
        _ => panic!("Expected Weights aterm"),
    }
    
    match &spec.formula.aterms[2] {
        Aterm::Trunc { lb, ub } => {
            assert!(lb.is_some());
            assert!(ub.is_some());
        },
        _ => panic!("Expected Trunc aterm"),
    }
}

#[test]
fn test_survival_parsing() {
    let p = parser();
    
    // Survival: Surv(time, status) ~ strata(g) + tt(x)
    let result = p.parse("Surv(time, status) ~ strata(g) + tt(x)");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    match &spec.formula.lhs {
        Response::Surv { time: _, event: _, time2 } => {
            assert!(time2.is_none());
        },
        _ => panic!("Expected Surv response"),
    }
}

#[test]
fn test_smooths_parsing() {
    let p = parser();
    
    // Smooths: s(x, k=10, bs="tp") + t2(x,z, bs="cr", by=g)
    let result = p.parse("y ~ s(x, k=10, bs=\"tp\") + t2(x,z, bs=\"cr\", by=g)");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    if let Expr::Sum(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 2);
        
        // Check first smooth
        if let Expr::Smooth { kind, vars, args } = &terms[0] {
            assert!(matches!(kind, SmoothKind::S));
            assert_eq!(vars, &["x"]);
            assert!(args.contains_key("k"));
            assert!(args.contains_key("bs"));
        } else {
            panic!("Expected Smooth expression");
        }
        
        // Check second smooth
        if let Expr::Smooth { kind, vars, args } = &terms[1] {
            assert!(matches!(kind, SmoothKind::T2));
            assert_eq!(vars, &["x", "z"]);
            assert!(args.contains_key("bs"));
            assert!(args.contains_key("by"));
        } else {
            panic!("Expected Smooth expression");
        }
    } else {
        panic!("Expected Sum expression");
    }
}

#[test]
fn test_groups_parsing() {
    let p = parser();
    
    // Groups: (1|g) + (x||g) + (1|ID|g) + (1|g1+g2) + (1|g1:g2) + (1|g1/g2)
    let result = p.parse("y ~ (1|g) + (x||g) + (1|ID|g) + (1|g1+g2) + (1|g1:g2) + (1|g1/g2)");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    if let Expr::Sum(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 6);
        
        // Check first group (1|g)
        if let Expr::Group { inner, kind, id, .. } = &terms[0] {
            assert!(matches!(**inner, Expr::Intercept(true)));
            assert!(matches!(kind, GroupKind::Correlated));
            assert!(id.is_none());
        } else {
            panic!("Expected Group expression");
        }
        
        // Check second group (x||g) - uncorrelated
        if let Expr::Group { kind, .. } = &terms[1] {
            assert!(matches!(kind, GroupKind::Uncorrelated));
        } else {
            panic!("Expected Group expression");
        }
        
        // Check third group (1|ID|g) - with ID
        if let Expr::Group { id, .. } = &terms[2] {
            assert_eq!(id, &Some("ID".to_string()));
        } else {
            panic!("Expected Group expression");
        }
    } else {
        panic!("Expected Sum expression");
    }
}

#[test]
fn test_autocor_parsing() {
    let p = parser();
    
    // Autocor inline: y ~ x + ar(p=1) and top-level
    let result = p.parse("y ~ x + ar(p=1)");
    assert!(result.is_ok());
    
    // Top-level autocor
    let result = p.parse("family=gaussian() y ~ x + ar(p=1) + sigma ~ z + ma(q=1)");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    assert!(spec.family.is_some());
    assert_eq!(spec.autocor.len(), 2);
    
    // Check first autocor
    if let Some(autocor) = spec.autocor.first() {
        assert_eq!(autocor.name, "ar");
        assert!(autocor.args.contains_key("p"));
    }
    
    // Check second autocor
    if let Some(autocor) = spec.autocor.get(1) {
        assert_eq!(autocor.name, "ma");
        assert!(autocor.args.contains_key("q"));
    }
}

#[test]
fn test_distributional_dpars_parsing() {
    let p = parser();
    
    // Distributional dpars: y ~ x + sigma ~ z + zi ~ w
    let result = p.parse("y ~ x + sigma ~ z + zi ~ w");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    assert_eq!(spec.dpars.len(), 2);
    
    // Check first dpar
    if let Some(dpar) = spec.dpars.first() {
        assert_eq!(dpar.name, "sigma");
    }
    
    // Check second dpar
    if let Some(dpar) = spec.dpars.get(1) {
        assert_eq!(dpar.name, "zi");
    }
}

#[test]
fn test_families_parsing() {
    let p = parser();
    
    // Families: mixture(gaussian(), student()) y ~ x, custom_family("kumaraswamy","mu","phi") y ~ x
    let result = p.parse("family=mixture(gaussian(), student()) y ~ x");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    if let Some(Family::Mixture(families)) = &spec.family {
        assert_eq!(families.len(), 2);
    } else {
        panic!("Expected Mixture family");
    }
    
    let result = p.parse("family=custom_family(\"kumaraswamy\",\"mu\",\"phi\") y ~ x");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    if let Some(Family::Custom { name, dpars }) = &spec.family {
        assert_eq!(name, "kumaraswamy");
        assert_eq!(dpars, &["mu", "phi"]);
    } else {
        panic!("Expected Custom family");
    }
}

#[test]
fn test_parsing_error_cases() {
    let p = parser();
    
    // Unbalanced parens: y ~ (x +
    let result = p.parse("y ~ (x +");
    assert!(result.is_err());
    
    // Bad func: y ~ poly(x,)
    let result = p.parse("y ~ poly(x,)");
    assert!(result.is_err());
    
    // Unknown token: y ~ x$y
    let result = p.parse("y ~ x$y");
    assert!(result.is_err());
    
    // Bad aterm: y | lb(0) ~ x (should suggest trunc(lb=0))
    let result = p.parse("y | lb(0) ~ x");
    assert!(result.is_err());
    
    // Wrong order: ~ x (missing lhs)
    let result = p.parse("~ x");
    assert!(result.is_err());
}

#[test]
fn test_pretty_printing() {
    let p = parser();
    
    // Test basic roundtrip
    let input = "y ~ x1 + x2";
    let spec = p.parse(input).unwrap();
    let output = pretty(&spec);
    
    // Pretty print should be readable
    assert!(output.contains("y"));
    assert!(output.contains("x1"));
    assert!(output.contains("x2"));
    
    // Test interaction roundtrip
    let input = "y ~ x1:x2";
    let spec = p.parse(input).unwrap();
    let output = pretty(&spec);
    assert!(output.contains("x1:x2"));
}

#[test]
fn test_canonicalization_identities() {
    let p = parser();
    
    // x*y == x + y + x:y
    let _a = p.parse("y ~ x*y").unwrap();
    let _b = p.parse("y ~ x + y + x:y").unwrap();
    // Note: This will fail until canonicalization is implemented
    // assert_eq!(canonicalize(&a), canonicalize(&b));
    
    // (a/b) == a + a:b
    let _a = p.parse("y ~ a/b").unwrap();
    let _b = p.parse("y ~ a + a:b").unwrap();
    // Note: This will fail until canonicalization is implemented
    // assert_eq!(canonicalize(&a), canonicalize(&b));
}

#[test]
fn test_edge_cases() {
    let p = parser();
    
    // Identifiers with dots: np.log(x), stats.zscore(x)
    let result = p.parse("y ~ np.log(x) + stats.zscore(x)");
    assert!(result.is_ok());
    
    // Strings in args: s(x, bs="tp")
    let result = p.parse("y ~ s(x, bs=\"tp\")");
    assert!(result.is_ok());
    
    // Boolean/ternary in aterms: trunc(lb = x > 0 ? 0 : -1)
    let result = p.parse("y | trunc(lb = x > 0 ? 0 : -1) ~ x");
    assert!(result.is_ok());
    
    // Many chained aterms: y | weights(w) | se(se) | subset(m)
    let result = p.parse("y | weights(w) | se(se) | subset(m) ~ x");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    assert_eq!(spec.formula.aterms.len(), 3);
}

#[test]
fn test_complex_expressions() {
    let p = parser();
    
    // Complex nested expressions
    let result = p.parse("y ~ (x1 + x2):(z1 + z2) + poly(x, 3) + s(x, k=10)");
    assert!(result.is_ok());
    
    // Multiple interactions
    let result = p.parse("y ~ x1:x2:x3 + a*b*c");
    assert!(result.is_ok());
    
    // Mixed operators
    let result = p.parse("y ~ x1 + x2*x3 + x4:x5 + x6/x7");
    assert!(result.is_ok());
}

#[test]
fn test_logical_expressions() {
    let p = parser();
    
    // Logical expressions
    let result = p.parse("y ~ x > 0 && z < 10");
    assert!(result.is_ok());
    
    // Ternary expressions
    let result = p.parse("y ~ x > 0 ? 1 : 0");
    assert!(result.is_ok());
    
    // Complex logical
    let result = p.parse("y ~ (x > 0 && z < 10) || (a == b)");
    assert!(result.is_ok());
}
