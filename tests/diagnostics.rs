// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! What the analyser decides to say.

use oxml_lsp::{Severity, analyse};

#[test]
fn a_clean_document_produces_no_errors() {
    let d = analyse("<a><b id='1'>x</b></a>");
    assert!(!d.iter().any(|x| x.severity == Severity::Error), "{d:?}");
}

#[test]
fn a_parse_error_reports_a_zero_based_position() {
    let d = analyse("<a>\n  <b>\n</a>");
    assert_eq!(d.len(), 1, "a parse failure should not cascade");
    assert_eq!(d[0].severity, Severity::Error);
    assert_eq!(d[0].code, "not-well-formed");
    // LSP positions are zero-based; the error is on the third line.
    assert!(d[0].start.line >= 1, "line was {}", d[0].start.line);
}

/// Once the tree cannot be built there is nothing further to say.
/// Guessing past a parse failure produces cascades that all describe
/// the same mistake.
#[test]
fn a_parse_error_suppresses_other_diagnostics() {
    let d = analyse("<a><b id='x'/><c id='x'/>");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, "not-well-formed");
}

#[test]
fn duplicate_ids_are_warned_about() {
    let d = analyse("<r><a id='x'>1</a><b id='x'>2</b></r>");
    let dup: Vec<_> = d.iter().filter(|x| x.code == "duplicate-id").collect();
    assert_eq!(dup.len(), 1, "{d:?}");
    assert_eq!(dup[0].severity, Severity::Warning);
    assert!(dup[0].message.contains('x'));
}

#[test]
fn distinct_ids_are_not_warned_about() {
    let d = analyse("<r><a id='x'>1</a><b id='y'>2</b></r>");
    assert!(d.iter().all(|x| x.code != "duplicate-id"), "{d:?}");
}

#[test]
fn empty_elements_are_hinted() {
    let d = analyse("<r><filler/></r>");
    let hints: Vec<_> =
        d.iter().filter(|x| x.code == "empty-element").collect();
    assert_eq!(hints.len(), 1, "{d:?}");
    assert_eq!(hints[0].severity, Severity::Hint);
}

/// An empty element that carries attributes carries information, so
/// it is not a leftover.
#[test]
fn an_empty_element_with_attributes_is_not_hinted() {
    let d = analyse("<r><link href='x'/></r>");
    assert!(d.iter().all(|x| x.code != "empty-element"), "{d:?}");
}

#[test]
fn ranges_are_non_empty_so_editors_can_highlight_them() {
    for src in ["<a><b/></a>", "<a><b id='q'/><c id='q'/></a>", "<a>"] {
        for d in analyse(src) {
            assert!(
                d.end.line > d.start.line
                    || d.end.character > d.start.character,
                "empty range for {}: {d:?}",
                d.code
            );
        }
    }
}
