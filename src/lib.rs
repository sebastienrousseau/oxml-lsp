// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml. All rights reserved.

//! Diagnostics for XML documents, in the shape an editor wants.
//!
//! The analysis lives here rather than in the binary so it can be
//! tested without a language-server transport in the way. An editor
//! integration is mostly plumbing; the part worth testing is what it
//! decides to say.

#![forbid(unsafe_code)]

/// How serious a diagnostic is, matching LSP's numbering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The document is broken.
    Error = 1,
    /// Legal, but probably not intended.
    Warning = 2,
    /// Informational.
    Information = 3,
    /// A gentle suggestion.
    Hint = 4,
}

/// A position in a document, zero-based as LSP requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Zero-based line.
    pub line: usize,
    /// Zero-based character offset within the line.
    pub character: usize,
}

/// One diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Where it starts.
    pub start: Position,
    /// Where it ends.
    pub end: Position,
    /// How serious.
    pub severity: Severity,
    /// What to tell the user.
    pub message: String,
    /// A stable identifier, so an editor can group or suppress.
    pub code: &'static str,
}

/// Analyse a document and return everything worth reporting.
///
/// A well-formedness error stops the analysis: once the tree cannot be
/// built there is nothing further to say, and guessing at structure
/// past a parse failure produces cascades of diagnostics that all
/// describe the same underlying mistake.
#[must_use]
pub fn analyse(source: &str) -> Vec<Diagnostic> {
    let doc = match oxml::parse(source) {
        Ok(doc) => doc,
        Err(e) => {
            let (line, column) = e.line_column(source);
            // `line_column` is 1-based for humans; LSP is 0-based.
            let start = Position {
                line: line.saturating_sub(1),
                character: column.saturating_sub(1),
            };
            return vec![Diagnostic {
                start,
                end: Position {
                    line: start.line,
                    character: start.character + 1,
                },
                severity: Severity::Error,
                message: e.to_string(),
                code: "not-well-formed",
            }];
        }
    };

    let mut out = Vec::new();
    lint_document(&doc, source, &mut out);
    out
}

fn lint_document(
    doc: &oxml::Document,
    source: &str,
    out: &mut Vec<Diagnostic>,
) {
    // Duplicated ids are legal XML but almost always a mistake, and
    // they break every tool that resolves references by id.
    let mut seen_ids: Vec<(String, usize)> = Vec::new();

    for id in doc.descendants() {
        if !doc.is_element(id) {
            continue;
        }
        if let Some(value) = doc.attribute(id, "id") {
            if let Some((_, first)) = seen_ids.iter().find(|(v, _)| v == value)
            {
                let at = locate(source, value);
                out.push(Diagnostic {
                    start: at,
                    end: Position {
                        line: at.line,
                        character: at.character + value.chars().count(),
                    },
                    severity: Severity::Warning,
                    message: format!(
                        "duplicate id `{value}`; first used on line {}",
                        first + 1
                    ),
                    code: "duplicate-id",
                });
            } else {
                seen_ids.push((value.to_owned(), at_line(source, value)));
            }
        }
    }

    // An empty element with no attributes carries no information, and
    // is usually a leftover.
    for id in doc.descendants() {
        let Some(name) = doc.element_name(id) else {
            continue;
        };
        if doc.children(id).is_empty()
            && doc.attribute_nodes(id).is_empty()
            && doc.parent(id) != Some(doc.root())
        {
            let at = locate(source, &format!("<{}", name.local));
            out.push(Diagnostic {
                start: at,
                end: Position {
                    line: at.line,
                    character: at.character + name.local.chars().count() + 1,
                },
                severity: Severity::Hint,
                message: format!(
                    "`{}` is empty and has no attributes",
                    name.local
                ),
                code: "empty-element",
            });
        }
    }
}

/// Find where a fragment first appears.
///
/// The tree does not carry source spans, so positions are recovered by
/// search. That is honest about its limits: for a repeated value it
/// finds the first occurrence, which is why the duplicate-id message
/// names the line rather than relying on the range alone.
fn locate(source: &str, needle: &str) -> Position {
    source.find(needle).map_or(
        Position {
            line: 0,
            character: 0,
        },
        |offset| {
            let before = &source[..offset];
            Position {
                line: before.matches('\n').count(),
                character: before
                    .rsplit('\n')
                    .next()
                    .map_or(0, |l| l.chars().count()),
            }
        },
    )
}

fn at_line(source: &str, needle: &str) -> usize {
    locate(source, needle).line
}
