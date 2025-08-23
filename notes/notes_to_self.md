# Notes

Some things to remember because I will forget.

### Why represent y as a bundle when family=binomial?

In classic R `glm` with `family=binomial`, the response can be a two-column matrix `cbind(successes, failures)`.

In `brms` with `y | trials(n)`, the response is one column (successes) plus a sidecar (trials) — functionally equivalent.

I internally represent binomial counts as a bundle (successes, trials) and exposing a helper to export the 2-column `cbind(successes, failures)` form when needed.

Why this design? The log-likelihood only needs `y` (successes) and `m` (trials). “Failures = m − y” is derived, so keeping (`y`, `m`) avoids accidental negatives and keeps validation simple (`0 ≤ y ≤ m`).

