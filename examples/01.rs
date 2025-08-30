// Example 01
// ==========
//  If you git clone this repo, you can run this example with:
// git clone https://github.com/alexhallam/polars-formula.git
// cd polars-formula
// cargo run --example 01
use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple API Demo ===\n");

    // Load data
    let df = CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;

    // Formula string
    let formula_str = "mpg ~ cyl + wt*hp + poly(disp, 4) - 1";

    // Step 1: Canonicalize (parse and canonicalize)
    println!("\n1. Parse and canonicalize formula");
    let spec = canonicalize(formula_str)?;
    print_formula(&spec);

    // Step 2: Print the full model spec
    println!("\n2. Full model specification:");
    print_modelspec(&spec);

    // Step 3: Materialize the formula
    println!("\n3. Materializing formula");
    let (y, x, _z) = materialize(&spec, &df)?;
    println!("   Results: y={}\n X={}\n", y, x);
    Ok(())
}
