use polars::prelude::*; // DataFrame andCsvReader
use polars_formula::{Color, Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple dataset
    let df: DataFrame = CsvReader::new(std::fs::File::open("examples/data/cake.csv")?).finish()?;

    // Original formula
    let formula_str = "angle ~ recipe * temperature + (1 | recipe:replicate)";
    println!("Original: {}", formula_str);

    // Colored version (original syntax preserved)
    let color_pretty = Color::default();
    println!("Colored:  {}", color_pretty.formula(formula_str));

    // Canonicalized version (for comparison)
    println!("Canonicalized: {}", color_pretty.formula(formula_str));

    // Materialize the formula
    let formula = Formula::parse(formula_str)?;
    let (y, x) = formula.materialize(&df, MaterializeOptions::default())?;

    // Print the results
    println!("y: {}", y);
    println!("X: {}", x);

    Ok(())
}
