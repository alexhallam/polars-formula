use chumsky::prelude::*;
use std::collections::HashMap;

use super::ast::*;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("parse error: {0}")]
    Generic(String),
}

/// Create a parser for statistical formula strings.
///
/// This function returns a parser that can parse R-style statistical formulas
/// into a `ModelSpec` structure. The parser supports the full range of formula
/// syntax including variables, interactions, polynomials, and more.
///
/// # Returns
///
/// Returns a parser that takes a stream of characters and produces a `ModelSpec`.
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use polars_formula::dsl::parser::parser;
///
/// let p = parser();
/// let result = p.parse("y ~ x1 + x2".chars().collect::<Vec<_>>());
/// assert!(result.is_ok());
/// ```
///
/// ## Complex Formula
/// ```rust
/// use polars_formula::dsl::parser::parser;
///
/// let p = parser();
/// let result = p.parse("mpg ~ wt*hp + poly(disp, 3) + (1|cyl)".chars().collect::<Vec<_>>());
/// assert!(result.is_ok());
/// ```
///
/// ## Error Handling
/// ```rust
/// use polars_formula::dsl::parser::parser;
///
/// let p = parser();
/// let result = p.parse("y ~~ x".chars().collect::<Vec<_>>()); // Invalid syntax
/// assert!(result.is_err());
/// ```
///
/// # Supported Syntax
///
/// - **Variables**: `x`, `income`, `age`
/// - **Response**: `y ~ ...` (left side of `~`)
/// - **Predictors**: `x1 + x2` (right side of `~`)
/// - **Interactions**: `x1:x2` (product terms)
/// - **Products**: `x1*x2` (expands to main effects + interactions)
/// - **Polynomials**: `poly(x, degree)`
/// - **Random Effects**: `(1|group)`, `(x|group)`
/// - **Smooth Terms**: `s(x)`, `t2(x,z)`, `te(x,y)`
/// - **Constants**: `1`, `0`
/// - **Intercept Removal**: `-1`
pub fn parser() -> impl Parser<char, ModelSpec, Error = Simple<char>> {
    // --- lexeme helpers ---
    let ident = text::ident().map(|s: String| s);
    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(|s| Expr::Num(s.parse::<f64>().unwrap()));
    let boolean = just("true")
        .to(Expr::Bool(true))
        .or(just("false").to(Expr::Bool(false)))
        .or(just("TRUE").to(Expr::Bool(true)))
        .or(just("FALSE").to(Expr::Bool(false)));
    let string = just('"')
        .ignore_then(
            filter(|c| *c != '"' && *c != '\\')
                .or(just('\\').ignore_then(any()))
                .repeated(),
        )
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Expr::Str);

    // dotted identifiers for np.log, foo.bar
    let dotted_ident = ident
        .separated_by(just('.'))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|v| v.join("."));

    // forward decls
    let expr = recursive(|expr| {
        // smooths: s(x, k=10, bs="tp"), t2(x,z), te(...), ti(...)
        let varlist = ident
            .clone()
            .separated_by(just(','))
            .at_least(1)
            .collect::<Vec<_>>();
        let named_arg = choice((
            just("k").to("k".to_string()),
            just("bs").to("bs".to_string()),
            just("by").to("by".to_string()),
            just("id").to("id".to_string()),
            just("sp").to("sp".to_string()),
            just("fx").to("fx".to_string()),
            just("knots").to("knots".to_string()),
            just("xt").to("xt".to_string()),
            just("pc").to("pc".to_string()),
            text::ident(),
        ))
        .then_ignore(just('=').padded())
        .then(expr.clone())
        .map(|(k, v)| (k, v));

        let smooth_args = named_arg
            .separated_by(just(','))
            .allow_trailing()
            .map(|kvs: Vec<(String, Expr)>| kvs.into_iter().collect::<HashMap<_, _>>());

        let smooth = choice((
            just("s")
                .ignore_then(just('('))
                .ignore_then(varlist.clone())
                .then(just(',').ignore_then(smooth_args.clone()).or_not())
                .then_ignore(just(')'))
                .map(|(vars, args)| Expr::Smooth {
                    kind: SmoothKind::S,
                    vars,
                    args: args.unwrap_or_default(),
                }),
            just("t2")
                .ignore_then(just('('))
                .ignore_then(varlist.clone())
                .then(just(',').ignore_then(smooth_args.clone()).or_not())
                .then_ignore(just(')'))
                .map(|(vars, args)| Expr::Smooth {
                    kind: SmoothKind::T2,
                    vars,
                    args: args.unwrap_or_default(),
                }),
            just("te")
                .ignore_then(just('('))
                .ignore_then(varlist.clone())
                .then(just(',').ignore_then(smooth_args.clone()).or_not())
                .then_ignore(just(')'))
                .map(|(vars, args)| Expr::Smooth {
                    kind: SmoothKind::TE,
                    vars,
                    args: args.unwrap_or_default(),
                }),
            just("ti")
                .ignore_then(just('('))
                .ignore_then(varlist.clone())
                .then(just(',').ignore_then(smooth_args.clone()).or_not())
                .then_ignore(just(')'))
                .map(|(vars, args)| Expr::Smooth {
                    kind: SmoothKind::TI,
                    vars,
                    args: args.unwrap_or_default(),
                }),
        ));

        // group terms: (1|g), (x||g), (1|ID|g)
        let group_op = choice((
            just(':').to(GroupOp::Cross),
            just('/').to(GroupOp::Nest),
            just('+').to(GroupOp::Split),
        ));
        let group_expr = ident
            .clone()
            .then((group_op.then(ident.clone())).repeated())
            .map(|(g1, tail)| {
                let mut v = vec![(g1, None)];
                for (op, name) in tail {
                    v.push((name, Some(op)));
                }
                GroupExpr(v)
            });

        let group_spec = group_expr.clone().map(GroupSpec::Expr).or(dotted_ident
            .clone()
            .then(
                just('(')
                    .ignore_then(expr.clone().separated_by(just(',')).allow_trailing())
                    .then_ignore(just(')')),
            )
            .map(|(name, args)| GroupSpec::Func { name, args }));

        let group_inner = choice((
            just('0').to(Expr::Intercept(false)),
            just('1').to(Expr::Intercept(true)),
        ))
        .or(expr.clone());

        let group_term = choice((
            just('(')
                .ignore_then(group_inner.clone())
                .then_ignore(just('|').padded())
                .then(group_spec.clone())
                .then_ignore(just(')'))
                .map(|(inner, spec)| Expr::Group {
                    inner: Box::new(inner),
                    spec,
                    kind: GroupKind::Correlated,
                    id: None,
                }),
            just('(')
                .ignore_then(group_inner.clone())
                .then_ignore(just('|').padded().then(just('|').padded()))
                .then(group_spec.clone())
                .then_ignore(just(')'))
                .map(|(inner, spec)| Expr::Group {
                    inner: Box::new(inner),
                    spec,
                    kind: GroupKind::Uncorrelated,
                    id: None,
                }),
            just('(')
                .ignore_then(group_inner.clone())
                .then_ignore(just('|').padded())
                .then_ignore(just("ID"))
                .then_ignore(just('|').padded())
                .then(group_spec.clone())
                .then_ignore(just(')'))
                .map(|(inner, spec)| Expr::Group {
                    inner: Box::new(inner),
                    spec,
                    kind: GroupKind::Correlated,
                    id: Some("ID".into()),
                }),
        ));

        // func_call (includes dotted)
        let args = expr.clone().separated_by(just(',')).allow_trailing();
        let func_call = dotted_ident
            .clone()
            .then(just('(').ignore_then(args.clone()).then_ignore(just(')')))
            .map(|(name, args)| Expr::Func { name, args });

        // atoms
        let atom = choice((
            number.clone(),
            boolean.clone(),
            string.clone(),
            smooth.clone(),
            group_term.clone(),
            func_call.clone(),
            ident.clone().map(Expr::Var),
            just('(').ignore_then(expr.clone()).then_ignore(just(')')),
            just('.').to(Expr::Dot),
            just('I')
                .ignore_then(just('('))
                .ignore_then(expr.clone())
                .then_ignore(just(')'))
                .map(|e| Expr::Identity(Box::new(e))),
            just('1').to(Expr::Intercept(true)),
            just('0').to(Expr::Intercept(false)),
        ))
        .padded();

        // pow: term ('^' (number | '(' sum ')'))?
        let pow = atom
            .clone()
            .then(
                just('^')
                    .ignore_then(choice((
                        number.clone(),
                        just('(').ignore_then(expr.clone()).then_ignore(just(')')),
                    )))
                    .or_not(),
            )
            .map(|(base, exp)| {
                if let Some(exp) = exp {
                    Expr::Pow {
                        base: Box::new(base),
                        exp: Box::new(exp),
                    }
                } else {
                    base
                }
            });

        // inter: pow (':' pow)*  -> Interaction([...])
        let inter = pow
            .clone()
            .separated_by(just(':').padded())
            .at_least(1)
            .map(|mut xs: Vec<Expr>| {
                if xs.len() == 1 {
                    xs.remove(0)
                } else {
                    Expr::Interaction(xs)
                }
            });

        // nest: inter (('/'|"%in%") inter)*
        let nest_op = just('/')
            .to(NestKind::Slash)
            .or(just("%in%").to(NestKind::In));
        let nest = inter
            .clone()
            .then((nest_op.then(inter.clone())).repeated())
            .map(|(first, rest)| {
                rest.into_iter().fold(first, |acc, (k, rhs)| Expr::Nest {
                    outer: Box::new(acc),
                    inner: Box::new(rhs),
                    kind: k,
                })
            });

        // prod: nest ('*' nest)*  (keep; expand later)
        let prod = nest.clone().separated_by(just('*')).at_least(1).map(|xs| {
            if xs.len() == 1 {
                xs.into_iter().next().unwrap()
            } else {
                Expr::Prod(xs)
            }
        });

        // sum: prod (('+'|'-') prod)*
        let sum = prod
            .clone()
            .then((one_of("+-").padded().then(prod.clone())).repeated())
            .map(|(head, tail)| {
                if tail.is_empty() {
                    return head;
                }
                let mut xs = vec![head];
                for (op, term) in tail {
                    if op == '-' {
                        xs.push(Expr::Func {
                            name: "NEG".into(),
                            args: vec![term],
                        });
                    } else {
                        xs.push(term);
                    }
                }
                Expr::Sum(xs)
            });

        sum.padded()
    });

    // LHS (response)
    let response_basic = ident
        .clone()
        .map(|v| Response::Var(v))
        .or(just("mvbind")
            .ignore_then(just('('))
            .ignore_then(ident.clone().separated_by(just(',')).at_least(2))
            .then_ignore(just(')'))
            .map(Response::Multi))
        .or(just("cbind")
            .ignore_then(just('('))
            .ignore_then(ident.clone().separated_by(just(',')).at_least(2))
            .then_ignore(just(')'))
            .map(Response::Multi))
        .or(just("Surv")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(','))
            .then(expr.clone())
            .then(just(',').ignore_then(expr.clone()).or_not())
            .then_ignore(just(')'))
            .map(|((time, event), time2)| Response::Surv { time, event, time2 }))
        .or(dotted_ident
            .clone()
            .then(
                just('(')
                    .ignore_then(expr.clone().separated_by(just(',')).allow_trailing())
                    .then_ignore(just(')')),
            )
            .map(|(name, args)| Response::Func { name, args }));

    // Proper aterm parsing
    let aterm = choice((
        just("se")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Se),
        just("weights")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Weights),
        just("trials")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Trials),
        just("cens")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Cens),
        just("trunc")
            .ignore_then(just('('))
            .ignore_then(
                just("lb")
                    .ignore_then(just('=').padded())
                    .ignore_then(expr.clone())
                    .or_not()
                    .then(
                        just(',')
                            .ignore_then(just("ub"))
                            .ignore_then(just('=').padded())
                            .ignore_then(expr.clone())
                            .or_not(),
                    ),
            )
            .then_ignore(just(')'))
            .map(|(lb, ub)| Aterm::Trunc { lb, ub }),
        just("subset")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Subset),
        just("rate")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Rate),
        just("thres")
            .ignore_then(just('('))
            .ignore_then(
                just("gr")
                    .ignore_then(just('=').padded())
                    .ignore_then(expr.clone())
                    .or_not(),
            )
            .then_ignore(just(')'))
            .map(|gr| Aterm::Thres { gr }),
        just("dec")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Dec),
        just("cat")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Cat),
        just("index")
            .ignore_then(just('('))
            .ignore_then(expr.clone())
            .then_ignore(just(')'))
            .map(Aterm::Index),
        just("vreal")
            .ignore_then(just('('))
            .ignore_then(expr.clone().separated_by(just(',')).at_least(1))
            .then_ignore(just(')'))
            .map(Aterm::VReal),
        just("vint")
            .ignore_then(just('('))
            .ignore_then(expr.clone().separated_by(just(',')).at_least(1))
            .then_ignore(just(')'))
            .map(Aterm::VInt),
        just("mi")
            .ignore_then(just('('))
            .ignore_then(just(')'))
            .to(Aterm::Mi),
    ));

    let aterm_chain = aterm.clone().separated_by(just(',')).collect::<Vec<_>>();

    let response = response_basic
        .clone()
        .then(just('|').padded().ignore_then(aterm_chain.clone()).or_not())
        .map(|(base, chain)| {
            let chain = chain.unwrap_or_default();
            // Handle y | trials(n) syntax
            if chain.len() == 1 {
                if let Aterm::Trials(trials_expr) = &chain[0] {
                    if let Response::Var(successes_var) = base {
                        return (
                            Response::BinomialTrials {
                                successes: Expr::Var(successes_var),
                                trials: trials_expr.clone(),
                            },
                            vec![],
                        );
                    }
                }
            }
            (base, chain)
        })
        .or(response_basic.clone().map(|base| (base, vec![])));

    // RHS
    let rhs = expr.clone();

    // Optional header
    let family_spec = dotted_ident
        .clone()
        .then(
            just('(')
                .ignore_then(expr.clone().separated_by(just(',')).allow_trailing())
                .then_ignore(just(')')),
        )
        .map(|(name, args)| Family::Builtin(name, args));

    let family = family_spec
        .clone()
        .or(just("mixture")
            .ignore_then(just('('))
            .ignore_then(family_spec.clone().separated_by(just(',')).at_least(2))
            .then_ignore(just(')'))
            .map(Family::Mixture))
        .or(just("custom_family")
            .ignore_then(just('('))
            .ignore_then(string.clone())
            .then(just(',').ignore_then(string.clone()).repeated())
            .then_ignore(just(')'))
            .map(|(name, dpars)| Family::Custom {
                name: if let Expr::Str(s) = name {
                    s
                } else {
                    unreachable!()
                },
                dpars: dpars
                    .into_iter()
                    .map(|e| if let Expr::Str(s) = e { s } else { "".into() })
                    .collect(),
            }));

    let link = dotted_ident
        .clone()
        .then(
            just('(')
                .ignore_then(expr.clone().separated_by(just(',')).allow_trailing())
                .then_ignore(just(')')),
        )
        .or(dotted_ident.clone().map(|n| (n, vec![])))
        .map(|(n, args)| Link::Named(n, args));

    let header = just("family")
        .ignore_then(just('=').padded())
        .ignore_then(family)
        .then(
            just(',')
                .ignore_then(just("link"))
                .ignore_then(just('=').padded())
                .ignore_then(link)
                .or_not(),
        );

    // Distributional parameter formulas
    let dpar_name = choice((
        just("sigma").to("sigma".to_string()),
        just("nu").to("nu".to_string()),
        just("phi").to("phi".to_string()),
        just("zi").to("zi".to_string()),
        just("hu").to("hu".to_string()),
        just("zoi").to("zoi".to_string()),
        just("coi").to("coi".to_string()),
        just("kappa").to("kappa".to_string()),
        just("beta").to("beta".to_string()),
        just("disc").to("disc".to_string()),
        just("bs").to("bs".to_string()),
        just("ndt").to("ndt".to_string()),
        just("bias").to("bias".to_string()),
    ));

    let dpar_formula = dpar_name
        .then_ignore(just('~'))
        .then(rhs.clone())
        .map(|(name, rhs)| Dpar { name, rhs });

    // Autocorrelation terms
    let autocor_call = choice((
        just("ar").to("ar"),
        just("ma").to("ma"),
        just("arma").to("arma"),
        just("cosy").to("cosy"),
        just("unstr").to("unstr"),
        just("sar").to("sar"),
        just("car").to("car"),
        just("fcor").to("fcor"),
    ))
    .then(
        just('(')
            .ignore_then(
                text::ident()
                    .then_ignore(just('=').padded())
                    .then(expr.clone())
                    .separated_by(just(','))
                    .allow_trailing(),
            )
            .then_ignore(just(')')),
    )
    .map(|(name, kvs)| {
        let mut args = HashMap::new();
        for (k, v) in kvs {
            args.insert(k.to_string(), v);
        }
        Autocor {
            name: name.to_string(),
            args,
        }
    });

    // Parse the main formula structure
    choice((
        // Full formula with dpars and autocor: [header] response ~ rhs + dpar1 ~ rhs1 + autocor1 + ... [, trailing_header]
        header
            .clone()
            .or_not()
            .then(
                response
                    .clone()
                    .then_ignore(just('~').padded())
                    .then(rhs.clone()),
            )
            .then(just('+').ignore_then(dpar_formula.clone()).repeated())
            .then(just('+').ignore_then(autocor_call.clone()).repeated())
            .then(just(',').padded().ignore_then(header.clone()).or_not())
            .then_ignore(end())
            .map(
                |((((leading_hdr, (resp, rhs)), dpars), acs), trailing_hdr)| {
                    let (lhs, aterms) = resp;
                    // Use trailing header if present, otherwise use leading header
                    let final_hdr = trailing_hdr.or(leading_hdr);
                    ModelSpec {
                        family: final_hdr.as_ref().map(|(f, _)| f.clone()),
                        link: final_hdr.and_then(|(_, lk)| lk),
                        formula: Formula { lhs, rhs, aterms },
                        dpars,
                        autocor: acs,
                    }
                },
            ),
        // Simple formula: [header] response ~ rhs [, trailing_header]
        header
            .clone()
            .or_not()
            .then(
                response
                    .clone()
                    .then_ignore(just('~').padded())
                    .then(rhs.clone()),
            )
            .then(just(',').padded().ignore_then(header.clone()).or_not())
            .then_ignore(end())
            .map(|((leading_hdr, (resp, rhs)), trailing_hdr)| {
                let (lhs, aterms) = resp;
                // Use trailing header if present, otherwise use leading header
                let final_hdr = trailing_hdr.or(leading_hdr);
                ModelSpec {
                    family: final_hdr.as_ref().map(|(f, _)| f.clone()),
                    link: final_hdr.and_then(|(_, lk)| lk),
                    formula: Formula { lhs, rhs, aterms },
                    dpars: vec![],
                    autocor: vec![],
                }
            }),
        // RHS-only: [header] rhs [, trailing_header]
        header
            .clone()
            .or_not()
            .then(rhs)
            .then(just(',').padded().ignore_then(header.clone()).or_not())
            .then_ignore(end())
            .map(|((leading_hdr, rhs), trailing_hdr)| {
                // Use trailing header if present, otherwise use leading header
                let final_hdr = trailing_hdr.or(leading_hdr);
                ModelSpec {
                    family: final_hdr.as_ref().map(|(f, _)| f.clone()),
                    link: final_hdr.and_then(|(_, lk)| lk),
                    formula: Formula {
                        lhs: Response::Var("".to_string()),
                        rhs,
                        aterms: vec![],
                    },
                    dpars: vec![],
                    autocor: vec![],
                }
            }),
    ))
}
