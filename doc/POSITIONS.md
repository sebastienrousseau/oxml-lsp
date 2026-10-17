<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Positions

## Three boundaries, on purpose

| | Base | Counted in |
|---|---|---|
| `oxml_lsp::Position` | **Zero** | Characters |
| The command-line output | **One** | Characters |
| LSP stdio ranges | **Zero** | UTF-16 code units |

The library is zero-based because LSP is. The transport converts scalar
columns to UTF-16 without changing the native API. The command line
adds one before printing, because a person counting lines starts at
one and every editor agrees with them.

Getting this backwards is easy — this README documented the library as
one-based until an example was written against it and did not compile.

## Characters, not bytes

`Position::character` counts Unicode characters from the start of the
line, not bytes.

`<a>éé</b>` reports character 6 and not byte 8. A byte offset would put
an editor's caret in the wrong place on any line containing a
non-ASCII character, which for XML is a great many lines.

The underlying parser reports a **byte** offset — deliberately, so that
each consumer converts to what it needs. This crate converts to
characters; something drawing a caret over raw bytes would not.

## UTF-16 at the transport boundary

LSP's default `positionEncoding` is **UTF-16 code units**. The native
analyser counts Unicode scalar values; the stdio transport converts
both endpoints before publishing diagnostics.

They agree for everything in the Basic Multilingual Plane, which is
almost everything. They disagree by one per non-BMP character — an
emoji, a rarer CJK ideograph — earlier on the same line, because those
are one character and two UTF-16 code units.

The adapter sorts the endpoints, scans each source character at most
once, and assigns the converted endpoints back to the original diagnostic
indices. For `d` diagnostics and `n` source characters, conversion costs
O(n + d log d) time and O(d) temporary space. It does not rescan the
document once per diagnostic or change diagnostic ordering. Protocol
tests cover BMP/non-BMP text, newlines, CRLF and end-of-input positions.

## Ranges

A `Diagnostic` carries `start` and `end`, not a single point, because
LSP underlines a range. Today the range is narrow — the construct that
failed — since the parser stops at the first violation and there is no
recovered region to underline.
