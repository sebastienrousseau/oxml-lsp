// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! `oxml-lsp` — diagnostics for XML, from the command line or an
//! editor.
//!
//! Run with a path to analyse a file and print diagnostics in a form
//! a person or a linter can read. `--stdio` runs the language-server
//! transport. Both entry points use the same analysis library.

#![forbid(unsafe_code)]

use std::io::Read as _;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!(
            "oxml-lsp — XML diagnostics\n\n\
             USAGE:\n    oxml-lsp [FILE]\n    oxml-lsp --stdio\n\n\
             Reads standard input when no file is given.\n\n\
             --stdio runs as a language server, speaking LSP over\n\
             standard input and output. This is the mode an editor\n\
             starts; it is not useful from a terminal.\n\n\
             EXIT STATUS:\n    0  no errors\n    1  at least one error\n"
        );
        return ExitCode::SUCCESS;
    }

    // `--stdio` is the conventional flag an editor passes to start a
    // language server. The protocol owns the streams from here.
    if args.iter().any(|a| a == "--stdio") {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        oxml_lsp::lsp::serve(stdin.lock(), stdout.lock());
        return ExitCode::SUCCESS;
    }

    let source = match read_source(args.first().map(String::as_str)) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("oxml-lsp: {error}");
            return ExitCode::from(2);
        }
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

fn read_source(path: Option<&str>) -> Result<String, String> {
    if let Some(path) = path {
        let file = std::fs::File::open(path)
            .map_err(|error| format!("cannot read {path}: {error}"))?;
        read_bounded(file)
            .map_err(|error| format!("cannot read {path}: {error}"))
    } else {
        read_bounded(std::io::stdin().lock())
            .map_err(|error| format!("cannot read stdin: {error}"))
    }
}

fn read_bounded(input: impl std::io::Read) -> Result<String, String> {
    const LIMIT: usize = 8 * 1024 * 1024;
    let mut source = String::new();
    let _ = input
        .take((LIMIT + 1) as u64)
        .read_to_string(&mut source)
        .map_err(|error| error.to_string())?;
    if source.len() > LIMIT {
        return Err("XML source limit exceeded (8 MiB)".to_owned());
    }
    Ok(source)
}
