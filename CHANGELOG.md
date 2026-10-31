# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.13] - 2026-10-31

### Security

- **Dual-lockfile security auditing.** Enforce `cargo audit --deny warnings`
  across both `Cargo.lock` and `fuzz/Cargo.lock` in
  `.github/workflows/security.yml`.

### Changed

- **Upgrade to `oxml-json` 0.0.12.** Lock `oxml-json = "=0.0.12"` across
  root and fuzz lockfiles.
- **Ecosystem release cycle v0.0.13.** Align package and fuzz harness
  metadata at `0.0.13` across the `oxml` suite.

## [0.0.12] - 2026-10-31

### Changed

- **Upgrade to `oxml` 0.0.12.** Align package and fuzz harness metadata
  at `0.0.12` and refresh root and fuzz lockfiles.
- **Automated GitHub release pipeline.** Add the `release-page` workflow
  job to verify the published crates.io archive, generate CycloneDX SBOMs
  and SHA-256 checksums, and attach SLSA build provenance attestations.
- **Scorecard pull-request SARIF parity.** Normalize `pull_request`
  Scorecard SARIF output so `supply-chain/branch-protection` and
  `supply-chain/online-scm` configurations are always reported.
- **CI and release action updates.** Advance `Swatinem/rust-cache` to
  v2.9.3, `taiki-e/install-action` to v2.88.0, `github/codeql-action` to
  v4.39.1, and `rust-lang/crates-io-auth-action` to v1.0.6.

## [0.0.11] - 2026-10-17

### Changed

- Pin Scorecard execution by verified container digest and disable public
  results publication. Retain SARIF artifacts, categorize posture findings
  in GitHub code scanning, and display workflow status instead of a score.

- Permit NCSA only for libfuzzer-sys 0.4.13 in the fuzz workspace;
  retain the root license policy and check both graphs in CI.
- Require an annotated tag signed by the approved public key, with the
  exact project/version subject, and use crates.io Trusted Publishing
  instead of a stored publishing token. Registry configuration still
  requires independent confirmation before tagging.

- Align unpublished harnesses at 0.0.11 and match example coverage to
  exact source paths, with negative regression contracts in local/CI gates.
- Wait for all actual release prerequisites before compiling, propagate
  publication failures and keep the laptop helper dry-run-only.

- Reject requests after shutdown with InvalidRequest and ignore document
  notifications until exit, as required by the LSP lifecycle contract.
- Reject structured/boolean request IDs before cloning or echoing them,
  retaining string, number and null identifiers.
- Convert diagnostic ranges to UTF-16 at the stdio boundary while
  preserving the native Unicode scalar API and diagnostic order.
- Bound LSP headers at 8 KiB and bodies at 16 MiB before allocation.
- Stop CLI file/stdin reads at the 8 MiB source budget without waiting
  for an oversized stream to close.
- Use the bounded, lexically strict JSON parser from oxml-json 0.0.11.
- Keep the laptop release helper dry-run-only and enforce it in CI.

- Apply explicit untrusted-document resource limits at the application
  boundary; query frontends also use per-call XPath evaluation budgets.

- Align the publishable package and family dependencies at 0.0.11,
  refresh root/fuzz lockfiles, and retain Rust 1.89.0 as the minimum.
- Validate closed issue/dependency changes against current sources and
  use pinned compiler, coverage, fuzz and audit tools in CI.

- **Breaking: the MSRV is now Rust 1.89.0** (was 1.86.0), one floor
  shared with the maintainer's other Rust crates.
- **The MSRV job checks its toolchain first.** It already set
  `RUSTUP_TOOLCHAIN`; it now also asserts `rustc --version` before
  building.

## [0.0.10] - 2026-10-05

### Changed

- **Upgrade to `oxml` 0.0.10.** Duplicate `xmlns` declarations and a
  `<?xml-stylesheet ...?>` impersonating the XML declaration are now
  diagnosed as the parser reports them.
- **Ecosystem release cycle v0.0.10.** Begin next iteration cycle across the
  entire lockstep suite.
- **Documentation migration.** Migrated website and rendered manual to
  `oxmllib.com/lsp` (`oxmllib.github.io`) using the Lucid SSG theme.

## [0.0.9] - 2026-10-04

### Changed

- **Upgrade to `oxml` 0.0.9 and `oxml-json` 0.0.9.** Accelerated XML parsing
  via chunked SWAR scanning and generational arena slot recycling.

## [0.0.8] - 2026-08-29

### Added

- **It speaks the Language Server Protocol.** `lsp::serve` reads and
  writes JSON-RPC over stdio with `Content-Length` framing, handling
  `initialize`, `shutdown`, `exit`, `textDocument/didOpen`,
  `didChange`, `didClose` and `publishDiagnostics`. Until now the
  crate was named for a protocol it did not speak, and said so.

  `Content-Length` counts **bytes**, not characters. A document
  containing an emoji makes the two differ, and a client that counts
  characters desynchronises the stream on its first such message;
  there is a test for exactly that. Sync is full-document
  (`textDocumentSync: 1`), which is correct before it is fast.

  `serve` is generic over its two ends, so it is driven from a buffer
  in tests rather than from a spawned process.

- A fuzz target over `analyse`, run for 300 seconds on every pull
  request, and a TESTING document describing what is checked and how.

### Changed

- JSON is now parsed by `oxml-json` rather than a private copy, so the
  MCP server and the language server cannot disagree about what a
  number means.

### Fixed

- The README and roadmap described the transport as forthcoming after
  it had shipped -- including an FAQ entry explaining why the crate
  was not a language server. Documentation that describes shipped work
  as pending is worse than none.

### Security

- Branch coverage measured and gated, CodeQL added, the Developer
  Certificate of Origin adopted and enforced, and Scorecard results
  kept out of code scanning.

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
