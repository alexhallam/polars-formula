use polars::prelude::*;
use polars_formula::{Formula, SimpleColoredPretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Colored Formula Demo\n");

    // Create a simple dataset
    let df: DataFrame =
        CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;

    // Test formulas with different complexity
    let formulas = vec![
        "mpg ~ wt + hp",
        "mpg ~ wt + hp + cyl + wt:hp",
        "mpg ~ poly(disp, 2) + wt * hp",
        "mpg ~ (1|cyl) + wt",
        "mpg | weights(wt) ~ hp + cyl",
    ];

    let color_pretty = SimpleColoredPretty::default();

    for (i, formula_str) in formulas.iter().enumerate() {
        println!("{}. Original formula:", i + 1);
        println!("   {}", color_pretty.formula(formula_str));
        println!();

        // Try to parse with simple formula parser
        match Formula::parse(formula_str) {
            Ok(_formula) => {
                println!("   ✅ Parsed successfully!");
                println!();
            }
            Err(e) => {
                println!("   ❌ Failed to parse: {}", e);
                println!();
            }
        }
    }

    // Demonstrate color configuration
    println!("🔧 Color Configuration Examples:\n");

    // Auto-detect colors
    let auto_colors = SimpleColoredPretty::default();
    println!("Auto-detected colors:");
    println!(
        "   {}",
        auto_colors.formula("y ~ x + poly(z, 2) + (1|group)")
    );
    println!();

    // Force colors on
    let forced_colors = SimpleColoredPretty::new(true);
    println!("Forced colors on:");
    println!(
        "   {}",
        forced_colors.formula("y ~ x + poly(z, 2) + (1|group)")
    );
    println!();

    // Force colors off
    let no_colors = SimpleColoredPretty::new(false);
    println!("Colors disabled:");
    println!("   {}", no_colors.formula("y ~ x + poly(z, 2) + (1|group)"));
    println!();

    // Demonstrate individual color methods
    println!("🎨 Individual Color Methods:\n");
    println!("Response: {}", color_pretty.response("mpg"));
    println!("Predictor: {}", color_pretty.predictor("wt"));
    println!("Operator: {}", color_pretty.operator("~"));
    println!("Function: {}", color_pretty.function("poly"));
    println!("Group: {}", color_pretty.group("(1|group)"));
    println!("Number: {}", color_pretty.number("2"));
    println!();

    Ok(())
}
