from __future__ import annotations

import polars as pl
from polars_ols import build_expressions_from_patsy_formula


def _parse_formula(formula: str):
    exprs, add_intercept = build_expressions_from_patsy_formula(
        formula, include_dependent_variable=True
    )
    return exprs[0], exprs[1:], add_intercept


def model_matrix(df: pl.DataFrame, formula: str) -> tuple[pl.Series, pl.DataFrame]:
    """Return response vector y and design matrix X from a Polars DataFrame.

    Args:
        df: Source Polars DataFrame
        formula: Patsy-style formula string (e.g., "y ~ x1 + x2 - 1")

    Returns:
        (y, X): y is a pl.Series; X is a pl.DataFrame of feature columns
    """
    y_expr, x_exprs, add_intercept = _parse_formula(formula)

    # Materialize y
    y = df.select(y_expr.alias("__y__")).to_series()

    # Materialize X columns
    cols: list[pl.Expr] = []
    if add_intercept:
        cols.append(pl.lit(1.0).alias("Intercept"))

    for i, expr in enumerate(x_exprs):
        # Try to derive a stable column name from the expression; fallback to x{i}
        try:
            name = expr.meta.output_name()  # type: ignore[attr-defined]
        except Exception:
            name = None
        if not name:
            name = f"x{i+1}"
        cols.append(expr.alias(name))

    X = df.select(cols)
    return y, X

