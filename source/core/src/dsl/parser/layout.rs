//! Layout resolution and constraint solver for the iconoglott DSL
//!
//! Resolves percentage-based dimensions, auto-sizing, and constraint-based positioning.

#![allow(dead_code)] // Public API - methods used externally

use super::ast::*;
use std::collections::HashMap;

/// Resolved layout rectangle with absolute coordinates
#[derive(Clone, Debug, Default)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn center_x(&self) -> f64 { self.x + self.width / 2.0 }
    pub fn center_y(&self) -> f64 { self.y + self.height / 2.0 }
    pub fn right(&self) -> f64 { self.x + self.width }
    pub fn bottom(&self) -> f64 { self.y + self.height }
}

/// Layout context holding parent constraints and computed values
#[derive(Clone, Debug)]
pub struct LayoutContext {
    /// Parent bounds (for percentage resolution)
    pub parent: LayoutRect,
    /// Computed layout for elements by ID/index
    pub computed: HashMap<String, LayoutRect>,
    /// Default content size for auto elements
    pub default_size: (f64, f64),
}

impl Default for LayoutContext {
    fn default() -> Self {
        Self {
            parent: LayoutRect::new(0.0, 0.0, 100.0, 100.0),
            computed: HashMap::new(),
            default_size: (32.0, 32.0),
        }
    }
}

impl LayoutContext {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            parent: LayoutRect::new(0.0, 0.0, width, height),
            ..Default::default()
        }
    }
    
    /// Create child context with new parent bounds
    pub fn child(&self, bounds: LayoutRect) -> Self {
        Self {
            parent: bounds,
            computed: self.computed.clone(),
            default_size: self.default_size,
        }
    }
}

/// Layout solver that resolves dimensions and constraints
#[derive(Default)]
pub struct LayoutSolver {
    /// Iteration limit for constraint solving
    max_iterations: usize,
}

impl LayoutSolver {
    pub fn new() -> Self {
        Self { max_iterations: 10 }
    }
    
    /// Resolve layout for a shape and its children
    pub fn resolve(&self, shape: &AstShape, ctx: &mut LayoutContext) -> LayoutRect {
        match shape.kind.as_str() {
            "layout" => self.resolve_layout_container(shape, ctx),
            "group" => self.resolve_group(shape, ctx),
            _ => self.resolve_shape(shape, ctx),
        }
    }
    
    /// Resolve a layout container (stack/row)
    fn resolve_layout_container(&self, shape: &AstShape, ctx: &mut LayoutContext) -> LayoutRect {
        let layout = self.extract_layout_props(shape);
        
        // Resolve container bounds
        let mut bounds = self.resolve_container_bounds(shape, ctx);
        
        // Apply padding
        let (pt, pr, pb, pl) = layout.padding.unwrap_or_default();
        let inner = LayoutRect::new(
            bounds.x + pl.resolve(bounds.width).unwrap_or(0.0),
            bounds.y + pt.resolve(bounds.height).unwrap_or(0.0),
            bounds.width - pl.resolve(bounds.width).unwrap_or(0.0) - pr.resolve(bounds.width).unwrap_or(0.0),
            bounds.height - pt.resolve(bounds.height).unwrap_or(0.0) - pb.resolve(bounds.height).unwrap_or(0.0),
        );
        
        // Layout children
        let is_horizontal = layout.direction.as_deref() != Some("vertical");
        let gap = layout.gap.resolve(if is_horizontal { inner.width } else { inner.height }).unwrap_or(0.0);
        
        let child_rects = self.layout_children(&shape.children, &inner, is_horizontal, gap, layout.justify, layout.align, ctx);
        
        // If auto-sized, update bounds based on children
        if self.is_auto_sized(shape) {
            let (content_w, content_h) = self.compute_content_size(&child_rects, is_horizontal, gap);
            if self.get_width_dim(shape).is_auto() {
                bounds.width = content_w + pl.resolve(bounds.width).unwrap_or(0.0) + pr.resolve(bounds.width).unwrap_or(0.0);
            }
            if self.get_height_dim(shape).is_auto() {
                bounds.height = content_h + pt.resolve(bounds.height).unwrap_or(0.0) + pb.resolve(bounds.height).unwrap_or(0.0);
            }
        }
        
        bounds
    }
    
    /// Layout children with flex-like distribution
    fn layout_children(
        &self,
        children: &[AstShape],
        container: &LayoutRect,
        is_horizontal: bool,
        gap: f64,
        justify: JustifyContent,
        align: AlignItems,
        ctx: &mut LayoutContext,
    ) -> Vec<LayoutRect> {
        if children.is_empty() { return Vec::new(); }
        
        let mut child_ctx = ctx.child(container.clone());
        let mut child_rects: Vec<LayoutRect> = Vec::with_capacity(children.len());
        
        // First pass: compute natural sizes
        let mut total_main: f64 = 0.0;
        let mut max_cross: f64 = 0.0;
        
        for child in children {
            let rect = self.resolve(child, &mut child_ctx);
            let (main, cross) = if is_horizontal { (rect.width, rect.height) } else { (rect.height, rect.width) };
            total_main += main;
            max_cross = max_cross.max(cross);
            child_rects.push(rect);
        }
        
        // Add gaps
        let total_gaps = gap * (children.len().saturating_sub(1)) as f64;
        let main_size = if is_horizontal { container.width } else { container.height };
        let cross_size = if is_horizontal { container.height } else { container.width };
        let remaining = (main_size - total_main - total_gaps).max(0.0);
        
        // Compute starting position and spacing based on justify
        let (mut pos, extra_gap) = match justify {
            JustifyContent::Start => (0.0, 0.0),
            JustifyContent::End => (remaining, 0.0),
            JustifyContent::Center => (remaining / 2.0, 0.0),
            JustifyContent::SpaceBetween if children.len() > 1 => {
                (0.0, remaining / (children.len() - 1) as f64)
            }
            JustifyContent::SpaceAround if !children.is_empty() => {
                let space = remaining / children.len() as f64;
                (space / 2.0, space)
            }
            JustifyContent::SpaceEvenly if !children.is_empty() => {
                let space = remaining / (children.len() + 1) as f64;
                (space, space)
            }
            _ => (0.0, 0.0),
        };
        
        // Second pass: position children
        for (i, rect) in child_rects.iter_mut().enumerate() {
            let (main, cross) = if is_horizontal { (rect.width, rect.height) } else { (rect.height, rect.width) };
            
            // Cross-axis alignment
            let cross_pos = match align {
                AlignItems::Start => 0.0,
                AlignItems::End => cross_size - cross,
                AlignItems::Center => (cross_size - cross) / 2.0,
                AlignItems::Stretch => 0.0, // Will need to resize
                AlignItems::Baseline => 0.0, // Simplified
            };
            
            if is_horizontal {
                rect.x = container.x + pos;
                rect.y = container.y + cross_pos;
                if align == AlignItems::Stretch { rect.height = cross_size; }
            } else {
                rect.x = container.x + cross_pos;
                rect.y = container.y + pos;
                if align == AlignItems::Stretch { rect.width = cross_size; }
            }
            
            pos += main + gap + (if i < children.len() - 1 { extra_gap } else { 0.0 });
        }
        
        child_rects
    }
    
    /// Compute content size from child rects
    fn compute_content_size(&self, rects: &[LayoutRect], is_horizontal: bool, gap: f64) -> (f64, f64) {
        if rects.is_empty() { return (0.0, 0.0); }
        
        let (mut total_main, mut max_cross) = (0.0, 0.0_f64);
        for rect in rects {
            let (main, cross) = if is_horizontal { (rect.width, rect.height) } else { (rect.height, rect.width) };
            total_main += main;
            max_cross = max_cross.max(cross);
        }
        total_main += gap * (rects.len().saturating_sub(1)) as f64;
        
        if is_horizontal { (total_main, max_cross) } else { (max_cross, total_main) }
    }
    
    /// Resolve bounds for a container
    fn resolve_container_bounds(&self, shape: &AstShape, ctx: &LayoutContext) -> LayoutRect {
        let (x, y) = self.resolve_position(shape, ctx);
        let width = self.resolve_width(shape, ctx);
        let height = self.resolve_height(shape, ctx);
        LayoutRect::new(x, y, width, height)
    }
    
    /// Resolve position from props
    fn resolve_position(&self, shape: &AstShape, ctx: &LayoutContext) -> (f64, f64) {
        // Check for anchor constraints
        let x = self.resolve_x_position(shape, ctx);
        let y = self.resolve_y_position(shape, ctx);
        (x, y)
    }
    
    fn resolve_x_position(&self, shape: &AstShape, ctx: &LayoutContext) -> f64 {
        // Check for center constraint
        if matches!(shape.props.get("_center_x"), Some(PropValue::Num(n)) if *n > 0.0) {
            let width = self.resolve_width(shape, ctx);
            return ctx.parent.x + (ctx.parent.width - width) / 2.0;
        }
        
        // Check for anchor constraints
        if let Some(PropValue::Dim(offset)) = shape.props.get("_anchor_left") {
            return ctx.parent.x + offset.resolve(ctx.parent.width).unwrap_or(0.0);
        }
        if let Some(PropValue::Dim(offset)) = shape.props.get("_anchor_right") {
            let width = self.resolve_width(shape, ctx);
            return ctx.parent.right() - width - offset.resolve(ctx.parent.width).unwrap_or(0.0);
        }
        
        // Regular at position
        match shape.props.get("at") {
            Some(PropValue::Pair(x, _)) => *x,
            Some(PropValue::PercentPair(x, _)) => ctx.parent.x + ctx.parent.width * x / 100.0,
            _ => ctx.parent.x,
        }
    }
    
    fn resolve_y_position(&self, shape: &AstShape, ctx: &LayoutContext) -> f64 {
        // Check for center constraint
        if matches!(shape.props.get("_center_y"), Some(PropValue::Num(n)) if *n > 0.0) {
            let height = self.resolve_height(shape, ctx);
            return ctx.parent.y + (ctx.parent.height - height) / 2.0;
        }
        
        // Check for anchor constraints
        if let Some(PropValue::Dim(offset)) = shape.props.get("_anchor_top") {
            return ctx.parent.y + offset.resolve(ctx.parent.height).unwrap_or(0.0);
        }
        if let Some(PropValue::Dim(offset)) = shape.props.get("_anchor_bottom") {
            let height = self.resolve_height(shape, ctx);
            return ctx.parent.bottom() - height - offset.resolve(ctx.parent.height).unwrap_or(0.0);
        }
        
        // Regular at position
        match shape.props.get("at") {
            Some(PropValue::Pair(_, y)) => *y,
            Some(PropValue::PercentPair(_, y)) => ctx.parent.y + ctx.parent.height * y / 100.0,
            _ => ctx.parent.y,
        }
    }
    
    fn resolve_width(&self, shape: &AstShape, ctx: &LayoutContext) -> f64 {
        self.get_width_dim(shape).resolve(ctx.parent.width).unwrap_or(ctx.default_size.0)
    }
    
    fn resolve_height(&self, shape: &AstShape, ctx: &LayoutContext) -> f64 {
        self.get_height_dim(shape).resolve(ctx.parent.height).unwrap_or(ctx.default_size.1)
    }
    
    fn get_width_dim(&self, shape: &AstShape) -> Dimension {
        if let Some(PropValue::Dim(d)) = shape.props.get("width") {
            return d.clone();
        }
        if let Some(PropValue::DimPair(dp)) = shape.props.get("size") {
            return dp.width.clone();
        }
        if let Some(PropValue::Pair(w, _)) = shape.props.get("size") {
            return Dimension::Px(*w);
        }
        Dimension::Auto
    }
    
    fn get_height_dim(&self, shape: &AstShape) -> Dimension {
        if let Some(PropValue::Dim(d)) = shape.props.get("height") {
            return d.clone();
        }
        if let Some(PropValue::DimPair(dp)) = shape.props.get("size") {
            return dp.height.clone();
        }
        if let Some(PropValue::Pair(_, h)) = shape.props.get("size") {
            return Dimension::Px(*h);
        }
        Dimension::Auto
    }
    
    fn is_auto_sized(&self, shape: &AstShape) -> bool {
        self.get_width_dim(shape).is_auto() || self.get_height_dim(shape).is_auto()
    }
    
    /// Extract LayoutProps from shape
    fn extract_layout_props(&self, shape: &AstShape) -> LayoutProps {
        if let Some(PropValue::Layout(props)) = shape.props.get("_layout") {
            return (**props).clone();
        }
        
        // Build from individual props
        let mut layout = LayoutProps::default();
        
        if let Some(PropValue::Str(d)) = shape.props.get("direction") {
            layout.direction = Some(d.clone());
        }
        if let Some(PropValue::Str(j)) = shape.props.get("justify") {
            layout.justify = match j.as_str() {
                "start" => JustifyContent::Start,
                "end" => JustifyContent::End,
                "center" => JustifyContent::Center,
                "spacebetween" | "space-between" => JustifyContent::SpaceBetween,
                "spacearound" | "space-around" => JustifyContent::SpaceAround,
                "spaceevenly" | "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::Start,
            };
        }
        if let Some(PropValue::Str(a)) = shape.props.get("align") {
            layout.align = match a.as_str() {
                "start" => AlignItems::Start,
                "end" => AlignItems::End,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::Start,
            };
        }
        if let Some(PropValue::Num(g)) = shape.props.get("gap") {
            layout.gap = Dimension::Px(*g);
        }
        if let Some(PropValue::Dim(d)) = shape.props.get("gap") {
            layout.gap = d.clone();
        }
        if matches!(shape.props.get("wrap"), Some(PropValue::Num(n)) if *n > 0.0) {
            layout.wrap = true;
        }
        
        layout
    }
    
    /// Resolve a simple group
    fn resolve_group(&self, shape: &AstShape, ctx: &mut LayoutContext) -> LayoutRect {
        let bounds = self.resolve_container_bounds(shape, ctx);
        let mut child_ctx = ctx.child(bounds.clone());
        
        for child in &shape.children {
            self.resolve(child, &mut child_ctx);
        }
        
        bounds
    }
    
    /// Resolve a simple shape
    fn resolve_shape(&self, shape: &AstShape, ctx: &LayoutContext) -> LayoutRect {
        let (x, y) = self.resolve_position(shape, ctx);
        let width = self.resolve_width(shape, ctx);
        let height = self.resolve_height(shape, ctx);
        LayoutRect::new(x, y, width, height)
    }
}

/// Convenience function to resolve layout for an AST
pub fn resolve_layout(ast: &AstNode, canvas_width: f64, canvas_height: f64) -> HashMap<String, LayoutRect> {
    let mut ctx = LayoutContext::new(canvas_width, canvas_height);
    let solver = LayoutSolver::new();
    
    if let AstNode::Scene(children) = ast {
        for (i, child) in children.iter().enumerate() {
            if let AstNode::Shape(shape) = child {
                let rect = solver.resolve(shape, &mut ctx);
                ctx.computed.insert(format!("shape_{}", i), rect);
            }
        }
    }
    
    ctx.computed
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_layout_shape(direction: &str, justify: JustifyContent, align: AlignItems) -> AstShape {
        let mut shape = AstShape::new("layout");
        shape.props.insert("direction".into(), PropValue::Str(direction.into()));
        shape.props.insert("justify".into(), PropValue::Str(format!("{:?}", justify).to_lowercase()));
        shape.props.insert("align".into(), PropValue::Str(format!("{:?}", align).to_lowercase()));
        shape
    }
    
    fn make_child(width: f64, height: f64) -> AstShape {
        let mut shape = AstShape::new("rect");
        shape.props.insert("size".into(), PropValue::Pair(width, height));
        shape
    }
    
    #[test]
    fn test_resolve_percentage_width() {
        let mut shape = AstShape::new("rect");
        shape.props.insert("width".into(), PropValue::Dim(Dimension::Percent(50.0)));
        shape.props.insert("height".into(), PropValue::Dim(Dimension::Px(30.0)));
        
        let ctx = LayoutContext::new(200.0, 100.0);
        let solver = LayoutSolver::new();
        let rect = solver.resolve_shape(&shape, &ctx);
        
        assert!((rect.width - 100.0).abs() < 0.001, "50% of 200 should be 100");
        assert!((rect.height - 30.0).abs() < 0.001);
    }
    
    #[test]
    fn test_layout_justify_center() {
        let mut layout = make_layout_shape("horizontal", JustifyContent::Center, AlignItems::Start);
        layout.props.insert("size".into(), PropValue::Pair(200.0, 100.0));
        layout.children = vec![make_child(40.0, 20.0), make_child(40.0, 20.0)];
        
        let mut ctx = LayoutContext::new(200.0, 100.0);
        let solver = LayoutSolver::new();
        solver.resolve(&layout, &mut ctx);
        
        // Children should be centered: (200 - 80) / 2 = 60 offset
    }
    
    #[test]
    fn test_layout_align_center() {
        let mut layout = make_layout_shape("horizontal", JustifyContent::Start, AlignItems::Center);
        layout.props.insert("size".into(), PropValue::Pair(200.0, 100.0));
        layout.children = vec![make_child(40.0, 20.0)];
        
        let mut ctx = LayoutContext::new(200.0, 100.0);
        let solver = LayoutSolver::new();
        solver.resolve(&layout, &mut ctx);
        
        // Child should be vertically centered: (100 - 20) / 2 = 40 y offset
    }
    
    #[test]
    fn test_anchor_constraint() {
        let mut shape = AstShape::new("rect");
        shape.props.insert("_anchor_right".into(), PropValue::Dim(Dimension::Px(10.0)));
        shape.props.insert("width".into(), PropValue::Dim(Dimension::Px(50.0)));
        shape.props.insert("height".into(), PropValue::Dim(Dimension::Px(30.0)));
        
        let ctx = LayoutContext::new(200.0, 100.0);
        let solver = LayoutSolver::new();
        let rect = solver.resolve_shape(&shape, &ctx);
        
        // Should be 10px from right edge: x = 200 - 50 - 10 = 140
        assert!((rect.x - 140.0).abs() < 0.001, "x should be 140, got {}", rect.x);
    }
    
    #[test]
    fn test_center_constraint() {
        let mut shape = AstShape::new("rect");
        shape.props.insert("_center_x".into(), PropValue::Num(1.0));
        shape.props.insert("_center_y".into(), PropValue::Num(1.0));
        shape.props.insert("width".into(), PropValue::Dim(Dimension::Px(50.0)));
        shape.props.insert("height".into(), PropValue::Dim(Dimension::Px(30.0)));
        
        let ctx = LayoutContext::new(200.0, 100.0);
        let solver = LayoutSolver::new();
        let rect = solver.resolve_shape(&shape, &ctx);
        
        // Should be centered: x = (200 - 50) / 2 = 75, y = (100 - 30) / 2 = 35
        assert!((rect.x - 75.0).abs() < 0.001, "x should be 75, got {}", rect.x);
        assert!((rect.y - 35.0).abs() < 0.001, "y should be 35, got {}", rect.y);
    }
}

