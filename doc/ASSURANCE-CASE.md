<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Assurance case

An assurance case is an argument, supported by evidence, that the
software is adequately secure for what it does. This one is
deliberately short: the strongest security claim this project makes is
about what it *cannot* do.

## What this software is

`oxml-lsp` is an XML linter and diagnostics library.

## What it consumes

Its inputs are XML documents supplied by an editor or on standard input. The threat model assumes every one of them is
hostile: a document written specifically to crash the parser, exhaust
memory, or reach something it should not.

## The claim

Untrusted input is checked at the boundaries documented in
[SECURITY.md](../SECURITY.md). This crate forbids unsafe Rust in its own
source. Those controls do not prove the absence of denial of service,
logic defects, or vulnerabilities in dependencies and runtimes.

## The argument

### Memory safety is structural, not tested for

Parsing is delegated to `oxml`, which never fetches external entities.

### Resource exhaustion is bounded, not merely unlikely

The concrete source, recursion and evaluation limits, together with
caller-owned limits, are recorded in [the threat model](../SECURITY.md).
A stack overflow aborts the process, so recursion must be bounded before
entering another frame; aborting is not a mitigation. These controls do
not constitute a process-wide memory or concurrency bound.

### Correctness is measured against an external standard

The project does not grade its own homework. Where an independent
conformance suite exists it is run, its denominator is published
alongside its rate, and the result is ratcheted so an unreviewed change
in either direction fails the build.

## The evidence

- `#![forbid(unsafe_code)]`, checked by a CI job.
- 41 tests over `analyse()`, every `Diagnostic` field, positions and exit codes, and the LSP transport added at 0.0.8 -- including byte-counted frames, bounded headers/bodies, bounded CLI stdin, UTF-16 ranges and rejection of requests after shutdown.
- Line coverage gated at a 95% floor.
- `analyse()` is linear in document size; a quadratic pass over attributes was found by benchmark and fixed, taking 1,088 ms to 9.5 ms.

## What this case does *not* claim

- It does not claim the absence of defects or memory corruption in
  dependencies, foreign runtimes or the compiler. Source-level unsafe-code
  prohibition is one control, not a proof about the entire process.
- It does not claim that parser budgets replace deployment limits.
  Callers must separately bound ingress, concurrent requests, wall-clock
  execution and process or WebAssembly memory.
- It does not claim independent review. This project has one
  maintainer, and no third party has audited it. That is recorded here
  rather than left to be inferred.

## Reporting a problem with this case

If you can construct an input that violates the claim above, that is a
vulnerability. See [SECURITY.md](../SECURITY.md).
