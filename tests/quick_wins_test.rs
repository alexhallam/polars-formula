use chumsky::Parser;
use polars::prelude::*;
use polars_formula::dsl::{canon::canonicalize, materialize::materialize, parser::parser};
use polars_formula::MaterializeOptions;

#[test]
fn test_current_broken_behaviors() {
    // Create test data
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "x1" => [1.0, 2.0, 3.0, 4.0],
        "x2" => [2.0, 3.0, 4.0, 5.0],
        "species" => ["setosa", "versicolor", "virginica", "setosa"]
    )
    .unwrap();

    let p = parser();

    // Test 1: poly() currently returns original series
    println!("Testing poly() function...");
    let poly_spec = p
        .parse("y ~ poly(x1, 2)".chars().collect::<Vec<_>>())
        .unwrap();
    let poly_canon = canonicalize(&poly_spec);
    let (_, x_poly, _) = materialize(&df, &poly_canon, MaterializeOptions::default()).unwrap();

    println!("  poly(x1, 2) returned {} columns", x_poly.width());
    println!("  Expected: 3 columns (intercept + x1, x1²)");
    println!("  Actual: {} column(s)", x_poly.width());

    // This should now pass - poly() is fixed!
    assert_eq!(
        x_poly.width(),
        3,
        "poly() should return 3 columns (intercept + 2 polynomial terms)"
    );

    // Test 2: Interactions currently don't work
    println!("Testing interactions...");
    let interaction_spec = p.parse("y ~ x1:x2".chars().collect::<Vec<_>>()).unwrap();
    let interaction_canon = canonicalize(&interaction_spec);
    let (_, x_interaction, _) =
        materialize(&df, &interaction_canon, MaterializeOptions::default()).unwrap();

    println!("  x1:x2 returned {} columns", x_interaction.width());
    println!("  Expected: 1 column (x1 * x2)");
    println!("  Actual: {} column(s)", x_interaction.width());

    // This should now pass - interactions are fixed!
    assert_eq!(
        x_interaction.width(),
        2,
        "x1:x2 should return 2 columns (intercept + interaction)"
    );

    // Test 3: Categorical variables not handled
    println!("Testing categorical variables...");
    let cat_spec = p.parse("y ~ species".chars().collect::<Vec<_>>()).unwrap();
    let cat_canon = canonicalize(&cat_spec);
    let (_, x_cat, _) = materialize(&df, &cat_canon, MaterializeOptions::default()).unwrap();

    println!("  species returned {} columns", x_cat.width());
    println!("  Expected: 2 columns (species_versicolor, species_virginica)");
    println!("  Actual: {} column(s)", x_cat.width());

    // This should now pass - categorical contrasts are implemented!
    assert_eq!(
        x_cat.width(),
        3,
        "species should return 3 columns (intercept + 2 contrasts)"
    );
}

#[test]
fn test_what_should_work() {
    // Create test data
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "x1" => [1.0, 2.0, 3.0, 4.0],
        "x2" => [2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();

    let p = parser();

    // Basic formulas should work
    let basic_spec = p.parse("y ~ x1 + x2".chars().collect::<Vec<_>>()).unwrap();
    let basic_canon = canonicalize(&basic_spec);
    let (y, x, z) = materialize(&df, &basic_canon, MaterializeOptions::default()).unwrap();

    assert_eq!(y.width(), 1, "Response should have 1 column");
    assert_eq!(
        x.width(),
        3,
        "Fixed effects should have 3 columns (intercept + x1 + x2)"
    );
    assert_eq!(z.width(), 0, "Random effects should be empty");

    // Identity function should work
    let identity_spec = p.parse("y ~ I(x1)".chars().collect::<Vec<_>>()).unwrap();
    let identity_canon = canonicalize(&identity_spec);
    let (_, x_identity, _) =
        materialize(&df, &identity_canon, MaterializeOptions::default()).unwrap();

    assert_eq!(
        x_identity.width(),
        2,
        "I(x1) should have 2 columns (intercept + x1)"
    );
}

// TODO: Add these tests once the features are implemented

#[test]
#[ignore]
fn test_poly_should_work() {
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "x" => [1.0, 2.0, 3.0, 4.0]
    )
    .unwrap();

    let p = parser();
    let spec = p
        .parse("y ~ poly(x, 2)".chars().collect::<Vec<_>>())
        .unwrap();
    let canon = canonicalize(&spec);
    let (_, x, _) = materialize(&df, &canon, MaterializeOptions::default()).unwrap();

    // Should return 2 columns: x and x²
    assert_eq!(x.width(), 2, "poly(x, 2) should return 2 columns");

    // Check column names
    let col_names: Vec<_> = x.get_column_names();
    assert!(
        col_names.iter().any(|name| name.as_str().contains("poly")),
        "Should have polynomial column"
    );
}

#[test]
#[ignore]
fn test_interactions_should_work() {
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "x1" => [1.0, 2.0, 3.0, 4.0],
        "x2" => [2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();

    let p = parser();
    let spec = p.parse("y ~ x1:x2".chars().collect::<Vec<_>>()).unwrap();
    let canon = canonicalize(&spec);
    let (_, x, _) = materialize(&df, &canon, MaterializeOptions::default()).unwrap();

    // Should return 1 column: x1 * x2
    assert_eq!(x.width(), 1, "x1:x2 should return 1 interaction column");

    // Check column name
    let col_names: Vec<_> = x.get_column_names();
    assert!(
        col_names
            .iter()
            .any(|name| name.as_str().contains("x1") && name.as_str().contains("x2")),
        "Should have interaction column"
    );
}

#[test]
#[ignore]
fn test_categorical_contrasts_should_work() {
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "species" => ["setosa", "versicolor", "virginica", "setosa"]
    )
    .unwrap();

    let p = parser();
    let spec = p.parse("y ~ species".chars().collect::<Vec<_>>()).unwrap();
    let canon = canonicalize(&spec);

    // Test treatment contrasts (default)
    let opts = MaterializeOptions::default();
    let (_, x, _) = materialize(&df, &canon, opts).unwrap();

    // Should return 2 columns: species_versicolor, species_virginica
    assert_eq!(
        x.width(),
        2,
        "species with treatment contrasts should return 2 columns"
    );

    // Check column names
    let col_names: Vec<_> = x.get_column_names();
    assert!(
        col_names
            .iter()
            .any(|name| name.as_str().contains("species")),
        "Should have species column"
    );
}

#[test]
#[ignore]
fn test_sparse_random_effects_should_work() {
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "group" => ["A", "B", "A", "B"]
    )
    .unwrap();

    let p = parser();
    let spec = p
        .parse("y ~ (1|group)".chars().collect::<Vec<_>>())
        .unwrap();
    let canon = canonicalize(&spec);
    let (_, x, z) = materialize(&df, &canon, MaterializeOptions::default()).unwrap();

    // Should return sparse Z matrix with 1 column (for group B)
    assert_eq!(
        z.width(),
        1,
        "Random intercept should return 1 sparse column"
    );

    // Check that Z is actually sparse (not dense one-hot)
    // This would require checking the internal representation
    // For now, just check it's not empty
    assert!(z.width() > 0, "Random effects should not be empty");
}
