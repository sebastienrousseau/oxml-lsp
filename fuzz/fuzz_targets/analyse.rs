#![no_main]
//! Arbitrary text must never panic `analyse`.
//!
//! An editor calls this on every keystroke, which means it is called
//! on text that is *not yet valid* far more often than on text that
//! is: half-typed tags, unbalanced quotes, a document mid-paste. A
//! panic is a crashed editor plugin.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    // The last valid zero-based line index, in the *LSP* line model:
    // a trailing newline opens a further, empty line.
    //
    // `str::lines()` disagrees -- it reports "abc\n" as one line --
    // and using it here produced a false positive within a minute of
    // fuzzing, on `<a id="="` followed by a newline. The diagnostic
    // pointing at line 1 was right; the assertion was wrong.
    let last_line = text.matches('\n').count();

    for d in oxml_lsp::analyse(text) {
        // Every diagnostic must point *into* the document it came
        // from. A position past the end crashes an editor that trusts
        // it, and is exactly the kind of off-by-one a fuzzer finds and
        // a test suite does not.
        assert!(
            d.start.line <= last_line,
            "diagnostic starts on line {} of a document whose last line is {}",
            d.start.line,
            last_line
        );
        assert!(
            d.end.line <= last_line,
            "diagnostic ends on line {} of a document whose last line is {}",
            d.end.line,
            last_line
        );
        // An end before its start is a range no editor can render.
        assert!(
            (d.start.line, d.start.character) <= (d.end.line, d.end.character),
            "diagnostic ends before it starts"
        );
    }
});
