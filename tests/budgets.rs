// SPDX-License-Identifier: MIT OR Apache-2.0
//! Analysis must reject documents outside the editor-facing resource budget.

use std::io::Write as _;
use std::io::{BufRead, Cursor, Read};
use std::process::{Child, Command, Stdio};

struct Process(Child);

impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn cli_stops_reading_oversized_stdin_without_waiting_for_eof() {
    let mut process = Process(
        Command::new(env!("CARGO_BIN_EXE_oxml-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start CLI"),
    );
    let mut input = process.0.stdin.take().expect("stdin");
    input
        .write_all(&vec![b' '; 8 * 1024 * 1024 + 1])
        .expect("write");
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(status) = process.0.try_wait().expect("status") {
            break Some(status);
        }
        if started.elapsed() >= std::time::Duration::from_secs(2) {
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    // Keep the writer open: rejecting only after read_to_string reaches
    // EOF would leave an untrusted stream free to grow the allocation.
    assert_eq!(status.and_then(|status| status.code()), Some(2));
    drop(input);
}

struct BodyProbe {
    input: Cursor<Vec<u8>>,
    body_reads: usize,
}

impl Read for BodyProbe {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.body_reads += 1;
        self.input.read(bytes)
    }
}

impl BufRead for BodyProbe {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.input.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.input.consume(amount);
    }
}

#[test]
fn an_oversized_frame_is_rejected_before_reading_its_body() {
    let header = format!("Content-Length: {}\r\n\r\n", 16 * 1024 * 1024 + 1);
    let mut input = BodyProbe {
        input: Cursor::new(header.into_bytes()),
        body_reads: 0,
    };
    oxml_lsp::lsp::serve(&mut input, Vec::new());
    assert_eq!(input.body_reads, 0, "oversized body was allocated and read");
}

#[test]
fn header_reads_stop_at_the_header_budget() {
    let mut input =
        Cursor::new(format!("X:{}", "x".repeat(16 * 1024)).into_bytes());
    oxml_lsp::lsp::serve(&mut input, Vec::new());
    assert!(input.position() <= 8193, "unbounded header was consumed");
}

#[test]
fn oversized_well_formed_documents_produce_a_limit_diagnostic() {
    let source = format!("<r>{}</r>", "x".repeat(8 * 1024 * 1024));
    let diagnostics = oxml_lsp::analyse(&source);
    assert!(!diagnostics.is_empty(), "analysis ignored the input budget");
    assert!(
        diagnostics[0].message.contains("limit"),
        "{:?}",
        diagnostics[0]
    );
}
