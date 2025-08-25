use chumsky::Parser;
use polars_formula::dsl::{ast::*, canon::*, parser::parser, pretty::pretty};

#[test]
fn test_canonicalize_star_expansion() {
    let p = parser();

    // Test star expansion: a*b -> a + b + a:b
    let result = p.parse("a*b");
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    // Should expand to sum with main effects and interaction
    if let Expr::Sum(terms) = &canonicalized.formula.rhs {
        assert!(terms.len() >= 3); // a, b, a:b

        // Should contain variables a and b
        let has_a = terms.iter().any(|t| matches!(t, Expr::Var(v) if v == "a"));
        let has_b = terms.iter().any(|t| matches!(t, Expr::Var(v) if v == "b"));
        let has_interaction = terms.iter().any(|t| matches!(t, Expr::Interaction(_)));

        assert!(has_a);
        assert!(has_b);
        assert!(has_interaction);
    } else {
        panic!("Expected Sum expression after canonicalization");
    }
}

#[test]
fn test_canonicalize_slash_expansion() {
    let p = parser();

    // Test slash expansion: a/b -> a + a:b
    let result = p.parse("a/b");
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    // Should expand to sum with main effect and interaction
    if let Expr::Sum(terms) = &canonicalized.formula.rhs {
        assert!(terms.len() >= 2); // a, a:b

        // Should contain variable a
        let has_a = terms.iter().any(|t| matches!(t, Expr::Var(v) if v == "a"));
        let has_interaction = terms.iter().any(|t| matches!(t, Expr::Interaction(_)));

        assert!(has_a);
        assert!(has_interaction);
    } else {
        panic!("Expected Sum expression after canonicalization");
    }
}

#[test]
fn test_canonicalize_nested_sums() {
    let p = parser();

    // Test nested sum flattening
    let result = p.parse("(a + b) + c");
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    // Should flatten to a single sum
    if let Expr::Sum(terms) = &canonicalized.formula.rhs {
        // All terms should be non-sum expressions (flattened)
        let all_non_sum = terms.iter().all(|t| !matches!(t, Expr::Sum(_)));
        assert!(all_non_sum);
        assert!(terms.len() >= 3);
    } else {
        panic!("Expected Sum expression after canonicalization");
    }
}

#[test]
fn test_canonicalize_interactions() {
    let p = parser();

    // Test interaction flattening
    let result = p.parse("(a:b):c");
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    // Should flatten interaction
    if let Expr::Interaction(terms) = &canonicalized.formula.rhs {
        assert!(terms.len() >= 3); // a, b, c
    } else {
        panic!("Expected Interaction expression after canonicalization");
    }
}

#[test]
fn test_expand_star_function() {
    // Test the canonicalize_expr function directly
    let prod_expr = Expr::Prod(vec![Expr::Var("x".to_string()), Expr::Var("y".to_string())]);

    let expanded = canonicalize_expr(prod_expr);

    if let Expr::Sum(terms) = expanded {
        assert!(terms.len() >= 3); // x, y, x:y

        let has_x = terms.iter().any(|t| matches!(t, Expr::Var(v) if v == "x"));
        let has_y = terms.iter().any(|t| matches!(t, Expr::Var(v) if v == "y"));
        let has_interaction = terms.iter().any(|t| matches!(t, Expr::Interaction(_)));

        assert!(has_x);
        assert!(has_y);
        assert!(has_interaction);
    } else {
        panic!("Expected Sum expression from canonicalize_expr");
    }
}

#[test]
fn test_canonicalize_complex_formula() {
    let p = parser();

    // Test complex formula with multiple transformations
    let result = p.parse("y ~ a*b + c/d + (x + y)");
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    // Should be a sum with expanded terms
    if let Expr::Sum(terms) = &canonicalized.formula.rhs {
        assert!(terms.len() > 3); // Should have many terms after expansion
    } else {
        panic!("Expected Sum expression after canonicalization");
    }
}

#[test]
fn test_canonicalize_preserves_other_constructs() {
    let p = parser();

    // Test that canonicalization preserves other constructs like powers
    let result = p.parse("x^2 + I(y)");
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    if let Expr::Sum(terms) = &canonicalized.formula.rhs {
        // Should contain power and identity expressions
        let has_power = terms.iter().any(|t| matches!(t, Expr::Pow { .. }));
        let has_identity = terms.iter().any(|t| matches!(t, Expr::Func { name, .. } if name == "I"));

        assert!(has_power);
        assert!(has_identity);
    } else {
        panic!("Expected Sum expression after canonicalization");
    }
}

#[test]
fn test_canonicalize_roundtrip() {
    let p = parser();

    // Test that canonicalization + pretty-printing produces valid syntax
    let input = "y ~ x1*x2 + z/w";
    let result = p.parse(input);
    assert!(result.is_ok());

    let spec = result.unwrap();
    let canonicalized = canonicalize(&spec);

    // Pretty print the canonicalized form
    let pretty_output = pretty(&canonicalized);

    // Should be able to parse the pretty-printed output
    let reparsed = p.parse(pretty_output.as_str());
    assert!(reparsed.is_ok(), "Failed to reparse: {}", pretty_output);
}

#[test]
fn test_canonicalize_identity_laws() {
    let p = parser();

    // Test algebraic identities

    // x*y should be equivalent to y*x after canonicalization
    let xy = p.parse("x*y").unwrap();
    let yx = p.parse("y*x").unwrap();

    let canon_xy = canonicalize(&xy);
    let canon_yx = canonicalize(&yx);

    // Both should produce the same canonical form (modulo ordering)
    let pretty_xy = pretty(&canon_xy);
    let pretty_yx = pretty(&canon_yx);

    // Should contain the same terms (though ordering might differ)
    assert!(pretty_xy.contains("x") && pretty_xy.contains("y"));
    assert!(pretty_yx.contains("x") && pretty_yx.contains("y"));
    assert!(pretty_xy.contains("x:y") || pretty_xy.contains("y:x"));
    assert!(pretty_yx.contains("x:y") || pretty_yx.contains("y:x"));
}
