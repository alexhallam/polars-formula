use owo_colors::OwoColorize;

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
    /// Create a new color config with explicit enabled/disabled state
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Create a color config that automatically detects terminal support
    pub fn auto() -> Self {
        Self::default()
    }

    /// Disable colors
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

/// Colored pretty-printer for formulas
pub struct ColoredPretty {
    config: ColorConfig,
}

impl ColoredPretty {
    pub fn new(config: ColorConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(ColorConfig::default())
    }

    pub fn disabled() -> Self {
        Self::new(ColorConfig::disabled())
    }

    /// Color a response variable
    pub fn response(&self, s: &str) -> String {
        if self.config.enabled {
            s.red().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color an operator
    pub fn operator(&self, s: &str) -> String {
        if self.config.enabled {
            s.yellow().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a function name
    pub fn function(&self, s: &str) -> String {
        if self.config.enabled {
            s.blue().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a group expression
    pub fn group(&self, s: &str) -> String {
        if self.config.enabled {
            s.green().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a number or constant
    pub fn number(&self, s: &str) -> String {
        if self.config.enabled {
            s.magenta().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a formula string with syntax highlighting
    pub fn formula(&self, formula: &str) -> String {
        if !self.config.enabled {
            return formula.to_string();
        }

        // Simple tokenization and coloring
        let mut result = String::new();
        let mut current = String::new();
        let mut in_function = false;
        let mut paren_depth = 0;
        let mut in_group = false;

        for ch in formula.chars() {
            match ch {
                '~' | '+' | '*' | ':' | '-' | '^' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_function, in_group));
                        current.clear();
                    }
                    // Color operator
                    result.push_str(&self.operator(&ch.to_string()));
                }
                '(' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_function, in_group));
                        current.clear();
                    }

                    paren_depth += 1;
                    if paren_depth == 1 && !in_function {
                        in_group = true;
                        result.push_str(&self.group("("));
                    } else {
                        result.push('(');
                    }
                }
                ')' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_function, in_group));
                        current.clear();
                    }

                    paren_depth -= 1;
                    if paren_depth == 0 && in_group {
                        in_group = false;
                        result.push_str(&self.group(")"));
                    } else {
                        result.push(')');
                    }

                    if in_function && paren_depth == 0 {
                        in_function = false;
                    }
                }
                ' ' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current, in_function, in_group));
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
            result.push_str(&self.color_token(&current, in_function, in_group));
        }

        result
    }

    fn color_token(&self, token: &str, in_function: bool, in_group: bool) -> String {
        if in_group {
            return self.group(token);
        }

        if in_function {
            return self.number(token);
        }

        // Check if it's a number
        if token.parse::<f64>().is_ok() || token == "1" || token == "0" {
            return self.number(token);
        }

        // Check if it's a function name (followed by parentheses)
        if token.chars().all(|c| c.is_alphabetic()) && !token.is_empty() {
            // This is a variable name - could be response or predictor
            // For simplicity, we'll treat single letters as response variables
            if token.len() == 1 {
                return self.response(token);
            } else {
                // Multi-letter variables - could be either, but let's treat as response for now
                return self.response(token);
            }
        }

        // Default to plain text
        token.to_string()
    }
}

impl Default for ColoredPretty {
    fn default() -> Self {
        Self::new(ColorConfig::default())
    }
}
