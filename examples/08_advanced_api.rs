use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced API Demo ===\n");

    // Load CBPP data for mixed effects example
    let df = CsvReader::new(std::fs::File::open("examples/data/cbpp.csv")?).finish()?;
    println!("1. Loaded CBPP data: {} rows × {} columns", df.height(), df.width());

    // Example 1: Mixed effects model
    println!("\n2. Mixed Effects Model Example");
    let mixed_formula = "incidence | trials(size) ~ period + (1|herd), family = binomial()";
    println!("   Formula: {}", mixed_formula);
    
    let spec = canonicalize(mixed_formula)?;
    println!("   Canonical form:");
    print_formula(&spec);
    
    let (y, x, z) = materialize(&spec, &df)?;
    println!("   Results: y={}x{}, X={}x{}, Z={}x{}", 
             y.height(), y.width(), x.height(), x.width(), z.height(), z.width());

    // Example 2: Complex interactions
    println!("\n3. Complex Interactions Example");
    let interaction_formula = "mpg ~ (cyl + wt) * hp + poly(disp, 3)";
    println!("   Formula: {}", interaction_formula);
    
    let spec2 = canonicalize(interaction_formula)?;
    println!("   Canonical form:");
    print_formula(&spec2);
    
    // Load mtcars for this example
    let mtcars_df = CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;
    let (y2, x2, z2) = materialize(&spec2, &mtcars_df)?;
    println!("   Results: y={}x{}, X={}x{}, Z={}x{}", 
             y2.height(), y2.width(), x2.height(), x2.width(), z2.height(), z2.width());
    println!("   X columns: {:?}", x2.get_column_names());

    println!("\n=== Advanced Demo Complete ===");
    Ok(())
}
