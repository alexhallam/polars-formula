use super::ast::*;

pub fn pretty(spec: &ModelSpec) -> String {
    let mut parts = Vec::new();

    // Add family/link header if present
    if let Some(family) = &spec.family {
        parts.push(format!("family={}", pretty_family(family)));

        if let Some(link) = &spec.link {
            parts.push(format!("link={}", pretty_link(link)));
        }
    }

    // Add main formula
    let lhs = pretty_response(&spec.formula.lhs);
    let rhs = pretty_expr(&spec.formula.rhs);

    // Add aterms to LHS if present
    let lhs_with_aterms = if spec.formula.aterms.is_empty() {
        lhs.clone()
    } else {
        let aterms_str = spec
            .formula
            .aterms
            .iter()
            .map(pretty_aterm)
            .collect::<Vec<_>>()
            .join(" | ");
        format!("{} | {}", lhs, aterms_str)
    };

    if lhs.is_empty() {
        parts.push(rhs);
    } else {
        parts.push(format!("{} ~ {}", lhs_with_aterms, rhs));
    }

    // Add distributional parameter formulas
    for dpar in &spec.dpars {
        parts.push(format!("{} ~ {}", dpar.name, pretty_expr(&dpar.rhs)));
    }

    // Add autocorrelation terms
    for autocor in &spec.autocor {
        let args_str = if autocor.args.is_empty() {
            String::new()
        } else {
            autocor
                .args
                .iter()
                .map(|(k, v)| format!("{}={}", k, pretty_expr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        parts.push(format!("{}({})", autocor.name, args_str));
    }

    parts.join(" + ")
}

fn pretty_family(family: &Family) -> String {
    match family {
        Family::Builtin(name, args) => {
            if args.is_empty() {
                format!("{}()", name)
            } else {
                let args_str = args.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
                format!("{}({})", name, args_str)
            }
        }
        Family::Mixture(families) => {
            let families_str = families
                .iter()
                .map(pretty_family)
                .collect::<Vec<_>>()
                .join(", ");
            format!("mixture({})", families_str)
        }
        Family::Custom { name, dpars } => {
            let dpars_str = dpars
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            format!("custom_family(\"{}\", {})", name, dpars_str)
        }
    }
}

fn pretty_link(link: &Link) -> String {
    match link {
        Link::Named(name, args) => {
            if args.is_empty() {
                name.clone()
            } else {
                let args_str = args.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
                format!("{}({})", name, args_str)
            }
        }
    }
}

fn pretty_response(response: &Response) -> String {
    match response {
        Response::Var(v) => v.clone(),
        Response::Multi(vars) => {
            format!("cbind({})", vars.join(", "))
        }
        Response::Surv { time, event, time2 } => {
            if let Some(time2) = time2 {
                format!(
                    "Surv({}, {}, {})",
                    pretty_expr(time),
                    pretty_expr(event),
                    pretty_expr(time2)
                )
            } else {
                format!("Surv({}, {})", pretty_expr(time), pretty_expr(event))
            }
        }
        Response::Func { name, args } => {
            let args_str = args.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
            format!("{}({})", name, args_str)
        }
        Response::BinomialTrials { successes, trials } => {
            format!("{} | trials({})", pretty_expr(successes), pretty_expr(trials))
        }
    }
}

fn pretty_aterm(aterm: &Aterm) -> String {
    match aterm {
        Aterm::Se(expr) => format!("se({})", pretty_expr(expr)),
        Aterm::Weights(expr) => format!("weights({})", pretty_expr(expr)),
        Aterm::Trials(expr) => format!("trials({})", pretty_expr(expr)),
        Aterm::Cens(expr) => format!("cens({})", pretty_expr(expr)),
        Aterm::Trunc { lb, ub } => {
            let mut parts = Vec::new();
            if let Some(lb) = lb {
                parts.push(format!("lb={}", pretty_expr(lb)));
            }
            if let Some(ub) = ub {
                parts.push(format!("ub={}", pretty_expr(ub)));
            }
            format!("trunc({})", parts.join(", "))
        }
        Aterm::Subset(expr) => format!("subset({})", pretty_expr(expr)),
        Aterm::Rate(expr) => format!("rate({})", pretty_expr(expr)),
        Aterm::Thres { gr } => {
            if let Some(gr) = gr {
                format!("thres(gr={})", pretty_expr(gr))
            } else {
                "thres()".to_string()
            }
        }
        Aterm::Dec(expr) => format!("dec({})", pretty_expr(expr)),
        Aterm::Cat(expr) => format!("cat({})", pretty_expr(expr)),
        Aterm::Index(expr) => format!("index({})", pretty_expr(expr)),
        Aterm::VReal(exprs) => {
            let args_str = exprs.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
            format!("vreal({})", args_str)
        }
        Aterm::VInt(exprs) => {
            let args_str = exprs.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
            format!("vint({})", args_str)
        }
        Aterm::Mi => "mi()".to_string(),
    }
}

pub fn pretty_expr(expr: &Expr) -> String {
    match expr {
        Expr::Num(n) => n.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Str(s) => format!("\"{}\"", s),
        Expr::Var(v) => v.clone(),
        Expr::Sum(terms) => terms
            .iter()
            .map(pretty_expr)
            .collect::<Vec<_>>()
            .join(" + "),
        Expr::Prod(terms) => terms
            .iter()
            .map(|t| match t {
                Expr::Sum(_) => format!("({})", pretty_expr(t)),
                _ => pretty_expr(t),
            })
            .collect::<Vec<_>>()
            .join(" * "),
        Expr::Interaction(terms) => terms
            .iter()
            .map(|t| match t {
                Expr::Sum(_) => format!("({})", pretty_expr(t)),
                _ => pretty_expr(t),
            })
            .collect::<Vec<_>>()
            .join(":"),
        Expr::Nest { outer, inner, kind } => {
            let op = match kind {
                NestKind::Slash => "/",
                NestKind::In => "%in%",
            };
            format!("{} {} {}", pretty_expr(outer), op, pretty_expr(inner))
        }
        Expr::Pow { base, exp } => {
            let base_str = match base.as_ref() {
                Expr::Sum(_) | Expr::Prod(_) => format!("({})", pretty_expr(base)),
                _ => pretty_expr(base),
            };
            format!("{}^{}", base_str, pretty_expr(exp))
        }
        Expr::Group {
            inner,
            spec,
            kind,
            id,
        } => {
            let inner_str = pretty_expr(inner);
            let spec_str = pretty_group_spec(spec);
            let sep = match kind {
                GroupKind::Correlated => "|",
                GroupKind::Uncorrelated => "||",
            };
            if let Some(id) = id {
                format!("({}|{}|{})", inner_str, id, spec_str)
            } else {
                format!("({}{}{})", inner_str, sep, spec_str)
            }
        }
        Expr::Smooth { kind, vars, args } => {
            let kind_str = match kind {
                SmoothKind::S => "s",
                SmoothKind::T2 => "t2",
                SmoothKind::TE => "te",
                SmoothKind::TI => "ti",
            };
            let mut parts = vec![vars.join(", ")];
            for (k, v) in args {
                parts.push(format!("{}={}", k, pretty_expr(v)));
            }
            format!("{}({})", kind_str, parts.join(", "))
        }
        Expr::Func { name, args } => {
            let args_str = args.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
            format!("{}({})", name, args_str)
        }
        Expr::Identity(inner) => {
            format!("I({})", pretty_expr(inner))
        }
        Expr::Intercept(true) => "1".to_string(),
        Expr::Intercept(false) => "0".to_string(),
        Expr::Dot => ".".to_string(),
    }
}

fn pretty_group_spec(spec: &GroupSpec) -> String {
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
                    format!("{}{}", op_str, name)
                } else {
                    name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(""),
        GroupSpec::Func { name, args } => {
            let args_str = args.iter().map(pretty_expr).collect::<Vec<_>>().join(", ");
            format!("{}({})", name, args_str)
        }
    }
}
