use chumsky::Parser;
use polars_formula::dsl::{ast::*, parser::parser};

#[test]
fn test_basic_formula_parsing() {
    let p = parser();

    // Test basic formula parsing
    let result = p.parse("y ~ x1 + x2");
    assert!(result.is_ok());

    let spec = result.unwrap();
    assert_eq!(spec.formula.lhs, Response::Var("y".to_string()));

    if let Expr::Sum(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 2);
    } else {
        panic!("Expected Sum expression");
    }
}

#[test]
fn test_smooth_parsing() {
    let p = parser();

    // Test smooth function parsing
    let result = p.parse("y ~ s(x, k=10, bs=\"tp\")");
    assert!(result.is_ok());

    let spec = result.unwrap();
    if let Expr::Smooth { kind, vars, args } = &spec.formula.rhs {
        assert_eq!(*kind, SmoothKind::S);
        assert_eq!(vars[0], "x");
        assert!(args.contains_key("k"));
        assert!(args.contains_key("bs"));
    } else {
        panic!("Expected Smooth expression");
    }
}

#[test]
fn test_group_parsing() {
    let p = parser();

    // Test random effects group parsing
    let result = p.parse("y ~ x + (1|group)");
    assert!(result.is_ok());

    let spec = result.unwrap();
    if let Expr::Sum(terms) = &spec.formula.rhs {
        if let Expr::Group {
            inner,
            spec: _,
            kind,
            id,
        } = &terms[1]
        {
            assert_eq!(*kind, GroupKind::Correlated);
            assert_eq!(*id, None);
            if let Expr::Intercept(true) = inner.as_ref() {
                // Expected
            } else {
                panic!("Expected intercept in group");
            }
        } else {
            panic!("Expected Group expression");
        }
    } else {
        panic!("Expected Sum expression");
    }
}

#[test]
fn test_aterm_parsing() {
    let p = parser();

    // Test aterm parsing
    let result = p.parse("y | weights(w) ~ x");
    assert!(result.is_ok());

    let spec = result.unwrap();
    assert_eq!(spec.formula.aterms.len(), 1);
    if let Aterm::Weights(_) = &spec.formula.aterms[0] {
        // Expected
    } else {
        panic!("Expected Weights aterm");
    }
}

#[test]
fn test_survival_parsing() {
    let p = parser();

    // Test survival response
    let result = p.parse("Surv(time, status) ~ x");
    assert!(result.is_ok());

    let spec = result.unwrap();
    match &spec.formula.lhs {
        Response::Surv {
            time: _,
            event: _,
            time2,
        } => {
            assert!(time2.is_none());
        }
        _ => panic!("Expected Surv response"),
    }
}

#[test]
fn test_family_parsing() {
    let p = parser();

    // Test family specification
    let result = p.parse("family=gaussian() y ~ x");
    assert!(result.is_ok());

    let spec = result.unwrap();
    assert!(spec.family.is_some());

    if let Some(Family::Builtin(name, args)) = &spec.family {
        assert_eq!(name, "gaussian");
        assert_eq!(args.len(), 0);
    } else {
        panic!("Expected Builtin family");
    }
}

#[test]
fn test_interaction_parsing() {
    let p = parser();

    // Test interaction parsing
    let result = p.parse("y ~ x1:x2");
    assert!(result.is_ok());

    let spec = result.unwrap();
    if let Expr::Interaction(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 2);
    } else {
        panic!("Expected Interaction expression");
    }
}

#[test]
fn test_power_parsing() {
    let p = parser();

    // Test power parsing
    let result = p.parse("y ~ x^2");
    assert!(result.is_ok());

    let spec = result.unwrap();
    if let Expr::Pow { base: _, exp: _ } = &spec.formula.rhs {
        // Expected
    } else {
        panic!("Expected Pow expression");
    }
}

#[test]
fn test_nesting_parsing() {
    let p = parser();

    // Test nesting parsing
    let result = p.parse("y ~ a/b");
    assert!(result.is_ok());

    let spec = result.unwrap();
    if let Expr::Nest {
        outer: _,
        inner: _,
        kind,
    } = &spec.formula.rhs
    {
        assert_eq!(*kind, NestKind::Slash);
    } else {
        panic!("Expected Nest expression");
    }
}

#[test]
fn test_complex_formula() {
    let p = parser();

    // Test complex formula with multiple constructs
    let result = p.parse("family=gaussian() y | weights(w) ~ x + s(z, k=5) + (1|group) + x:z");
    assert!(result.is_ok());

    let spec = result.unwrap();
    assert!(spec.family.is_some());
    assert!(!spec.formula.aterms.is_empty());

    if let Expr::Sum(terms) = &spec.formula.rhs {
        assert!(terms.len() >= 4); // x, s(z), (1|group), x:z
    } else {
        panic!("Expected Sum expression with multiple terms");
    }
}
