//! Position utilities for converting byte offsets to line/column positions.
//!
//! This module provides utilities for converting between byte offsets (used internally
//! in Bobbin's `Span` type) and line/column positions (used by LSP and Godot editor).

/// Line/column position (0-indexed).
///
/// Both line and column are 0-indexed for LSP compatibility.
/// When using with Godot editor, add 1 to both values (Godot uses 1-indexed positions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// 0-indexed line number.
    pub line: u32,
    /// 0-indexed column (in bytes for UTF-8, or UTF-16 code units for LSP fallback).
    pub column: u32,
}

/// Index for efficient offset-to-position conversion.
///
/// Pre-computes line start offsets for O(log n) position lookups.
/// Supports both UTF-8 byte columns and UTF-16 code unit columns (for LSP fallback).
#[derive(Debug)]
pub struct LineIndex {
    /// Byte offsets of each line start (0 is always the first entry).
    line_starts: Vec<usize>,
    /// Original source text (kept for UTF-16 column conversion).
    source: String,
}

impl LineIndex {
    /// Create a new line index for the given source.
    ///
    /// # Example
    ///
    /// ```
    /// use bobbin_syntax::LineIndex;
    ///
    /// let source = "line1\nline2\nline3";
    /// let index = LineIndex::new(source);
    ///
    /// let pos = index.line_col(6); // 'l' of "line2"
    /// assert_eq!(pos.line, 1);
    /// assert_eq!(pos.column, 0);
    /// ```
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];

        for (offset, c) in source.char_indices() {
            if c == '\n' {
                // Line start is the byte after the newline
                line_starts.push(offset + 1);
            }
        }

        Self {
            line_starts,
            source: source.to_string(),
        }
    }

    /// Convert a byte offset to line/column (0-indexed).
    ///
    /// The column is in bytes (UTF-8 code units).
    ///
    /// # Example
    ///
    /// ```
    /// use bobbin_syntax::LineIndex;
    ///
    /// let source = "hello\nworld";
    /// let index = LineIndex::new(source);
    ///
    /// // First line
    /// assert_eq!(index.line_col(0).line, 0);
    /// assert_eq!(index.line_col(0).column, 0);
    ///
    /// // Second line starts at byte 6
    /// assert_eq!(index.line_col(6).line, 1);
    /// assert_eq!(index.line_col(6).column, 0);
    /// ```
    pub fn line_col(&self, offset: usize) -> SourcePosition {
        // Binary search for the line containing this offset
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert_pos) => insert_pos.saturating_sub(1),
        };

        let line_start = self.line_starts[line];
        let column = offset.saturating_sub(line_start);

        SourcePosition {
            line: line as u32,
            column: column as u32,
        }
    }

    /// Convert a byte offset to UTF-16 column offset.
    ///
    /// This is needed for LSP clients that don't support UTF-8 position encoding.
    /// UTF-16 column counts code units (surrogate pairs count as 2).
    ///
    /// # Example
    ///
    /// ```
    /// use bobbin_syntax::LineIndex;
    ///
    /// let source = "a𐐀b"; // 𐐀 is 4 bytes in UTF-8, 2 code units in UTF-16
    /// let index = LineIndex::new(source);
    ///
    /// // 'b' is at byte 5 (a=1, 𐐀=4)
    /// // In UTF-16: 'b' is at column 3 (a=1, 𐐀=2, b starts at 3)
    /// assert_eq!(index.utf16_col(5), 3);
    /// ```
    pub fn utf16_col(&self, offset: usize) -> u32 {
        let pos = self.line_col(offset);
        let line_start = self.line_starts[pos.line as usize];
        let line_end = offset;

        // Get the slice from line start to offset
        let line_prefix = &self.source[line_start..line_end];

        // Count UTF-16 code units
        line_prefix
            .chars()
            .map(|c| if c.len_utf16() > 1 { 2 } else { 1 })
            .sum::<u32>()
    }

    /// Convert a byte offset to an LSP-compatible position.
    ///
    /// Returns a tuple of (line, column) where:
    /// - If `use_utf16` is false, column is in bytes (UTF-8)
    /// - If `use_utf16` is true, column is in UTF-16 code units
    pub fn to_lsp_position(&self, offset: usize, use_utf16: bool) -> SourcePosition {
        let pos = self.line_col(offset);

        if use_utf16 {
            SourcePosition {
                line: pos.line,
                column: self.utf16_col(offset),
            }
        } else {
            pos
        }
    }

    /// Get the number of lines in the source.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let index = LineIndex::new("");
        assert_eq!(index.line_count(), 1);
        let pos = index.line_col(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_single_line() {
        let source = "hello";
        let index = LineIndex::new(source);

        assert_eq!(index.line_count(), 1);

        let pos = index.line_col(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        let pos = index.line_col(4);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 4);
    }

    #[test]
    fn test_multiple_lines() {
        let source = "line1\nline2\nline3";
        let index = LineIndex::new(source);

        assert_eq!(index.line_count(), 3);

        // 'l' of line1
        let pos = index.line_col(0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        // '1' of line1
        let pos = index.line_col(4);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 4);

        // 'l' of line2 (byte 6 - after "line1\n")
        let pos = index.line_col(6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);

        // 'l' of line3 (byte 12 - after "line1\nline2\n")
        let pos = index.line_col(12);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_unicode_utf8() {
        // "café" - é is 2 bytes in UTF-8
        let source = "café";
        let index = LineIndex::new(source);

        // 'c' at byte 0
        assert_eq!(index.line_col(0).column, 0);

        // 'a' at byte 1
        assert_eq!(index.line_col(1).column, 1);

        // 'f' at byte 2
        assert_eq!(index.line_col(2).column, 2);

        // 'é' at byte 3-4
        assert_eq!(index.line_col(3).column, 3);

        // After 'é' (past end)
        assert_eq!(index.line_col(5).column, 5);
    }

    #[test]
    fn test_unicode_utf16() {
        // 𐐀 is a surrogate pair in UTF-16 (4 bytes in UTF-8)
        let source = "a𐐀b";
        let index = LineIndex::new(source);

        // 'a' at byte 0, UTF-16 column 0
        assert_eq!(index.utf16_col(0), 0);

        // 𐐀 at bytes 1-4, UTF-16 column 1 (start of the character)
        assert_eq!(index.utf16_col(1), 1);

        // 'b' at byte 5, UTF-16 column 3 (a=1, 𐐀=2, so b starts at 3)
        assert_eq!(index.utf16_col(5), 3);
    }

    #[test]
    fn test_windows_line_endings() {
        // Windows uses \r\n - we only track \n
        let source = "line1\r\nline2";
        let index = LineIndex::new(source);

        // 'l' of line2 is at byte 7 (after "line1\r\n")
        let pos = index.line_col(7);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_to_lsp_position() {
        let source = "a𐐀b";
        let index = LineIndex::new(source);

        // UTF-8 mode: byte offsets
        let pos = index.to_lsp_position(5, false);
        assert_eq!(pos.column, 5);

        // UTF-16 mode: code unit offsets
        let pos = index.to_lsp_position(5, true);
        assert_eq!(pos.column, 3);
    }
}
