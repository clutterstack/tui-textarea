use crate::cursor::CursorMove;
use crate::input::Key;
use crate::textarea::TextArea;
#[cfg(feature = "wrap")]
use textwrap::Options;

/// Mouse handling functionality for TextArea
impl<'a> TextArea<'a> {
    /// Helper method for calculate_effective_wrap_width when wrap feature is not available
    #[cfg(all(feature = "mouse", not(feature = "wrap")))]
    fn calculate_effective_wrap_width(&self, area_width: u16) -> usize {
        let mut wrap_width = area_width as usize;
        
        // Subtract line number width if enabled
        if self.line_number_style().is_some() {
            let lnum_len = crate::util::num_digits(self.lines().len());
            wrap_width = wrap_width.saturating_sub((lnum_len + 2) as usize);
        }
        
        wrap_width.max(1) // Ensure minimum width of 1
    }
    /// Handle mouse events and route to appropriate handler
    #[cfg(feature = "mouse")]
    pub fn handle_mouse_event(&mut self, key: Key, widget_area: crate::ratatui::layout::Rect) -> bool {
        match key {
            Key::MouseClick(x, y) => self.handle_mouse_click(x, y, widget_area),
            Key::MouseDrag(x, y) => self.handle_mouse_drag(x, y, widget_area),
            Key::MouseUp(x, y) => self.handle_mouse_up(x, y, widget_area),
            _ => false,
        }
    }

    /// Handle mouse click events for text selection and cursor positioning
    #[cfg(feature = "mouse")]
    pub fn handle_mouse_click(&mut self, screen_x: u16, screen_y: u16, widget_area: crate::ratatui::layout::Rect) -> bool {
        // Calculate the actual text area, accounting for block borders if present
        let text_area = if let Some(block) = self.block() {
            block.inner(widget_area)
        } else {
            widget_area
        };
        
        // Convert screen coordinates to text area relative coordinates
        let rel_x = screen_x.saturating_sub(text_area.x);
        let rel_y = screen_y.saturating_sub(text_area.y);
        
        // Check if click is within the text area bounds
        if rel_x >= text_area.width || rel_y >= text_area.height {
            return false;
        }
        
        if let Some((row, col)) = self.screen_to_logical_position(rel_x, rel_y, text_area.width, text_area.height) {
            // Start selection on mouse down
            self.selection_start = Some((row, col));
            self.move_cursor(CursorMove::Jump(row as u16, col as u16));
            true
        } else {
            false
        }
    }

    /// Handle mouse drag events for extending text selection
    #[cfg(feature = "mouse")]
    pub fn handle_mouse_drag(&mut self, screen_x: u16, screen_y: u16, widget_area: crate::ratatui::layout::Rect) -> bool {
        // Calculate the actual text area, accounting for block borders if present
        let text_area = if let Some(block) = self.block() {
            block.inner(widget_area)
        } else {
            widget_area
        };
        
        // Convert screen coordinates to text area relative coordinates
        let rel_x = screen_x.saturating_sub(text_area.x);
        let rel_y = screen_y.saturating_sub(text_area.y);
        
        // Check if drag is within the text area bounds
        if rel_x >= text_area.width || rel_y >= text_area.height {
            return false;
        }
        
        if let Some((row, col)) = self.screen_to_logical_position(rel_x, rel_y, text_area.width, text_area.height) {
            // Extend selection to current drag position
            self.move_cursor(CursorMove::Jump(row as u16, col as u16));
            true
        } else {
            false
        }
    }

    /// Handle mouse up events for finalizing text selection
    #[cfg(feature = "mouse")]
    pub fn handle_mouse_up(&mut self, screen_x: u16, screen_y: u16, widget_area: crate::ratatui::layout::Rect) -> bool {
        // Calculate the actual text area, accounting for block borders if present
        let text_area = if let Some(block) = self.block() {
            block.inner(widget_area)
        } else {
            widget_area
        };
        
        // Convert screen coordinates to text area relative coordinates
        let rel_x = screen_x.saturating_sub(text_area.x);
        let rel_y = screen_y.saturating_sub(text_area.y);
        
        // Check if release is within the text area bounds
        if rel_x >= text_area.width || rel_y >= text_area.height {
            return false;
        }
        
        if let Some((row, col)) = self.screen_to_logical_position(rel_x, rel_y, text_area.width, text_area.height) {
            // Finalize selection at current position
            self.move_cursor(CursorMove::Jump(row as u16, col as u16));
            true
        } else {
            false
        }
    }

    /// Convert screen coordinates to logical text position
    #[cfg(feature = "mouse")]
    pub fn screen_to_logical_position(&self, rel_x: u16, rel_y: u16, area_width: u16, _area_height: u16) -> Option<(usize, usize)> {
        // Get the current viewport information
        let (top_row, _) = self.viewport.scroll_top();
        let display_line_index = rel_y as usize;
        
        // Check if wrapping is enabled and handle accordingly
        #[cfg(feature = "wrap")]
        let wrap_enabled = self.wrap_enabled();
        #[cfg(not(feature = "wrap"))]
        let wrap_enabled = false;
        
        if wrap_enabled {
            #[cfg(feature = "wrap")]
            return self.screen_to_logical_position_wrapped(rel_x, display_line_index, area_width, top_row as usize);
        } else {
            return self.screen_to_logical_position_unwrapped(rel_x, display_line_index, top_row as usize);
        }
        
        #[cfg(not(feature = "wrap"))]
        self.screen_to_logical_position_unwrapped(rel_x, display_line_index, top_row as usize)
    }

    /// Convert screen coordinates to logical position when wrapping is disabled
    #[cfg(feature = "mouse")]
    fn screen_to_logical_position_unwrapped(&self, rel_x: u16, display_line_index: usize, top_row: usize) -> Option<(usize, usize)> {
        let logical_row = top_row + display_line_index;
        
        // Check if the logical row exists
        if logical_row >= self.lines().len() {
            return None;
        }
        
        // Account for line numbers if enabled
        let lnum_width = self.calculate_line_number_width();
        if lnum_width > 0 && rel_x < lnum_width {
            // Click was on line numbers, position at start of line
            return Some((logical_row, 0));
        }
        
        let adjusted_x = if lnum_width > 0 {
            rel_x.saturating_sub(lnum_width)
        } else {
            rel_x
        };
        
        let logical_col = std::cmp::min(adjusted_x as usize, self.lines()[logical_row].chars().count());
        Some((logical_row, logical_col))
    }

    /// Convert screen coordinates to logical position when wrapping is enabled
    #[cfg(all(feature = "mouse", feature = "wrap"))]
    fn screen_to_logical_position_wrapped(&self, rel_x: u16, display_line_index: usize, area_width: u16, top_row: usize) -> Option<(usize, usize)> {
        let wrap_width = self.calculate_effective_wrap_width(area_width);
        let lnum_width = self.calculate_line_number_width();
        let mut current_display_line = 0;
        // Create Options and set preserve_trailing_space
        let options = Options::new(wrap_width).preserve_trailing_space(true);

        
        // Walk through logical lines to find which one contains our target display line
        for (logical_row, line_text) in self.lines().iter().enumerate() {
            if logical_row < top_row {
                continue; // Skip lines above the viewport
            }
            
            let wrapped_lines = textwrap::wrap(line_text, &options);
            let wrapped_line_count = wrapped_lines.len().max(1); // Empty lines still take one display line
            
            if current_display_line + wrapped_line_count > display_line_index {
                // This logical line contains our target display line
                let wrapped_line_offset = display_line_index - current_display_line;
                
                if wrapped_line_offset < wrapped_lines.len() {
                    let wrapped_line = &wrapped_lines[wrapped_line_offset];
                    
                    // Account for line numbers if enabled
                    if lnum_width > 0 && rel_x < lnum_width {
                        // Click was on line numbers
                        let char_offset = wrapped_lines.iter()
                            .take(wrapped_line_offset)
                            .map(|line| line.chars().count())
                            .sum::<usize>();
                        return Some((logical_row, char_offset));
                    }
                    
                    let adjusted_x = if lnum_width > 0 {
                        rel_x.saturating_sub(lnum_width)
                    } else {
                        rel_x
                    };
                    
                    let char_in_wrapped_line = std::cmp::min(adjusted_x as usize, wrapped_line.chars().count());
                    
                    // Calculate the absolute character position in the original line
                    let char_offset = wrapped_lines.iter()
                        .take(wrapped_line_offset)
                        .map(|line| line.chars().count())
                        .sum::<usize>();
                    
                    let logical_col = char_offset + char_in_wrapped_line;
                    return Some((logical_row, logical_col));
                } else {
                    // Click was on an empty area after the last wrapped line
                    return Some((logical_row, line_text.chars().count()));
                }
            }
            
            current_display_line += wrapped_line_count;
        }
        
        None // Click was beyond all text
    }

    /// Convert logical cursor position to screen coordinates
    #[cfg(feature = "mouse")]
    pub fn logical_to_screen_position(&self, area_width: u16, area_height: u16) -> Option<(u16, u16)> {
        let (logical_row, logical_col) = self.cursor();
        let (top_row, left_col) = self.viewport.scroll_top();
        
        // Check if cursor is outside viewable area vertically
        if logical_row < top_row as usize {
            return None; // Cursor is above visible area
        }
        
        #[cfg(feature = "wrap")]
        let wrap_enabled = self.wrap_enabled();
        #[cfg(not(feature = "wrap"))]
        let wrap_enabled = false;
        
        if wrap_enabled {
            #[cfg(feature = "wrap")]
            return self.logical_to_screen_position_wrapped(logical_row, logical_col, area_width, area_height, top_row as usize);
        } else {
            return self.logical_to_screen_position_unwrapped(logical_row, logical_col, area_width, area_height, top_row as usize, left_col);
        }
        
        #[cfg(not(feature = "wrap"))]
        self.logical_to_screen_position_unwrapped(logical_row, logical_col, area_width, area_height, top_row as usize, left_col)
    }

    /// Convert logical position to screen coordinates when wrapping is disabled
    #[cfg(feature = "mouse")]
    fn logical_to_screen_position_unwrapped(&self, logical_row: usize, logical_col: usize, area_width: u16, area_height: u16, top_row: usize, left_col: u16) -> Option<(u16, u16)> {
        // Check if cursor row is within visible area
        let screen_y = logical_row.checked_sub(top_row)?;
        if screen_y >= area_height as usize {
            return None; // Cursor is below visible area
        }
        
        // Get line and calculate visual position
        if logical_row >= self.lines().len() {
            return None;
        }
        
        let line = &self.lines()[logical_row];
        let visual_col = line.chars().take(logical_col).map(|c| {
            if c == '\t' {
                self.tab_len as usize
            } else {
                unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
            }
        }).sum::<usize>();
        
        // Account for horizontal scrolling and line numbers
        let lnum_width = self.calculate_line_number_width();
        let screen_x = visual_col.saturating_sub(left_col as usize) + lnum_width as usize;
        
        if screen_x >= area_width as usize {
            return None; // Cursor is outside visible area horizontally
        }
        
        Some((screen_x as u16, screen_y as u16))
    }

    /// Convert logical position to screen coordinates when wrapping is enabled
    #[cfg(all(feature = "mouse", feature = "wrap"))]
    fn logical_to_screen_position_wrapped(&self, logical_row: usize, logical_col: usize, area_width: u16, area_height: u16, top_row: usize) -> Option<(u16, u16)> {
        let wrap_width = self.calculate_effective_wrap_width(area_width);
        let lnum_width = self.calculate_line_number_width();
        let mut current_display_line = 0;

        // Create Options and set preserve_trailing_space
        let options = Options::new(wrap_width).preserve_trailing_space(true);

        
        // Count display lines before the cursor's logical row
        for row in top_row..logical_row {
            if row >= self.lines().len() {
                break;
            }
            let wrapped_lines = textwrap::wrap(&self.lines()[row], &options);
            current_display_line += wrapped_lines.len().max(1);
        }
        
        // Check if we're already beyond the visible area
        if current_display_line >= area_height as usize {
            return None;
        }
        
        // Handle the cursor's logical row
        if logical_row < self.lines().len() {
            let line = &self.lines()[logical_row];
            let wrapped_lines = textwrap::wrap(line, &options);
            
            if wrapped_lines.is_empty() {
                // Empty line case
                let screen_y = current_display_line;
                if screen_y >= area_height as usize {
                    return None;
                }
                return Some((lnum_width, screen_y as u16));
            }
            
            // Find which wrapped line contains the cursor
            let mut char_count = 0;
            for (wrapped_idx, wrapped_line) in wrapped_lines.iter().enumerate() {
                let line_char_count = wrapped_line.chars().count();
                
                if logical_col <= char_count + line_char_count {
                    // Cursor is in this wrapped line
                    let screen_y = current_display_line + wrapped_idx;
                    if screen_y >= area_height as usize {
                        return None;
                    }
                    
                    let char_in_wrapped_line = logical_col - char_count;
                    let visual_x = wrapped_line.chars().take(char_in_wrapped_line).map(|c| {
                        if c == '\t' {
                            self.tab_len as usize
                        } else {
                            unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
                        }
                    }).sum::<usize>();
                    
                    let screen_x = lnum_width as usize + visual_x;
                    
                    return Some((screen_x as u16, screen_y as u16));
                }
                
                char_count += line_char_count;
            }
            
            // Cursor is beyond the end of the line
            let last_wrapped_idx = wrapped_lines.len() - 1;
            let screen_y = current_display_line + last_wrapped_idx;
            if screen_y >= area_height as usize {
                return None;
            }
            
            let last_wrapped_line = &wrapped_lines[last_wrapped_idx];
            let visual_x = last_wrapped_line.chars().map(|c| {
                if c == '\t' {
                    self.tab_len as usize
                } else {
                    unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
                }
            }).sum::<usize>();
            
            let screen_x = lnum_width as usize + visual_x;
            Some((screen_x as u16, screen_y as u16))
        } else {
            None
        }
    }
}