# Security Policy

## Supported versions

oxml-lsp is pre-1.0. Only the most recent `0.0.x` release receives security
fixes.

## Reporting a vulnerability

Please report privately, not as a public issue:

- GitHub's [private vulnerability reporting](https://github.com/sebastienrousseau/oxml-lsp/security/advisories/new)
- or email <sebastian.rousseau@gmail.com>

You should get an acknowledgement within 72 hours.

## Threat model

The stdio transport accepts client-controlled JSON messages. Headers
are capped at 8 KiB and bodies at 16 MiB before body allocation. The shared
JSON parser also limits source size and nesting. Analysis uses the strict
core XML profile, including an 8 MiB source/decoded-document limit.

Internal entities may expand within core budgets. The parser does not
automatically fetch external entities or documents. Diagnostic positions
and messages are data, not executable commands. Editor clients and the
launching process remain separate trust boundaries.

**Memory safety.** This crate forbids unsafe Rust in its own source.
Dependencies and runtimes have separate safety obligations. This
does not rule out resource exhaustion, logic defects or protocol errors.

## What is not a vulnerability

- Rejecting a document other parsers accept, when the document is not
  well-formed. Please still report it — it is a bug, just not a
  security one.
