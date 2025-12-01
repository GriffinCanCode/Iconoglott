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

    #[test]
    fn test_element_id_stability() {
        let id1 = ElementId::new(0, ElementKind::Rect.as_u8());
        let id2 = ElementId::new(0, ElementKind::Rect.as_u8());
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_element_id_uniqueness() {
        let id1 = ElementId::new(0, ElementKind::Rect.as_u8());
        let id2 = ElementId::new(1, ElementKind::Rect.as_u8());
        let id3 = ElementId::new(0, ElementKind::Circle.as_u8());
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_idgen_monotonic() {
        let gen = IdGen::default();
        assert_eq!(gen.next(), 0);
        assert_eq!(gen.next(), 1);
        assert_eq!(gen.next(), 2);
    }

    #[test]
    fn test_content_hash_determinism() {
        let h1 = ContentHash::from_svg("<rect x=\"0\"/>");
        let h2 = ContentHash::from_svg("<rect x=\"0\"/>");
        assert_eq!(h1, h2);
    }
}

