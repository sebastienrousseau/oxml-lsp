<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

## Where this is

**Not a language server yet.** The LSP JSON-RPC transport is not
implemented. What exists is `analyse()` and a command-line front end
over it.

That is stated at the top of the README rather than near the bottom,
and the crates.io description now says "diagnostics engine" rather than
"language server", because a crate called `oxml-lsp` that does not
speak LSP will otherwise be installed by someone expecting one.

## The order

**1. LSP transport.** `initialize`, `textDocument/didOpen`,
`didChange`, `didClose`, `publishDiagnostics`. A thin layer over
`analyse()`: positions are already zero-based and severities already
carry LSP's numbers, so most of it is framing and dispatch.

Open question: `positionEncoding`. See [POSITIONS.md](POSITIONS.md).

**2. Incremental sync.** Full-document sync first, because it is
correct and simple. A large document reparsed on every keystroke is
the reason to move on from it, and the arena parser is fast enough that
"large" here is larger than it sounds — worth measuring before
optimising.

**3. Schema-aware diagnostics** via `xmlschema`. This is where
diagnostics stop being one-per-document: a well-formed document can
violate its schema in many independent ways, and reporting all of them
is correct. See [DIAGNOSTICS.md](DIAGNOSTICS.md).

**4. Completion** from a schema — element and attribute names valid at
the cursor.

## Why the analysis is a library

Steps 3 and 4 add diagnostic *sources*, not front ends. Keeping
`analyse()` a pure function of a `&str` means the CLI, the transport
and any future embedding are all thin layers over the same tested code,
and none of them has to be running for the analysis to be tested.

## Not planned

- **Formatting.** It needs a serialiser, and the library reads only.
- **XSLT.**
- **Error recovery** for well-formedness. See
  [DIAGNOSTICS.md](DIAGNOSTICS.md).
