use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple dataset
    let df: DataFrame = CsvReader::new(std::fs::File::open("examples/data/cake.csv")?).finish()?;

    // Original formula
    let formula_str = "angle ~ recipe * temperature + (1 | recipe:replicate)";
    println!("Original: {}", formula_str);

    // Step 1: Parse and canonicalize
    let spec = canonicalize(formula_str)?;

    // Step 2: Print canonical formula with colors
    println!("Canonicalized:");
    print_formula(&spec);

    // Step 3: Print full model spec
    println!("\nFull model specification:");
    print_modelspec(&spec);

    // Step 4: Materialize the formula
    let (y, x, z) = materialize(&spec, &df)?;

    // Print the results
    println!("\nResults:");
    println!("y: {}", y);
    println!("X: {}", x);
    println!("Z: {}", z);

    Ok(())
}
