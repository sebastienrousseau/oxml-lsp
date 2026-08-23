// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! End-to-end behaviour of the `oxml-lsp` binary.
//!
//! The analysis itself is unit-tested in the library. What these cover
//! is the contract an editor task or a CI step sees: which stream each
//! line lands on, and the exit status, which is the only part a script
//! can branch on.

use std::io::Write as _;
use std::process::{Command, Stdio};

struct Output {
    stdout: String,
    stderr: String,
    code: i32,
}

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oxml-lsp"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary runs");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin.as_bytes())
        .expect("write");
    let out = child.wait_with_output().expect("wait");
    Output {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().expect("exited normally"),
    }
}

/// Write `contents` to a uniquely named file and return its path.
fn temp_file(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("oxml-lsp-test-{name}.xml"));
    std::fs::write(&path, contents).expect("write temp file");
    path
}

#[test]
fn a_clean_document_exits_zero() {
    let out = run(&[], "<a><b>text</b></a>");
    assert_eq!(out.code, 0, "{out:?}", out = out.stderr);
    assert!(out.stdout.contains("no diagnostics"), "{}", out.stdout);
    assert!(out.stderr.is_empty(), "{}", out.stderr);
}

#[test]
fn a_malformed_document_exits_one() {
    // The distinction a CI step branches on.
    let out = run(&[], "<a><b></a>");
    assert_eq!(out.code, 1);
    assert!(out.stdout.contains("error"), "{}", out.stdout);
    assert!(out.stdout.contains("not-well-formed"), "{}", out.stdout);
}

#[test]
fn diagnostics_carry_a_one_based_position() {
    // Editors and humans both count from one; the library counts from
    // zero, and the binary is where that is translated.
    let out = run(&[], "<a><b></a>");
    let line = out.stdout.lines().next().expect("a diagnostic");
    let mut parts = line.split(':');
    let l: usize = parts.next().expect("line").parse().expect("numeric line");
    let c: usize = parts.next().expect("col").parse().expect("numeric column");
    assert!(l >= 1, "line was {l}");
    assert!(c >= 1, "column was {c}");
}

#[test]
fn a_warning_alone_still_exits_zero() {
    // Only errors fail the run; a warning is information, and exiting
    // non-zero for one would make the tool unusable in a pipeline.
    let out = run(&[], "<a><b/><b/></a>");
    assert_eq!(out.code, 0, "stdout: {} stderr: {}", out.stdout, out.stderr);
    if !out.stdout.contains("no diagnostics") {
        assert!(!out.stdout.contains("error:"), "{}", out.stdout);
    }
}

#[test]
fn a_duplicate_id_is_reported() {
    let out = run(&[], r#"<a><b id="x"/><c id="x"/></a>"#);
    assert!(out.stdout.contains("duplicate-id"), "{}", out.stdout);
}

#[test]
fn a_file_argument_is_read() {
    let path = temp_file("ok", "<a><b>text</b></a>");
    let out = run(&[path.to_str().expect("utf-8 path")], "");
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(out.stdout.contains("no diagnostics"), "{}", out.stdout);
    drop(std::fs::remove_file(&path));
}

#[test]
fn a_file_argument_reports_its_diagnostics() {
    let path = temp_file("bad", "<a><b></a>");
    let out = run(&[path.to_str().expect("utf-8 path")], "");
    assert_eq!(out.code, 1);
    assert!(out.stdout.contains("not-well-formed"), "{}", out.stdout);
    drop(std::fs::remove_file(&path));
}

#[test]
fn an_unreadable_file_is_distinct_from_an_invalid_one() {
    // Exit 2, not 1: "I could not read it" and "it is wrong" are
    // different outcomes and a script must be able to tell them apart.
    let out = run(&["/nonexistent/path/to/nothing.xml"], "");
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("cannot read"), "{}", out.stderr);
    assert!(out.stdout.is_empty(), "{}", out.stdout);
}

#[test]
fn errors_go_to_stderr_not_stdout() {
    // Otherwise a diagnostic ends up inside whatever consumes stdout.
    let out = run(&["/nonexistent/file.xml"], "");
    assert!(out.stderr.contains("oxml-lsp:"), "{}", out.stderr);
    assert!(!out.stdout.contains("oxml-lsp:"), "{}", out.stdout);
}

#[test]
fn help_is_available_under_both_flags() {
    for flag in ["-h", "--help"] {
        let out = run(&[flag], "");
        assert_eq!(out.code, 0, "{flag}");
        assert!(out.stdout.contains("USAGE"), "{flag}: {}", out.stdout);
        assert!(out.stdout.contains("EXIT STATUS"), "{flag}");
    }
}

#[test]
fn help_wins_over_a_file_argument() {
    // A user asking for help while a path is still on the line should
    // get help, not an analysis of the file.
    let path = temp_file("help", "<a><b></a>");
    let out = run(&[path.to_str().expect("utf-8 path"), "--help"], "");
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("USAGE"), "{}", out.stdout);
    drop(std::fs::remove_file(&path));
}

#[test]
fn empty_input_is_a_diagnostic_not_a_crash() {
    let out = run(&[], "");
    assert!(out.code == 0 || out.code == 1, "code was {}", out.code);
    assert!(!out.stdout.is_empty() || !out.stderr.is_empty());
}
