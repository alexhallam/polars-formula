use chumsky::Parser;
use polars_formula::dsl::parser::parser;

#[test]
fn debug_parser_issues() {
    let p = parser();
    println!("=== Testing smooth parsing ===");
    let result = p.parse("s(x, k=10, bs=\"tp\")");
    println!("smooth result: {:?}", result);

    println!("\n=== Testing aterm parsing ===");
    let result = p.parse("y | weights(w) ~ x");
    println!("aterm result: {:?}", result);

    println!("\n=== Testing family parsing ===");
    let result = p.parse("family=gaussian() y ~ x");
    println!("family result: {:?}", result);

    println!("\n=== Testing survival parsing ===");
    let result = p.parse("Surv(time, status) ~ x");
    println!("survival result: {:?}", result);

    println!("\n=== Testing complex formula ===");
    let result = p.parse("y ~ x1 + x2 | weights(w)");
    println!("complex result: {:?}", result);

    println!("\n=== Testing individual components ===");
    let result = p.parse("y ~ x");
    println!("basic formula: {:?}", result);
    let result = p.parse("y | weights(w) ~ x");
    println!("formula with aterm: {:?}", result);
    let result = p.parse("family=gaussian() y ~ x");
    println!("formula with family: {:?}", result);

    println!("\n=== Testing I() function ===");
    let result = p.parse("y ~ I(x)");
    println!("I() function: {:?}", result);

    println!("\n=== Testing power expression ===");
    let result = p.parse("y ~ x^2");
    println!("power expression: {:?}", result);

    println!("\n=== Testing group parsing ===");
    let result = p.parse("y ~ (1|group)");
    println!("simple group: {:?}", result);

    let result = p.parse("y ~ (1|recipe:replicate)");
    println!("group with interaction: {:?}", result);

    let result = p.parse("y ~ x + (1|group)");
    println!("formula with group: {:?}", result);

    println!("\n=== Testing the exact failing formula ===");
    let result = p.parse("angle ~ recipe * temperature + (1 | recipe:replicate)");
    println!("exact formula: {:?}", result);

    // Let's break it down
    println!("\n=== Breaking down the formula ===");
    let result = p.parse("angle ~ recipe * temperature");
    println!("part 1 (product): {:?}", result);

    let result = p.parse("angle ~ recipe + temperature");
    println!("part 2 (sum): {:?}", result);

    let result = p.parse("angle ~ (1 | recipe:replicate)");
    println!("part 3 (group): {:?}", result);
}
