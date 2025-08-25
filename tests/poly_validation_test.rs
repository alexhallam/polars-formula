use polars::prelude::*;
use polars_formula::*;

#[test]
fn test_poly_degree_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Create a small dataset with only 5 unique values
    let df = df!(
        "x" => &[1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0] // 5 unique values, 10 total
    )?;

    println!("Testing with dataset: {} rows, {} unique values", df.height(), df.column("x")?.unique()?.len());

    // Test that degree 5 (equal to unique count) fails
    let formula_str = "x ~ poly(x, 5) - 1";
    let formula = Formula::parse(formula_str)?;
    
    let result = formula.materialize(&df, MaterializeOptions::default());
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    println!("Degree 5 error message: {}", error_msg);
    assert!(error_msg.contains("'degree' must be less than number of unique points"));
    assert!(error_msg.contains("Got degree=5, unique points=5"));

    // Test that degree 4 (less than unique count) succeeds
    let formula_str = "x ~ poly(x, 4) - 1";
    let formula = Formula::parse(formula_str)?;
    
    let result = formula.materialize(&df, MaterializeOptions::default());
    match &result {
        Ok(_) => println!("Degree 4 succeeded as expected"),
        Err(e) => println!("Degree 4 failed with error: {}", e),
    }
    assert!(result.is_ok());

    // Test with mtcars dataset
    let df = CsvReader::new(std::fs::File::open("examples/data/mtcars.csv")?).finish()?;
    let unique_disp = df.column("disp")?.unique()?.len();
    println!("Mtcars dataset: {} rows, {} unique disp values", df.height(), unique_disp);
    
    // Test that degree equal to unique count fails
    let formula_str = format!("mpg ~ poly(disp, {}) - 1", unique_disp);
    let formula = Formula::parse(&formula_str)?;
    
    let result = formula.materialize(&df, MaterializeOptions::default());
    assert!(result.is_err());
    
    let error_msg = result.unwrap_err().to_string();
    println!("Mtcars degree {} error message: {}", unique_disp, error_msg);
    assert!(error_msg.contains("'degree' must be less than number of unique points"));

    // Test that degree less than unique count succeeds
    let formula_str = format!("mpg ~ poly(disp, {}) - 1", unique_disp - 1);
    let formula = Formula::parse(&formula_str)?;
    
    let result = formula.materialize(&df, MaterializeOptions::default());
    match &result {
        Ok(_) => println!("Mtcars degree {} succeeded as expected", unique_disp - 1),
        Err(e) => println!("Mtcars degree {} failed with error: {}", unique_disp - 1, e),
    }
    assert!(result.is_ok());

    println!("✅ All poly() degree validation tests passed");
    Ok(())
}
