use polars_formula::SimpleColoredPretty;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Hex Color Constants Demo\n");

    // Create a colored pretty printer
    let color_pretty = SimpleColoredPretty::default();

    // Test formulas with the current colors
    let formulas = vec![
        "mpg ~ wt + hp - 1",
        "y ~ x + poly(z, 2) + (1|group)",
        "response ~ predictor1 + predictor2 * predictor3",
    ];

    for (i, formula_str) in formulas.iter().enumerate() {
        println!(
            "{}. Formula: {}",
            i + 1,
            color_pretty.formula(formula_str)
        );
    }

    Ok(())
}
