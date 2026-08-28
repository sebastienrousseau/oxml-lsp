# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.7] - 2026-08-28

### Added

- **Benchmarks.** `analyse()` runs on every keystroke in an editor, so
  the figure that matters is latency on a small buffer rather than
  throughput on a large file. The benchmark earned its place
  immediately: it is what found `analyse()` being quadratic in the
  number of attributes, which cost 1,088 ms on a document that now
  takes 9.5.

- A **gate script** (`./scripts/gate.sh`) running everything CI runs,
  and a **publish script** running the same checks a release does.

- An **Examples** job in CI. `examples/lint.sh` asserts its output and
  says so in its own header, and the README said the invocations in it
  "fail CI when they stop being true" -- but nothing ran them.

### Changed

- Built on oxml 0.0.7, which reads a document from any `BufRead`. The
  suite ships one version number across all six crates.

- The README now follows the same shape as the rest of the suite, and
  gained the Benchmarks, Ecosystem comparison, Documentation and
  Acknowledgements sections it lacked. The comparison says plainly
  that `lemminx` is what to use if you want a working XML language
  server today.

### Removed

- `criterion` from dev-dependencies. Nothing referenced it -- the
  benchmark is `harness = false` and times with `Instant` -- and it
  cost about a minute of compilation on every `cargo bench`.

## [0.0.6] - 2026-08-26

### Changed

- Built on oxml 0.0.6 and xmlschema 0.0.6. The suite ships one version
  number across all six crates.

  xmlschema 0.0.6 is the substantial half of this release: its W3C
  conformance pass rate moved from 71.7% to 95.6%, and its coverage of
  the suite -- the share of tests that produce an answer meaning
  anything -- from 27.0% to 87.6%. Schemas this crate previously read
  as valid, and did not enforce, are now either enforced or reported
  as unenforceable.

## [0.0.5] - 2026-08-24

### Changed

- Built on oxml 0.0.5, which completes `XPath` 1.0: all thirteen axes
  and all 27 functions.

  **One behaviour change reaches expressions passed through this
  crate.** A function name outside the specification's library, or a
  call with the wrong number of arguments, used to compile and evaluate
  to an empty node-set. Both are now compile errors, reported with an
  offset. `starts-with("abc")` answered `true` before, because the
  absent argument read as the empty string.

  Six functions that previously answered `""` now work:
  `substring-before`, `substring-after`, `translate`, `name`, `id` and
  `lang`. So do the `following`, `preceding` and `namespace` axes.

## [0.0.4] - 2026-08-24

### Added

- Rebuilt on oxml 0.0.4. No new surface here; the diagnostics come
  from the core parser, which gained a great deal of correctness.

## [0.0.3] - 2026-08-22

### Added

- Initial release. Language server for XML documents, powered by oxml
- Tracks the version line of the [`oxml`](https://github.com/sebastienrousseau/oxml)
  core, so a given version of any suite member is built and tested against
  the matching core.

[0.0.3]: https://github.com/sebastienrousseau/oxml-lsp/releases/tag/v0.0.3
