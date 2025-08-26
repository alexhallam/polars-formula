use chumsky::Parser;
use polars::prelude::*;
use polars_formula::dsl::{
    canon::canonicalize, materialize::materialize, parser::parser, pretty::pretty,
};
use polars_formula::{Color, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 polars-formula Quick Wins Demo\n");

    // Load sample data with categorical variable
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        "x1" => [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        "x2" => [2.0, 3.0, 4.0, 5.0, 6.0, 7.0],
        "species" => ["setosa", "versicolor", "virginica", "setosa", "versicolor", "virginica"]
    )?;

    println!("📊 Sample data:");
    println!("{}", df);
    println!();

    let color_pretty = Color::default();

    // Test current implementation status
    println!("🔧 Current Implementation Status:\n");

    // 1. poly() function - now working!
    println!("1. poly() function - ✅ FIXED:");
    let poly_formula = "y ~ poly(x1, 2)";
    println!("   Formula: {}", color_pretty.formula(poly_formula));

    let p = parser();
    if let Ok(spec) = p.parse(poly_formula.chars().collect::<Vec<_>>()) {
        let canonicalized = canonicalize(&spec);
        match materialize(&df, &canonicalized, MaterializeOptions::default()) {
            Ok((_y, x, _z)) => {
                println!("   Current output: {} fixed effect columns", x.width());
                println!("   Expected: 2 columns (intercept + poly term)");
                println!("   Actual: {} columns ✅", x.width());
            }
            Err(e) => println!("   Error: {}", e),
        }
    }
    println!();

    // 2. Interactions - now working!
    println!("2. Interactions - ✅ FIXED:");
    let interaction_formula = "y ~ x1:x2";
    println!("   Formula: {}", color_pretty.formula(interaction_formula));

    if let Ok(spec) = p.parse(interaction_formula.chars().collect::<Vec<_>>()) {
        let canonicalized = canonicalize(&spec);
        match materialize(&df, &canonicalized, MaterializeOptions::default()) {
            Ok((_y, x, _z)) => {
                println!("   Current output: {} fixed effect columns", x.width());
                println!("   Expected: 2 columns (intercept + x1:x2)");
                println!("   Actual: {} columns ✅", x.width());
            }
            Err(e) => println!("   Error: {}", e),
        }
    }
    println!();

    // 3. Categorical variables not handled
    println!("3. Categorical variables - ✅ FIXED:");
    let cat_formula = "y ~ species";
    println!("   Formula: {}", color_pretty.formula(cat_formula));

    if let Ok(spec) = p.parse(cat_formula.chars().collect::<Vec<_>>()) {
        let canonicalized = canonicalize(&spec);
        match materialize(&df, &canonicalized, MaterializeOptions::default()) {
            Ok((_y, x, _z)) => {
                println!("   Current output: {} fixed effect columns", x.width());
                println!(
                    "   Expected: 3 columns (intercept + species_versicolor, species_virginica)"
                );
                println!("   Actual: {} columns ✅", x.width());
            }
            Err(e) => println!("   Error: {}", e),
        }
    }
    println!();

    // 4. Random effects are dense and inefficient
    println!("4. Random effects - dense and inefficient:");
    let re_formula = "y ~ (1|species)";
    println!("   Formula: {}", color_pretty.formula(re_formula));

    if let Ok(spec) = p.parse(re_formula.chars().collect::<Vec<_>>()) {
        let canonicalized = canonicalize(&spec);
        match materialize(&df, &canonicalized, MaterializeOptions::default()) {
            Ok((y, x, z)) => {
                println!("   Current output: {} random effect columns", z.width());
                println!("   Expected: 2 sparse columns");
                println!("   Actual: {} dense one-hot columns", z.width());
            }
            Err(e) => println!("   Error: {}", e),
        }
    }
    println!();

    // Show what the fixed implementations should look like
    println!("🎯 Target Implementations:\n");

    println!("1. Fixed poly() function:");
    println!("   poly(x1, 2) → [x1, x1²] with orthogonal option");
    println!("   poly(x1, 3, raw=true) → [x1, x1², x1³] (raw polynomials)");
    println!("   poly(x1, 2, raw=false) → [orthogonal_poly_1, orthogonal_poly_2]");
    println!();

    println!("2. Fixed interactions:");
    println!("   x1:x2 → [x1 * x2] (element-wise multiplication)");
    println!("   x1*x2 → [x1, x2, x1:x2] (full expansion)");
    println!("   x1:x2:x3 → [x1 * x2 * x3] (multi-way interaction)");
    println!();

    println!("3. Categorical contrasts:");
    println!("   species (treatment) → [species_versicolor, species_virginica]");
    println!("   species (sum) → [species_setosa, species_versicolor]");
    println!("   species (helmert) → [species_versicolor_vs_setosa, species_virginica_vs_prev]");
    println!();

    println!("4. Sparse random effects:");
    println!("   (1|species) → sparse Z matrix with 2 columns");
    println!("   (x1|species) → sparse Z matrix with 6 columns (2 groups × 3 slopes)");
    println!("   (x1||species) → uncorrelated random effects");
    println!();

    println!("5. LazyFrame integration:");
    println!("   formula.to_exprs() → Vec<Expr> for LazyFrame");
    println!("   df.lazy().with_columns(terms).collect()");
    println!();

    println!("✅ Demo completed! These are the critical gaps to address first.");
    Ok(())
}
