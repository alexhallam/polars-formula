use proptest::prelude::*;
use polars_formula::dsl::{parser::parser, canon::*, pretty::pretty};
use chumsky::Parser;

// Strategy for generating valid identifiers
fn identifier_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "x".to_string(), "y".to_string(), "z".to_string(),
        "var1".to_string(), "var2".to_string(), "var3".to_string(),
        "age".to_string(), "income".to_string(), "weight".to_string(),
        "treatment".to_string(), "dose".to_string(), "group".to_string(),
    ])
}

// Strategy for generating basic formulas
fn basic_formula_strategy() -> impl Strategy<Value = String> {
    (
        identifier_strategy(),
        identifier_strategy(),
    ).prop_map(|(lhs, rhs)| format!("{} ~ {}", lhs, rhs))
}

// Strategy for generating interaction formulas
fn interaction_formula_strategy() -> impl Strategy<Value = String> {
    (
        identifier_strategy(),
        identifier_strategy(),
        identifier_strategy(),
    ).prop_map(|(lhs, var1, var2)| format!("{} ~ {}:{}", lhs, var1, var2))
}

// Strategy for generating sum formulas
fn sum_formula_strategy() -> impl Strategy<Value = String> {
    (
        identifier_strategy(),
        identifier_strategy(),
        identifier_strategy(),
    ).prop_map(|(lhs, var1, var2)| format!("{} ~ {} + {}", lhs, var1, var2))
}

// Strategy for generating product formulas
fn product_formula_strategy() -> impl Strategy<Value = String> {
    (
        identifier_strategy(),
        identifier_strategy(),
        identifier_strategy(),
    ).prop_map(|(lhs, var1, var2)| format!("{} ~ {} * {}", lhs, var1, var2))
}

// Strategy for generating polynomial formulas
fn poly_formula_strategy() -> impl Strategy<Value = String> {
    (
        identifier_strategy(),
        identifier_strategy(),
        1..=3u8,
    ).prop_map(|(lhs, var, degree)| format!("{} ~ poly({}, {})", lhs, var, degree))
}

// Strategy for generating complex formulas
fn complex_formula_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "y ~ x1 + x2 + x1:x2".to_string(),
        "y ~ x1 * x2".to_string(),
        "y ~ x1 / x2".to_string(),
        "y ~ poly(x, 2)".to_string(),
        "y ~ I(x)".to_string(),
        "y ~ x^2".to_string(),
    ])
}

proptest! {
    #[test]
    fn test_basic_formula_roundtrip(formula in basic_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse basic formula: {}", formula);
        
        let spec = result.unwrap();
        let pretty_output = pretty(&spec);
        
        // Should be able to reparse the pretty-printed output
        let reparsed = p.parse(pretty_output.as_str());
        prop_assert!(reparsed.is_ok(), "Failed to reparse: {}", pretty_output);
    }
    
    #[test]
    fn test_interaction_formula_roundtrip(formula in interaction_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse interaction formula: {}", formula);
        
        let spec = result.unwrap();
        let pretty_output = pretty(&spec);
        
        // Should be able to reparse the pretty-printed output
        let reparsed = p.parse(pretty_output.as_str());
        prop_assert!(reparsed.is_ok(), "Failed to reparse: {}", pretty_output);
    }
    
    #[test]
    fn test_sum_formula_roundtrip(formula in sum_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse sum formula: {}", formula);
        
        let spec = result.unwrap();
        let pretty_output = pretty(&spec);
        
        // Should be able to reparse the pretty-printed output
        let reparsed = p.parse(pretty_output.as_str());
        prop_assert!(reparsed.is_ok(), "Failed to reparse: {}", pretty_output);
    }
    
    #[test]
    fn test_product_formula_roundtrip(formula in product_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse product formula: {}", formula);
        
        let spec = result.unwrap();
        let canonicalized = canonicalize(&spec);
        let pretty_output = pretty(&canonicalized);
        
        // Should be able to reparse the canonicalized output
        let reparsed = p.parse(pretty_output.as_str());
        prop_assert!(reparsed.is_ok(), "Failed to reparse canonicalized: {}", pretty_output);
    }
    
    #[test]
    fn test_poly_formula_roundtrip(formula in poly_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse poly formula: {}", formula);
        
        let spec = result.unwrap();
        let pretty_output = pretty(&spec);
        
        // Should be able to reparse the pretty-printed output
        let reparsed = p.parse(pretty_output.as_str());
        prop_assert!(reparsed.is_ok(), "Failed to reparse: {}", pretty_output);
    }
    
    #[test]
    fn test_complex_formula_roundtrip(formula in complex_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse complex formula: {}", formula);
        
        let spec = result.unwrap();
        let canonicalized = canonicalize(&spec);
        let pretty_output = pretty(&canonicalized);
        
        // Should be able to reparse the canonicalized output
        let reparsed = p.parse(pretty_output.as_str());
        prop_assert!(reparsed.is_ok(), "Failed to reparse canonicalized: {}", pretty_output);
    }
    
    #[test]
    fn test_canonicalization_idempotence(formula in complex_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse formula: {}", formula);
        
        let spec = result.unwrap();
        let canonicalized1 = canonicalize(&spec);
        let canonicalized2 = canonicalize(&canonicalized1);
        
        // Canonicalization should be idempotent
        let pretty1 = pretty(&canonicalized1);
        let pretty2 = pretty(&canonicalized2);
        prop_assert_eq!(pretty1, pretty2, "Canonicalization not idempotent");
    }
    
    #[test]
    fn test_star_expansion_identity(formula in product_formula_strategy()) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse product formula: {}", formula);
        
        let spec = result.unwrap();
        let canonicalized = canonicalize(&spec);
        
        // After canonicalization, a*b should be equivalent to a + b + a:b
        let pretty_output = pretty(&canonicalized);
        prop_assert!(pretty_output.contains("+"), "Product should expand to sum");
    }
    
    #[test]
    fn test_slash_expansion_identity(formula in prop::sample::select(vec![
        "y ~ x1/x2".to_string(),
        "y ~ a/b".to_string(),
        "y ~ var1/var2".to_string(),
    ])) {
        let p = parser();
        let result = p.parse(formula.as_str());
        prop_assert!(result.is_ok(), "Failed to parse slash formula: {}", formula);
        
        let spec = result.unwrap();
        let canonicalized = canonicalize(&spec);
        
        // After canonicalization, a/b should be equivalent to a + a:b
        let pretty_output = pretty(&canonicalized);
        prop_assert!(pretty_output.contains("+"), "Slash should expand to sum");
    }
    
    #[test]
    fn test_parser_error_handling(invalid_formula in prop::sample::select(vec![
        "".to_string(),
        "~".to_string(),
        "y ~".to_string(),
        "~ x".to_string(),
        "y ~~ x".to_string(),
        "y ~ x +".to_string(),
        "y ~ x *".to_string(),
        "y ~ x:".to_string(),
        "y ~ poly(,2)".to_string(),
        "y ~ s(".to_string(),
        "y ~ s(x,".to_string(),
    ])) {
        let p = parser();
        let result = p.parse(invalid_formula.as_str());
        
        // Invalid formulas should fail to parse
        prop_assert!(result.is_err(), "Invalid formula should fail to parse: {}", invalid_formula);
    }
}
