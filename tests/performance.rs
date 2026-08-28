// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-lsp. All rights reserved.

//! `analyse` must stay fast enough for an editor.
//!
//! This file exists because a benchmark, added at the same time,
//! found `analyse` was quadratic in document size. Nothing else here
//! would have noticed: every correctness test used a small document,
//! and they all passed at 1,088 ms for 480 KB just as they do at 9 ms.

/// `analyse` must stay linear in document size.
///
/// It was quadratic: the duplicate-id check searched a `Vec` linearly
/// and, worse, scanned the whole source to find each id *in the branch
/// where the id was not a duplicate*. A document with five thousand
/// distinct ids performed five thousand full-source scans and reported
/// nothing. A benchmark measured 1.8 ms for 200 entries and 1,088 ms
/// for 5,000 — twenty-six times the input for six hundred times the
/// work. An editor calls this on every keystroke.
///
/// A timing assertion is a blunt instrument and this one is
/// deliberately loose: the fix took the large case to about 10 ms, and
/// the bound is 40 times that. It is not measuring performance, it is
/// catching a return to quadratic, which showed up as a hundredfold
/// difference rather than a subtle one.
#[test]
fn analyse_stays_linear_in_document_size() {
    use std::fmt::Write as _;
    use std::time::Instant;

    let mut doc = String::from("<config>\n");
    for i in 0..5_000 {
        let _ = write!(doc, "  <entry id=\"e{i}\"><n>{i}</n></entry>\n");
    }
    doc.push_str("</config>\n");

    // Fastest of three: contention can only make a run slower, and a
    // single sample on a shared runner is not evidence of anything.
    let mut best = f64::INFINITY;
    for _ in 0..3 {
        let start = Instant::now();
        let found = oxml_lsp::analyse(&doc);
        best = best.min(start.elapsed().as_secs_f64());
        assert!(
            found
                .iter()
                .all(|d| d.severity != oxml_lsp::Severity::Error),
            "the document is well-formed"
        );
    }

    assert!(
        best < 0.4,
        "analysing {} KB took {:.0} ms; it was quadratic once and this \
         is what that looked like",
        doc.len() / 1024,
        best * 1e3
    );
}
