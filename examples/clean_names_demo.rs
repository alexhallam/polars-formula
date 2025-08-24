use polars::prelude::*;
use polars_formula::{make_clean_names, Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 polars-formula Clean Names Demo\n");

    // Read data from CSV file
    println!("📊 Loading data from mtcars.csv...");

    let df = CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;

    println!("{}", df);

    // Test the make_clean_names function directly
    println!("\n🔧 make_clean_names() Examples:");
    let examples = vec![
        "poly(x,2)^1",
        "x1:x2",
        "treatment:dose",
        "Column Name!",
        "poly(income,3)^2",
        "mpg",
        "wt",
        "hp",
        "wt:hp",
        "poly(wt,2)^1",
    ];

    for example in examples {
        let cleaned = make_clean_names(example);
        println!("  '{}' → '{}'", example, cleaned);
    }

    // Test formula materialization with real data
    println!("\n📈 Formula Materialization with Real Data:");
    let formula_text = "mpg ~ wt + hp + wt:hp";
    println!("Formula: {}", formula_text);
    let formula = Formula::parse(formula_text)?;
    let (y, x) = formula.materialize(&df, MaterializeOptions::default())?;

    println!("  Response variable columns: {:?}", y.get_column_names());
    println!("  Design matrix columns:");
    for (i, name) in x.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    // Test formula materialization without clean names
    println!("\n📈 Formula Materialization without Clean Names:");
    let opts = MaterializeOptions {
        clean_names: false,
        ..Default::default()
    };
    let (y, x) = formula.materialize(&df, opts)?;

    println!("  Response variable columns: {:?}", y.get_column_names());
    println!("  Design matrix columns:");
    for (i, name) in x.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    // Test polynomial regression with real data
    println!("\n📈 Polynomial Regression Example:");
    let poly_formula_text = "mpg ~ poly(wt, 2)";
    println!("Formula: {}", poly_formula_text);
    let poly_formula = Formula::parse(poly_formula_text)?;
    let (y, x) = poly_formula.materialize(&df, MaterializeOptions::default())?;

    println!("  Response variable columns: {:?}", y.get_column_names());
    println!("  Design matrix columns:");
    for (i, name) in x.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    // Test complex formula with multiple interactions
    println!("\n📈 Complex Formula Example:");
    let complex_formula_text = "mpg ~ wt + hp + cyl + wt:hp + poly(disp, 2)";
    println!("Formula: {}", complex_formula_text);
    let complex_formula = Formula::parse(complex_formula_text)?;
    let (y, x) = complex_formula.materialize(&df, MaterializeOptions::default())?;

    println!("  Response variable columns: {:?}", y.get_column_names());
    println!("  Design matrix columns:");
    for (i, name) in x.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    println!("\n✅ Demo completed successfully!");
    Ok(())
}
