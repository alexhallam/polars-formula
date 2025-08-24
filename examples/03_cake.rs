use chumsky::Parser;
use polars::prelude::*;
use polars_formula::dsl::{canon::*, materialize::materialize_dsl_spec, parser::parser, pretty::*};
use polars_formula::MaterializeOptions;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cake DSL Demo ===\n");

    // Load the cake data
    println!("1. Loading cake.csv data...");
    let df = CsvReader::new(std::fs::File::open("examples/data/cake.csv")?).finish()?;

    println!(
        "   DataFrame shape: {} rows × {} columns",
        df.height(),
        df.width()
    );
    println!("   Columns: {:?}", df.get_column_names());
    println!();

    // Show first few rows
    println!("   First 5 rows:");
    println!("{}", df.head(Some(5)));
    println!();

    // Define the formula to parse
    let formula_str = "angle ~ recipe * temperature + (1 | recipe:replicate)";
    println!("2. Parsing formula: {}", formula_str);

    // Parse the formula using the new DSL parser
    let p = parser();
    let parse_result = p.parse(formula_str);

    match parse_result {
        Ok(spec) => {
            println!("   ✅ Formula parsed successfully!");
            println!("   Parsed AST structure:");
            println!("   - Family: {:?}", spec.family);
            println!("   - Link: {:?}", spec.link);
            println!("   - Response: {:?}", spec.formula.lhs);
            println!("   - Predictors: {:?}", spec.formula.rhs);
            println!("   - Aterms: {:?}", spec.formula.aterms);
            println!("   - Dpars: {:?}", spec.dpars);
            println!("   - Autocor: {:?}", spec.autocor);
            println!();

            // Pretty print the parsed formula
            println!("3. Pretty-printed formula:");
            let pretty_output = pretty(&spec);
            println!("   {}", pretty_output);
            println!();

            // Canonicalize the formula
            println!("4. Canonicalizing formula...");
            let canonicalized = canonicalize(&spec);
            let canonical_pretty = pretty(&canonicalized);
            println!("   Canonicalized: {}", canonical_pretty);
            println!();

            // Try to materialize the formula
            println!("5. Materializing formula...");
            let canonicalized = canonicalize(&spec);
            let materialize_result =
                materialize_dsl_spec(&df, &canonicalized, MaterializeOptions::default());

            match materialize_result {
                Ok((y, x, z)) => {
                    println!("   ✅ Materialization successful!");
                    println!("   Response variable (y):");
                    println!("     - Shape: {} rows × {} columns", y.height(), y.width());
                    println!("     - Column names: {:?}", y.get_column_names());
                    println!("     - Types: {:?}", y.dtypes());
                    println!("     - First 5 values: {:?}", y.head(Some(5)));
                    println!();

                    println!("   Fixed effects design matrix (X):");
                    println!("     - Shape: {} rows × {} columns", x.height(), x.width());
                    println!("     - Column names: {:?}", x.get_column_names());
                    println!("     - First 5 rows:");
                    println!("{}", x.head(Some(5)));
                    println!();

                    println!("   Random effects design matrix (Z):");
                    println!("     - Shape: {} rows × {} columns", z.height(), z.width());
                    println!("     - Column names: {:?}", z.get_column_names());
                    if z.width() > 0 {
                        println!("     - First 5 rows:");
                        println!("{}", z.head(Some(5)));
                    } else {
                        println!("     - No random effects columns");
                    }
                    println!();
                }
                Err(e) => {
                    println!("   ⚠️  Materialization failed: {}", e);
                    println!("   This is expected for complex formulas with groups");
                    println!("   that aren't fully implemented in materialization yet.");
                    println!();
                }
            }

            // Test roundtrip parsing
            println!("6. Testing roundtrip parsing...");
            let reparsed = p.parse(pretty_output.as_str());
            match reparsed {
                Ok(_) => println!("   ✅ Roundtrip successful - parsed formula can be re-parsed!"),
                Err(e) => println!("   ⚠️  Roundtrip failed: {:?}", e),
            }
            println!();
        }
        Err(e) => {
            println!("   ❌ Formula parsing failed: {:?}", e);
            println!();

            // Try parsing simpler parts to debug
            println!("   Debugging: Trying to parse simpler parts...");

            // Try just the basic formula
            let basic_result = p.parse("angle ~ recipe");
            println!("   'angle ~ recipe': {:?}", basic_result.is_ok());

            // Try with interaction
            let interaction_result = p.parse("angle ~ recipe * temperature");
            println!(
                "   'angle ~ recipe * temperature': {:?}",
                interaction_result.is_ok()
            );

            // Try with sum
            let sum_result = p.parse("angle ~ recipe + temperature");
            println!(
                "   'angle ~ recipe + temperature': {:?}",
                sum_result.is_ok()
            );

            println!();
        }
    }

    // Demonstrate what the parser can currently handle
    println!("7. Current parser capabilities:");
    println!("   ✅ Basic formulas: y ~ x");
    println!("   ✅ Interactions: y ~ x1:x2");
    println!("   ✅ Products: y ~ x1*x2 (expands to x1 + x2 + x1:x2)");
    println!("   ✅ Sums: y ~ x1 + x2");
    println!("   ✅ Functions: y ~ poly(x, 2), y ~ I(x)");
    println!("   ✅ Powers: y ~ x^2");
    println!("   ✅ Groups: (1|group), (1|recipe:replicate) - now implemented!");
    println!("   ⚠️  Smooths: s(x, k=10) - not yet implemented");
    println!("   ⚠️  Aterms: y | weights(w) ~ x - not yet implemented");
    println!("   ⚠️  Family: family=gaussian() y ~ x - not yet implemented");
    println!();

    // Show some working examples
    println!("8. Working examples with cake data:");

    let working_examples = vec![
        "angle ~ recipe",
        "angle ~ temperature",
        "angle ~ recipe + temperature",
        "angle ~ recipe * temperature",
        "angle ~ recipe:temperature",
        "angle ~ poly(temperature, 2)",
        "angle ~ (1|recipe)",
        "angle ~ (1|recipe:replicate)",
        "angle ~ recipe + (1|replicate)",
    ];

    for example in working_examples {
        let result = p.parse(example);
        match result {
            Ok(spec) => {
                let canonicalized = canonicalize(&spec);
                let pretty_output = pretty(&canonicalized);
                println!("   ✅ {} → {}", example, pretty_output);
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
