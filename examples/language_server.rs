// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-lsp. All rights reserved.

//! Driving the language server without an editor.
//!
//! Run with:
//!
//! ```text
//! cargo run --example language_server
//! ```
//!
//! An editor starts `oxml-lsp --stdio` and speaks to it over the
//! process's own streams. `serve` is generic over its two ends, so the
//! same code can be driven from a buffer — which is what makes it
//! testable, and what this example shows.

use std::io::Cursor;

/// Wrap a message in the header the protocol requires.
///
/// `Content-Length` is a count of **bytes**. A document containing an
/// emoji makes bytes and characters differ, and a client that counts
/// characters desynchronises the stream on its first such message.
fn framed(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

fn main() {
    let conversation: String = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///example.xml","text":"<a><b></a>"}}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#,
        r#"{"jsonrpc":"2.0","method":"exit"}"#,
    ]
    .iter()
    .map(|m| framed(m))
    .collect();

    let mut replies = Vec::new();
    oxml_lsp::lsp::serve(Cursor::new(conversation.into_bytes()), &mut replies);

    let text = String::from_utf8(replies).expect("replies are utf-8");
    println!("{}", text.replace('\r', ""));

    // The middle reply is a `publishDiagnostics` notification: the
    // document has a mismatched tag, and the server pushed that to the
    // client without being asked. That push is the whole point of a
    // language server over a linter you run by hand.
    assert!(
        text.contains("publishDiagnostics"),
        "the server should have pushed diagnostics"
    );
}
