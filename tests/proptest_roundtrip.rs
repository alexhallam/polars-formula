use chumsky::Parser;
use polars_formula::dsl::{parser::parser, pretty::pretty};
use proptest::prelude::*;

fn ident_strat() -> impl Strategy<Value = String> {
    // Avoid reserved words minimally (real impl: filter a set)
    "[A-Za-z_][A-Za-z0-9_\\.]{0,12}".prop_filter("reserved", |s| s != "family" && s != "link")
}

// Tiny generator for expressions (grow over time)
fn expr_strat() -> impl Strategy<Value = String> {
    let atom = prop_oneof![
        ident_strat(),
        "[0-9]{1,3}(\\.[0-9]{1,2})?".prop_map(|s| s),
        Just("1".to_string()),
        Just("0".to_string()),
        (ident_strat(), ident_strat()).prop_map(|(a, b)| format!("{a}:{b}")),
        (ident_strat(), ident_strat()).prop_map(|(a, b)| format!("{a}*{b}")),
        (ident_strat(), 1u8..=3).prop_map(|(a, k)| format!("poly({a},{k})")),
        (ident_strat(), ident_strat()).prop_map(|(a, b)| format!("({a}+{b})^2")),
    ];
    atom
}

fn formula_strat() -> impl Strategy<Value = String> {
    (ident_strat(), expr_strat()).prop_map(|(y, rhs)| format!("{y} ~ {rhs}"))
}

proptest! {
  #[test]
  fn parse_pretty_parse_equiv(s in formula_strat()) {
      let p = parser();
      let spec1 = p.parse(s.as_str()).unwrap();
      let pretty1 = pretty(&spec1);

      let spec2 = parser().parse(pretty1.as_str()).unwrap();

      // For now compare pretty forms (once canonicalize is implemented, compare AST)
      let pretty2 = pretty(&spec2);
      prop_assert_eq!(pretty1, pretty2);
  }
}
