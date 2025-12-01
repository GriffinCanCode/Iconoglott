//! Core rendering utilities with incremental diffing

use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::diff::{self, Patch};
use crate::scene::Scene;

/// Compute diff between two scenes for minimal updates
pub fn diff_scenes(old: &Scene, new: &Scene) -> Vec<Patch> {
    let result = diff::diff(old, new);
    if result.needs_full_redraw() {
        return vec![Patch::FullRedraw];
    }
    result.patches
}

/// Python-exposed patch data structure
#[derive(Debug, Clone)]
#[pyclass]
pub struct RenderPatch {
    #[pyo3(get)]
    pub op: String,
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
        // Attrs are embedded in the dict when needed
        Ok(None)
    }

    fn __repr__(&self) -> String {
        format!("RenderPatch(op={:?}, idx={:?})", self.op, self.idx)
    }
}

impl From<Patch> for RenderPatch {
    fn from(p: Patch) -> Self {
        match p {
            Patch::None => Self { op: "none".into(), idx: None, svg: None, from_idx: None, to_idx: None },
            Patch::FullRedraw => Self { op: "full_redraw".into(), idx: None, svg: None, from_idx: None, to_idx: None },
            Patch::Add { idx, svg } => Self { op: "add".into(), idx: Some(idx), svg: Some(svg), from_idx: None, to_idx: None },
            Patch::Remove { idx } => Self { op: "remove".into(), idx: Some(idx), svg: None, from_idx: None, to_idx: None },
            Patch::Update { idx, attrs: _ } => Self { op: "update".into(), idx: Some(idx), svg: None, from_idx: None, to_idx: None },
            Patch::Move { from, to } => Self { op: "move".into(), idx: None, svg: None, from_idx: Some(from), to_idx: Some(to) },
            Patch::Reorder { order: _ } => Self { op: "reorder".into(), idx: None, svg: None, from_idx: None, to_idx: None },
            Patch::UpdateDefs { svg } => Self { op: "update_defs".into(), idx: None, svg: Some(svg), from_idx: None, to_idx: None },
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
