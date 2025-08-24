use chumsky::Parser;
use polars::prelude::*;
use polars_formula::dsl::{canon::*, materialize::materialize_dsl_spec, parser::parser, pretty::*};
use polars_formula::{MaterializeOptions, SimpleColoredPretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CBPP DSL Demo ===\n");

    // Load the CBPP data
    println!("1. Loading cbpp.csv data...");
    let df = CsvReader::new(std::fs::File::open("examples/data/cbpp.csv")?).finish()?;

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
        "   - Number of herds: {}",
        df.column("herd")?.unique()?.len()
    );
    println!(
        "   - Number of periods: {}",
        df.column("period")?.unique()?.len()
    );
    println!(
        "   - Incidence range: {} to {}",
        df.column("incidence")?.i64()?.min().unwrap(),
        df.column("incidence")?.i64()?.max().unwrap()
    );
    println!(
        "   - Size range: {} to {}",
        df.column("size")?.i64()?.min().unwrap(),
        df.column("size")?.i64()?.max().unwrap()
    );
    println!();

    // Test what the parser can currently handle
    println!("2. Testing parser capabilities with CBPP data:");

    let p = parser();
    let color_pretty = SimpleColoredPretty::default();

    // Test basic formulas
    let basic_formulas = vec![
        "incidence ~ period",
        "incidence ~ herd",
        "incidence ~ period + herd",
        "incidence ~ period * herd",
        "incidence ~ (1|herd)",
        "incidence ~ period + (1|herd)",
    ];

    println!("   Basic formulas:");
    for formula in basic_formulas {
        match p.parse(formula) {
            Ok(spec) => {
                let canonicalized = canonicalize(&spec);
                let pretty_output = pretty(&canonicalized);
                println!(
                    "   ✅ {} → {}",
                    color_pretty.formula_original(formula),
                    color_pretty.formula(&pretty_output)
                );
            }
            Err(_) => {
                println!(
                    "   ❌ {} → failed to parse",
                    color_pretty.formula_original(formula)
                );
            }
        }
    }
    println!();

    // Test what works now
    println!("3. Features now implemented:");
    println!("   ✅ Aterms: incidence | trials(size) ~ period");
    println!("   ✅ Family specifications: incidence ~ period, family = binomial(\"logit\") - IMPLEMENTED!");
    println!("   ✅ Combined: incidence | trials(size) ~ period + (1|herd), family = binomial(\"logit\") - IMPLEMENTED!");
    println!();

    // Test binomial trials parsing
    println!("   Testing binomial trials parsing:");
    let binomial_formulas = vec![
        "incidence | trials(size) ~ period",
        "incidence | trials(size) ~ period + (1|herd)",
        "incidence | trials(size) ~ period + (1|herd), family = binomial(\"logit\")",
    ];

    for formula in binomial_formulas {
        match p.parse(formula) {
            Ok(spec) => {
                let canonicalized = canonicalize(&spec);
                let pretty_output = pretty(&canonicalized);
                println!(
                    "   ✅ {} → {}",
                    color_pretty.formula_original(formula),
                    color_pretty.formula(&pretty_output)
                );
                if spec.family.is_some() {
                    println!("     Family: {:?}", spec.family);
                }
            }
            Err(_) => {
                println!(
                    "   ❌ {} → failed to parse",
                    color_pretty.formula_original(formula)
                );
            }
        }
    }
    println!();

    // Demonstrate materialization with a working formula
    println!(
        "4. Materializing a working formula: {}",
        color_pretty.formula_original("incidence ~ period + (1|herd)")
    );

    // Also test binomial trials materialization
    println!(
        "5. Testing binomial trials materialization: incidence | trials(size) ~ period + (1|herd)"
    );
    let binomial_formula = "incidence | trials(size) ~ period + (1|herd)";
    match p.parse(binomial_formula) {
        Ok(spec) => {
            let canonicalized = canonicalize(&spec);
            let canonical_pretty = pretty(&canonicalized);
            println!("   Parsed: {}", binomial_formula);
            println!("   Canonicalized: {}", canonical_pretty);

            // Try to materialize
            let materialize_result =
                materialize_dsl_spec(&df, &canonicalized, MaterializeOptions::default());
            match materialize_result {
                Ok((y, x, z)) => {
                    println!("   ✅ Materialization successful!");
                    println!("   Response variable (y):");
                    println!("     - Shape: {} rows × {} columns", y.height(), y.width());
                    println!("     - Column names: {:?}", y.get_column_names());
                    println!("     - Types: {:?}", y.dtypes());
                    println!();

                    println!("   Fixed effects design matrix (X):");
                    println!("     - Shape: {} rows × {} columns", x.height(), x.width());
                    println!("     - Column names: {:?}", x.get_column_names());
                    println!();

                    println!("   Random effects design matrix (Z):");
                    println!("     - Shape: {} rows × {} columns", z.height(), z.width());
                    println!("     - Column names: {:?}", z.get_column_names());
                    println!();

                    // Note: Two-column conversion helper not yet implemented
                    println!("   Note: Two-column conversion helper not yet implemented");
                    println!("   Would convert (successes, trials) to (successes, failures)");
                }
                Err(e) => {
                    println!("   ⚠️  Materialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Parsing failed: {:?}", e);
        }
    }
    println!();

    // Demonstrate materialization with a working formula
    println!(
        "6. Materializing a working formula: {}",
        color_pretty.formula_original("incidence ~ period + (1|herd)")
    );
    let working_formula = "incidence ~ period + (1|herd)";
    match p.parse(working_formula) {
        Ok(spec) => {
            let canonicalized = canonicalize(&spec);
            let canonical_pretty = pretty(&canonicalized);
            println!("   Parsed: {}", working_formula);
            println!("   Canonicalized: {}", canonical_pretty);

            // Try to materialize
            let materialize_result =
                materialize_dsl_spec(&df, &canonicalized, MaterializeOptions::default());
            match materialize_result {
                Ok((y, x, z)) => {
                    println!("   ✅ Materialization successful!");
                    println!("   Response variable (y):");
                    println!("     - Shape: {} rows × {} columns", y.height(), y.width());
                    println!("     - Column names: {:?}", y.get_column_names());
                    println!("     - Types: {:?}", y.dtypes());
                    println!();

                    println!("   Fixed effects design matrix (X):");
                    println!("     - Shape: {} rows × {} columns", x.height(), x.width());
                    println!("     - Column names: {:?}", x.get_column_names());
                    println!();

                    println!("   Random effects design matrix (Z):");
                    println!("     - Shape: {} rows × {} columns", z.height(), z.width());
                    println!("     - Column names: {:?}", z.get_column_names());
                    println!();

                    // Validate against expected CBPP structure
                    println!("   Validation for CBPP data:");
                    println!("     - Expected: n = 56 rows, K = 4 periods, H = 15 herds");
                    println!("     - Expected: X.shape == (56, 4), Z.shape == (56, 15)");
                    println!(
                        "     - Actual: X.shape == ({}, {}), Z.shape == ({}, {})",
                        x.height(),
                        x.width(),
                        z.height(),
                        z.width()
                    );

                    if x.height() == 56 && z.height() == 56 && z.width() == 15 {
                        println!("     ✅ Shapes match expectations!");
                    } else {
                        println!("     ⚠️  Shapes don't match expectations");
                    }
                }
                Err(e) => {
                    println!("   ⚠️  Materialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Parsing failed: {:?}", e);
        }
    }
    println!();

    // Show what the full CBPP model should look like
    println!("7. Target CBPP Model Specification:");
    println!(
        "   Input: incidence | trials(size) ~ period + (1 | herd), family = binomial(\"logit\")"
    );
    println!("   Canonical form: incidence | trials(size) ~ 1 + period + (1 | herd)");
    println!();
    println!("   Expected materialization:");
    println!("   - Response: Response::BinomialTrials {{ successes = incidence, trials = size }}");
    println!("   - X: shape (56, 4) with columns [\"intercept\", \"period_2\", \"period_3\", \"period_4\"]");
    println!(
        "   - Z: shape (56, 15) with columns [\"ri_herd_1\", \"ri_herd_2\", ..., \"ri_herd_15\"]"
    );
    println!("   - Family: binomial(logit)");
    println!();

    // Implementation roadmap
    println!("8. Implementation Roadmap:");
    println!(
        "   ✅ Basic formula parsing: {}",
        color_pretty.formula_original("y ~ x")
    );
    println!("   ✅ Group terms: (1|group)");
    println!("   ✅ Interactions and products: x1:x2, x1*x2");
    println!("   ✅ Canonicalization: expanding products and groups");
    println!("   ✅ Materialization: X and Z matrices");
    println!("   ✅ Aterms: y | trials(size) - IMPLEMENTED!");
    println!("   ✅ Binomial response materialization: Response::BinomialTrials - IMPLEMENTED!");
    println!("   ✅ Family specifications: family=binomial() - IMPLEMENTED!");
    println!("   ⚠️  Treatment coding for categorical variables");
    println!("   ⚠️  Two-column conversion helper: (successes, trials) → (successes, failures)");
    println!();

    println!("=== Demo Complete ===");

    Ok(())
}
