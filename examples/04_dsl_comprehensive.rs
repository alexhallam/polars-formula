use chumsky::Parser;
use polars::prelude::*;
use polars_formula::dsl::{
    canon::canonicalize, materialize::materialize, parser::parser, pretty::pretty,
};
use polars_formula::{MaterializeOptions, Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 polars-formula DSL Comprehensive Demo\n");

    // Load sample data
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x1" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x2" => [2.0, 3.0, 4.0, 5.0, 6.0],
        "group" => ["A", "A", "B", "B", "C"]
    )?;

    println!("📊 Sample data:");
    println!("{}", df);
    println!();

    // Create colored pretty printer
    let color_pretty = Color::default();

    // Test various formula types
    let formulas = vec![
        "y ~ x1 + x2",
        "y ~ x1 * x2", // Expands to x1 + x2 + x1:x2
        "y ~ x1:x2",
        "y ~ poly(x1, 2)",
        "y ~ (1|group)",
        "y ~ x1 + (1|group)",
        "y ~ x1 + x2, family=gaussian()",
    ];

    for (i, formula_str) in formulas.iter().enumerate() {
        println!("{}. Formula: {}", i + 1, color_pretty.formula(formula_str));

        let p = parser();
        match p.parse(formula_str.chars().collect::<Vec<_>>()) {
            Ok(spec) => {
                println!("   ✅ Parsed successfully");

                // Canonicalize
                let canonicalized = canonicalize(&spec);
                let canonical_str = pretty(&canonicalized);
                println!(
                    "   📝 Canonicalized: {}",
                    color_pretty.formula(&canonical_str)
                );

                // Try to materialize
                match materialize(&df, &canonicalized, MaterializeOptions::default()) {
                    Ok((y, x, z)) => {
                        println!("   🎯 Materialized:");
                        println!("     - Response: {} columns", y.width());
                        println!("     - Fixed effects: {} columns", x.width());
                        println!("     - Random effects: {} columns", z.width());
                    }
                    Err(e) => {
                        println!("   ⚠️  Materialization failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Parse failed: {:?}", e);
            }
        }
        println!();
    }

    // Demonstrate advanced features
    println!("🔧 Advanced Features Demo:");

    // Complex formula with multiple features
    let complex_formula = "y ~ x1 * x2 + poly(x1, 2) + (1|group)";
    println!("Complex formula: {}", color_pretty.formula(complex_formula));

    let p = parser();
    if let Ok(spec) = p.parse(complex_formula.chars().collect::<Vec<_>>()) {
        let canonicalized = canonicalize(&spec);
        let canonical_str = pretty(&canonicalized);
        println!("Canonicalized: {}", color_pretty.formula(&canonical_str));

        // Show the structure
        println!("Formula structure:");
        println!("  - Response: {:?}", spec.formula.lhs);
        println!("  - Predictors: {:?}", spec.formula.rhs);
        println!("  - Family: {:?}", spec.family);
        println!("  - Link: {:?}", spec.link);
    }

    println!("\n✅ Demo completed successfully!");
    Ok(())
}
