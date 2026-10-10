# Contributing

Use Bun and the committed `bun.lock`. Rust dependencies use the workspace
`Cargo.lock`. Start with the setup and validation commands in the README.

Keep detection in `gamespec-core::scanner`, comparisons in the pure engine,
and platform or UI glue outside those modules. Never use hardware model numbers,
generation, clock speeds or vendor names to rank performance.

For catalog changes, cite an accessible primary benchmark source, the test date,
metric, tested hardware variant and dataset. Preserve the uncertainty policy.
Do not combine scores from different test setups. Add cases covering unknown
models and ambiguous equivalence. Benchmarks are approximations for game
requirements, not FPS predictions.

Run the checks documented in the README before submitting a pull request.
Tests must work without a GPU. Include screenshots for UI changes, describe any
native detection changes and avoid committing real hardware reports or user paths.
