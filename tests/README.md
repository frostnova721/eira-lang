# Tests

Run the active suite with `cargo test --locked`.

Tests are separated by responsibility:

- `scanner_test.rs`: tokens, escapes, and interpolation.
- `parser_test.rs`: syntax, precedence, and AST structure.
- `weave_analyser_test.rs`: semantic analysis and inferred weaves. The helper checks
  collected `Augury` diagnostics as well as returned errors.
- `compiler_test.rs`: full compilation, diagnostic locations, invalid programs,
  and import errors.
- `runtime_test.rs`: source-to-output tests through the CLI, covering control flow,
  iterator cycles (decks, inclusive ranges, boundaries, and accumulation),
  recursion, captures, signs and attunements, decks, imports, and `--no-run`.
- `native_spells_test.rs`: native spell registry and method resolution.
- `common/mod.rs`: isolated temporary source files, cleanup, and compiler setup.

Run one group with, for example, `cargo test --locked --test runtime_test`.
Runtime success tests check exact stdout and empty stderr because the CLI currently
returns exit status zero even on language errors. Tests do not rely on benchmark timings.

Three runtime regressions are explicitly ignored until their implementation is fixed:

- Nested cycles overwrite the outer iteration value and can panic in the VM.
- Returning a closure from a spell with parameters carries the outer spell's
  argument metadata into later calls of the closure.
- Passing a bare source filename to the CLI resolves sibling imports from `/`
  rather than the source directory. Import coverage uses an absolute source path.

Run these reproducers with `cargo test --locked --test runtime_test -- --ignored`.
They assert the intended behavior and are expected to fail currently; remove their
`ignore` attributes when fixing the underlying bugs.
