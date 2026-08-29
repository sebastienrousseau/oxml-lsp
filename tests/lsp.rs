// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-lsp. All rights reserved.

//! The language server, driven over an in-memory pipe.
//!
//! `serve` is generic over its two ends precisely so these can exist.
//! Spawning a process instead would measure process start and pipe
//! setup alongside the protocol, and could not assert on what was
//! written without a second pipe to read it back.

use std::io::Cursor;

/// Frame a message the way a client would.
fn framed(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

/// Run a conversation and split the replies back apart.
fn converse(messages: &[&str]) -> Vec<String> {
    let input: String = messages.iter().map(|m| framed(m)).collect();
    let mut out = Vec::new();
    oxml_lsp::lsp::serve(Cursor::new(input.into_bytes()), &mut out);
    let text = String::from_utf8(out).expect("replies are utf-8");

    // Split on the header, keeping the bodies.
    let mut bodies = Vec::new();
    let mut rest = text.as_str();
    while let Some(start) = rest.find("Content-Length: ") {
        let after = &rest[start + "Content-Length: ".len()..];
        let (len, remainder) =
            after.split_once("\r\n\r\n").expect("a framed message");
        let len: usize = len.trim().parse().expect("a numeric length");
        assert!(
            remainder.len() >= len,
            "header claims {len} bytes but only {} remain",
            remainder.len()
        );
        bodies.push(remainder[..len].to_owned());
        rest = &remainder[len..];
    }
    bodies
}

#[test]
fn initialize_advertises_full_document_sync() {
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    ]);
    assert_eq!(replies.len(), 1, "got {replies:?}");
    assert!(
        replies[0].contains("\"textDocumentSync\":1"),
        "{}",
        replies[0]
    );
    assert!(replies[0].contains("\"id\":1"), "{}", replies[0]);
}

#[test]
fn opening_a_document_publishes_diagnostics() {
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///a.xml","text":"<a><b></a>"}}}"#,
    ]);
    assert_eq!(replies.len(), 2, "got {replies:?}");
    let published = &replies[1];
    assert!(published.contains("publishDiagnostics"), "{published}");
    assert!(published.contains("file:///a.xml"), "{published}");
    // A mismatched tag is an error, severity 1.
    assert!(published.contains("\"severity\":1"), "{published}");
}

#[test]
fn a_clean_document_publishes_an_empty_list() {
    // `<a><b/></a>` would *not* do: the linter reports an empty
    // element as a hint, so a document being well-formed is not the
    // same as it having nothing to say. This one has nothing to say.
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///ok.xml","text":"<a>text</a>"}}}"#,
    ]);
    assert_eq!(replies.len(), 1, "got {replies:?}");
    // The list must be present and empty, not absent: an editor clears
    // its squiggles on an empty list and leaves them on no message.
    assert!(replies[0].contains("\"diagnostics\":[]"), "{}", replies[0]);
}

#[test]
fn changing_a_document_republishes() {
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///a.xml","text":"<a>ok</a>"}}}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///a.xml"},"contentChanges":[{"text":"<a><b></a>"}]}}"#,
    ]);
    assert_eq!(replies.len(), 2, "got {replies:?}");
    assert!(replies[0].contains("\"diagnostics\":[]"), "{}", replies[0]);
    assert!(replies[1].contains("\"severity\":1"), "{}", replies[1]);
}

#[test]
fn closing_a_document_clears_its_diagnostics() {
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///a.xml","text":"<a><b></a>"}}}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///a.xml"}}}"#,
    ]);
    assert_eq!(replies.len(), 2, "got {replies:?}");
    assert!(replies[1].contains("\"diagnostics\":[]"), "{}", replies[1]);
}

#[test]
fn shutdown_is_answered_and_exit_ends_the_session() {
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","id":9,"method":"shutdown"}"#,
        r#"{"jsonrpc":"2.0","method":"exit"}"#,
        r#"{"jsonrpc":"2.0","id":10,"method":"initialize","params":{}}"#,
    ]);
    // The initialize after exit must not be answered: the session is
    // over, and replying would mean the loop kept running.
    assert_eq!(replies.len(), 1, "got {replies:?}");
    assert!(replies[0].contains("\"id\":9"), "{}", replies[0]);
}

#[test]
fn an_unknown_request_is_answered_but_a_notification_is_not() {
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","method":"textDocument/somethingElse","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{}}"#,
    ]);
    // A client waiting on a request it sent must get something back;
    // a notification by definition draws nothing.
    assert_eq!(replies.len(), 1, "got {replies:?}");
    assert!(replies[0].contains("-32601"), "{}", replies[0]);
}

#[test]
fn malformed_json_does_not_end_the_session() {
    let replies = converse(&[
        "not json at all",
        r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{}}"#,
    ]);
    assert_eq!(replies.len(), 2, "got {replies:?}");
    assert!(replies[0].contains("-32700"), "{}", replies[0]);
    assert!(replies[1].contains("\"id\":2"), "{}", replies[1]);
}

#[test]
fn the_content_length_header_counts_bytes_not_characters() {
    // A document containing an emoji makes the two differ. If the
    // server counted characters, this reply would be truncated and
    // every message after it would be misframed — which `converse`
    // asserts against when it splits the stream.
    let replies = converse(&[
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///emoji.xml","text":"<a>😀</a>"}}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"initialize","params":{}}"#,
    ]);
    assert_eq!(replies.len(), 2, "got {replies:?}");
    assert!(
        replies[1].contains("\"id\":4"),
        "stream desynchronised: {replies:?}"
    );
}
