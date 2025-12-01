//! SVG fragment memoization cache
//!
//! Content-addressed cache for rendered SVG fragments.
//! Avoids re-rendering unchanged elements during incremental updates.

use std::collections::HashMap;
use crate::id::ContentHash;

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

    #[test]
    fn test_cache_insert_get() {
        let mut cache = RenderCache::new(10);
        let hash = ContentHash::from_svg("<rect/>");
        cache.insert(hash, "<rect/>".into());
        assert_eq!(cache.get(&hash), Some("<rect/>"));
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
}

