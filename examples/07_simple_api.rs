use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple API Demo ===\n");

    // Load data
    let df = CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;
    println!(
        "1. Loaded mtcars data: {} rows × {} columns",
        df.height(),
        df.width()
    );

    // Formula string
    let formula_str = "mpg ~ cyl + wt*hp + poly(disp, 2) - 1";
    println!("\n2. Original formula: {}", formula_str);

    // Step 1: Canonicalize (parse and canonicalize)
    println!("\n3. Canonicalizing formula...");
    let spec = canonicalize(formula_str)?;

    // Step 2: Print the canonical formula with colors
    println!("   Canonical formula:");
    print_formula(&spec);

    // Step 3: Print the full model spec
    println!("\n4. Full model specification:");
    print_modelspec(&spec);

    // Step 4: Materialize the formula
    println!("\n5. Materializing formula");
    let (y, x, z) = materialize(&spec, &df)?;
    println!("   Results: y={}\n, X={}\n, Z={}", y, x, z);
    Ok(())
}
