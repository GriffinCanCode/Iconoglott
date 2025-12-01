//! Stable element identity system with content-addressed hashing
//!
//! Separates identity (what makes an element unique) from content (detecting changes).
//! Uses FNV-1a for fast hashing with good distribution.

use std::sync::atomic::{AtomicU64, Ordering};

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Fast FNV-1a hasher for identity computation
#[derive(Debug, Clone, Copy)]
pub struct Fnv1a(u64);

impl Default for Fnv1a {
    fn default() -> Self { Self(FNV_OFFSET) }
}

impl Fnv1a {
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }

    #[inline]
    pub fn write_u8(&mut self, v: u8) { self.update(&[v]); }
    
    #[inline]
    pub fn write_u32(&mut self, v: u32) { self.update(&v.to_le_bytes()); }
    
    #[inline]
    pub fn write_u64(&mut self, v: u64) { self.update(&v.to_le_bytes()); }
    
    #[inline]
    pub fn write_f32(&mut self, v: f32) { self.update(&v.to_bits().to_le_bytes()); }

    #[inline]
    pub fn write_str(&mut self, s: &str) { self.update(s.as_bytes()); }

    #[inline]
    pub fn finish(self) -> u64 { self.0 }
}

/// Stable element identity - unique within a scene across mutations
/// 
/// Identity = hash(creation_order, kind_discriminant, key_properties)
/// Key properties are the "identity-defining" props (position, not style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(pub u64);

impl ElementId {
    /// Create identity from creation order and kind
    pub fn new(order: u64, kind: u8) -> Self {
        let mut h = Fnv1a::default();
        h.write_u64(order);
        h.write_u8(kind);
        Self(h.finish())
    }

    /// Create identity with additional key bytes
    pub fn with_key(order: u64, kind: u8, key: &[u8]) -> Self {
        let mut h = Fnv1a::default();
        h.write_u64(order);
        h.write_u8(kind);
        h.update(key);
        Self(h.finish())
    }
}

/// Content hash for detecting element changes (full property comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(pub u64);

impl ContentHash {
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut h = Fnv1a::default();
        h.update(data);
        Self(h.finish())
    }

    pub fn from_svg(svg: &str) -> Self { Self::from_bytes(svg.as_bytes()) }
}

/// Monotonic ID generator for stable element ordering
#[derive(Debug)]
pub struct IdGen(AtomicU64);

impl Default for IdGen {
    fn default() -> Self { Self(AtomicU64::new(0)) }
}

impl IdGen {
    pub fn next(&self) -> u64 { self.0.fetch_add(1, Ordering::Relaxed) }
    
    pub fn reset(&self) { self.0.store(0, Ordering::Relaxed); }
}

impl Clone for IdGen {
    fn clone(&self) -> Self { Self(AtomicU64::new(self.0.load(Ordering::Relaxed))) }
}

/// Kind discriminant for element types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Rect = 0,
    Circle = 1,
    Ellipse = 2,
    Line = 3,
    Path = 4,
    Polygon = 5,
    Text = 6,
    Image = 7,
    Group = 8,
    Gradient = 9,
    Filter = 10,
}

impl ElementKind {
    pub fn as_u8(self) -> u8 { self as u8 }
    
    pub fn name(self) -> &'static str {
        match self {
            Self::Rect => "rect",
            Self::Circle => "circle",
            Self::Ellipse => "ellipse",
            Self::Line => "line",
            Self::Path => "path",
            Self::Polygon => "polygon",
            Self::Text => "text",
            Self::Image => "image",
            Self::Group => "group",
            Self::Gradient => "gradient",
            Self::Filter => "filter",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Fnv1a tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_fnv1a_default() {
        let h = Fnv1a::default();
        assert_eq!(h.finish(), FNV_OFFSET);
    }

    #[test]
    fn test_fnv1a_update_empty() {
        let mut h = Fnv1a::default();
        h.update(&[]);
        assert_eq!(h.finish(), FNV_OFFSET);
    }

    #[test]
    fn test_fnv1a_deterministic() {
        let mut h1 = Fnv1a::default();
        let mut h2 = Fnv1a::default();
        h1.update(b"hello");
        h2.update(b"hello");
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_fnv1a_different_input() {
        let mut h1 = Fnv1a::default();
        let mut h2 = Fnv1a::default();
        h1.update(b"hello");
        h2.update(b"world");
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_fnv1a_write_u8() {
        let mut h = Fnv1a::default();
        h.write_u8(42);
        assert_ne!(h.finish(), FNV_OFFSET);
    }

    #[test]
    fn test_fnv1a_write_u32() {
        let mut h = Fnv1a::default();
        h.write_u32(123456);
        assert_ne!(h.finish(), FNV_OFFSET);
    }

    #[test]
    fn test_fnv1a_write_u64() {
        let mut h = Fnv1a::default();
        h.write_u64(0xDEADBEEF);
        assert_ne!(h.finish(), FNV_OFFSET);
    }

    #[test]
    fn test_fnv1a_write_f32() {
        let mut h1 = Fnv1a::default();
        let mut h2 = Fnv1a::default();
        h1.write_f32(3.14159);
        h2.write_f32(3.14159);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn test_fnv1a_write_str() {
        let mut h = Fnv1a::default();
        h.write_str("test string");
        assert_ne!(h.finish(), FNV_OFFSET);
    }

    #[test]
    fn test_fnv1a_order_matters() {
        let mut h1 = Fnv1a::default();
        let mut h2 = Fnv1a::default();
        h1.update(b"ab");
        h2.update(b"ba");
        assert_ne!(h1.finish(), h2.finish());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ElementId tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_element_id_stability() {
        let id1 = ElementId::new(0, ElementKind::Rect.as_u8());
        let id2 = ElementId::new(0, ElementKind::Rect.as_u8());
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_element_id_uniqueness_by_order() {
        let id1 = ElementId::new(0, ElementKind::Rect.as_u8());
        let id2 = ElementId::new(1, ElementKind::Rect.as_u8());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_element_id_uniqueness_by_kind() {
        let id1 = ElementId::new(0, ElementKind::Rect.as_u8());
        let id2 = ElementId::new(0, ElementKind::Circle.as_u8());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_element_id_with_key() {
        let id1 = ElementId::with_key(0, 0, b"key1");
        let id2 = ElementId::with_key(0, 0, b"key1");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_element_id_with_key_different() {
        let id1 = ElementId::with_key(0, 0, b"key1");
        let id2 = ElementId::with_key(0, 0, b"key2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_element_id_hash_impl() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ElementId::new(0, 0));
        set.insert(ElementId::new(1, 0));
        set.insert(ElementId::new(0, 0)); // duplicate
        assert_eq!(set.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ContentHash tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_content_hash_determinism() {
        let h1 = ContentHash::from_svg("<rect x=\"0\"/>");
        let h2 = ContentHash::from_svg("<rect x=\"0\"/>");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_from_bytes() {
        let h1 = ContentHash::from_bytes(b"test");
        let h2 = ContentHash::from_bytes(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_content_hash_different() {
        let h1 = ContentHash::from_svg("<rect/>");
        let h2 = ContentHash::from_svg("<circle/>");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_content_hash_whitespace_matters() {
        let h1 = ContentHash::from_svg("<rect />");
        let h2 = ContentHash::from_svg("<rect/>");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_content_hash_case_sensitive() {
        let h1 = ContentHash::from_svg("<Rect/>");
        let h2 = ContentHash::from_svg("<rect/>");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_content_hash_empty() {
        let h1 = ContentHash::from_svg("");
        let h2 = ContentHash::from_svg("");
        assert_eq!(h1, h2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IdGen tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_idgen_monotonic() {
        let gen = IdGen::default();
        assert_eq!(gen.next(), 0);
        assert_eq!(gen.next(), 1);
        assert_eq!(gen.next(), 2);
    }

    #[test]
    fn test_idgen_reset() {
        let gen = IdGen::default();
        gen.next();
        gen.next();
        gen.reset();
        assert_eq!(gen.next(), 0);
    }

    #[test]
    fn test_idgen_clone() {
        let gen1 = IdGen::default();
        gen1.next();
        gen1.next();
        let gen2 = gen1.clone();
        assert_eq!(gen1.next(), gen2.next());
    }

    #[test]
    fn test_idgen_large_sequence() {
        let gen = IdGen::default();
        for i in 0..1000 {
            assert_eq!(gen.next(), i);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ElementKind tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_element_kind_as_u8() {
        assert_eq!(ElementKind::Rect.as_u8(), 0);
        assert_eq!(ElementKind::Circle.as_u8(), 1);
        assert_eq!(ElementKind::Ellipse.as_u8(), 2);
        assert_eq!(ElementKind::Line.as_u8(), 3);
        assert_eq!(ElementKind::Path.as_u8(), 4);
        assert_eq!(ElementKind::Polygon.as_u8(), 5);
        assert_eq!(ElementKind::Text.as_u8(), 6);
        assert_eq!(ElementKind::Image.as_u8(), 7);
        assert_eq!(ElementKind::Group.as_u8(), 8);
        assert_eq!(ElementKind::Gradient.as_u8(), 9);
        assert_eq!(ElementKind::Filter.as_u8(), 10);
    }

    #[test]
    fn test_element_kind_name() {
        assert_eq!(ElementKind::Rect.name(), "rect");
        assert_eq!(ElementKind::Circle.name(), "circle");
        assert_eq!(ElementKind::Ellipse.name(), "ellipse");
        assert_eq!(ElementKind::Line.name(), "line");
        assert_eq!(ElementKind::Path.name(), "path");
        assert_eq!(ElementKind::Polygon.name(), "polygon");
        assert_eq!(ElementKind::Text.name(), "text");
        assert_eq!(ElementKind::Image.name(), "image");
        assert_eq!(ElementKind::Group.name(), "group");
        assert_eq!(ElementKind::Gradient.name(), "gradient");
        assert_eq!(ElementKind::Filter.name(), "filter");
    }

    #[test]
    fn test_element_kind_copy() {
        let k1 = ElementKind::Rect;
        let k2 = k1;
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_element_kind_eq() {
        assert_eq!(ElementKind::Rect, ElementKind::Rect);
        assert_ne!(ElementKind::Rect, ElementKind::Circle);
    }
}

