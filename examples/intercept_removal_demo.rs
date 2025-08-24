use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Intercept Removal Demo with -1 Syntax\n");

    // Create a simple dataset
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x1" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x2" => [2.0, 4.0, 6.0, 8.0, 10.0]
    )?;

    println!("Dataset:");
    println!("{}", df);

    // Test 1: Default formula (with intercept)
    println!("\n📊 Test 1: Default formula (with intercept)");
    println!("Formula: 'y ~ x1 + x2'");
    let formula1 = Formula::parse("y ~ x1 + x2")?;
    let (y1, X1) = formula1.materialize(&df, MaterializeOptions::default())?;

    println!("Response variable shape: {} rows × {} columns", y1.height(), y1.width());
    println!("Design matrix columns:");
    for (i, name) in X1.get_column_names().iter().enumerate() {
        println!("  {}: {}", i, name);
    }
    println!(
        "Design matrix shape: {} rows × {} columns",
        X1.height(),
        X1.width()
    );

    // Test 2: Formula with -1 (without intercept)
    println!("\n📊 Test 2: Formula with -1 (without intercept)");
    println!("Formula: 'y ~ x1 + x2 - 1'");
    let formula2 = Formula::parse("y ~ x1 + x2 - 1")?;
    let (y2, X2) = formula2.materialize(&df, MaterializeOptions::default())?;

    println!("Response variable shape: {} rows × {} columns", y2.height(), y2.width());
    println!("Design matrix columns:");
    for (i, name) in X2.get_column_names().iter().enumerate() {
        println!("  {}: {}", i, name);
    }
    println!(
        "Design matrix shape: {} rows × {} columns",
        X2.height(),
        X2.width()
    );

    // Test 3: Compare with MaterializeOptions
    println!("\n📊 Test 3: Compare with MaterializeOptions");
    println!("Formula: 'y ~ x1 + x2' with rhs_intercept: false");
    let opts_no_intercept = MaterializeOptions {
        rhs_intercept: false,
        ..Default::default()
    };
    let (y3, X3) = formula1.materialize(&df, opts_no_intercept)?;

    println!("Response variable shape: {} rows × {} columns", y3.height(), y3.width());
    println!("Design matrix columns:");
    for (i, name) in X3.get_column_names().iter().enumerate() {
        println!("  {}: {}", i, name);
    }
    println!(
        "Design matrix shape: {} rows × {} columns",
        X3.height(),
        X3.width()
    );

    // Test 4: Complex formula with -1
    println!("\n📊 Test 4: Complex formula with -1");
    println!("Formula: 'y ~ x1 + x2 + x1:x2 + poly(x1, 2) - 1'");
    let formula4 = Formula::parse("y ~ x1 + x2 + x1:x2 + poly(x1, 2) - 1")?;
    let (y4, X4) = formula4.materialize(&df, MaterializeOptions::default())?;

    println!("Response variable shape: {} rows × {} columns", y4.height(), y4.width());
    println!("Design matrix columns:");
    for (i, name) in X4.get_column_names().iter().enumerate() {
        println!("  {}: {}", i, name);
    }
    println!(
        "Design matrix shape: {} rows × {} columns",
        X4.height(),
        X4.width()
    );

    // Verify that -1 and MaterializeOptions produce the same result
    println!("\n✅ Verification:");
    println!(
        "Test 2 (-1 syntax) and Test 3 (MaterializeOptions) should have the same number of columns"
    );
    println!("Test 2 columns: {}", X2.width());
    println!("Test 3 columns: {}", X3.width());
    assert_eq!(
        X2.width(),
        X3.width(),
        "Both methods should produce the same number of columns"
    );

    println!("\n🎉 All tests passed! The -1 syntax correctly removes the intercept.");
    Ok(())
}
