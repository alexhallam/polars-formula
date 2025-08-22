use polars::prelude::*;
use polars_formula::{make_clean_names, Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 polars-formula Clean Names Demo\n");

    // Create sample data
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x1" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x2" => [2.0, 3.0, 4.0, 5.0, 6.0]
    )?;

    println!("📊 Sample Data:");
    println!("{:?}", df);

    // Test the make_clean_names function directly
    println!("\n🔧 make_clean_names() Examples:");
    let examples = vec![
        "poly(x,2)^1",
        "x1:x2",
        "treatment:dose",
        "Column Name!",
        "poly(income,3)^2",
    ];

    for example in examples {
        let cleaned = make_clean_names(example);
        println!("  '{}' → '{}'", example, cleaned);
    }

    // Test formula materialization with clean names (default)
    println!("\n📈 Formula Materialization with Clean Names (default):");
    let formula = Formula::parse("y ~ x1 + poly(x2,2) + x1:x2")?;
    let (y, X) = formula.materialize(&df, MaterializeOptions::default())?;

    println!("  Response variable: {}", y.name());
    println!("  Design matrix columns:");
    for (i, name) in X.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    // Test formula materialization without clean names
    println!("\n📈 Formula Materialization without Clean Names:");
    let opts = MaterializeOptions {
        clean_names: false,
        ..Default::default()
    };
    let (y, X) = formula.materialize(&df, opts)?;

    println!("  Response variable: {}", y.name());
    println!("  Design matrix columns:");
    for (i, name) in X.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    println!("\n✅ Demo completed successfully!");
    Ok(())
}
