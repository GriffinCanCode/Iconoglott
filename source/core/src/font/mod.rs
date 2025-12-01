//! Font metrics for accurate text layout
//!
//! Provides glyph-level measurements for common system fonts and supports
//! loading custom fonts via ttf-parser. All metrics are normalized to 1em.

use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Font Metrics Types
// ─────────────────────────────────────────────────────────────────────────────

/// Metrics for a single glyph, normalized to 1em (units_per_em = 1.0)
#[derive(Clone, Copy, Debug)]
pub struct GlyphMetrics {
    pub advance_width: f32,
    pub left_bearing: f32,
}

impl Default for GlyphMetrics {
    fn default() -> Self { Self { advance_width: 0.5, left_bearing: 0.0 } }
}

/// Full font metrics including vertical dimensions and glyph widths
#[derive(Clone, Debug)]
pub struct FontMetrics {
    pub ascender: f32,      // Height above baseline (normalized)
    pub descender: f32,     // Depth below baseline (negative, normalized)
    pub line_gap: f32,      // Extra space between lines (normalized)
    pub cap_height: f32,    // Height of capital letters (normalized)
    pub x_height: f32,      // Height of lowercase x (normalized)
    pub avg_char_width: f32, // Average character width (normalized)
    widths: HashMap<char, f32>, // Per-character advance widths
}

impl Default for FontMetrics {
    fn default() -> Self { DEFAULT_SANS_SERIF.clone() }
}

impl FontMetrics {
    /// Get advance width for a character (normalized to 1em)
    #[inline]
    pub fn char_width(&self, c: char) -> f32 {
        *self.widths.get(&c).unwrap_or(&self.avg_char_width)
    }

    /// Measure text width at given font size
    pub fn measure_width(&self, text: &str, size: f32) -> f32 {
        text.chars().map(|c| self.char_width(c)).sum::<f32>() * size
    }

    /// Measure text height at given font size  
    pub fn measure_height(&self, size: f32) -> f32 {
        (self.ascender - self.descender) * size
    }

    /// Full text bounds: (width, height, baseline_offset)
    pub fn measure(&self, text: &str, size: f32) -> TextMetrics {
        TextMetrics {
            width: self.measure_width(text, size),
            height: self.measure_height(size),
            ascender: self.ascender * size,
            descender: self.descender * size,
        }
    }

    /// Line height (ascender - descender + line_gap)
    #[inline]
    pub fn line_height(&self, size: f32) -> f32 {
        (self.ascender - self.descender + self.line_gap) * size
    }
}

/// Text measurement result
#[derive(Clone, Copy, Debug, Default)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub ascender: f32,   // Distance from baseline to top
    pub descender: f32,  // Distance from baseline to bottom (negative)
}

// ─────────────────────────────────────────────────────────────────────────────
// Bundled Font Metrics (Common System Fonts)
// ─────────────────────────────────────────────────────────────────────────────

lazy_static::lazy_static! {
    /// Default sans-serif metrics (Arial/Helvetica-like)
    pub static ref DEFAULT_SANS_SERIF: FontMetrics = FontMetrics {
        ascender: 0.88,
        descender: -0.12,
        line_gap: 0.0,
        cap_height: 0.72,
        x_height: 0.52,
        avg_char_width: 0.52,
        widths: build_sans_serif_widths(),
    };

    /// Serif metrics (Times-like)
    pub static ref DEFAULT_SERIF: FontMetrics = FontMetrics {
        ascender: 0.89,
        descender: -0.22,
        line_gap: 0.0,
        cap_height: 0.66,
        x_height: 0.45,
        avg_char_width: 0.48,
        widths: build_serif_widths(),
    };

    /// Monospace metrics (Courier-like)
    pub static ref DEFAULT_MONO: FontMetrics = FontMetrics {
        ascender: 0.83,
        descender: -0.17,
        line_gap: 0.0,
        cap_height: 0.57,
        x_height: 0.43,
        avg_char_width: 0.60,
        widths: build_mono_widths(),
    };

    /// Font family to metrics lookup
    static ref FONT_METRICS: HashMap<&'static str, &'static FontMetrics> = {
        let mut m = HashMap::new();
        // Sans-serif families
        for name in &["Arial", "Helvetica", "Verdana", "Tahoma", "Trebuchet MS", 
                      "system-ui", "sans-serif", "-apple-system", "BlinkMacSystemFont",
                      "Segoe UI", "Roboto", "Ubuntu", "Cantarell", "Noto Sans",
                      "Liberation Sans", "SF Pro", "Inter"] {
            m.insert(*name, &*DEFAULT_SANS_SERIF);
        }
        // Serif families
        for name in &["Times", "Times New Roman", "Georgia", "Palatino", "serif",
                      "Cambria", "Book Antiqua", "Noto Serif", "Liberation Serif"] {
            m.insert(*name, &*DEFAULT_SERIF);
        }
        // Monospace families
        for name in &["Courier", "Courier New", "monospace", "Consolas", 
                      "Monaco", "Menlo", "Liberation Mono", "DejaVu Sans Mono",
                      "SF Mono", "JetBrains Mono", "Fira Code", "Source Code Pro"] {
            m.insert(*name, &*DEFAULT_MONO);
        }
        m
    };
}

/// Get metrics for a font family (falls back to sans-serif)
pub fn get_metrics(font_family: &str) -> &'static FontMetrics {
    // Try exact match first
    if let Some(m) = FONT_METRICS.get(font_family) {
        return m;
    }
    // Try first font in comma-separated list
    if let Some(first) = font_family.split(',').next() {
        let trimmed = first.trim().trim_matches('"').trim_matches('\'');
        if let Some(m) = FONT_METRICS.get(trimmed) {
            return m;
        }
    }
    // Detect by keywords
    let lower = font_family.to_lowercase();
    if lower.contains("mono") || lower.contains("code") || lower.contains("courier") {
        return &DEFAULT_MONO;
    }
    if lower.contains("times") || lower.contains("serif") && !lower.contains("sans") {
        return &DEFAULT_SERIF;
    }
    &DEFAULT_SANS_SERIF
}

/// Measure text with given font family and size
pub fn measure_text(text: &str, font_family: &str, size: f32) -> TextMetrics {
    get_metrics(font_family).measure(text, size)
}

// ─────────────────────────────────────────────────────────────────────────────
// Character Width Tables (normalized to 1em)
// ─────────────────────────────────────────────────────────────────────────────

fn build_sans_serif_widths() -> HashMap<char, f32> {
    let mut w = HashMap::with_capacity(128);
    // Based on Arial/Helvetica metrics (normalized to 1em)
    // Lowercase
    w.insert('a', 0.556); w.insert('b', 0.556); w.insert('c', 0.500);
    w.insert('d', 0.556); w.insert('e', 0.556); w.insert('f', 0.278);
    w.insert('g', 0.556); w.insert('h', 0.556); w.insert('i', 0.222);
    w.insert('j', 0.222); w.insert('k', 0.500); w.insert('l', 0.222);
    w.insert('m', 0.833); w.insert('n', 0.556); w.insert('o', 0.556);
    w.insert('p', 0.556); w.insert('q', 0.556); w.insert('r', 0.333);
    w.insert('s', 0.500); w.insert('t', 0.278); w.insert('u', 0.556);
    w.insert('v', 0.500); w.insert('w', 0.722); w.insert('x', 0.500);
    w.insert('y', 0.500); w.insert('z', 0.500);
    // Uppercase
    w.insert('A', 0.667); w.insert('B', 0.667); w.insert('C', 0.722);
    w.insert('D', 0.722); w.insert('E', 0.667); w.insert('F', 0.611);
    w.insert('G', 0.778); w.insert('H', 0.722); w.insert('I', 0.278);
    w.insert('J', 0.500); w.insert('K', 0.667); w.insert('L', 0.556);
    w.insert('M', 0.833); w.insert('N', 0.722); w.insert('O', 0.778);
    w.insert('P', 0.667); w.insert('Q', 0.778); w.insert('R', 0.722);
    w.insert('S', 0.667); w.insert('T', 0.611); w.insert('U', 0.722);
    w.insert('V', 0.667); w.insert('W', 0.944); w.insert('X', 0.667);
    w.insert('Y', 0.667); w.insert('Z', 0.611);
    // Digits
    for d in '0'..='9' { w.insert(d, 0.556); }
    // Punctuation & symbols
    w.insert(' ', 0.278); w.insert('!', 0.278); w.insert('"', 0.355);
    w.insert('#', 0.556); w.insert('$', 0.556); w.insert('%', 0.889);
    w.insert('&', 0.667); w.insert('\'', 0.191); w.insert('(', 0.333);
    w.insert(')', 0.333); w.insert('*', 0.389); w.insert('+', 0.584);
    w.insert(',', 0.278); w.insert('-', 0.333); w.insert('.', 0.278);
    w.insert('/', 0.278); w.insert(':', 0.278); w.insert(';', 0.278);
    w.insert('<', 0.584); w.insert('=', 0.584); w.insert('>', 0.584);
    w.insert('?', 0.556); w.insert('@', 1.015); w.insert('[', 0.278);
    w.insert('\\', 0.278); w.insert(']', 0.278); w.insert('^', 0.469);
    w.insert('_', 0.556); w.insert('`', 0.333); w.insert('{', 0.334);
    w.insert('|', 0.260); w.insert('}', 0.334); w.insert('~', 0.584);
    w
}

fn build_serif_widths() -> HashMap<char, f32> {
    let mut w = HashMap::with_capacity(128);
    // Based on Times New Roman metrics (normalized to 1em)
    // Lowercase
    w.insert('a', 0.444); w.insert('b', 0.500); w.insert('c', 0.444);
    w.insert('d', 0.500); w.insert('e', 0.444); w.insert('f', 0.333);
    w.insert('g', 0.500); w.insert('h', 0.500); w.insert('i', 0.278);
    w.insert('j', 0.278); w.insert('k', 0.500); w.insert('l', 0.278);
    w.insert('m', 0.778); w.insert('n', 0.500); w.insert('o', 0.500);
    w.insert('p', 0.500); w.insert('q', 0.500); w.insert('r', 0.333);
    w.insert('s', 0.389); w.insert('t', 0.278); w.insert('u', 0.500);
    w.insert('v', 0.500); w.insert('w', 0.722); w.insert('x', 0.500);
    w.insert('y', 0.500); w.insert('z', 0.444);
    // Uppercase
    w.insert('A', 0.722); w.insert('B', 0.667); w.insert('C', 0.667);
    w.insert('D', 0.722); w.insert('E', 0.611); w.insert('F', 0.556);
    w.insert('G', 0.722); w.insert('H', 0.722); w.insert('I', 0.333);
    w.insert('J', 0.389); w.insert('K', 0.722); w.insert('L', 0.611);
    w.insert('M', 0.889); w.insert('N', 0.722); w.insert('O', 0.722);
    w.insert('P', 0.556); w.insert('Q', 0.722); w.insert('R', 0.667);
    w.insert('S', 0.556); w.insert('T', 0.611); w.insert('U', 0.722);
    w.insert('V', 0.722); w.insert('W', 0.944); w.insert('X', 0.722);
    w.insert('Y', 0.722); w.insert('Z', 0.611);
    // Digits  
    for d in '0'..='9' { w.insert(d, 0.500); }
    // Punctuation
    w.insert(' ', 0.250); w.insert('!', 0.333); w.insert('"', 0.408);
    w.insert('#', 0.500); w.insert('$', 0.500); w.insert('%', 0.833);
    w.insert('&', 0.778); w.insert('\'', 0.180); w.insert('(', 0.333);
    w.insert(')', 0.333); w.insert('*', 0.500); w.insert('+', 0.564);
    w.insert(',', 0.250); w.insert('-', 0.333); w.insert('.', 0.250);
    w.insert('/', 0.278); w.insert(':', 0.278); w.insert(';', 0.278);
    w.insert('<', 0.564); w.insert('=', 0.564); w.insert('>', 0.564);
    w.insert('?', 0.444); w.insert('@', 0.921); w.insert('[', 0.333);
    w.insert('\\', 0.278); w.insert(']', 0.333); w.insert('^', 0.469);
    w.insert('_', 0.500); w.insert('`', 0.333); w.insert('{', 0.480);
    w.insert('|', 0.200); w.insert('}', 0.480); w.insert('~', 0.541);
    w
}

fn build_mono_widths() -> HashMap<char, f32> {
    // All characters same width in monospace
    let mut w = HashMap::with_capacity(128);
    for c in ' '..='~' { w.insert(c, 0.60); }
    w
}

// ─────────────────────────────────────────────────────────────────────────────
// TTF Parser Integration (optional font loading)
// ─────────────────────────────────────────────────────────────────────────────

/// Parse font metrics from raw TTF/OTF data
#[cfg(feature = "font-parsing")]
pub fn parse_font_data(data: &[u8]) -> Option<FontMetrics> {
    use ttf_parser::Face;
    
    let face = Face::parse(data, 0).ok()?;
    let units = face.units_per_em() as f32;
    let scale = 1.0 / units;
    
    let mut widths = HashMap::new();
    for c in ' '..='~' {
        if let Some(glyph_id) = face.glyph_index(c) {
            if let Some(advance) = face.glyph_hor_advance(glyph_id) {
                widths.insert(c, advance as f32 * scale);
            }
        }
    }
    
    let avg_char_width = if widths.is_empty() { 0.5 } 
        else { widths.values().sum::<f32>() / widths.len() as f32 };
    
    Some(FontMetrics {
        ascender: face.ascender() as f32 * scale,
        descender: face.descender() as f32 * scale,
        line_gap: face.line_gap() as f32 * scale,
        cap_height: face.capital_height().unwrap_or((face.ascender() as f32 * 0.75) as i16) as f32 * scale,
        x_height: face.x_height().unwrap_or((face.ascender() as f32 * 0.5) as i16) as f32 * scale,
        avg_char_width,
        widths,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_text_sans() {
        let m = measure_text("Hello", "Arial", 16.0);
        assert!(m.width > 30.0 && m.width < 50.0, "width={}", m.width);
        assert!(m.height > 14.0 && m.height < 18.0, "height={}", m.height);
    }

    #[test]
    fn test_measure_text_mono() {
        let m = measure_text("iii", "Courier", 16.0);
        let m2 = measure_text("WWW", "Courier", 16.0);
        // Monospace: same width regardless of character
        assert!((m.width - m2.width).abs() < 0.01, "mono widths differ: {} vs {}", m.width, m2.width);
    }

    #[test]
    fn test_font_family_fallback() {
        // Unknown font falls back to sans-serif
        let m1 = get_metrics("UnknownFont");
        let m2 = get_metrics("Arial");
        assert_eq!(m1.avg_char_width, m2.avg_char_width);
    }

    #[test]
    fn test_comma_separated_fonts() {
        let m = get_metrics("'Helvetica Neue', Helvetica, Arial, sans-serif");
        assert_eq!(m.avg_char_width, DEFAULT_SANS_SERIF.avg_char_width);
    }

    #[test]
    fn test_variable_width() {
        let m = get_metrics("Arial");
        // 'i' should be narrower than 'W'
        assert!(m.char_width('i') < m.char_width('W'));
        // 'm' should be wider than average
        assert!(m.char_width('m') > m.avg_char_width);
    }
}

