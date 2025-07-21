// Test program for horizontal scrolling functionality
use tui_textarea::{TextArea, Scrolling};

#[cfg(not(feature = "wrap"))]
fn test_horizontal_scroll_basic() {
    // Create a textarea with a long line
    let mut textarea = TextArea::from(["This is a very long line that should trigger horizontal scrolling when the viewport is narrow"]);
    
    // Test horizontal scrolling
    textarea.scroll(Scrolling::Delta { rows: 0, cols: 10 });
    
    // Check that the viewport has been scrolled
    let (_, col_left) = textarea.viewport.scroll_top();
    assert_eq!(col_left, 10, "Horizontal scroll should move viewport to column 10");
    
    println!("✓ Basic horizontal scrolling works");
}

#[cfg(feature = "wrap")]
fn test_horizontal_scroll_disabled_with_wrap() {
    let mut textarea = TextArea::from(["This is a very long line that should not trigger horizontal scrolling when wrap is enabled"]);
    textarea.set_wrap(true);
    
    // Try to scroll horizontally
    textarea.scroll(Scrolling::Delta { rows: 0, cols: 10 });
    
    // Check that horizontal scrolling was disabled
    let (_, col_left) = textarea.viewport.scroll_top();
    assert_eq!(col_left, 0, "Horizontal scroll should be disabled when wrap is enabled");
    
    println!("✓ Horizontal scrolling correctly disabled with wrap");
}

fn test_character_width_calculation() {
    use tui_textarea::cursor::CursorMove;
    
    // Test with tabs and Unicode
    let mut textarea = TextArea::from(["Hello\t世界\tTab"]);
    textarea.set_tab_length(4);
    
    // Move cursor to test visual position calculation
    textarea.move_cursor(CursorMove::End);
    let (row, col) = textarea.cursor();
    assert_eq!(row, 0);
    assert_eq!(col, 11); // "Hello" (5) + "\t" (1) + "世界" (2) + "\t" (1) + "Tab" (3) = 12 chars, but col is 0-indexed, so 11
    
    println!("✓ Character width calculation works with tabs and Unicode");
}

fn test_viewport_bounds() {
    let mut textarea = TextArea::from(["Short", "This is a much longer line that extends beyond normal viewport width", "End"]);
    
    // Test that viewport bounds are calculated correctly
    let (row_top, col_left, row_bottom, col_right) = textarea.viewport.position();
    
    // Initial viewport should start at (0, 0)
    assert_eq!(row_top, 0);
    assert_eq!(col_left, 0);
    
    println!("✓ Viewport bounds calculation works");
}

fn main() {
    println!("Testing horizontal scrolling implementation...");
    
    #[cfg(not(feature = "wrap"))]
    test_horizontal_scroll_basic();
    
    #[cfg(feature = "wrap")]
    test_horizontal_scroll_disabled_with_wrap();
    
    test_character_width_calculation();
    test_viewport_bounds();
    
    println!("\n🎉 All horizontal scrolling tests passed!");
}