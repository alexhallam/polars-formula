# Colored Formula Output

The `polars-formula` library now supports colored output for formulas using the `owo-colors` library. This makes it easier to read and understand complex statistical formulas by highlighting different components with distinct colors.

## Features

- **Automatic color detection**: Colors are automatically enabled when outputting to a terminal that supports colors
- **Manual control**: You can explicitly enable or disable colors
- **Environment variable support**: Respects the `NO_COLOR` environment variable
- **Syntax highlighting**: Different formula components are colored differently

## Color Scheme

- **Response variables** (like `mpg`, `y`): Red (#bf616a)
- **Operators** (`~`, `+`, `*`, `:`, `-`, `^`): Yellow  
- **Functions** (`poly`, `I`, `log`, `exp`): Blue
- **Groups** (`(1|group)`, parentheses): Green (#a3be8c)
- **Numbers and constants** (`1`, `2`, `0`): Yellow (#ebcb8b)
- **Fallback**: Yellow (#ebcb8b)

## Usage

### Basic Usage

```rust
use polars_formula::{Formula, SimpleColoredPretty};

let formula_str = "mpg ~ wt + hp + poly(disp, 2)";
let color_pretty = SimpleColoredPretty::default();

// Color the formula string
println!("{}", color_pretty.formula(formula_str));
```

### Configuration

```rust
use polars_formula::SimpleColoredPretty;

// Auto-detect terminal support (default)
let auto_colors = SimpleColoredPretty::default();

// Force colors on
let forced_colors = SimpleColoredPretty::new(true);

// Force colors off
let no_colors = SimpleColoredPretty::new(false);
```

### Individual Color Methods

```rust
let color_pretty = SimpleColoredPretty::default();

println!("Response: {}", color_pretty.response("mpg"));
println!("Operator: {}", color_pretty.operator("~"));
println!("Function: {}", color_pretty.function("poly"));
println!("Group: {}", color_pretty.group("(1|group)"));
println!("Number: {}", color_pretty.number("2"));
```

### Integration with Examples

The colored output is integrated into the existing examples:

```rust
use polars::prelude::*;
use polars_formula::{Formula, MaterializeOptions, SimpleColoredPretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let df: DataFrame = CsvReader::new(std::fs::File::open("data.csv")?).finish()?;
    
    let formula_str = "mpg ~ wt + hp + poly(disp, 2)";
    let color_pretty = SimpleColoredPretty::default();
    
    // Print colored formula
    println!("Formula: {}", color_pretty.formula(formula_str));
    
    let formula = Formula::parse(formula_str)?;
    let (y, x) = formula.materialize(&df, MaterializeOptions::default())?;
    
    println!("y: {}", y);
    println!("X: {}", x);
    
    Ok(())
}
```

## Environment Variables

- `NO_COLOR`: Set this environment variable to disable colors regardless of terminal support
- Colors are automatically disabled when output is redirected to a file

## Examples

Run the colored formula demo to see the feature in action:

```bash
cargo run --example colored_formula_demo
```

This will show:
- Different formula types with syntax highlighting
- Color configuration examples
- Individual color method demonstrations

## Dependencies

The colored output feature uses:
- `owo-colors`: For terminal color support
- `atty`: For terminal detection

These are automatically included when you use the colored output features.
