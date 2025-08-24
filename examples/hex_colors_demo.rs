use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions, SimpleColoredPretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Hex Color Constants Demo\n");

    // Show the current hex color constants
    println!("Current hex color constants:");
    println!("  Response color: #d08770 (orange-red)");
    println!("  Predictor color: #b48ead (purple)");
    println!("  Operator color: #ebcb8b (yellow)");
    println!();

    // Create a colored pretty printer
    let color_pretty = SimpleColoredPretty::default();

    // Test formulas with the current colors
    let formulas = vec![
        "mpg ~ wt + hp",
        "y ~ x + poly(z, 2) + (1|group)",
        "response ~ predictor1 + predictor2 * predictor3",
    ];

    for (i, formula_str) in formulas.iter().enumerate() {
        println!("{}. Formula: {}", i + 1, color_pretty.formula(formula_str));
    }

    println!("\n💡 To use the exact hex colors:");
    println!("1. The hex constants are defined at the top of src/simple_colored.rs");
    println!("2. You can modify them to change the color scheme");
    println!("3. The colors are now applied using ANSI escape codes");
    println!("4. Change the hex values above to customize the colors!");

    Ok(())
}
