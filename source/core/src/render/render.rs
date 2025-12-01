//! Core rendering utilities with incremental diffing

use pyo3::prelude::*;
use pyo3::types::PyDict;
use super::diff::{self, DiffOp, IndexedScene};
use crate::scene::Scene;

/// Compute diff between two scenes for minimal updates
pub fn diff_scenes(old: &Scene, new: &Scene) -> Vec<DiffOp> {
    let result = diff::diff(old, new);
    if result.needs_full_redraw() {
        return vec![DiffOp::FullRedraw];
    }
    result.ops
}

/// Python-exposed patch data structure
#[derive(Debug, Clone)]
#[pyclass]
pub struct RenderPatch {
    #[pyo3(get)]
    pub op: String,
    #[pyo3(get)]
    pub id: Option<u64>,
    #[pyo3(get)]
    pub idx: Option<usize>,
    #[pyo3(get)]
    pub svg: Option<String>,
    #[pyo3(get)]
    pub from_idx: Option<usize>,
    #[pyo3(get)]
    pub to_idx: Option<usize>,
}

#[pymethods]
impl RenderPatch {
    fn attrs(&self, _py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        Ok(None)
    }

    fn __repr__(&self) -> String {
        format!("RenderPatch(op={:?}, id={:?}, idx={:?})", self.op, self.id, self.idx)
    }
}

impl From<DiffOp> for RenderPatch {
    fn from(op: DiffOp) -> Self {
        match op {
            DiffOp::None => Self { 
                op: "none".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None 
            },
            DiffOp::FullRedraw => Self { 
                op: "full_redraw".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None 
            },
            DiffOp::Add { id, idx, svg } => Self { 
                op: "add".into(), id: Some(id), idx: Some(idx), svg: Some(svg), from_idx: None, to_idx: None 
            },
            DiffOp::Remove { id, idx } => Self { 
                op: "remove".into(), id: Some(id), idx: Some(idx), svg: None, from_idx: None, to_idx: None 
            },
            DiffOp::Update { id, idx, attrs: _, svg } => Self { 
                op: "update".into(), id: Some(id), idx: Some(idx), svg, from_idx: None, to_idx: None 
            },
            DiffOp::Move { id, from, to } => Self { 
                op: "move".into(), id: Some(id), svg: None, idx: None, from_idx: Some(from), to_idx: Some(to) 
            },
            DiffOp::UpdateDefs { svg } => Self { 
                op: "update_defs".into(), id: None, idx: None, svg: Some(svg), from_idx: None, to_idx: None 
            },
        }
    }
}

/// Compute patches between old and new scenes (Python interface)
#[pyfunction]
pub fn compute_patches(old: &Scene, new: &Scene) -> Vec<RenderPatch> {
    diff_scenes(old, new).into_iter().map(RenderPatch::from).collect()
}

/// Check if scenes need full redraw (fast path for Python)
#[pyfunction]
pub fn needs_redraw(old: &Scene, new: &Scene) -> bool {
    !diff::diff(old, new).is_empty()
}

/// Index a scene for O(1) element lookups (exposed for caching)
#[pyfunction]
pub fn index_scene(scene: &Scene) -> usize {
    IndexedScene::from_scene(scene).len()
}
