use owo_colors::OwoColorize;

/// Simple colored pretty-printer for formulas
pub struct SimpleColoredPretty {
    enabled: bool,
}

impl SimpleColoredPretty {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn default() -> Self {
        Self {
            enabled: std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout),
        }
    }

    pub fn disabled() -> Self {
        Self { enabled: false }
    }

    /// Color a formula string with simple syntax highlighting
    pub fn formula(&self, formula: &str) -> String {
        if !self.enabled {
            return formula.to_string();
        }

        let mut result = String::new();
        let mut current = String::new();

        for ch in formula.chars() {
            match ch {
                '~' | '+' | '*' | ':' | '-' | '^' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current));
                        current.clear();
                    }
                    // Color operator
                    result.push_str(&ch.to_string().yellow().to_string());
                }
                '(' | ')' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current));
                        current.clear();
                    }
                    // Color parentheses
                    result.push_str(&ch.to_string().green().to_string());
                }
                ' ' => {
                    // Flush current token
                    if !current.is_empty() {
                        result.push_str(&self.color_token(&current));
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
            result.push_str(&self.color_token(&current));
        }

        result
    }

    fn color_token(&self, token: &str) -> String {
        // Check if it's a number
        if token.parse::<f64>().is_ok() || token == "1" || token == "0" {
            return token.yellow().to_string();
        }

        // Check if it's a function name (common ones)
        let functions = ["poly", "I", "log", "exp", "sin", "cos", "tan"];
        if functions.contains(&token) {
            return token.blue().to_string();
        }

        // Check if it's a single letter (likely response variable)
        if token.len() == 1 && token.chars().all(|c| c.is_alphabetic()) {
            return token.red().to_string();
        }

        // Multi-letter variables - treat as response variables
        if token.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return token.red().to_string();
        }

        // Default to fallback color
        token.yellow().to_string()
    }

    /// Color a response variable
    pub fn response(&self, s: &str) -> String {
        if self.enabled {
            s.red().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color an operator
    pub fn operator(&self, s: &str) -> String {
        if self.enabled {
            s.yellow().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a function name
    pub fn function(&self, s: &str) -> String {
        if self.enabled {
            s.blue().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a group expression
    pub fn group(&self, s: &str) -> String {
        if self.enabled {
            s.green().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a number or constant
    pub fn number(&self, s: &str) -> String {
        if self.enabled {
            s.yellow().to_string()
        } else {
            s.to_string()
        }
    }
}

impl Default for SimpleColoredPretty {
    fn default() -> Self {
        Self::new(std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout))
    }
}
