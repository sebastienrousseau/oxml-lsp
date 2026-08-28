// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-lsp. All rights reserved.

//! Linting a document and reading the diagnostics.
//!
//! Run with:
//!
//! ```text
//! cargo run --example lint_a_document
//! ```
//!
//! This is what an editor integration calls. The crate is named for a
//! protocol it does not yet speak -- there is no JSON-RPC transport --
//! so `analyse` is the whole public surface, and this is how to use
//! it directly in the meantime.

use oxml_lsp::{Severity, analyse};

const DOC: &str = r#"<?xml version="1.0"?>
<config>
  <entry id="a" id="a">duplicate attribute</entry>
  <entry>unclosed
</config>
"#;

fn main() {
    let diagnostics = analyse(DOC);

    println!(
        "{} diagnostic(s) for {} bytes\n",
        diagnostics.len(),
        DOC.len()
    );
    for d in &diagnostics {
        // Positions are zero-based, as the Language Server Protocol
        // defines them; an editor showing line 1 means line 0 here.
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Information => "info",
            Severity::Hint => "hint",
        };
        println!(
            "{}:{}: {severity}: {}",
            d.start.line, d.start.character, d.message
        );
    }

    // Well-formed is not the same as silent. `analyse` also lints,
    // so a document with no *errors* can still carry hints -- an
    // editor should not treat a non-empty result as a failure.
    let clean = analyse("<a><b/></a>");
    let errors = clean
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    println!(
        "\nwell-formed document: {} diagnostic(s), {errors} error(s)",
        clean.len()
    );
    for d in &clean {
        println!("  {:?}: {}", d.severity, d.message);
    }
    assert_eq!(errors, 0, "well-formed means no errors, not no output");
}
