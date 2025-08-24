use crate::dsl::ast::*;
use chumsky::Parser;
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

    /// Color a formula string using AST parsing for accuracy
    pub fn formula(&self, formula: &str) -> String {
        if !self.enabled {
            return formula.to_string();
        }

        // Try to parse with DSL parser first for accurate coloring
        match crate::dsl::parser::parser().parse(formula.chars().collect::<Vec<_>>()) {
            Ok(spec) => self.pretty_model_spec(&spec),
            Err(_) => {
                // Fallback to simple string parsing if DSL parsing fails
                self.formula_simple_parse(formula)
            }
        }
    }

    /// Color a formula using the parsed AST structure (most accurate)
    pub fn pretty_model_spec(&self, spec: &ModelSpec) -> String {
        if !self.enabled {
            return crate::dsl::pretty::pretty(spec);
        }

        let mut parts = Vec::new();

        // Add family/link header if present
        if let Some(family) = &spec.family {
            parts.push(format!("family={}", self.pretty_family(family)));

            if let Some(link) = &spec.link {
                parts.push(format!("link={}", self.pretty_link(link)));
            }
        }

        // Add main formula
        let lhs = self.pretty_response(&spec.formula.lhs);
        let rhs = self.pretty_expr(&spec.formula.rhs);

        // Add aterms to LHS if present
        let lhs_with_aterms = if spec.formula.aterms.is_empty() {
            lhs.clone()
        } else {
            let aterms_str = spec
                .formula
                .aterms
                .iter()
                .map(|aterm| self.pretty_aterm(aterm))
                .collect::<Vec<_>>()
                .join(&self.operator(" | "));
            format!("{} {} {}", lhs, self.operator("|"), aterms_str)
        };

        if lhs.is_empty() {
            parts.push(rhs);
        } else {
            parts.push(format!(
                "{} {} {}",
                lhs_with_aterms,
                self.operator("~"),
                rhs
            ));
        }

        // Add distributional parameter formulas
        for dpar in &spec.dpars {
            parts.push(format!(
                "{} {} {}",
                dpar.name,
                self.operator("~"),
                self.pretty_expr(&dpar.rhs)
            ));
        }

        // Add autocorrelation terms
        for autocor in &spec.autocor {
            let args_str = if autocor.args.is_empty() {
                String::new()
            } else {
                autocor
                    .args
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, self.pretty_expr(v)))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "))
            };
            parts.push(format!(
                "{}{}{}{}",
                self.function(&autocor.name),
                self.operator("("),
                args_str,
                self.operator(")")
            ));
        }

        parts.join(&self.operator(" + "))
    }

    /// Fallback method for simple string parsing
    fn formula_simple_parse(&self, formula: &str) -> String {
        let mut result = String::new();
        let mut current = String::new();
        let mut tokens = Vec::new();
        let mut in_lhs = true; // Track if we're in the left-hand side (response)

        for ch in formula.chars() {
            match ch {
                '~' | '+' | '*' | ':' | '-' | '^' => {
                    // Flush current token
                    if !current.is_empty() {
                        tokens.push((current.clone(), in_lhs));
                        current.clear();
                    }
                    if ch == '~' {
                        in_lhs = false; // After ~, we're in the right-hand side
                    }
                    tokens.push((ch.to_string(), false)); // Operators are never responses
                }
                '(' | ')' => {
                    // Flush current token
                    if !current.is_empty() {
                        tokens.push((current.clone(), in_lhs));
                        current.clear();
                    }
                    tokens.push((ch.to_string(), false)); // Parentheses are never responses
                }
                ' ' => {
                    // Flush current token
                    if !current.is_empty() {
                        tokens.push((current.clone(), in_lhs));
                        current.clear();
                    }
                    tokens.push((" ".to_string(), false));
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // Flush remaining token
        if !current.is_empty() {
            tokens.push((current.clone(), in_lhs));
        }

        // Now color the tokens based on their position
        for (token, is_response) in tokens {
            if token == " " {
                result.push(' ');
            } else if token == "~"
                || token == "+"
                || token == "*"
                || token == ":"
                || token == "-"
                || token == "^"
            {
                result.push_str(&token.yellow().to_string());
            } else if token == "(" || token == ")" {
                result.push_str(&token.blue().to_string());
            } else if is_response {
                result.push_str(&token.red().to_string());
            } else {
                result.push_str(&token.blue().to_string());
            }
        }

        result
    }

    /// Color a response variable
    pub fn response(&self, s: &str) -> String {
        if self.enabled {
            s.red().to_string()
        } else {
            s.to_string()
        }
    }

    /// Color a predictor variable
    pub fn predictor(&self, s: &str) -> String {
        if self.enabled {
            s.blue().to_string()
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

    /// Color a function name (same as predictor)
    pub fn function(&self, s: &str) -> String {
        self.predictor(s)
    }

    /// Color a group expression (same as predictor)
    pub fn group(&self, s: &str) -> String {
        self.predictor(s)
    }

    /// Color a number or constant (same as predictor)
    pub fn number(&self, s: &str) -> String {
        self.predictor(s)
    }

    // AST-based pretty printing methods
    fn pretty_family(&self, family: &Family) -> String {
        match family {
            Family::Builtin(name, args) => {
                if args.is_empty() {
                    format!(
                        "{}{}{}",
                        self.function(name),
                        self.operator("("),
                        self.operator(")")
                    )
                } else {
                    let args_str = args
                        .iter()
                        .map(|arg| self.pretty_expr(arg))
                        .collect::<Vec<_>>()
                        .join(&self.operator(", "));
                    format!(
                        "{}{}{}{}",
                        self.function(name),
                        self.operator("("),
                        args_str,
                        self.operator(")")
                    )
                }
            }
            Family::Mixture(families) => {
                let families_str = families
                    .iter()
                    .map(|f| self.pretty_family(f))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function("mixture"),
                    self.operator("("),
                    families_str,
                    self.operator(")")
                )
            }
            Family::Custom { name, dpars } => {
                let dpars_str = dpars
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}{}{}",
                    self.function("custom_family"),
                    self.operator("(\""),
                    name,
                    self.operator("\", "),
                    dpars_str,
                    self.operator(")")
                )
            }
        }
    }

    fn pretty_link(&self, link: &Link) -> String {
        match link {
            Link::Named(name, args) => {
                if args.is_empty() {
                    self.function(name)
                } else {
                    let args_str = args
                        .iter()
                        .map(|arg| self.pretty_expr(arg))
                        .collect::<Vec<_>>()
                        .join(&self.operator(", "));
                    format!(
                        "{}{}{}{}",
                        self.function(name),
                        self.operator("("),
                        args_str,
                        self.operator(")")
                    )
                }
            }
        }
    }

    fn pretty_response(&self, response: &Response) -> String {
        match response {
            Response::Var(v) => self.response(v),
            Response::Multi(vars) => {
                let vars_str = vars
                    .iter()
                    .map(|v| self.response(v))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function("cbind"),
                    self.operator("("),
                    vars_str,
                    self.operator(")")
                )
            }
            Response::Surv { time, event, time2 } => {
                if let Some(time2) = time2 {
                    format!(
                        "{}{}{}{}{}{}{}{}",
                        self.function("Surv"),
                        self.operator("("),
                        self.pretty_expr(time),
                        self.operator(", "),
                        self.pretty_expr(event),
                        self.operator(", "),
                        self.pretty_expr(time2),
                        self.operator(")")
                    )
                } else {
                    format!(
                        "{}{}{}{}{}{}",
                        self.function("Surv"),
                        self.operator("("),
                        self.pretty_expr(time),
                        self.operator(", "),
                        self.pretty_expr(event),
                        self.operator(")")
                    )
                }
            }
            Response::Func { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| self.pretty_expr(arg))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function(name),
                    self.operator("("),
                    args_str,
                    self.operator(")")
                )
            }
            Response::BinomialTrials { successes, trials } => {
                format!(
                    "{} {} {}{}{}{}",
                    self.pretty_expr(successes),
                    self.operator("|"),
                    self.function("trials"),
                    self.operator("("),
                    self.pretty_expr(trials),
                    self.operator(")")
                )
            }
        }
    }

    fn pretty_aterm(&self, aterm: &Aterm) -> String {
        match aterm {
            Aterm::Se(expr) => format!(
                "{}{}{}{}",
                self.function("se"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Weights(expr) => format!(
                "{}{}{}{}",
                self.function("weights"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Trials(expr) => format!(
                "{}{}{}{}",
                self.function("trials"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Cens(expr) => format!(
                "{}{}{}{}",
                self.function("cens"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Trunc { lb, ub } => {
                let mut parts = Vec::new();
                if let Some(lb) = lb {
                    parts.push(format!("lb={}", self.pretty_expr(lb)));
                }
                if let Some(ub) = ub {
                    parts.push(format!("ub={}", self.pretty_expr(ub)));
                }
                format!(
                    "{}{}{}{}",
                    self.function("trunc"),
                    self.operator("("),
                    parts.join(&self.operator(", ")),
                    self.operator(")")
                )
            }
            Aterm::Subset(expr) => format!(
                "{}{}{}{}",
                self.function("subset"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Rate(expr) => format!(
                "{}{}{}{}",
                self.function("rate"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Thres { gr } => {
                if let Some(gr) = gr {
                    format!(
                        "{}{}{}{}",
                        self.function("thres"),
                        self.operator("(gr="),
                        self.pretty_expr(gr),
                        self.operator(")")
                    )
                } else {
                    format!(
                        "{}{}{}",
                        self.function("thres"),
                        self.operator("("),
                        self.operator(")")
                    )
                }
            }
            Aterm::Dec(expr) => format!(
                "{}{}{}{}",
                self.function("dec"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Cat(expr) => format!(
                "{}{}{}{}",
                self.function("cat"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::Index(expr) => format!(
                "{}{}{}{}",
                self.function("index"),
                self.operator("("),
                self.pretty_expr(expr),
                self.operator(")")
            ),
            Aterm::VReal(exprs) => {
                let args_str = exprs
                    .iter()
                    .map(|expr| self.pretty_expr(expr))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function("vreal"),
                    self.operator("("),
                    args_str,
                    self.operator(")")
                )
            }
            Aterm::VInt(exprs) => {
                let args_str = exprs
                    .iter()
                    .map(|expr| self.pretty_expr(expr))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function("vint"),
                    self.operator("("),
                    args_str,
                    self.operator(")")
                )
            }
            Aterm::Mi => format!(
                "{}{}{}",
                self.function("mi"),
                self.operator("("),
                self.operator(")")
            ),
        }
    }

    fn pretty_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Num(n) => self.number(&n.to_string()),
            Expr::Bool(b) => self.number(&b.to_string()),
            Expr::Str(s) => format!("{}{}{}", self.operator("\""), s, self.operator("\"")),
            Expr::Var(v) => self.predictor(v), // Variables in expressions are predictors
            Expr::Sum(terms) => terms
                .iter()
                .map(|term| self.pretty_expr(term))
                .collect::<Vec<_>>()
                .join(&self.operator(" + ")),
            Expr::Prod(terms) => terms
                .iter()
                .map(|t| match t {
                    Expr::Sum(_) => format!(
                        "{}{}{}",
                        self.operator("("),
                        self.pretty_expr(t),
                        self.operator(")")
                    ),
                    _ => self.pretty_expr(t),
                })
                .collect::<Vec<_>>()
                .join(&self.operator(" * ")),
            Expr::Interaction(terms) => terms
                .iter()
                .map(|t| match t {
                    Expr::Sum(_) => format!(
                        "{}{}{}",
                        self.operator("("),
                        self.pretty_expr(t),
                        self.operator(")")
                    ),
                    _ => self.pretty_expr(t),
                })
                .collect::<Vec<_>>()
                .join(&self.operator(":")),
            Expr::Nest { outer, inner, kind } => {
                let op = match kind {
                    NestKind::Slash => "/",
                    NestKind::In => "%in%",
                };
                format!(
                    "{} {} {}",
                    self.pretty_expr(outer),
                    self.operator(op),
                    self.pretty_expr(inner)
                )
            }
            Expr::Pow { base, exp } => {
                let base_str = match base.as_ref() {
                    Expr::Sum(_) | Expr::Prod(_) => format!(
                        "{}{}{}",
                        self.operator("("),
                        self.pretty_expr(base),
                        self.operator(")")
                    ),
                    _ => self.pretty_expr(base),
                };
                format!(
                    "{}{}{}",
                    base_str,
                    self.operator("^"),
                    self.pretty_expr(exp)
                )
            }
            Expr::Group {
                inner,
                spec,
                kind,
                id,
            } => {
                let inner_str = self.pretty_expr(inner);
                let spec_str = self.pretty_group_spec(spec);
                let sep = match kind {
                    GroupKind::Correlated => "|",
                    GroupKind::Uncorrelated => "||",
                };
                if let Some(id) = id {
                    format!(
                        "{}{}{}{}{}{}{}",
                        self.group("("),
                        inner_str,
                        self.operator("|"),
                        id,
                        self.operator("|"),
                        spec_str,
                        self.group(")")
                    )
                } else {
                    format!(
                        "{}{}{}{}",
                        self.group("("),
                        inner_str,
                        self.operator(sep),
                        format!("{}{}", spec_str, self.group(")")),
                    )
                }
            }
            Expr::Smooth { kind, vars, args } => {
                let kind_str = match kind {
                    SmoothKind::S => "s",
                    SmoothKind::T2 => "t2",
                    SmoothKind::TE => "te",
                    SmoothKind::TI => "ti",
                };
                let mut parts = vec![vars.join(&self.operator(", "))];
                for (k, v) in args {
                    parts.push(format!("{}={}", k, self.pretty_expr(v)));
                }
                format!(
                    "{}{}{}{}",
                    self.function(kind_str),
                    self.operator("("),
                    parts.join(&self.operator(", ")),
                    self.operator(")")
                )
            }
            Expr::Func { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| self.pretty_expr(arg))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function(name),
                    self.operator("("),
                    args_str,
                    self.operator(")")
                )
            }
            Expr::Identity(inner) => {
                format!(
                    "{}{}{}{}",
                    self.function("I"),
                    self.operator("("),
                    self.pretty_expr(inner),
                    self.operator(")")
                )
            }
            Expr::Intercept(true) => self.number("1"),
            Expr::Intercept(false) => self.number("0"),
            Expr::Dot => self.operator("."),
        }
    }

    fn pretty_group_spec(&self, spec: &GroupSpec) -> String {
        match spec {
            GroupSpec::Expr(group_expr) => group_expr
                .0
                .iter()
                .map(|(name, op)| {
                    if let Some(op) = op {
                        let op_str = match op {
                            GroupOp::Cross => ":",
                            GroupOp::Nest => "/",
                            GroupOp::Split => "+",
                        };
                        format!("{}{}", self.operator(op_str), name)
                    } else {
                        name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(""),
            GroupSpec::Func { name, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| self.pretty_expr(arg))
                    .collect::<Vec<_>>()
                    .join(&self.operator(", "));
                format!(
                    "{}{}{}{}",
                    self.function(name),
                    self.operator("("),
                    args_str,
                    self.operator(")")
                )
            }
        }
    }
}

impl Default for SimpleColoredPretty {
    fn default() -> Self {
        Self::new(std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout))
    }
}
