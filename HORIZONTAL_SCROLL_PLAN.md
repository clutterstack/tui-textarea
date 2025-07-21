# Plan: Add Horizontal Scrolling to Line-Based tui-textarea

## Overview
Re-implement horizontal scrolling for non-wrapped text by leveraging existing infrastructure and following idiomatic Rust patterns similar to those previously used in the Paragraph widget.

## Key Findings
- **Infrastructure exists but is disabled**: The `Viewport` already tracks horizontal bounds and `CursorMove::InViewport` handles horizontal clamping
- **Current implementation**: `viewport.scroll()` ignores the `cols` parameter (sets `col = 0`)
- **Conflict resolution**: Horizontal scrolling should only work when `wrap_enabled = false`

## Implementation Plan

### 1. Re-enable Horizontal Scrolling in Viewport (`src/widget.rs`)
- **Modify `Viewport::scroll()`**: Remove the hardcoded `col = 0` and use the actual `cols` parameter
- **Add horizontal scroll logic**: Use existing `apply_scroll()` helper for both row and column updates
- **Conditional behavior**: Only apply horizontal scrolling when wrap is disabled

### 2. Add Horizontal Line Clipping (`src/widget.rs`)
- **Modify `text_lines()` method**: Add horizontal offset calculation based on viewport
- **Create line slicing logic**: 
  - Calculate visible character range based on `col_top` and viewport width
  - Handle Unicode width calculations using existing `unicode-width` crate patterns
  - Account for tab expansion using existing `DisplayTextBuilder` patterns
- **Preserve line highlighting**: Ensure `line_spans()` output works correctly with horizontal clipping

### 3. Update Cursor Viewport Logic (`src/scroll.rs` & `src/textarea.rs`)
- **Enhance scroll triggering**: Automatically scroll horizontally when cursor moves outside viewport
- **Modify `scroll_with_shift()`**: Use `CursorMove::InViewport` to ensure cursor remains visible
- **Add horizontal scroll methods**: Enable programmatic horizontal scrolling via `Scrolling::Delta`

### 4. Character Width Handling
- **Tab handling**: Respect existing tab width settings and `DisplayTextBuilder` logic
- **Unicode support**: Use existing `UnicodeWidthChar` patterns for proper character width calculation
- **Line number compatibility**: Ensure horizontal scrolling works with line numbers enabled

### 5. Integration Points
- **Wrap conflict prevention**: Add guards to disable horizontal scrolling when `wrap_enabled = true`
- **Preserve existing API**: Maintain backward compatibility for existing `scroll()` calls
- **Update documentation**: Add examples and notes about horizontal scrolling availability

## Technical Approach

### Character-Based Scrolling
- Use character positions rather than byte positions for consistent behavior
- Leverage existing `char_indices()` patterns from the codebase
- Handle wide characters (tabs, Unicode) using established width calculation methods

### Efficient Line Slicing
- Pre-calculate visible character range before creating spans
- Slice line content at character boundaries, not byte boundaries
- Reuse existing `LineHighlighter` for styling the visible portion

### Cursor Tracking
- Automatically adjust horizontal scroll when cursor moves beyond viewport edges
- Use existing `next_scroll_top()` pattern for smooth horizontal scrolling
- Maintain cursor visibility during text editing operations

## Expected Benefits
- **High-quality idiomatic Rust**: Follows existing codebase patterns and conventions
- **Performance**: Minimal overhead when wrap is enabled (no change to existing behavior)
- **Compatibility**: Works seamlessly with existing features (line numbers, search, selection)
- **User experience**: Natural horizontal navigation for long lines without wrapping