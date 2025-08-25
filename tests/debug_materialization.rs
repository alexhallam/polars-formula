use polars::prelude::*;
use polars_formula::dsl::{parser::parser, materialize::materialize, canon::*, pretty::pretty};
use polars_formula::MaterializeOptions;
use chumsky::Parser;

#[test]
fn test_group_materialization() {
    // Create a simple test DataFrame
    let df = df!(
        "y" => [1.0, 2.0, 3.0, 4.0],
        "x" => [1.0, 2.0, 3.0, 4.0],
        "group" => ["A", "A", "B", "B"]
    ).unwrap();
    
    let p = parser();
    
    // Test 1: Simple random intercept
    println!("=== Test 1: Simple random intercept ===");
    let formula1 = "y ~ (1|group)";
    let parsed1 = p.parse(formula1).unwrap();
    let canonicalized1 = canonicalize(&parsed1);
    println!("Canonicalized: {}", pretty(&canonicalized1));
    let result1 = materialize(&df, &canonicalized1, MaterializeOptions::default());
    match result1 {
        Ok((_y, x, z)) => {
            println!("✅ Success!");
            println!("X shape: {}x{}", x.height(), x.width());
            println!("X columns: {:?}", x.get_column_names());
            println!("Z shape: {}x{}", z.height(), z.width());
            println!("Z columns: {:?}", z.get_column_names());
        }
        Err(e) => println!("❌ Error: {:?}", e),
    }
    
    // Test 2: Random slope
    println!("\n=== Test 2: Random slope ===");
    let formula2 = "y ~ (x|group)";
    let parsed2 = p.parse(formula2).unwrap();
    let canonicalized2 = canonicalize(&parsed2);
    println!("Canonicalized: {}", pretty(&canonicalized2));
    let result2 = materialize(&df, &canonicalized2, MaterializeOptions::default());
    match result2 {
        Ok((_y, x, z)) => {
            println!("✅ Success!");
            println!("X shape: {}x{}", x.height(), x.width());
            println!("X columns: {:?}", x.get_column_names());
            println!("Z shape: {}x{}", z.height(), z.width());
            println!("Z columns: {:?}", z.get_column_names());
        }
        Err(e) => println!("❌ Error: {:?}", e),
    }
    
    // Test 3: Mixed model
    println!("\n=== Test 3: Mixed model ===");
    let formula3 = "y ~ x + (x|group)";
    let parsed3 = p.parse(formula3).unwrap();
    let canonicalized3 = canonicalize(&parsed3);
    println!("Canonicalized: {}", pretty(&canonicalized3));
    let result3 = materialize(&df, &canonicalized3, MaterializeOptions::default());
    match result3 {
        Ok((_y, x, z)) => {
            println!("✅ Success!");
            println!("X shape: {}x{}", x.height(), x.width());
            println!("X columns: {:?}", x.get_column_names());
            println!("Z shape: {}x{}", z.height(), z.width());
            println!("Z columns: {:?}", z.get_column_names());
        }
        Err(e) => println!("❌ Error: {:?}", e),
    }
}
