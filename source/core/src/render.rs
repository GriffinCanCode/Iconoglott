//! Core rendering utilities

use crate::scene::Scene;

/// Compute diff between two scenes for minimal updates
pub fn diff_scenes(old: &Scene, new: &Scene) -> Vec<RenderOp> {
    let mut ops = Vec::new();
    
    // Canvas change requires full redraw
    if old.width != new.width || old.height != new.height || old.background != new.background {
        ops.push(RenderOp::FullRedraw);
        return ops;
    }
    
    // For now, simple full redraw on any change
    // Future: element-level diffing with stable IDs
    if old.to_json() != new.to_json() {
        ops.push(RenderOp::FullRedraw);
    }
    
    ops
}

/// Render operation for incremental updates
#[derive(Debug, Clone)]
pub enum RenderOp {
    FullRedraw,
    AddElement(usize),
    RemoveElement(usize),
    UpdateElement(usize),
}

