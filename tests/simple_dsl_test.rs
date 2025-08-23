use polars_formula::dsl::{parser::parser, ast::*, pretty::pretty};
use chumsky::Parser;

#[test]
fn test_simple_formula() {
    let p = parser();
    
    // Test RHS-only formula (this should work)
    let result = p.parse("x1 + x2");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    assert_eq!(spec.formula.lhs, Response::Var("".to_string()));
    
    // Test that RHS is parsed as a Sum
    if let Expr::Sum(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 2);
    } else {
        panic!("Expected Sum expression");
    }
}

#[test]
fn test_simple_interaction() {
    let p = parser();
    
    // Test interaction parsing
    let result = p.parse("x1:x2");
    assert!(result.is_ok());
    
    let spec = result.unwrap();
    if let Expr::Interaction(terms) = &spec.formula.rhs {
        assert_eq!(terms.len(), 2);
    } else {
        panic!("Expected Interaction expression");
    }
}

#[test]
fn test_family_spec() {
    let p = parser();
    
    // Test family specification
    let result = p.parse("family=gaussian() x");
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
fn test_pretty_print() {
    let p = parser();
    
    let input = "x1 + x2";
    let spec = p.parse(input).unwrap();
    let output = pretty(&spec);
    
    // Pretty print should be readable
    assert!(output.contains("x1"));
    assert!(output.contains("x2"));
}
