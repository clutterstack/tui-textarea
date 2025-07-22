use crate::ratatui::buffer::Buffer;
use crate::ratatui::layout::Rect;
use crate::ratatui::text::Span;
use crate::ratatui::widgets::Widget;
use crate::ratatui::style::Style;
use crate::textarea::TextArea;
use crate::util::num_digits;
#[cfg(feature = "wrap")]
use crate::highlight::extract_segment_spans;
#[cfg(feature = "ratatui")]
use ratatui::text::Line;
use std::cmp;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "wrap")]
use textwrap::Options;

#[cfg(feature = "tuirs")]
use tui::text::Spans as Line;

// &mut 'a (u16, u16, u16, u16) is not available since `render` method takes immutable reference of TextArea
// instance. In the case, the TextArea instance cannot be accessed from any other objects since it is mutablly
// borrowed.
//
// `ratatui::Frame::render_stateful_widget` would be an assumed way to render a stateful widget. But at this
// point we stick with using `ratatui::Frame::render_widget` because it is simpler API. Users don't need to
// manage states of textarea instances separately.
// https://docs.rs/ratatui/latest/ratatui/terminal/struct.Frame.html#method.render_stateful_widget
#[derive(Default, Debug)]
pub struct Viewport(AtomicU64);

impl Clone for Viewport {
    fn clone(&self) -> Self {
        let u = self.0.load(Ordering::Relaxed);
        Viewport(AtomicU64::new(u))
    }
}

impl Viewport {
    pub fn scroll_top(&self) -> (u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        ((u >> 16) as u16, u as u16)
    }

    pub fn rect(&self) -> (u16, u16, u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        let width = (u >> 48) as u16;
        let height = (u >> 32) as u16;
        let row = (u >> 16) as u16;
        let col = u as u16;
        (row, col, width, height)
    }

    pub fn position(&self) -> (u16, u16, u16, u16) {
        let (row_top, col_top, width, height) = self.rect();
        let row_bottom = row_top.saturating_add(height).saturating_sub(1);
        let col_bottom = col_top.saturating_add(width).saturating_sub(1);

        (
            row_top,
            col_top,
            cmp::max(row_top, row_bottom),
            cmp::max(col_top, col_bottom),
        )
    }

    fn store(&self, row: u16, col: u16, width: u16, height: u16) {
        // Pack four u16 values into one u64 value
        let u =
            ((width as u64) << 48) | ((height as u64) << 32) | ((row as u64) << 16) | col as u64;
        self.0.store(u, Ordering::Relaxed);
    }

    pub fn scroll(&mut self, rows: i16, cols: i16) {
        fn apply_scroll(pos: u16, delta: i16) -> u16 {
            if delta >= 0 {
                pos.saturating_add(delta as u16)
            } else {
                pos.saturating_sub(-delta as u16)
            }
        }

        let u = self.0.get_mut();
        let row = apply_scroll((*u >> 16) as u16, rows);
        let col = apply_scroll(*u as u16, cols);
        *u = (*u & 0xffff_ffff_0000_0000) | ((row as u64) << 16) | (col as u64);
    }
}

#[inline]
fn next_scroll_top(prev_top: u16, cursor: u16, len: u16) -> u16 {
    if cursor < prev_top {
        cursor
    } else if prev_top + len <= cursor {
        cursor + 1 - len
    } else {
        prev_top
    }
}

/// Calculate the visual width of a character, handling tabs and Unicode width
fn char_visual_width(c: char, position: usize, tab_len: u8) -> usize {
    match c {
        '\t' => {
            if tab_len == 0 {
                0
            } else {
                tab_len as usize - (position % tab_len as usize)
            }
        }
        _ => {
            use unicode_width::UnicodeWidthChar;
            c.width().unwrap_or(0)
        }
    }
}

/// Calculate visual position range for horizontal clipping
/// Returns (start_char_idx, end_char_idx, start_visual_offset)
fn calculate_horizontal_range(
    line: &str,
    col_left: u16,
    viewport_width: u16,
    tab_len: u8,
) -> (usize, usize, usize) {
    let col_left = col_left as usize;
    let col_right = col_left + viewport_width as usize;
    
    let mut visual_pos = 0;
    let mut start_char_idx = 0;
    let mut start_visual_offset = 0;
    let mut found_start = false;
    
    // Find the starting character index
    for (char_idx, c) in line.char_indices() {
        let char_width = char_visual_width(c, visual_pos, tab_len);
        
        if !found_start {
            if visual_pos + char_width > col_left {
                start_char_idx = char_idx;
                start_visual_offset = if visual_pos < col_left {
                    col_left - visual_pos
                } else {
                    0
                };
                found_start = true;
            }
        }
        
        visual_pos += char_width;
        
        // Find the ending character index
        if found_start && visual_pos >= col_right {
            // Find the next character boundary
            if let Some((end_idx, _)) = line[char_idx..].char_indices().next() {
                return (start_char_idx, char_idx + end_idx, start_visual_offset);
            }
            return (start_char_idx, line.len(), start_visual_offset);
        }
    }
    
    // If we haven't found the start, the entire line is before the viewport
    if !found_start {
        return (line.len(), line.len(), 0);
    }
    
    // The line ends within the viewport
    (start_char_idx, line.len(), start_visual_offset)
}

impl<'a> TextArea<'a> {
    fn text_lines(&'a self, top_row: usize, height: usize, area_width: u16) -> Vec<Line<'a>> {
        let lines_len = self.lines().len();
        let lnum_len = num_digits(lines_len);
        let line_number_style = self.line_number_style();
        let show_line_numbers = line_number_style.is_some();

        #[cfg(feature = "wrap")]
        let wrap_enabled = self.wrap_enabled();
        #[cfg(not(feature = "wrap"))]
        let wrap_enabled = false;

        if wrap_enabled {
            return self.render_wrapped_lines(top_row, height, area_width, lnum_len, show_line_numbers, line_number_style);
        }

        self.render_unwrapped_lines(top_row, height, area_width, lnum_len, show_line_numbers)
    }

    fn render_wrapped_lines(
        &'a self,
        top_row: usize,
        height: usize,
        area_width: u16,
        lnum_len: u8,
        show_line_numbers: bool,
        line_number_style: Option<Style>,
    ) -> Vec<Line<'a>> {
        #[cfg(not(feature = "wrap"))]
        {
            Vec::new()
        }

        #[cfg(feature = "wrap")]
        {
            use crate::ratatui::text::{Line, Span};

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
            }

            lines
        }
    }

    fn render_unwrapped_lines(
        &'a self,
        top_row: usize,
        height: usize,
        area_width: u16,
        lnum_len: u8,
        show_line_numbers: bool,
    ) -> Vec<Line<'a>> {
        let lines_len = self.lines().len();
        let bottom_row = top_row.saturating_add(height).min(lines_len);
        let mut lines = Vec::new();

        let (_, col_left) = self.viewport.scroll_top();
        let mut viewport_width = area_width;

        if show_line_numbers {
            viewport_width = viewport_width.saturating_sub((lnum_len + 2) as u16);
        }

        for (i, line) in self.lines()[top_row..bottom_row].iter().enumerate() {
            let rendered_line = if col_left > 0 || line.chars().count() > viewport_width as usize {
                let (start, end, _) = calculate_horizontal_range(
                    line,
                    col_left,
                    viewport_width,
                    self.tab_length(),
                );
                if start < line.len() {
                    self.line_spans(&line[start..end], top_row + i, lnum_len)
                } else {
                    self.line_spans("", top_row + i, lnum_len)
                }
            } else {
                self.line_spans(line.as_str(), top_row + i, lnum_len)
            };

            lines.push(rendered_line);
        }

        lines
    }

    fn placeholder_lines(&'a self) -> Vec<Line<'a>> {
        let cursor = Span::styled(" ", self.cursor_style);
        let text = Span::raw(self.placeholder.as_str());
        vec![Line::from(vec![cursor, text])]
    }

    fn scroll_top_row(&self, prev_top: u16, height: u16) -> u16 {
        next_scroll_top(prev_top, self.cursor().0 as u16, height)
    }
    
    fn scroll_left_col(&self, prev_left: u16, width: u16) -> u16 {
        #[cfg(feature = "wrap")]
        let wrap_enabled = self.wrap_enabled();
        #[cfg(not(feature = "wrap"))]
        let wrap_enabled = false;
        
        if wrap_enabled {
            return 0; // No horizontal scrolling when wrap is enabled
        }
        
        // Calculate available width for text content
        let mut text_width = width;
        if self.line_number_style().is_some() {
            let lines_len = self.lines().len();
            let lnum_len = num_digits(lines_len);
            text_width = text_width.saturating_sub((lnum_len + 2) as u16);
        }
        
        // Calculate cursor visual position considering tabs and Unicode width
        let (cursor_row, cursor_col) = self.cursor();
        if cursor_row >= self.lines().len() {
            return prev_left;
        }
        
        let line = &self.lines()[cursor_row];
        let mut visual_pos = 0;
        
        for (char_idx, c) in line.char_indices() {
            if line[..char_idx].chars().count() >= cursor_col {
                break;
            }
            visual_pos += char_visual_width(c, visual_pos, self.tab_length());
        }
        
        next_scroll_top(prev_left, visual_pos as u16, text_width)
    }

    fn render_lines(&self, lines: Vec<Line<'a>>, area: Rect, buf: &mut Buffer) {
        use crate::ratatui::layout::Alignment;
        
        for (i, line) in lines.into_iter().enumerate() {
            let y = area.y + i as u16;
            
            // Bounds check - don't render lines outside the text area
            if y >= area.y + area.height {
                break;
            }
            
            // Create a single-line area for this line
            let line_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };
            
            // Apply alignment manually for each line
            let aligned_line = match self.alignment() {
                Alignment::Left => line,
                Alignment::Center => line.centered(),
                Alignment::Right => line.right_aligned(),
            };
            
            // Render the line widget
            aligned_line.render(line_area, buf);
        }
    }
}

impl Widget for &TextArea<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Rect { width, height, .. } = if let Some(b) = self.block() {
            b.inner(area)
        } else {
            area
        };

        let (top_row, left_col) = self.viewport.scroll_top();
        let top_row = self.scroll_top_row(top_row, height);
        let left_col = self.scroll_left_col(left_col, width);

        let lines = if !self.placeholder.is_empty() && self.is_empty() {
            self.placeholder_lines()
        } else {
            self.text_lines(top_row as _, height as _, width)
        };

        // To get fine control over the text color and the surrrounding block they have to be rendered separately
        // see https://github.com/ratatui/ratatui/issues/144
        let mut text_area = area;
        if let Some(b) = self.block() {
            text_area = b.inner(area);
            // ratatui does not need `clone()` call because `Block` implements `WidgetRef` and `&T` implements `Widget`
            // where `T: WidgetRef`. So `b.render` internally calls `b.render_ref` and it doesn't move out `self`.
            #[cfg(feature = "tuirs")]
            let b = b.clone();
            b.render(area, buf)
        }

        // Store scroll position for rendering on the next tick
        self.viewport.store(top_row, left_col, width, height);

        self.render_lines(lines, text_area, buf);
    }
}
