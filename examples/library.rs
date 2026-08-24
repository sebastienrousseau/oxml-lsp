// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! Using `analyse()` directly.
//!
//! The analysis is a library rather than a binary so that it can be
//! embedded: in a build tool, a test harness, or -- eventually -- an
//! LSP transport, which will be a thin layer over this same function.
//!
//! Run with:
//!
//! ```text
//! cargo run --example library
//! ```

use oxml_lsp::{Severity, analyse};

const BROKEN: &str =
    "<config>\n  <name>ok</name>\n  <port>8080</hostname>\n</config>";

fn main() {
    println!("== a well-formed document ==");
    let clean = analyse("<a><b>x</b></a>");
    assert!(clean.is_empty(), "nothing to report");
    println!("  {} diagnostics", clean.len());

    println!("\n== a mismatched end tag ==");
    let found = analyse(BROKEN);
    // One diagnostic, not a cascade. The parser stops at the first
    // violation, because everything after it would be a consequence of
    // it rather than a separate problem.
    assert_eq!(found.len(), 1, "one diagnostic per document");

    for d in &found {
        println!("  severity : {:?}", d.severity);
        println!(
            "  range    : {}:{} to {}:{}  (zero-based)",
            d.start.line, d.start.character, d.end.line, d.end.character
        );
        println!("  code     : {}", d.code);
        println!("  message  : {}", d.message);

        assert_eq!(d.severity, Severity::Error);
        assert_eq!(
            d.code, "not-well-formed",
            "the code is stable; match on it"
        );
        // Zero-based, as LSP requires. The command-line front end adds
        // one before printing, because a person counts from one.
        assert_eq!(d.start.line, 2, "the third line");
        assert!(d.end.line >= d.start.line, "a range runs forwards");
    }

    println!("\n== drawing a caret ==");
    // The position is exposed rather than pre-formatted so a consumer
    // can render it however it needs -- a caret here, an LSP `Range`
    // later.
    for d in &found {
        let line = BROKEN.lines().nth(d.start.line).unwrap_or_default();
        println!("{:>3} | {line}", d.start.line + 1);
        println!("    | {}^ {}", " ".repeat(d.start.character), d.message);
    }

    println!("\n== severities are numbered as LSP numbers them ==");
    // So the transport layer, when it lands, is a cast rather than a
    // match.
    for severity in [
        Severity::Error,
        Severity::Warning,
        Severity::Information,
        Severity::Hint,
    ] {
        println!("  {severity:?} = {}", severity as u8);
    }
}
