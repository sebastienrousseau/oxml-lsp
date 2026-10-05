<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Roadmap

## Where this is

**It speaks LSP.** The transport landed in 0.0.8: `initialize`,
`shutdown`, `exit`, `textDocument/didOpen`, `didChange`, `didClose`
and `publishDiagnostics`, framed with `Content-Length` over stdio.
`analyse()` and the command-line front end remain, and the transport
is a thin layer over the same function.

`Content-Length` counts **bytes**, not characters. A document
containing an emoji makes the two differ, and a client that counts
characters desynchronises the stream on its first such message; there
is a test for exactly that.

Sync is full-document (`textDocumentSync: 1`). See the order below.

## The order

**1. Incremental sync.** Full-document sync first, because it is
correct and simple. A large document reparsed on every keystroke is
the reason to move on from it, and the arena parser is fast enough that
"large" here is larger than it sounds — worth measuring before
optimising.

**2. Schema-aware diagnostics** via `xmlschema`. This is where
diagnostics stop being one-per-document: a well-formed document can
violate its schema in many independent ways, and reporting all of them
is correct. See [DIAGNOSTICS.md](DIAGNOSTICS.md).

**3. Completion** from a schema — element and attribute names valid at
the cursor.

## Why the analysis is a library

Steps 2 and 3 add diagnostic *sources*, not front ends. Keeping
`analyse()` a pure function of a `&str` means the CLI, the transport
and any future embedding are all thin layers over the same tested code,
and none of them has to be running for the analysis to be tested.

## Not planned

- **Formatting.** It needs a serialiser, and the library reads only.
- **XSLT.**
- **Error recovery** for well-formedness. See
  [DIAGNOSTICS.md](DIAGNOSTICS.md).
