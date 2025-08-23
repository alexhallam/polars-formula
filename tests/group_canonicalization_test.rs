use chumsky::Parser;
use polars_formula::dsl::{canon::*, parser::parser, pretty::pretty};

#[test]
fn test_group_canonicalization() {
    let p = parser();

    // Test 1: Simple random intercept
    let formula1 = "y ~ (1|group)";
    let parsed1 = p.parse(formula1).unwrap();
    let canonicalized1 = canonicalize(&parsed1);
    let pretty1 = pretty(&canonicalized1);
    assert_eq!(pretty1, "y ~ (1|group)");

    // Test 2: Random slope - should expand to random intercept + random slope
    let formula2 = "y ~ (x|group)";
    let parsed2 = p.parse(formula2).unwrap();
    let canonicalized2 = canonicalize(&parsed2);
    let pretty2 = pretty(&canonicalized2);
    assert_eq!(pretty2, "y ~ (1|group) + (0 + x|group)");

    // Test 3: Mixed model with fixed and random effects
    let formula3 = "y ~ x + (x|group)";
    let parsed3 = p.parse(formula3).unwrap();
    let canonicalized3 = canonicalize(&parsed3);
    let pretty3 = pretty(&canonicalized3);
    assert_eq!(pretty3, "y ~ x + (1|group) + (0 + x|group)");

    // Test 4: Sleep study example
    let formula4 = "Reaction ~ Days + (Days|Subject)";
    let parsed4 = p.parse(formula4).unwrap();
    let canonicalized4 = canonicalize(&parsed4);
    let pretty4 = pretty(&canonicalized4);
    assert_eq!(
        pretty4,
        "Reaction ~ Days + (1|Subject) + (0 + Days|Subject)"
    );

    // Test 5: Complex nested groups
    let formula5 = "y ~ (x|group1:group2)";
    let parsed5 = p.parse(formula5).unwrap();
    let canonicalized5 = canonicalize(&parsed5);
    let pretty5 = pretty(&canonicalized5);
    assert_eq!(pretty5, "y ~ (1|group1:group2) + (0 + x|group1:group2)");

    println!("✅ All group canonicalization tests passed!");
    println!("   Random intercept: {} → {}", formula1, pretty1);
    println!("   Random slope: {} → {}", formula2, pretty2);
    println!("   Mixed model: {} → {}", formula3, pretty3);
    println!("   Sleep study: {} → {}", formula4, pretty4);
    println!("   Nested groups: {} → {}", formula5, pretty5);
}
