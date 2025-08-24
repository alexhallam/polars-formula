# Notes

Some things to remember because I will forget.

### Why represent y as a bundle when family=binomial?

In classic R `glm` with `family=binomial`, the response can be a two-column matrix `cbind(successes, failures)`.

In `brms` with `y | trials(n)`, the response is one column (successes) plus a sidecar (trials) — functionally equivalent.

I internally represent binomial counts as a bundle (successes, trials) and exposing a helper to export the 2-column `cbind(successes, failures)` form when needed.

Why this design? The log-likelihood only needs `y` (successes) and `m` (trials). “Failures = m − y” is derived, so keeping (`y`, `m`) avoids accidental negatives and keeps validation simple (`0 ≤ y ≤ m`).

### Why do we need to canonicalize the formula?

We need to canonicalize the formula to ensure that the formula is in a consistent format. This is important because it allows us to compare formulas and ensure that they are equivalent.

### Why have a parser decoupled from the materializer?

The parser is responsible for parsing the formula into an AST. The materializer is responsible for materializing the AST into a design matrix.

### Why would I ever want to just parse a formula?

You might want to parse a formula to get the AST, or to pretty-print it.

I also hope that others will be able to build their own modeling tools on top of this. It may be useful to have a way to parse a formula without materializing it.

### Why do we need to materialize the formula?

This is the primary deliverable of the library. Polars dataframes materialized against a formula should make downstream modeling easier.