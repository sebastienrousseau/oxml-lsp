// SPDX-License-Identifier: MIT OR Apache-2.0
//! Convert transport ranges without changing the native scalar-based API.

use crate::{Diagnostic, Position};
use std::iter::Peekable;
use std::str::Chars;

pub(super) fn utf16(text: &str, diagnostics: &mut [Diagnostic]) {
    let mut endpoints: Vec<_> = diagnostics
        .iter()
        .enumerate()
        .flat_map(|(index, diagnostic)| {
            [
                (
                    diagnostic.start.line,
                    diagnostic.start.character,
                    index,
                    false,
                ),
                (diagnostic.end.line, diagnostic.end.character, index, true),
            ]
        })
        .collect();
    endpoints.sort_unstable();
    let mut cursor = Cursor {
        chars: text.chars().peekable(),
        line: 0,
        scalar: 0,
        units: 0,
    };
    // Scan each source character at most once. Sorting two endpoints per
    // diagnostic costs O(d log d) time and O(d) space, not O(d * input).
    // Assignment by original index preserves diagnostic order.
    for (line, character, index, end) in endpoints {
        let converted = cursor.convert(Position { line, character });
        if end {
            diagnostics[index].end = converted;
        } else {
            diagnostics[index].start = converted;
        }
    }
}

struct Cursor<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    scalar: usize,
    units: usize,
}

impl Cursor<'_> {
    fn next_line(&mut self) -> bool {
        for ch in self.chars.by_ref() {
            if ch == '\n' {
                self.line += 1;
                self.scalar = 0;
                self.units = 0;
                return true;
            }
            self.scalar += 1;
            self.units += ch.len_utf16();
        }
        false
    }

    fn convert(&mut self, target: Position) -> Position {
        while self.line < target.line {
            if !self.next_line() {
                return Position {
                    line: self.line,
                    character: self.units,
                };
            }
        }
        while self.scalar < target.character {
            let Some(&ch) = self.chars.peek() else { break };
            if ch == '\n' {
                break;
            }
            let _ = self.chars.next();
            self.scalar += 1;
            self.units += ch.len_utf16();
        }
        Position {
            line: self.line,
            character: self.units,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;
    use crate::Position;

    #[test]
    fn out_of_range_lines_stop_at_the_utf16_end_of_input() {
        let mut cursor = Cursor {
            chars: "😀\nX😀".chars().peekable(),
            line: 0,
            scalar: 0,
            units: 0,
        };
        assert_eq!(
            cursor.convert(Position {
                line: 99,
                character: 0
            }),
            Position {
                line: 1,
                character: 3
            },
        );
    }
}
