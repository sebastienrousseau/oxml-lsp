---
title: oxml-lsp — Language Server Protocol for XML
description: XML diagnostics and linting over the Language Server Protocol, powered by oxml.
hide:
  - navigation
  - toc
---

<section class="dot-hero" markdown>

# oxml-lsp

<p class="tagline">XML diagnostics, linting, and position mapping over the Language Server Protocol (LSP) — powered by oxml with zero unsafe code.</p>

<div class="buttons">
  <a class="primary" href="https://docs.rs/oxml-lsp">API Docs →</a>
  <a href="https://github.com/sebastienrousseau/oxml-lsp">GitHub</a>
  <a href="DIAGNOSTICS/">Diagnostics</a>
  <a href="POSITIONS/">Positions</a>
</div>

</section>

## What's inside

<div class="grid cards" markdown>

- :material-code-tags:{ .lg .middle } **LSP 3.17 protocol**

    ---

    Full Language Server Protocol support over stdio and JSON-RPC for Neovim, VS Code, Helix, and Emacs.

    [→ Setup Guide](README.md)

- :material-alert-circle-outline:{ .lg .middle } **Rich diagnostics**

    ---

    Precise well-formedness errors, missing close tags, schema validation errors, and entity limit violations.

    [→ Diagnostics Guide](DIAGNOSTICS.md)

- :material-crosshairs-gps:{ .lg .middle } **Byte-accurate positions**

    ---

    Accurate UTF-8 and UTF-16 position translations mapping parser arena offsets directly to editor cursor positions.

    [→ Positions](POSITIONS.md)

- :material-shield-check:{ .lg .middle } **Zero `unsafe` code**

    ---

    `#![forbid(unsafe_code)]` at crate root. Resilient against malformed, adversarial, or explosive XML payloads.

    [→ Assurance Case](ASSURANCE-CASE.md)

</div>

## Quick start

Install `oxml-lsp`:

```bash
cargo install oxml-lsp
```

Configure in your editor (e.g., Neovim with `nvim-lspconfig`):

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "xml",
  callback = function()
    vim.lsp.start({
      name = "oxml-lsp",
      cmd = { "oxml-lsp" },
      root_dir = vim.fs.dirname(vim.fs.find({ ".git" }, { upward = true })[1]),
    })
  end,
})
```

## Where to next

- [**Diagnostics**](DIAGNOSTICS.md) — Diagnostic error codes and severity levels.
- [**Positions**](POSITIONS.md) — How oxml translates document offsets to editor positions.
- [**Assurance Case**](ASSURANCE-CASE.md) — Safety guarantees, protocol bounds, and threat analysis.
- [**Testing**](TESTING.md) — LSP harness tests and regression corpus.
- [**Roadmap**](ROADMAP.md) — Protocol extensions and completion support.

## Current release

- Release notes: [GitHub Releases](https://github.com/sebastienrousseau/oxml-lsp/releases)
- Crates.io: [crates.io/crates/oxml-lsp](https://crates.io/crates/oxml-lsp)
- Repository: [sebastienrousseau/oxml-lsp](https://github.com/sebastienrousseau/oxml-lsp)
