// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! `oxml-lsp` — diagnostics for XML, from the command line or an
//! editor.
//!
//! Run with a path to analyse a file and print diagnostics in a form
//! a person or a linter can read. The language-server transport is
//! deliberately not implemented yet: the analysis in [`oxml_lsp`] is
//! the part worth having, and wiring it to LSP's JSON-RPC framing is
//! plumbing that should follow a decision about which client to
//! target first.

#![forbid(unsafe_code)]

use std::io::Read as _;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "oxml-lsp — XML diagnostics\n\n\
             USAGE:\n    oxml-lsp [FILE]\n\n\
             Reads standard input when no file is given.\n\n\
             EXIT STATUS:\n    0  no errors\n    1  at least one error\n"
        );
        return ExitCode::SUCCESS;
    }

    let source = if let Some(path) = args.first() {
        match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("oxml-lsp: cannot read {path}: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("oxml-lsp: cannot read stdin: {e}");
            return ExitCode::from(2);
        }
        buf
    };

    let diagnostics = oxml_lsp::analyse(&source);
    if diagnostics.is_empty() {
        println!("no diagnostics");
        return ExitCode::SUCCESS;
    }

    let mut had_error = false;
    for d in &diagnostics {
        let label = match d.severity {
            oxml_lsp::Severity::Error => {
                had_error = true;
                "error"
            }
            oxml_lsp::Severity::Warning => "warning",
            oxml_lsp::Severity::Information => "info",
            oxml_lsp::Severity::Hint => "hint",
        };
        println!(
            "{}:{}: {label}: {} [{}]",
            d.start.line + 1,
            d.start.character + 1,
            d.message,
            d.code
        );
    }

    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
