use chumsky::Parser;
use polars::prelude::*;
use polars_formula::dsl::{canon::*, materialize::materialize_dsl_spec, parser::parser, pretty::*};
use polars_formula::{MaterializeOptions, SimpleColoredPretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sleep Study DSL Demo ===\n");

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

    // Define the formula to parse
    let formula_str = "Reaction ~ Days + (Days | Subject)";
    let color_pretty = SimpleColoredPretty::default();
    println!(
        "2. Parsing formula: {}",
        color_pretty.formula(formula_str)
    );

    // Parse the formula using the new DSL parser
    let p = parser();
    let parse_result = p.parse(formula_str.chars().collect::<Vec<_>>());

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
            println!("   {}", color_pretty.formula(&pretty_output));
            println!();

            // Canonicalize the formula
            println!("4. Canonicalizing formula...");
            let canonicalized = canonicalize(&spec);
            let canonical_pretty = pretty(&canonicalized);
            println!(
                "   Canonicalized: {}",
                color_pretty.formula(&canonical_pretty)
            );
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
                    println!("     - First 5 rows:");
                    println!("{}", y.head(Some(5)));
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
            let reparsed = p.parse(pretty_output.as_str().chars().collect::<Vec<_>>());
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
            let basic_result = p.parse("Reaction ~ Days".chars().collect::<Vec<_>>());
            println!("   'Reaction ~ Days': {:?}", basic_result.is_ok());

            // Try with group
            let group_result = p.parse("Reaction ~ (Days | Subject)".chars().collect::<Vec<_>>());
            println!(
                "   'Reaction ~ (Days | Subject)': {:?}",
                group_result.is_ok()
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
    println!("   ✅ Groups: (1|group), (Days|Subject) - now implemented!");
    println!("   ⚠️  Smooths: s(x, k=10) - not yet implemented");
    println!("   ⚠️  Aterms: y | weights(w) ~ x - not yet implemented");
    println!("   ⚠️  Family: family=gaussian() y ~ x - not yet implemented");
    println!();

    // Show some working examples with sleep study data
    println!("8. Working examples with sleep study data:");

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
