<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-lsp</h1>

<p align="center">
  Diagnostics for XML documents, powered by <a href="https://github.com/sebastienrousseau/oxml">oxml</a>.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-lsp/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-lsp/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-lsp"><img src="https://img.shields.io/crates/v/oxml-lsp.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-lsp"><img src="https://img.shields.io/badge/docs.rs-oxml-lsp-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/oxml-lsp"><img src="https://img.shields.io/badge/lib.rs-oxml-lsp-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/oxml-lsp"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/oxml-lsp?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```bash
cargo install --git https://github.com/sebastienrousseau/oxml-lsp
```

## Usage

```bash
oxml-lsp document.xml
cat document.xml | oxml-lsp
```

```text
3:5: warning: duplicate id `x`; first used on line 2 [duplicate-id]
7:3: hint: `filler` is empty and has no attributes [empty-element]
```

## Diagnostics

| Code | Severity | What it means |
|---|---|---|
| `not-well-formed` | Error | The document could not be parsed |
| `duplicate-id` | Warning | Two elements share an `id` |
| `empty-element` | Hint | An element with no children and no attributes |

Duplicate ids are legal XML but almost always a mistake — they break
every tool that resolves references by id.

## Design

The analysis lives in the library, not the binary, so it can be tested
without a language-server transport in the way. An editor integration
is mostly plumbing; the part worth testing is what it decides to say.

**A parse error suppresses everything else.** Once the tree cannot be
built there is nothing further to say, and guessing at structure past a
parse failure produces cascades of diagnostics that all describe the
same underlying mistake.

## Status

The analysis and a command-line front end work. The LSP JSON-RPC
transport is not implemented yet — that should follow a decision about
which editor to target first, and the useful part is already here and
testable.

## The oxml suite

Every member ships the **same version number**, so there is never a
compatibility table to consult. Versions advance in `0.0.1` steps along
the `0.0.x` line; `0.1.0` follows `0.0.999`.

| Crate | What it is |
|---|---|
| [`oxml`](https://github.com/sebastienrousseau/oxml) | Core — parser, tree, XPath 1.0 |
| [`oxml-cli`](https://github.com/sebastienrousseau/oxml-cli) | Command-line querying and validation |
| [`oxml-lsp`](https://github.com/sebastienrousseau/oxml-lsp) | Diagnostics for editors |
| [`oxml-mcp`](https://github.com/sebastienrousseau/oxml-mcp) | Model Context Protocol server |
| [`oxml-wasm`](https://github.com/sebastienrousseau/oxml-wasm) | WebAssembly bindings |
| [`xmlschema`](https://github.com/sebastienrousseau/xmlschema) | XSD validation |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). By participating you agree to
the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
