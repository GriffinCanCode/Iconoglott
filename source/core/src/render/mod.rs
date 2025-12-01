//! Rendering, diffing, caching, and undo/redo

mod cache;
mod command;
mod diff;
mod render;

pub use cache::{CacheStats, CachedRenderer, RenderCache};
pub use command::{CommandHistory, SceneCommand};
pub use diff::{DiffOp, DiffResult, IndexedElement, IndexedScene, Patch, diff, element_kind};
pub use render::{RenderPatch, compute_patches, diff_scenes, index_scene, needs_redraw};
