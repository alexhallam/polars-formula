use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // print working that this is still in development
    println!("🚧 This example is still in development is not working as expected.");

    // Load data
    let df = CsvReader::new(std::fs::File::open("examples/data/cbpp.csv")?).finish()?;

    // Formula string
    let formula_str: &'static str =
        "incidence | trials(size) ~ period + (1|herd), family = binomial()";

    // Step 1: Parse and canonicalize
    println!("Original:  {}", formula_str);
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
