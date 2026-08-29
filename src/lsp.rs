// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-lsp. All rights reserved.

//! The Language Server Protocol, over stdio.
//!
//! This is the transport the crate is named for. Until now it linted
//! from a command line and the README said plainly that it was not a
//! language server; it now speaks enough of the protocol to be one for
//! diagnostics.
//!
//! # What it implements
//!
//! - `initialize` / `initialized`, advertising full text-document sync
//! - `textDocument/didOpen`, `didChange`, `didClose`
//! - `textDocument/publishDiagnostics`, pushed after every open and
//!   change
//! - `shutdown` / `exit`
//!
//! # What it does not
//!
//! Completion, hover, formatting, go-to-definition and schema
//! awareness. Those need more than [`analyse`]
//! produces, and claiming them in the server capabilities while
//! returning nothing is worse than not advertising them: an editor
//! that believes the advertisement stops offering its own fallback.
//!
//! # Framing
//!
//! LSP is JSON-RPC with an HTTP-style header. Every message is
//! `Content-Length: N`, a blank line, then exactly N **bytes** of
//! JSON. Bytes, not characters — a document containing an emoji makes
//! those differ, and a server that counts characters desynchronises the
//! stream on the first non-ASCII message it sends.

use std::collections::HashMap;
use std::io::{BufRead, Write};

use crate::{Severity, analyse};
use oxml_json::{self as json, Json};

/// Serve the protocol over a byte stream until the client exits.
///
/// Generic over its two ends so a test can drive it through an
/// in-memory pipe: the alternative is spawning a process, which
/// measures the operating system alongside the thing under test and
/// cannot assert on what was written without a second pipe.
pub fn serve<R: BufRead, W: Write>(input: R, mut output: W) {
    let mut server = Server::new();
    let mut reader = input;

    while let Some(message) = read_message(&mut reader) {
        let Ok(request) = json::parse(&message) else {
            // A malformed message is answered, not fatal: JSON-RPC
            // defines a parse error for exactly this, and dropping the
            // connection would lose every later message too.
            let _ = write_message(
                &mut output,
                &error_response(&Json::Null, -32700, "parse error"),
            );
            continue;
        };

        for reply in server.handle(&request) {
            if write_message(&mut output, &reply).is_err() {
                return;
            }
        }
        if server.exited {
            return;
        }
    }
}

/// One connection's state.
struct Server {
    /// Open documents by URI. The client owns the text; the server
    /// keeps a copy because `didChange` sends the new content and
    /// diagnostics are computed from it.
    documents: HashMap<String, String>,
    /// Set by `exit`.
    exited: bool,
    /// Set by `shutdown`. A request after it is an error per the
    /// specification, rather than something to serve anyway.
    shutting_down: bool,
}

impl Server {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
            exited: false,
            shutting_down: false,
        }
    }

    /// Handle one message, returning whatever should be sent back.
    ///
    /// A notification produces no response but may produce
    /// *notifications of its own* — `didOpen` answers with diagnostics
    /// — so this returns a list rather than an `Option`.
    fn handle(&mut self, request: &Json) -> Vec<Json> {
        let method = request.get("method").and_then(Json::as_str).unwrap_or("");
        let id = request.get("id").cloned().unwrap_or(Json::Null);
        let params = request.get("params");

        match method {
            "initialize" => vec![ok_response(&id, initialize_result())],
            "initialized" => Vec::new(),
            "shutdown" => {
                self.shutting_down = true;
                vec![ok_response(&id, Json::Null)]
            }
            "exit" => {
                self.exited = true;
                Vec::new()
            }
            "textDocument/didOpen" => {
                let Some((uri, text)) = document_from_open(params) else {
                    return Vec::new();
                };
                let _ = self.documents.insert(uri.clone(), text.clone());
                vec![diagnostics_notification(&uri, &text)]
            }
            "textDocument/didChange" => {
                let Some((uri, text)) = document_from_change(params) else {
                    return Vec::new();
                };
                let _ = self.documents.insert(uri.clone(), text.clone());
                vec![diagnostics_notification(&uri, &text)]
            }
            "textDocument/didClose" => {
                if let Some(uri) = uri_of(params) {
                    let _ = self.documents.remove(&uri);
                    // Clear the client's diagnostics for a document it
                    // no longer has open; otherwise stale squiggles
                    // outlive the file.
                    return vec![publish(&uri, Vec::new())];
                }
                Vec::new()
            }
            // An unknown *notification* draws nothing; an unknown
            // *request* has an id and must be answered, or the client
            // waits for a reply that never comes.
            _ if matches!(id, Json::Null) => Vec::new(),
            _ => vec![error_response(&id, -32601, "method not found")],
        }
    }
}

/// The capabilities this server actually has.
fn initialize_result() -> Json {
    Json::object(vec![(
        "capabilities",
        Json::object(vec![
            // 1 is full sync: the client sends the whole document on
            // every change. Incremental sync would need the server to
            // apply ranges, and applying them wrongly produces
            // diagnostics for text the user never wrote.
            ("textDocumentSync", Json::Number(1.0)),
        ]),
    )])
}

/// The `publishDiagnostics` notification for a document.
fn diagnostics_notification(uri: &str, text: &str) -> Json {
    let diagnostics = analyse(text)
        .into_iter()
        .map(|d| {
            Json::object(vec![
                (
                    "range",
                    Json::object(vec![
                        ("start", position(d.start.line, d.start.character)),
                        ("end", position(d.end.line, d.end.character)),
                    ]),
                ),
                (
                    "severity",
                    Json::Number(f64::from(severity_code(d.severity))),
                ),
                ("code", Json::str(d.code)),
                ("source", Json::str("oxml")),
                ("message", Json::str(d.message)),
            ])
        })
        .collect();
    publish(uri, diagnostics)
}

fn publish(uri: &str, diagnostics: Vec<Json>) -> Json {
    Json::object(vec![
        ("jsonrpc", Json::str("2.0")),
        ("method", Json::str("textDocument/publishDiagnostics")),
        (
            "params",
            Json::object(vec![
                ("uri", Json::str(uri)),
                ("diagnostics", Json::Array(diagnostics)),
            ]),
        ),
    ])
}

fn position(line: usize, character: usize) -> Json {
    // The cast is lossy above 2^53 lines, which is not a document.
    #[allow(clippy::cast_precision_loss)]
    Json::object(vec![
        ("line", Json::Number(line as f64)),
        ("character", Json::Number(character as f64)),
    ])
}

/// LSP severities are 1-4, most severe first.
const fn severity_code(s: Severity) -> u8 {
    match s {
        Severity::Error => 1,
        Severity::Warning => 2,
        Severity::Information => 3,
        Severity::Hint => 4,
    }
}

fn uri_of(params: Option<&Json>) -> Option<String> {
    Some(
        params?
            .get("textDocument")?
            .get("uri")?
            .as_str()?
            .to_owned(),
    )
}

fn document_from_open(params: Option<&Json>) -> Option<(String, String)> {
    let doc = params?.get("textDocument")?;
    Some((
        doc.get("uri")?.as_str()?.to_owned(),
        doc.get("text")?.as_str()?.to_owned(),
    ))
}

/// Full sync sends one change containing the whole document.
fn document_from_change(params: Option<&Json>) -> Option<(String, String)> {
    let uri = uri_of(params)?;
    let Json::Array(changes) = params?.get("contentChanges")? else {
        return None;
    };
    let text = changes.last()?.get("text")?.as_str()?.to_owned();
    Some((uri, text))
}

fn ok_response(id: &Json, result: Json) -> Json {
    Json::object(vec![
        ("jsonrpc", Json::str("2.0")),
        ("id", id.clone()),
        ("result", result),
    ])
}

fn error_response(id: &Json, code: i32, message: &str) -> Json {
    Json::object(vec![
        ("jsonrpc", Json::str("2.0")),
        ("id", id.clone()),
        (
            "error",
            Json::object(vec![
                ("code", Json::Number(f64::from(code))),
                ("message", Json::str(message)),
            ]),
        ),
    ])
}

/// Read one framed message, or `None` at end of input.
///
/// Headers are case-insensitive and terminated by a blank line. Only
/// `Content-Length` matters; `Content-Type` is accepted and ignored,
/// and anything else is skipped rather than treated as an error — a
/// client sending a header this does not know about is not a client
/// worth disconnecting.
fn read_message<R: BufRead>(reader: &mut R) -> Option<String> {
    let mut length: Option<usize> = None;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            // End of input and a broken pipe are the same thing here:
            // the client is gone and there is no message to read.
            Ok(0) | Err(_) => return None,
            Ok(_) => {}
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().ok();
            }
        }
    }

    // A message with no Content-Length cannot be read: there is no
    // other delimiter in the protocol.
    let length = length?;
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body).ok()?;
    String::from_utf8(body).ok()
}

/// Write one framed message.
fn write_message<W: Write>(out: &mut W, message: &Json) -> std::io::Result<()> {
    let body = message.to_json();
    // `len()` on a `String` is bytes, which is what the header must
    // carry. Counting characters here would desynchronise the stream
    // on the first message containing a character outside ASCII.
    write!(out, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    out.flush()
}
