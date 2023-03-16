//! HTML escaping.
//!
//! This module contains one unit struct, which can be used to HTML-escape a
//! string of text (for use in a format string).

use std::fmt;

/// Wrapper struct which will emit the HTML-escaped version of the contained
/// string when passed to a format string.
///
/// This is suitable for escaping text content, not attribute values.
pub(crate) struct Escape<'a>(pub &'a str);

impl<'a> fmt::Display for Escape<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Because the internet is always right, turns out there's not that many
        // characters to escape: http://stackoverflow.com/questions/7381974
        // We choose not to escape `"` and `'` here because they are only relevant
        // inside attribute values, and this is only used for escaping text content.
        let Escape(s) = *self;
        let pile_o_bits = s;
        let mut last = 0;
        for (i, ch) in s.char_indices() {
            let s = match ch {
                '>' => "&gt;",
                '<' => "&lt;",
                '&' => "&amp;",
                _ => continue,
            };
            fmt.write_str(&pile_o_bits[last..i])?;
            fmt.write_str(s)?;
            // NOTE: we only expect single byte characters here - which is fine as long as we
            // only match single byte characters
            last = i + 1;
        }

        if last < s.len() {
            fmt.write_str(&pile_o_bits[last..])?;
        }
        Ok(())
    }
}
