use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MaterializeOptions {
    /// Whether to include an intercept term in the design matrix.
    pub rhs_intercept: bool,
    /// Name to use for the intercept column when `rhs_intercept` is `true`.
    pub intercept_name: &'static str,
    /// Whether to clean column names using `make_clean_names()`.
    pub clean_names: bool,
}

impl Default for MaterializeOptions {
    fn default() -> Self {
        Self {
            rhs_intercept: true,
            intercept_name: "intercept",
            clean_names: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelSpec {
    pub family: Option<Family>,
    pub link: Option<Link>,
    pub formula: Formula,
    pub dpars: Vec<Dpar>,
    pub autocor: Vec<Autocor>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Family {
    Builtin(String, Vec<Expr>), // gaussian(), binomial(link=..)
    Mixture(Vec<Family>),       // mixture(gaussian(), student())
    Custom { name: String, dpars: Vec<String> }, // custom_family("kumaraswamy","mu","phi")
}

#[derive(Debug, Clone, PartialEq)]
pub enum Link {
    Named(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub lhs: Response,
    pub rhs: Expr,
    pub aterms: Vec<Aterm>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Var(String),
    Multi(Vec<String>), // cbind/mvbind
    Surv {
        time: Expr,
        event: Expr,
        time2: Option<Expr>,
    },
    Func {
        name: String,
        args: Vec<Expr>,
    }, // for future e.g. Hist(...)
    BinomialTrials {
        successes: Expr,
        trials: Expr,
    }, // y | trials(n) syntax
}

#[derive(Debug, Clone, PartialEq)]
pub enum Aterm {
    Se(Expr),
    Weights(Expr),
    Trials(Expr),
    Cens(Expr),
    Trunc { lb: Option<Expr>, ub: Option<Expr> },
    Subset(Expr),
    Rate(Expr),
    Thres { gr: Option<Expr> },
    Dec(Expr),
    Cat(Expr),
    Index(Expr),
    VReal(Vec<Expr>),
    VInt(Vec<Expr>),
    Mi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dpar {
    pub name: String,
    pub rhs: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Autocor {
    pub name: String,
    pub args: HashMap<String, Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Bool(bool),
    Str(String),
    Var(String),
    Sum(Vec<Expr>),         // flattened
    Prod(Vec<Expr>),        // for sugar `*` before expansion
    Interaction(Vec<Expr>), // a:b:c  (flattened)
    Nest {
        outer: Box<Expr>,
        inner: Box<Expr>,
        kind: NestKind,
    }, // a/b, b %in% a
    Pow {
        base: Box<Expr>,
        exp: Box<Expr>,
    }, // (a+b)^2
    Group {
        inner: Box<Expr>,
        spec: GroupSpec,
        kind: GroupKind,
        id: Option<String>,
    },
    Smooth {
        kind: SmoothKind,
        vars: Vec<String>,
        args: HashMap<String, Expr>,
    },
    Func {
        name: String,
        args: Vec<Expr>,
    },
    Identity(Box<Expr>), // I(...)
    Intercept(bool),     // 1 or 0
    Dot,                 // .
}

#[derive(Debug, Clone, PartialEq)]
pub enum NestKind {
    Slash,
    In,
} // '/' or '%in%'

#[derive(Debug, Clone, PartialEq)]
pub enum GroupKind {
    Correlated,
    Uncorrelated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupSpec {
    Expr(GroupExpr),
    Func { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupOp {
    Cross,
    Nest,
    Split,
} // ':', '/', '+'

#[derive(Debug, Clone, PartialEq)]
pub struct GroupExpr(pub Vec<(String, Option<GroupOp>)>); // [(g1,None),(g2,Some(Cross)),...]

#[derive(Debug, Clone, PartialEq)]
pub enum SmoothKind {
    S,
    T2,
    TE,
    TI,
}
