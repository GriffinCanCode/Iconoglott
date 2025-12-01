//! Rendering, diffing, and caching

mod cache;
mod diff;
mod render;

pub use cache::{CacheStats, CachedRenderer, RenderCache};
pub use diff::{DiffOp, DiffResult, IndexedElement, IndexedScene, Patch, diff, element_kind};
pub use render::{RenderPatch, compute_patches, diff_scenes, index_scene, needs_redraw};

