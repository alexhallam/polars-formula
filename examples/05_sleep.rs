use polars::prelude::*;
use polars_formula::{canonicalize, materialize, print_formula, print_modelspec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sleep Study Demo ===\n");

    // Load the sleep study data
    println!("1. Loading sleep_study.csv data...");
    let df = CsvReader::new(std::fs::File::open("examples/data/sleep_study.csv")?).finish()?;

    println!(
        "   DataFrame shape: {} rows × {} columns",
        df.height(),
        df.width()
    );
    println!("   Columns: {:?}", df.get_column_names());
    println!();

    // Show first few rows
    println!("   First 10 rows:");
    println!("{}", df.head(Some(10)));
    println!();

    // Show data summary
    println!("   Data summary:");
    println!(
        "   - Number of subjects: {}",
        df.column("Subject")?.unique()?.len()
    );
    println!(
        "   - Days range: {} to {}",
        df.column("Days")?.i64()?.min().unwrap(),
        df.column("Days")?.i64()?.max().unwrap()
    );
    println!(
        "   - Reaction range: {:.2} to {:.2}",
        df.column("Reaction")?.f64()?.min().unwrap(),
        df.column("Reaction")?.f64()?.max().unwrap()
    );
    println!();

    // Define the formula to parse - testing both syntaxes
    let formula_str = "Reaction ~ 1 + Days + (1 + Days|Subject)";
    println!("2. Original formula: {}", formula_str);

    // Canonicalize the formula
    println!("3. Canonicalizing formula...");
    let spec = canonicalize(formula_str)?;
    println!("   Canonicalized:");
    print_formula(&spec);

    // Print full model specification
    println!("4. Full model specification:");
    print_modelspec(&spec);

    // Materialize the formula
    println!("5. Materializing formula...");
    let (y, x, z) = materialize(&spec, &df)?;
    println!("   ✅ Materialization successful!");
    println!("   Response variable (y):");
    println!("     - Shape: {} rows × {} columns", y.height(), y.width());
    println!("     - Column names: {:?}", y.get_column_names());
    println!("     - First 5 rows:");
    println!("{}", y.head(Some(5)));

    println!("   Fixed effects design matrix (X):");
    println!("     - Shape: {} rows × {} columns", x.height(), x.width());
    println!("     - Column names: {:?}", x.get_column_names());
    println!("     - First 5 rows:");
    println!("{}", x.head(Some(5)));

    println!("   Random effects design matrix (Z):");
    println!("     - Shape: {} rows × {} columns", z.height(), z.width());
    println!("     - Column names: {:?}", z.get_column_names());
    if z.height() > 0 {
        println!("     - First 5 rows:");
        println!("{}", z.head(Some(5)));
    } else {
        println!("     - No random effects columns");
    }

    // Test the alternative syntax
    println!("\n6. Testing alternative syntax: (Days|Subject)");
    let formula_str2 = "Reaction ~ 1 + Days + (Days|Subject)";
    println!("   Formula: {}", formula_str2);

    let spec2 = canonicalize(formula_str2)?;
    println!("   Canonicalized:");
    print_formula(&spec2);

    let (y2, x2, z2) = materialize(&spec2, &df)?;
    println!(
        "   Z matrix shape: {} rows × {} columns",
        z2.height(),
        z2.width()
    );
    println!("   Z matrix columns: {:?}", z2.get_column_names());

    // Show some working examples with sleep study data
    println!("\n7. Working examples with sleep study data:");

    let working_examples = vec![
        "Reaction ~ Days",
        "Reaction ~ Subject",
        "Reaction ~ Days + Subject",
        "Reaction ~ Days * Subject",
        "Reaction ~ Days:Subject",
        "Reaction ~ poly(Days, 2)",
        "Reaction ~ (1|Subject)",
        "Reaction ~ (Days|Subject)",
        "Reaction ~ Days + (1|Subject)",
        "Reaction ~ Days + (Days|Subject)",
    ];

    for example in working_examples {
        match canonicalize(example) {
            Ok(_spec) => {
                println!("   ✅ {} → parsed successfully", example);
            }
            Err(_) => {
                println!("   ❌ {} → failed to parse", example);
            }
        }
    }

    println!();
    println!("=== Demo Complete ===");

    Ok(())
}
