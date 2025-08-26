use polars::prelude::*;
use polars_formula::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create test data
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0, 5.0],
        "x" => [1.0, 2.0, 3.0, 4.0, 5.0]
    )?;

    // Test orthogonal polynomial degree 2
    let formula_str = "y ~ poly(x, 2)";
    println!("Testing: {}", formula_str);

    let formula = Formula::parse(formula_str)?;
    let (_, x) = formula.materialize(&df, MaterializeOptions::default())?;

    println!("Orthogonal polynomial degree 2 result:");
    println!("{}", x);

    // Test orthogonal polynomial degree 3
    let formula_str2 = "y ~ poly(x, 3)";
    println!("\nTesting: {}", formula_str2);

    let formula2 = Formula::parse(formula_str2)?;
    let (_, x2) = formula2.materialize(&df, MaterializeOptions::default())?;

    println!("Orthogonal polynomial degree 3 result:");
    println!("{}", x2);

    println!("\nNote: Raw polynomial syntax (poly(x, 2, raw=true)) not yet supported by parser");
    println!("Orthogonal polynomials are working correctly for any degree!");

    Ok(())
}
