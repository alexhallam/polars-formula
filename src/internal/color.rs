use owo_colors::{colors::CustomColor, OwoColorize};

// Define custom colors using const generics
// type ResponseColor = CustomColor<191, 97, 106>;
type ResponseColor = CustomColor<163, 190, 140>;
//type TermColor = CustomColor<208, 135, 112>;
type OperatorColor = CustomColor<208, 135, 112>;
type TermColor = CustomColor<235, 203, 139>;
/// Configuration for colored output
#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub enabled: bool,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout),
        }
    }
}

impl ColorConfig {
    // Note: These methods are kept for potential future use but are currently unused
    #[allow(dead_code)]
    /// Create a new color config with explicit enabled/disabled state
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    #[allow(dead_code)]
    /// Create a color config that automatically detects terminal support
    pub fn auto() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    /// Disable colors
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

/// Colored pretty-printer for formulas
pub struct Color {
    config: ColorConfig,
}

impl Color {
    pub fn new(config: ColorConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(ColorConfig::default())
    }

    #[allow(dead_code)]
    pub fn disabled() -> Self {
        Self::new(ColorConfig::disabled())
    }

    /// Color a response variable (rgb(191, 97, 106))
    pub fn response(&self, s: &str) -> String {
        if self.config.enabled {
            s.fg::<ResponseColor>().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a term/predictor variable (rgb(180, 142, 173))
    pub fn term(&self, s: &str) -> String {
        if self.config.enabled {
            s.fg::<TermColor>().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color an operator (rgb(235, 203, 139))
    pub fn operator(&self, s: &str) -> String {
        if self.config.enabled {
            s.fg::<OperatorColor>().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a function name (uses term color as fallback)
    #[allow(dead_code)]
    pub fn function(&self, s: &str) -> String {
        self.term(s)
    }

    /// Color a group expression (uses term color as fallback)
    #[allow(dead_code)]
    pub fn group(&self, s: &str) -> String {
        self.term(s)
    }

    /// Color a number or constant (uses term color as fallback)
    pub fn number(&self, s: &str) -> String {
        self.term(s)
    }

    /// Color a formula string with syntax highlighting
    pub fn formula(&self, formula: &str) -> String {
        if !self.config.enabled {
            return formula.to_string();
        }

        // Simple tokenization and coloring
        let mut result = String::new();
        let mut current = String::new();
        let mut in_lhs = true; // Track if we're in the left-hand side (response)

        for ch in formula.chars() {
            match ch {
                '~' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_lhs));
                        current.clear();
                    }
                    // After ~, we're in the right-hand side
                    in_lhs = false;
                    result.push_str(&self.operator(&ch.to_string()));
                }
                '+' | '*' | ':' | '-' | '^' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_lhs));
                        current.clear();
                    }
                    // Color operator
                    result.push_str(&self.operator(&ch.to_string()));
                }
                '(' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_lhs));
                        current.clear();
                    }
                    // Color parentheses with operator color
                    result.push_str(&self.operator("("));
                }
                ')' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_lhs));
                        current.clear();
                    }
                    // Color parentheses with operator color
                    result.push_str(&self.operator(")"));
                }
                ' ' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_lhs));
                        current.clear();
                    }
                    result.push(' ');
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // Flush remaining token
        if !current.is_empty() {
            result.push_str(&self.color_token(&current, in_lhs));
        }

        result
    }

    fn color_token(&self, token: &str, is_response: bool) -> String {
        // Check if it's a number
        if token.parse::<f64>().is_ok() || token == "1" || token == "0" {
            return self.number(token);
        }

        // Check if it's an operator
        if token == "~"
            || token == "+"
            || token == "*"
            || token == ":"
            || token == "-"
            || token == "^"
        {
            return self.operator(token);
        }

        // Check if it's a function name (contains only alphabetic characters)
        if token.chars().all(|c| c.is_alphabetic()) && !token.is_empty() {
            // If it's in the LHS (response), color it as response
            if is_response {
                return self.response(token);
            } else {
                // Otherwise, it's a term/predictor
                return self.term(token);
            }
        }

        // Default to term color (fallback)
        self.term(token)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(ColorConfig::default())
    }
}
