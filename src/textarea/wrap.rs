#![cfg(feature = "wrap")]

use crate::textarea::TextArea;
use crate::ratatui::text::{Line, Span};

/// Text wrapping functionality for TextArea
impl<'a> TextArea<'a> {
    // Render content wrapped with `textwrap` 
    fn render_wrapped_lines(
        &'a self,
        top_row: usize,
        height: usize,
        area_width: u16,
        lnum_len: u8,
        show_line_numbers: bool,
        line_number_style: Option<Style>,
    ) -> Vec<Line<'a>> {
            const LNUM_PADDING: usize = 2;
            let mut wrap_width = area_width as usize;

            if show_line_numbers {
                wrap_width = wrap_width.saturating_sub(lnum_len as usize + LNUM_PADDING);
            }

            if let Some(custom_width) = self.wrap_width() {
                wrap_width = custom_width;
            }

            wrap_width = wrap_width.max(1);
            
            // Create Options and set preserve_trailing_space
            let options = Options::new(wrap_width).preserve_trailing_space(true);

            let mut lines = Vec::new();
            let mut display_row = 0;

            for (logical_row, line_text) in self.lines().iter().enumerate() {
                let wrapped_lines = textwrap::wrap(line_text, &options);

                if display_row + wrapped_lines.len() <= top_row {
                    // Skip this line entirely
                    display_row += wrapped_lines.len();
                    continue;
                }

                for (wrap_index, wrapped_line) in wrapped_lines.iter().enumerate() {
                    if display_row >= top_row + height {
                        break;
                    }

                    if display_row >= top_row {
                        let mut spans = Vec::new();

                        if show_line_numbers {
                            if wrap_index == 0 {
                                let style = line_number_style.expect("checked already");
                                let lnum = format!(" {:>width$} ", logical_row + 1, width = lnum_len as usize);
                                spans.push(Span::styled(lnum, style));
                            } else {
                                let padding = " ".repeat(lnum_len as usize + LNUM_PADDING);
                                spans.push(Span::raw(padding));
                            }
                        }

                        // Get fully highlighted line (handles selection, cursor, search, etc.)
                        let full_highlighted_line = self.line_spans(line_text, logical_row, 0);
                        
                        // Calculate character range for this wrapped segment
                        let (segment_start_char, segment_end_char) = 
                            TextArea::calculate_segment_char_range(line_text, &wrapped_lines, wrap_index);
                        
                        // Extract spans for this segment from the highlighted line
                        let segment_spans = extract_segment_spans(
                            full_highlighted_line,
                            segment_start_char,
                            segment_end_char,
                        );
                        
                        // Add extracted spans to our line, or fallback if empty
                        if segment_spans.is_empty() {
                            spans.push(Span::styled(wrapped_line.to_string(), self.style()));
                        } else {
                            spans.extend(segment_spans);
                        }
                        lines.push(Line::from(spans));
                    }

                    display_row += 1;
                }

                if display_row >= top_row + height {
                    break;
                }

            lines
        }
    }

    /// Enable or disable text wrapping.
    /// When enabled, long lines will be wrapped to fit within the text area width.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// textarea.set_wrap(true);
    /// assert!(textarea.wrap_enabled());
    /// ```
    pub fn set_wrap(&mut self, enabled: bool) {
        self.wrap_enabled = enabled;
    }

    /// Check if text wrapping is enabled.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// assert!(!textarea.wrap_enabled());
    /// textarea.set_wrap(true);
    /// assert!(textarea.wrap_enabled());
    /// ```
    pub fn wrap_enabled(&self) -> bool {
        self.wrap_enabled
    }

    /// Set the wrap width. If `None`, wrapping will use the available text area width.
    /// If `Some(width)`, lines will be wrapped at the specified character width.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// textarea.set_wrap(true);
    /// textarea.set_wrap_width(Some(80));
    /// assert_eq!(textarea.wrap_width(), Some(80));
    /// ```
    pub fn set_wrap_width(&mut self, width: Option<usize>) {
        self.wrap_width = width;
    }

    /// Get the current wrap width setting.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// assert_eq!(textarea.wrap_width(), None);
    /// textarea.set_wrap_width(Some(80));
    /// assert_eq!(textarea.wrap_width(), Some(80));
    /// ```
    pub fn wrap_width(&self) -> Option<usize> {
        self.wrap_width
    }

    /// Calculate the effective wrap width considering line numbers and custom width settings
    pub fn calculate_effective_wrap_width(&self, area_width: u16) -> usize {
        let mut wrap_width = area_width as usize;
        
        // Subtract line number width if enabled
        if self.line_number_style().is_some() {
            let lnum_len = crate::util::num_digits(self.lines().len());
            wrap_width = wrap_width.saturating_sub((lnum_len + 2) as usize);
        }
        
        // Use custom wrap width if set
        if let Some(custom_width) = self.wrap_width() {
            wrap_width = custom_width;
        }
        
        wrap_width.max(1) // Ensure minimum width of 1
    }
}