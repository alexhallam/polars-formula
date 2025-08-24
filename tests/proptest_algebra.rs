use polars_formula::dsl::{parser::parser, canon::canonicalize};
use chumsky::Parser;

#[test]
fn star_equals_main_plus_interact() {
    let p = parser();
    let a = p.parse("y ~ x*y").unwrap();
    let b = p.parse("y ~ x + y + x:y").unwrap();
    assert_eq!(canonicalize(&a), canonicalize(&b));
}

#[test]
fn nesting_shorthand() {
    let p = parser();
    let a = p.parse("y ~ a/b").unwrap();
    let b = p.parse("y ~ a + a:b").unwrap();
    assert_eq!(canonicalize(&a), canonicalize(&b));
}

#[test]
fn intercept_normalization() {
    let p = parser();
    let a = p.parse("y ~ 1 + x - 1").unwrap();
    let b = p.parse("y ~ x - 1").unwrap();
    assert_eq!(canonicalize(&a), canonicalize(&b));
}
