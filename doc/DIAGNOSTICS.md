<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Diagnostics

## One per document

A well-formedness error stops the analysis. There is at most one
diagnostic, and that is deliberate.

The alternative is error recovery: guess what the malformed document
meant, carry on, and report everything else found. In an editor that
produces cascades — eleven squiggles for one missing `>`, ten of them
consequences of the first. The user fixes the first and the other ten
disappear, which teaches them to distrust all of them.

It also produces disagreement. Recovery means guessing, two
implementations guess differently, and an editor and a build tool end
up disagreeing about the same file.

When schema validation arrives this changes: a *valid* document can
have many independent violations, and reporting all of them is
correct. The rule is that well-formedness stops, validity accumulates.

## Codes

| Code | Meaning |
|---|---|
| `not-well-formed` | The document violates a well-formedness rule |

The code is **stable**. Match on it, group by it, suppress by it. The
`message` is human-facing and may be reworded in any release.

That split exists because an editor's suppression list keyed on a
message string breaks the first time the wording improves.

## Severities

`Severity`'s discriminants are LSP's numbers — `Error = 1`,
`Warning = 2`, `Information = 3`, `Hint = 4` — so the transport is a
cast.

Only `Error` is produced today. `Warning` is where a legal-but-suspect
construct would go: a duplicated namespace declaration, an empty
attribute where a schema expects content.

## What is not diagnosed

- **Validity.** A document can be well-formed and still violate its
  schema. That needs `xmlschema`, and it is on the roadmap.
- **Style.** Indentation, attribute order, line length. A formatter's
  job, and there is no formatter.
- **Anything requiring the external DTD subset.** The parser never
  fetches one, by the same design that forecloses XXE.
