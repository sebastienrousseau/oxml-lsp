<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<h1 align="center">oxml-lsp</h1>

<p align="center">
  XML diagnostics — the analysis engine behind a language server, and a
  command-line front end for it. Powered by
  <a href="https://github.com/sebastienrousseau/oxml">oxml</a>, with zero
  <code>unsafe</code> code.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/oxml-lsp/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/oxml-lsp/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/oxml-lsp"><img src="https://img.shields.io/crates/v/oxml-lsp.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/oxml-lsp"><img src="https://img.shields.io/badge/docs.rs-oxml--lsp-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
</p>

---

> ### Status: not yet a language server
>
> **The LSP JSON-RPC transport is not implemented.** What exists is the
> analysis engine — `analyse()` returns positioned diagnostics — and a
> command-line front end that prints them.
>
> That is genuinely useful today for linting XML in CI or a
> `Makefile`, and it is not what "LSP" leads you to expect, so it is
> the first thing on this page rather than a note near the bottom.
>
> The transport is a thin layer over `analyse()` and follows a
> decision about which editor to target first.

---

## Contents

- [What it does today](#what-it-does-today)
- [Install](#install)
- [Quick Start](#quick-start)
- [The oxml ecosystem](#the-oxml-ecosystem)
- [Library API](#library-api)
- [Diagnostics](#diagnostics)
- [Exit status](#exit-status)
- [Design](#design)
- [Roadmap](#roadmap)
- [Examples](#examples)
- [When not to use oxml-lsp](#when-not-to-use-oxml-lsp)
- [FAQ](#faq)
- [Development](#development)
- [Security](#security)
- [License](#license)

---

## What it does today

- **A library.** `analyse(&str) -> Vec<Diagnostic>`, each with a
  severity, a line and column, a message and a stable code.
- **A command-line linter.** Reads a file or standard input, prints
  diagnostics, exits non-zero if any is an error.

What it does **not** do: speak LSP, offer completion, format, or
validate against a schema.

## Install

```bash
cargo install oxml-lsp
```

As a library:

```toml
[dependencies]
oxml-lsp = "0.0.5"
```

## Quick Start

```bash
$ oxml-lsp config.xml
3:13: error: at byte 39: </hostname> closes <port> [not-well-formed]
```

```bash
$ oxml-lsp valid.xml
no diagnostics
```

Reads standard input when given no file, so it composes:

```bash
curl -s https://example.com/feed.xml | oxml-lsp
find . -name '*.xml' -print0 | xargs -0 -n1 oxml-lsp
```

## The oxml ecosystem

| Crate | What it is |
|---|---|
| [`oxml`](https://github.com/sebastienrousseau/oxml) | The library: parser, tree, XPath 1.0 |
| [`xmlschema`](https://github.com/sebastienrousseau/xmlschema) | XSD validation |
| [`oxml-cli`](https://github.com/sebastienrousseau/oxml-cli) | The command line |
| [`oxml-wasm`](https://github.com/sebastienrousseau/oxml-wasm) | WebAssembly bindings |
| [`oxml-mcp`](https://github.com/sebastienrousseau/oxml-mcp) | Model Context Protocol server |
| **`oxml-lsp`** | **This crate — diagnostics** |

All six ship one version number, in steps of 0.0.1.

## Library API

```rust
use oxml_lsp::{Severity, analyse};

let diagnostics = analyse("<a><b></a>");
assert_eq!(diagnostics.len(), 1);

let d = &diagnostics[0];
assert_eq!(d.severity, Severity::Error);
assert_eq!(d.code, "not-well-formed");
// Positions are zero-based, as LSP requires.
assert_eq!(d.start.line, 0);
```

| Item | |
|---|---|
| `analyse(source) -> Vec<Diagnostic>` | Every diagnostic for a document |
| `Diagnostic` | `start`, `end`, `severity`, `message`, `code` |
| `Position` | `line`, `character` — **zero-based**, as LSP requires |
| `Severity` | `Error`, `Warning`, `Information`, `Hint` — numbered as LSP numbers them |

Both are shaped for the transport that is coming: positions are
zero-based and `Severity`'s discriminants are LSP's numbers, so the
layer will be a cast rather than a conversion.

The command-line front end adds one to both before printing, because a
person counting lines starts at one.

## Diagnostics

**One per document, at most, today.** A parse failure stops the parse,
and continuing past one produces cascades of diagnostics that are all
consequences of the first — an editor showing eleven squiggles for one
missing `>` is worse than one.

Positions in the **library** are zero-based, as LSP requires. The
**command line** prints them one-based, because that is how a person
counts.

Either way the character offset is counted in characters, not bytes,
so a multi-byte character earlier on the line does not shift it.

| Code | Meaning |
|---|---|
| `not-well-formed` | The document violates a well-formedness rule |

The code is stable and is what you match on. The message is
human-facing and may be reworded.

## Exit status

| Code | Meaning |
|---|---|
| 0 | No errors |
| 1 | At least one error |
| 2 | Could not read the file or standard input |

The distinction between 1 and 2 matters in a script: "this document has
a problem" and "you gave me a path that does not exist" need different
responses, and a tool that returns 1 for both makes `set -e` a
liability.

## Design

**The analysis is a library, not a binary.** `analyse()` takes a `&str`
and returns owned diagnostics. No I/O, no protocol, no global state —
so it is testable without an editor, and the CLI and the eventual LSP
transport are both thin layers over the same function.

**No error recovery.** The parser stops at the first well-formedness
violation. Recovery means guessing what a malformed document meant, and
two implementations guessing differently is how an editor and a build
tool come to disagree about a file.

**Positions come from the parser's byte offsets.** `oxml` returns an
offset rather than a formatted message precisely so that a consumer can
turn it into whatever it needs — a line and column here, a caret
elsewhere, an LSP `Range` later.

## Roadmap

In the order it makes sense to build:

1. **LSP transport** — `initialize`, `textDocument/didOpen`,
   `didChange`, `publishDiagnostics`. A thin layer over `analyse()`.
2. **Incremental sync**, so a large document is not reparsed on every
   keystroke.
3. **Schema-aware diagnostics** via `xmlschema`, which is where
   validity errors rather than well-formedness errors come from.
4. **Completion** from a schema.

Steps 3 and 4 are why the analysis is a library: they add diagnostic
sources, not a new front end.

## Examples

[`examples/`](examples/) asserts its output, so the invocations in this
README fail CI when they stop being true.

| Example | What it shows |
|---|---|
| [`lint.sh`](examples/lint.sh) | Files, standard input, exit codes |
| [`library.rs`](examples/library.rs) | `analyse()` and every field of a `Diagnostic` |

## When not to use oxml-lsp

- **You want a working language server today.** This is not one yet.
  Use `lemminx` (Java) or `vscode-xml`.
- **You need schema validation.** Use `oxml-cli validate` or
  `xmlschema` directly.
- **You need formatting.** `xmllint --format`.
- **You need every error, not the first.** There is no recovery.

## FAQ

### Why is it called `oxml-lsp` if it is not a language server?

Because that is what it is being built into, and the crate name was
claimed when the suite was published. The gap is stated at the top of
this page rather than buried, and the crates.io description says
"diagnostics engine" rather than "language server".

### Can I use it in CI now?

Yes — that is the useful part today:

```yaml
- run: find . -name '*.xml' -print0 | xargs -0 -n1 oxml-lsp
```

Exits non-zero on the first malformed document.

### Why only one diagnostic per document?

Because the parser stops at the first violation, and a recovering
parser produces cascades where every diagnostic after the first is a
consequence of it. One accurate diagnostic beats eleven guesses.

### How do I get diagnostics for schema violations?

Not from here yet. Use `oxml-cli validate schema.xsd doc.xml`.

### Are the positions LSP-compatible?

Nearly. `Position` is already zero-based and `Severity`'s discriminants
are already LSP's numbers.

The remaining gap is that `character` counts **Unicode characters**
while LSP's default `positionEncoding` counts **UTF-16 code units**.
They agree for everything in the Basic Multilingual Plane and disagree
by one per emoji or other non-BMP character earlier on the same line.
The transport layer will either convert or negotiate `utf-32`, which
LSP 3.17 allows.

### Does it fetch anything?

No. No network code, and external entities are never dereferenced.

### Which editors work with it?

None, directly, until the transport lands. You can wire the CLI into
any editor that runs a linter on save.

## Development

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
./examples/lint.sh
cargo run --example library
```

## Security

No network access. External entities never dereferenced, so a document
cannot make the linter read `/etc/passwd`. Entity expansion and
recursion bounded. `#![forbid(unsafe_code)]`.

See
<https://github.com/sebastienrousseau/oxml/blob/main/doc/SECURITY-MODEL.md>.

## License

Licensed under either of Apache-2.0 ([LICENSE-APACHE](LICENSE-APACHE))
or MIT ([LICENSE-MIT](LICENSE-MIT)), at your option.
