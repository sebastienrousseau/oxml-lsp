// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 oxml-lsp. All rights reserved.

//! What `analyse` costs, by document shape.
//!
//! An editor calls this on every keystroke, so the figure that matters
//! is not throughput on a large file but latency on a small one: a
//! language server that takes 20 ms to lint a 4 KB buffer feels slow
//! in a way that a parser doing the same work in a batch job does not.
//!
//! Reported per document rather than in MB/s for that reason, and the
//! shapes are chosen for what an editor actually holds open: a small
//! well-formed file, the same file mid-edit with an unclosed tag, and
//! one large enough to be uncomfortable.
//!
//! Absolute figures here describe the machine as much as the code —
//! see `oxml`'s `doc/BENCHMARKS.md`. Compare runs, not numbers.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

/// A document of `n` elements, well-formed.
fn document(n: usize) -> String {
    let mut s = String::from("<?xml version=\"1.0\"?>\n<config>\n");
    for i in 0..n {
        let _ = write!(
            s,
            "  <entry id=\"e{i}\" enabled=\"true\">\n    \
             <name>Entry {i}</name>\n    \
             <value>{i}</value>\n  </entry>\n"
        );
    }
    s.push_str("</config>\n");
    s
}

/// The fastest of `rounds` runs.
///
/// Contention can only make a run slower, so the fastest is the least
/// perturbed sample. A mean would mostly measure whatever else the
/// machine was doing.
fn fastest(rounds: usize, mut f: impl FnMut()) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..rounds {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed().as_secs_f64());
    }
    best
}

fn main() {
    let cases: Vec<(String, String)> = vec![
        ("small (10 entries)".to_owned(), document(10)),
        ("editor buffer (200 entries)".to_owned(), document(200)),
        ("large (5,000 entries)".to_owned(), document(5_000)),
        (
            "mid-edit, unclosed tag".to_owned(),
            document(200).replace("</config>", ""),
        ),
        ("not well-formed".to_owned(), "<a><b></a>".to_owned()),
    ];

    println!("analyse, fastest of 50 runs\n");
    for (name, doc) in &cases {
        let secs = fastest(50, || {
            let _ = black_box(oxml_lsp::analyse(black_box(doc)));
        });
        let diagnostics = oxml_lsp::analyse(doc).len();
        println!(
            "  {name:<30} {:>8.3} ms  ({:>3} diagnostics, {} KB)",
            secs * 1e3,
            diagnostics,
            doc.len() / 1024
        );
    }
}
