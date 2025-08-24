use chumsky::Parser;
use polars_formula::dsl::parser::parser;

#[test]
fn debug_parser() {
    let p = parser();

    // Test with a simple formula
    let result = p.parse("y ~ x");
    println!("Parse result: {:?}", result);

    // Test with just the response part
    let result = p.parse("y");
    println!("Response only result: {:?}", result);
}
