use polars::prelude::*;
use polars_formula::{make_clean_names, Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 polars-formula basic formula demo\n");

    // Read data from CSV file
    println!("📊 Loading data from mtcars.csv...");

    let df = CsvReader::new(std::fs::File::open("examples/mtcars.csv")?).finish()?;

    println!("{}", df);

    // Test complex formula with multiple interactions
    let complex_formula_text = "mpg ~ wt + hp + cyl + wt:hp + poly(disp, 2) - 1";
    println!("Formula: {}", complex_formula_text);
    let complex_formula: Formula = Formula::parse(complex_formula_text)?;
    let (y, x) = complex_formula.materialize(&df, MaterializeOptions::default())?;

    println!("  Response variable: {}", y.name());
    println!("  Design matrix columns:");
    for (i, name) in x.get_column_names().iter().enumerate() {
        println!("    {}: {}", i, name.as_str());
    }

    println!("\n✅ Demo completed successfully!");
    Ok(())
}
