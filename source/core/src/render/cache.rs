//! SVG fragment memoization cache
//!
//! Content-addressed cache for rendered SVG fragments.
//! Avoids re-rendering unchanged elements during incremental updates.

use std::collections::HashMap;
use crate::hash::ContentHash;

/// Cache entry with SVG and hit count for LRU eviction
#[derive(Debug, Clone)]
struct CacheEntry {
    svg: String,
    hits: u32,
}

/// Memoization cache for rendered SVG fragments
#[derive(Debug)]
pub struct RenderCache {
    entries: HashMap<ContentHash, CacheEntry>,
    max_size: usize,
}

impl Default for RenderCache {
    fn default() -> Self { Self::new(1024) }
}

impl RenderCache {
    pub fn new(max_size: usize) -> Self {
        Self { entries: HashMap::with_capacity(max_size), max_size }
    }

    /// Get cached SVG for content hash
    pub fn get(&mut self, hash: &ContentHash) -> Option<&str> {
        self.entries.get_mut(hash).map(|e| {
            e.hits = e.hits.saturating_add(1);
            e.svg.as_str()
        })
    }

    /// Store SVG with content hash
    pub fn insert(&mut self, hash: ContentHash, svg: String) {
        if self.entries.len() >= self.max_size {
            self.evict_lru();
        }
        self.entries.insert(hash, CacheEntry { svg, hits: 1 });
    }

    /// Get or compute SVG fragment
    pub fn get_or_insert<F>(&mut self, hash: ContentHash, f: F) -> &str 
    where F: FnOnce() -> String {
        if !self.entries.contains_key(&hash) {
            let svg = f();
            self.insert(hash, svg);
        }
        self.get(&hash).unwrap()
    }

    /// Evict lowest-hit entry
    fn evict_lru(&mut self) {
        if let Some(&hash) = self.entries.iter()
            .min_by_key(|(_, e)| e.hits)
            .map(|(h, _)| h) {
            self.entries.remove(&hash);
        }
    }

    /// Clear all cached fragments
    pub fn clear(&mut self) { self.entries.clear(); }

    /// Number of cached entries
    pub fn len(&self) -> usize { self.entries.len() }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_hits: u32 = self.entries.values().map(|e| e.hits).sum();
        let total_size: usize = self.entries.values().map(|e| e.svg.len()).sum();
        CacheStats {
            entries: self.entries.len(),
            total_hits,
            total_bytes: total_size,
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_hits: u32,
    pub total_bytes: usize,
}

/// Cached scene renderer with fragment memoization
pub struct CachedRenderer {
    cache: RenderCache,
}

impl Default for CachedRenderer {
    fn default() -> Self { Self::new() }
}

impl CachedRenderer {
    pub fn new() -> Self { Self { cache: RenderCache::default() } }

    pub fn with_capacity(size: usize) -> Self {
        Self { cache: RenderCache::new(size) }
    }

    /// Get SVG fragment, using cache if available
    pub fn render_element<F>(&mut self, hash: ContentHash, render: F) -> &str 
    where F: FnOnce() -> String {
        self.cache.get_or_insert(hash, render)
    }

    /// Clear the fragment cache
    pub fn invalidate(&mut self) { self.cache.clear(); }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats { self.cache.stats() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // RenderCache tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cache_new() {
        let cache = RenderCache::new(100);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_default() {
        let cache = RenderCache::default();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_insert_get() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("<rect/>");
        cache.insert(hash, "<rect/>".into());
        assert_eq!(cache.get(&hash), Some("<rect/>"));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("<nonexistent/>");
        assert_eq!(cache.get(&hash), None);
    }

    #[test]
    fn test_cache_overwrite() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("<test/>");
        cache.insert(hash, "<old/>".into());
        cache.insert(hash, "<new/>".into());
        assert_eq!(cache.get(&hash), Some("<new/>"));
    }

    #[test]
    fn test_cache_multiple_entries() {
        let mut cache = RenderCache::new(10);
        let h1 = ContentHash::from_svg("<a/>");
        let h2 = ContentHash::from_svg("<b/>");
        let h3 = ContentHash::from_svg("<c/>");
        cache.insert(h1, "<a/>".into());
        cache.insert(h2, "<b/>".into());
        cache.insert(h3, "<c/>".into());
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&h1), Some("<a/>"));
        assert_eq!(cache.get(&h2), Some("<b/>"));
        assert_eq!(cache.get(&h3), Some("<c/>"));
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = RenderCache::new(2);
        let h1 = ContentHash::from_svg("<rect/>");
        let h2 = ContentHash::from_svg("<circle/>");
        let h3 = ContentHash::from_svg("<ellipse/>");
        
        cache.insert(h1, "<rect/>".into());
        cache.insert(h2, "<circle/>".into());
        // Access h2 to increase hits
        cache.get(&h2);
        // Insert h3, should evict h1 (fewer hits)
        cache.insert(h3, "<ellipse/>".into());
        
        assert_eq!(cache.len(), 2);
        assert!(cache.get(&h1).is_none());
        assert!(cache.get(&h2).is_some());
    }

    #[test]
    fn test_cache_eviction_lru_order() {
        let mut cache = RenderCache::new(3);
        let h1 = ContentHash::from_svg("<1/>");
        let h2 = ContentHash::from_svg("<2/>");
        let h3 = ContentHash::from_svg("<3/>");
        let h4 = ContentHash::from_svg("<4/>");

        cache.insert(h1, "<1/>".into());
        cache.insert(h2, "<2/>".into());
        cache.insert(h3, "<3/>".into());

        // Access h1 and h3 multiple times
        cache.get(&h1);
        cache.get(&h1);
        cache.get(&h3);
        cache.get(&h3);
        cache.get(&h3);

        // h2 has lowest hits, should be evicted
        cache.insert(h4, "<4/>".into());
        assert!(cache.get(&h2).is_none());
        assert!(cache.get(&h1).is_some());
        assert!(cache.get(&h3).is_some());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = RenderCache::new(10);
        cache.insert(ContentHash::from_svg("<x/>"), "<x/>".into());
        cache.insert(ContentHash::from_svg("<y/>"), "<y/>".into());
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_get_or_insert() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("<path/>");
        let mut computed = false;
        
        let svg = cache.get_or_insert(hash, || {
            computed = true;
            "<path/>".into()
        });
        assert!(computed);
        assert_eq!(svg, "<path/>");

        // Second call should use cache
        computed = false;
        let svg2 = cache.get_or_insert(hash, || {
            computed = true;
            "<path/>".into()
        });
        assert!(!computed);
        assert_eq!(svg2, "<path/>");
    }

    #[test]
    fn test_get_or_insert_expensive_compute() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("<complex/>");
        let mut call_count = 0;

        for _ in 0..5 {
            let _ = cache.get_or_insert(hash, || {
                call_count += 1;
                "<computed/>".into()
            });
        }

        assert_eq!(call_count, 1); // Only computed once
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = RenderCache::new(10);
        let h1 = ContentHash::from_svg("<test1/>");
        let h2 = ContentHash::from_svg("<test2/>");
        
        cache.insert(h1, "<test1/>".into());
        cache.insert(h2, "<test2test2/>".into());
        
        // Access to bump hits
        cache.get(&h1);
        cache.get(&h1);
        cache.get(&h2);
        
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert!(stats.total_hits >= 3);
        assert!(stats.total_bytes > 0);
    }

    #[test]
    fn test_cache_stats_empty() {
        let cache = RenderCache::new(10);
        let stats = cache.stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.total_bytes, 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CachedRenderer tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cached_renderer_new() {
        let renderer = CachedRenderer::new();
        assert_eq!(renderer.stats().entries, 0);
    }

    #[test]
    fn test_cached_renderer_with_capacity() {
        let renderer = CachedRenderer::with_capacity(100);
        assert_eq!(renderer.stats().entries, 0);
    }

    #[test]
    fn test_cached_renderer_render_element() {
        let mut renderer = CachedRenderer::new();
        let hash = ContentHash::from_svg("<path d=\"M 0 0\"/>");
        
        let mut computed = false;
        let svg = renderer.render_element(hash, || {
            computed = true;
            "<path d=\"M 0 0\"/>".into()
        });
        assert!(computed);
        assert_eq!(svg, "<path d=\"M 0 0\"/>");

        computed = false;
        let svg2 = renderer.render_element(hash, || {
            computed = true;
            "<path d=\"M 0 0\"/>".into()
        });
        assert!(!computed);
        assert_eq!(svg2, "<path d=\"M 0 0\"/>");
    }

    #[test]
    fn test_cached_renderer_invalidate() {
        let mut renderer = CachedRenderer::new();
        let hash = ContentHash::from_svg("<test/>");
        renderer.render_element(hash, || "<test/>".into());
        assert_eq!(renderer.stats().entries, 1);
        renderer.invalidate();
        assert_eq!(renderer.stats().entries, 0);
    }

    #[test]
    fn test_cached_renderer_stats() {
        let mut renderer = CachedRenderer::new();
        let hash = ContentHash::from_svg("<stat/>");
        renderer.render_element(hash, || "<stat/>".into());
        let stats = renderer.stats();
        assert_eq!(stats.entries, 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Edge cases
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cache_empty_string() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("");
        cache.insert(hash, "".into());
        assert_eq!(cache.get(&hash), Some(""));
    }

    #[test]
    fn test_cache_large_entry() {
        let mut cache = RenderCache::new(10);
        let large_svg = "x".repeat(10000);
        let hash = ContentHash::from_svg(&large_svg);
        cache.insert(hash, large_svg.clone());
        assert_eq!(cache.get(&hash), Some(large_svg.as_str()));
    }

    #[test]
    fn test_cache_size_one() {
        let mut cache = RenderCache::new(1);
        let h1 = ContentHash::from_svg("<a/>");
        let h2 = ContentHash::from_svg("<b/>");
        
        cache.insert(h1, "<a/>".into());
        cache.insert(h2, "<b/>".into());
        
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&h2).is_some());
    }
}

