<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Positions

## Two conventions, on purpose

| | Base | Counted in |
|---|---|---|
| `oxml_lsp::Position` | **Zero** | Characters |
| The command-line output | **One** | Characters |

The library is zero-based because LSP is, and the transport layer that
is coming should be a cast rather than a conversion. The command line
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

## The UTF-16 gap

LSP's default `positionEncoding` is **UTF-16 code units**. This crate
counts characters.

They agree for everything in the Basic Multilingual Plane, which is
almost everything. They disagree by one per non-BMP character — an
emoji, a rarer CJK ideograph — earlier on the same line, because those
are one character and two UTF-16 code units.

Two ways to close it when the transport lands:

1. Convert on the way out, counting `c.len_utf16()`.
2. Negotiate `positionEncoding: "utf-32"`, which LSP 3.17 permits and
   which matches what is already stored.

Option 2 is preferable where the client supports it, because a
conversion that runs per diagnostic is a conversion that can be wrong.

## Ranges

A `Diagnostic` carries `start` and `end`, not a single point, because
LSP underlines a range. Today the range is narrow — the construct that
failed — since the parser stops at the first violation and there is no
recovered region to underline.
