//! Core rendering utilities with incremental diffing

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyDict;

use super::diff::{self, DiffOp, IndexedScene};
use crate::scene::Scene;

/// Compute diff between two scenes for minimal updates
pub fn diff_scenes(old: &Scene, new: &Scene) -> Vec<DiffOp> {
    let result = diff::diff(old, new);
    if result.needs_full_redraw() { vec![DiffOp::FullRedraw] } else { result.ops }
}

/// Patch data structure for incremental updates
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyclass(get_all))]
pub struct RenderPatch {
    pub op: String, pub id: Option<u64>, pub idx: Option<usize>,
    pub svg: Option<String>, pub from_idx: Option<usize>, pub to_idx: Option<usize>,
}

#[cfg(feature = "python")]
#[pymethods]
impl RenderPatch {
    fn attrs(&self, _py: Python<'_>) -> PyResult<Option<Py<PyDict>>> { Ok(None) }
    fn __repr__(&self) -> String { format!("RenderPatch(op={:?}, id={:?}, idx={:?})", self.op, self.id, self.idx) }
}

impl From<DiffOp> for RenderPatch {
    fn from(op: DiffOp) -> Self {
        match op {
            DiffOp::None => Self { op: "none".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None },
            DiffOp::FullRedraw => Self { op: "full_redraw".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None },
            DiffOp::Add { id, idx, svg } => Self { op: "add".into(), id: Some(id), idx: Some(idx), svg: Some(svg), from_idx: None, to_idx: None },
            DiffOp::Remove { id, idx } => Self { op: "remove".into(), id: Some(id), idx: Some(idx), svg: None, from_idx: None, to_idx: None },
            DiffOp::Update { id, idx, svg, .. } => Self { op: "update".into(), id: Some(id), idx: Some(idx), svg, from_idx: None, to_idx: None },
            DiffOp::Move { id, from, to } => Self { op: "move".into(), id: Some(id), svg: None, idx: None, from_idx: Some(from), to_idx: Some(to) },
            DiffOp::UpdateDefs { svg } => Self { op: "update_defs".into(), id: None, idx: None, svg: Some(svg), from_idx: None, to_idx: None },
        }
    }
}

#[cfg_attr(feature = "python", pyfunction)]
pub fn compute_patches(old: &Scene, new: &Scene) -> Vec<RenderPatch> {
    diff_scenes(old, new).into_iter().map(RenderPatch::from).collect()
}

#[cfg_attr(feature = "python", pyfunction)]
pub fn needs_redraw(old: &Scene, new: &Scene) -> bool { !diff::diff(old, new).is_empty() }

#[cfg_attr(feature = "python", pyfunction)]
pub fn index_scene(scene: &Scene) -> usize { IndexedScene::from_scene(scene).len() }
