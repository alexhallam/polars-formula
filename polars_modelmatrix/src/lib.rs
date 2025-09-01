use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3_polars::prelude::{PyDataFrame, PySeries};
use polars_formula::{Formula, MaterializeOptions};

/// Generate response vector and design matrix from a Polars DataFrame using formula syntax.
#[pyfunction]
fn model_matrix(formula: &str, df: PyDataFrame) -> PyResult<(PySeries, PyDataFrame)> {
    let rust_df = df.into_df();

    let parsed = Formula::parse(formula).map_err(|e| PyValueError::new_err(e.to_string()))?;

    let (y_series, x_df) = parsed
        .materialize(&rust_df, MaterializeOptions::default())
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok((PySeries(y_series), PyDataFrame(x_df)))
}

/// A Python module implemented in Rust.
#[pymodule]
fn polars_modelmatrix(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(model_matrix, m)?)?;
    Ok(())
}
