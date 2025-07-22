use crate::textarea::TextArea;

/// Text wrapping functionality for TextArea
impl<'a> TextArea<'a> {
    /// Enable or disable text wrapping.
    /// When enabled, long lines will be wrapped to fit within the text area width.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// textarea.set_wrap(true);
    /// assert!(textarea.wrap_enabled());
    /// ```
    #[cfg(feature = "wrap")]
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
    #[cfg(feature = "wrap")]
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
    #[cfg(feature = "wrap")]
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
    #[cfg(feature = "wrap")]
    pub fn wrap_width(&self) -> Option<usize> {
        self.wrap_width
    }

    /// Calculate the effective wrap width considering line numbers and custom width settings
    #[cfg(feature = "wrap")]
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