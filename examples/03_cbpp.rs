use chumsky::Parser;
use polars::prelude::*; // DataFrame andCsvReader
use polars_formula::dsl::canon::canonicalize;
use polars_formula::dsl::parser::parser;
use polars_formula::dsl::pretty::pretty;
use polars_formula::{Color, Formula, MaterializeOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // print working that this is still in development
    println!("🚧 This example is still in development is not working as expected.");

    // Load data
    let df = CsvReader::new(std::fs::File::open("examples/data/cbpp.csv")?).finish()?;

    // Formula string
    let formula_str: &'static str =
        "incidence | trials(size) ~ period + (1|herd), family = binomial()";

    // Colored print
    let color_pretty = Color::default();
    println!("Original:  {}", color_pretty.formula(formula_str));

    // Parse and canonicalize the formula
    let model_spec = parser()
        .parse(formula_str.chars().collect::<Vec<_>>())
        .map_err(|e| format!("Parse error: {:?}", e))?;
    let canonicalized = canonicalize(&model_spec);
    let canonicalized_str = pretty(&canonicalized);
    println!(
        "Canonicalized: {}",
        color_pretty.formula(&canonicalized_str)
    );

    // Parse the formula
    let formula: Formula = Formula::parse(formula_str)?;
    let (y, x) = formula.materialize(&df, MaterializeOptions::default())?;

    // Print spec
    //println!("Spec: {}", pretty(&formula.spec));

    // Print the results
    println!("y: {}", y);
    println!("X: {}", x);

    Ok(())
}
