//! Path boolean operations using sweep-line algorithm
//!
//! Implements Bentley-Ottmann sweep for segment intersection detection
//! and polygon boolean operations (union, intersection, difference, XOR).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Floating point comparison tolerance
const EPS: f64 = 1e-10;

/// 2D point with f64 precision for robust geometric computations
#[derive(Clone, Copy, Debug, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self { Self { x, y } }
    pub fn dot(self, o: Point) -> f64 { self.x * o.x + self.y * o.y }
    pub fn cross(self, o: Point) -> f64 { self.x * o.y - self.y * o.x }
    pub fn sub(self, o: Point) -> Point { Point::new(self.x - o.x, self.y - o.y) }
    pub fn add(self, o: Point) -> Point { Point::new(self.x + o.x, self.y + o.y) }
    pub fn scale(self, s: f64) -> Point { Point::new(self.x * s, self.y * s) }
    pub fn len2(self) -> f64 { self.x * self.x + self.y * self.y }
    pub fn len(self) -> f64 { self.len2().sqrt() }
    
    fn cmp_xy(&self, o: &Point) -> Ordering {
        match fcmp(self.x, o.x) {
            Ordering::Equal => fcmp(self.y, o.y),
            ord => ord,
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, o: &Self) -> bool { (self.x - o.x).abs() < EPS && (self.y - o.y).abs() < EPS }
}

impl Eq for Point {}

impl PartialOrd for Point {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp_xy(o)) }
}

impl Ord for Point {
    fn cmp(&self, o: &Self) -> Ordering { self.cmp_xy(o) }
}

/// Line segment with origin polygon index and edge index
#[derive(Clone, Debug)]
pub struct Segment {
    pub p0: Point,
    pub p1: Point,
    pub poly_idx: usize,
    pub edge_idx: usize,
}

impl Segment {
    pub fn new(p0: Point, p1: Point, poly_idx: usize, edge_idx: usize) -> Self {
        // Ensure p0 <= p1 in sweep order (left to right, then bottom to top)
        if p0.cmp_xy(&p1) == Ordering::Greater {
            Self { p0: p1, p1: p0, poly_idx, edge_idx }
        } else {
            Self { p0, p1, poly_idx, edge_idx }
        }
    }
    
    /// Evaluate y-coordinate at given x (assumes segment spans x)
    pub fn y_at(&self, x: f64) -> f64 {
        if (self.p1.x - self.p0.x).abs() < EPS { return self.p0.y.min(self.p1.y); }
        let t = (x - self.p0.x) / (self.p1.x - self.p0.x);
        self.p0.y + t * (self.p1.y - self.p0.y)
    }
    
    /// Check if segment is nearly vertical
    pub fn is_vertical(&self) -> bool { (self.p1.x - self.p0.x).abs() < EPS }
}

/// Sweep-line event types
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind { Start, End, Intersection }

/// Event in the sweep-line algorithm
#[derive(Clone, Debug)]
pub struct Event {
    pub point: Point,
    pub kind: EventKind,
    pub seg_idx: usize,
    pub other_idx: Option<usize>, // For intersection events
}

impl Event {
    fn start(point: Point, seg_idx: usize) -> Self {
        Self { point, kind: EventKind::Start, seg_idx, other_idx: None }
    }
    fn end(point: Point, seg_idx: usize) -> Self {
        Self { point, kind: EventKind::End, seg_idx, other_idx: None }
    }
    fn intersection(point: Point, seg_idx: usize, other_idx: usize) -> Self {
        Self { point, kind: EventKind::Intersection, seg_idx, other_idx: Some(other_idx) }
    }
}

impl PartialEq for Event {
    fn eq(&self, o: &Self) -> bool { self.point == o.point && self.kind == o.kind }
}
impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) }
}

impl Ord for Event {
    fn cmp(&self, o: &Self) -> Ordering {
        // Min-heap: reverse ordering (smaller x first, then smaller y)
        match o.point.cmp_xy(&self.point) {
            Ordering::Equal => {
                // Process starts before intersections before ends
                let kind_ord = |k: &EventKind| match k {
                    EventKind::Start => 0,
                    EventKind::Intersection => 1,
                    EventKind::End => 2,
                };
                kind_ord(&o.kind).cmp(&kind_ord(&self.kind))
            }
            ord => ord,
        }
    }
}

/// Boolean operation type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoolOp {
    Union,
    Intersection,
    Difference,  // A - B
    Xor,
}

/// Simple polygon represented as a list of vertices
#[derive(Clone, Debug, Default)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub is_hole: bool,
}

impl Polygon {
    pub fn new(vertices: Vec<Point>) -> Self { Self { vertices, is_hole: false } }
    
    pub fn with_hole(vertices: Vec<Point>, is_hole: bool) -> Self {
        Self { vertices, is_hole }
    }
    
    /// Compute signed area (positive = CCW, negative = CW)
    pub fn signed_area(&self) -> f64 {
        if self.vertices.len() < 3 { return 0.0; }
        let mut area = 0.0;
        let n = self.vertices.len();
        for i in 0..n {
            let j = (i + 1) % n;
            area += self.vertices[i].cross(self.vertices[j]);
        }
        area * 0.5
    }
    
    /// Check if polygon is counter-clockwise
    pub fn is_ccw(&self) -> bool { self.signed_area() > 0.0 }
    
    /// Reverse vertex order
    pub fn reverse(&mut self) { self.vertices.reverse(); }
    
    /// Ensure CCW winding for outer contours, CW for holes
    pub fn normalize(&mut self) {
        if self.is_hole && self.is_ccw() { self.reverse(); }
        else if !self.is_hole && !self.is_ccw() { self.reverse(); }
    }
    
    /// Generate segments from polygon edges
    pub fn to_segments(&self, poly_idx: usize) -> Vec<Segment> {
        let n = self.vertices.len();
        if n < 2 { return vec![]; }
        (0..n).map(|i| {
            let j = (i + 1) % n;
            Segment::new(self.vertices[i], self.vertices[j], poly_idx, i)
        }).collect()
    }
    
    /// Point-in-polygon test using ray casting
    pub fn contains(&self, p: Point) -> bool {
        let n = self.vertices.len();
        if n < 3 { return false; }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let vi = self.vertices[i];
            let vj = self.vertices[j];
            if ((vi.y > p.y) != (vj.y > p.y)) &&
               (p.x < (vj.x - vi.x) * (p.y - vi.y) / (vj.y - vi.y) + vi.x) {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}

/// Compute intersection point of two segments (if any)
pub fn segment_intersection(s1: &Segment, s2: &Segment) -> Option<Point> {
    let d1 = s1.p1.sub(s1.p0);
    let d2 = s2.p1.sub(s2.p0);
    let cross = d1.cross(d2);
    
    if cross.abs() < EPS { return None; } // Parallel or collinear
    
    let diff = s2.p0.sub(s1.p0);
    let t1 = diff.cross(d2) / cross;
    let t2 = diff.cross(d1) / cross;
    
    // Check if intersection is within both segments (exclusive of endpoints)
    if t1 > EPS && t1 < 1.0 - EPS && t2 > EPS && t2 < 1.0 - EPS {
        Some(s1.p0.add(d1.scale(t1)))
    } else {
        None
    }
}

/// Compare two floats with epsilon tolerance
fn fcmp(a: f64, b: f64) -> Ordering {
    if (a - b).abs() < EPS { Ordering::Equal }
    else if a < b { Ordering::Less }
    else { Ordering::Greater }
}

/// Bentley-Ottmann sweep line algorithm for finding all segment intersections
pub struct SweepLine {
    segments: Vec<Segment>,
    events: BinaryHeap<Event>,
    active: Vec<usize>,           // Indices of active segments
    sweep_x: f64,                 // Current sweep line position
    intersections: Vec<(usize, usize, Point)>, // (seg1, seg2, point)
}

impl SweepLine {
    pub fn new(segments: Vec<Segment>) -> Self {
        let mut events = BinaryHeap::new();
        
        // Add start and end events for each segment
        for (i, seg) in segments.iter().enumerate() {
            events.push(Event::start(seg.p0, i));
            events.push(Event::end(seg.p1, i));
        }
        
        Self {
            segments,
            events,
            active: Vec::new(),
            sweep_x: f64::NEG_INFINITY,
            intersections: Vec::new(),
        }
    }
    
    /// Find all intersections using sweep line
    pub fn find_intersections(mut self) -> Vec<(usize, usize, Point)> {
        while let Some(event) = self.events.pop() {
            self.sweep_x = event.point.x;
            
            match event.kind {
                EventKind::Start => self.handle_start(event.seg_idx),
                EventKind::End => self.handle_end(event.seg_idx),
                EventKind::Intersection => {
                    if let Some(other) = event.other_idx {
                        self.handle_intersection(event.seg_idx, other, event.point);
                    }
                }
            }
        }
        
        self.intersections
    }
    
    fn handle_start(&mut self, seg_idx: usize) {
        // Find position to insert in active list (sorted by y at sweep_x)
        let y = self.segments[seg_idx].y_at(self.sweep_x);
        let pos = self.active.iter().position(|&i| {
            self.segments[i].y_at(self.sweep_x) > y
        }).unwrap_or(self.active.len());
        
        self.active.insert(pos, seg_idx);
        
        // Check for intersections with neighbors
        if pos > 0 { self.check_intersection(self.active[pos - 1], seg_idx); }
        if pos + 1 < self.active.len() { self.check_intersection(seg_idx, self.active[pos + 1]); }
    }
    
    fn handle_end(&mut self, seg_idx: usize) {
        if let Some(pos) = self.active.iter().position(|&i| i == seg_idx) {
            self.active.remove(pos);
            // Check if neighbors now intersect
            if pos > 0 && pos < self.active.len() {
                self.check_intersection(self.active[pos - 1], self.active[pos]);
            }
        }
    }
    
    fn handle_intersection(&mut self, seg1: usize, seg2: usize, point: Point) {
        // Record intersection
        self.intersections.push((seg1, seg2, point));
        
        // Swap segments in active list
        let pos1 = self.active.iter().position(|&i| i == seg1);
        let pos2 = self.active.iter().position(|&i| i == seg2);
        
        if let (Some(p1), Some(p2)) = (pos1, pos2) {
            self.active.swap(p1, p2);
            
            // Check for new intersections with new neighbors
            let (lo, hi) = if p1 < p2 { (p1, p2) } else { (p2, p1) };
            if lo > 0 { self.check_intersection(self.active[lo - 1], self.active[lo]); }
            if hi + 1 < self.active.len() { self.check_intersection(self.active[hi], self.active[hi + 1]); }
        }
    }
    
    fn check_intersection(&mut self, seg1: usize, seg2: usize) {
        if let Some(pt) = segment_intersection(&self.segments[seg1], &self.segments[seg2]) {
            // Only add if intersection is to the right of sweep line
            if pt.x > self.sweep_x + EPS {
                self.events.push(Event::intersection(pt, seg1, seg2));
            }
        }
    }
}

/// Result of boolean operation
#[derive(Clone, Debug, Default)]
pub struct BoolResult {
    pub contours: Vec<Polygon>,
}

impl BoolResult {
    /// Convert to SVG path data
    pub fn to_path_d(&self) -> String {
        self.contours.iter()
            .filter(|c| c.vertices.len() >= 3)
            .map(|c| {
                let mut d = String::new();
                for (i, p) in c.vertices.iter().enumerate() {
                    if i == 0 { d.push_str(&format!("M{:.4} {:.4}", p.x, p.y)); }
                    else { d.push_str(&format!(" L{:.4} {:.4}", p.x, p.y)); }
                }
                d.push_str(" Z");
                d
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Greiner-Hormann polygon clipping algorithm
/// More robust for complex polygons than Martinez-Rueda
pub struct PolygonClipper {
    subject: Polygon,
    clip: Polygon,
}

impl PolygonClipper {
    pub fn new(subject: Polygon, clip: Polygon) -> Self {
        Self { subject, clip }
    }
    
    /// Perform boolean operation
    pub fn compute(&self, op: BoolOp) -> BoolResult {
        if self.subject.vertices.len() < 3 || self.clip.vertices.len() < 3 {
            return BoolResult::default();
        }
        
        // Use Sutherland-Hodgman for simple convex clipping cases
        // For general polygons, use Weiler-Atherton or sweep-line based approach
        match op {
            BoolOp::Intersection => self.intersection(),
            BoolOp::Union => self.union(),
            BoolOp::Difference => self.difference(),
            BoolOp::Xor => self.xor(),
        }
    }
    
    fn intersection(&self) -> BoolResult {
        // Sutherland-Hodgman for convex clip polygon
        let mut output = self.subject.vertices.clone();
        
        let n = self.clip.vertices.len();
        for i in 0..n {
            if output.is_empty() { break; }
            
            let edge_start = self.clip.vertices[i];
            let edge_end = self.clip.vertices[(i + 1) % n];
            
            let input = std::mem::take(&mut output);
            let m = input.len();
            
            for j in 0..m {
                let current = input[j];
                let next = input[(j + 1) % m];
                
                let curr_inside = is_left(edge_start, edge_end, current);
                let next_inside = is_left(edge_start, edge_end, next);
                
                if curr_inside {
                    output.push(current);
                    if !next_inside {
                        if let Some(pt) = line_intersection(edge_start, edge_end, current, next) {
                            output.push(pt);
                        }
                    }
                } else if next_inside {
                    if let Some(pt) = line_intersection(edge_start, edge_end, current, next) {
                        output.push(pt);
                    }
                }
            }
        }
        
        let mut result = BoolResult::default();
        if output.len() >= 3 {
            result.contours.push(Polygon::new(output));
        }
        result
    }
    
    fn union(&self) -> BoolResult {
        // For union, we need to trace the outer boundary
        // Use Weiler-Atherton approach
        self.weiler_atherton(BoolOp::Union)
    }
    
    fn difference(&self) -> BoolResult {
        self.weiler_atherton(BoolOp::Difference)
    }
    
    fn xor(&self) -> BoolResult {
        // XOR = (A - B) ∪ (B - A)
        let a_minus_b = self.weiler_atherton(BoolOp::Difference);
        let clipper_rev = PolygonClipper::new(self.clip.clone(), self.subject.clone());
        let b_minus_a = clipper_rev.weiler_atherton(BoolOp::Difference);
        
        BoolResult {
            contours: a_minus_b.contours.into_iter()
                .chain(b_minus_a.contours)
                .collect(),
        }
    }
    
    /// Weiler-Atherton polygon clipping
    fn weiler_atherton(&self, op: BoolOp) -> BoolResult {
        // Find all intersection points
        let intersections = self.find_edge_intersections();
        
        if intersections.is_empty() {
            // No intersections - check containment
            return self.handle_no_intersections(op);
        }
        
        // Build intersection graph and trace contours
        self.trace_contours(&intersections, op)
    }
    
    fn find_edge_intersections(&self) -> Vec<IntersectionPoint> {
        let mut intersections = Vec::new();
        
        let sn = self.subject.vertices.len();
        let cn = self.clip.vertices.len();
        
        for i in 0..sn {
            let s0 = self.subject.vertices[i];
            let s1 = self.subject.vertices[(i + 1) % sn];
            
            for j in 0..cn {
                let c0 = self.clip.vertices[j];
                let c1 = self.clip.vertices[(j + 1) % cn];
                
                if let Some((pt, t_s, t_c)) = line_intersection_params(s0, s1, c0, c1) {
                    if t_s > EPS && t_s < 1.0 - EPS && t_c > EPS && t_c < 1.0 - EPS {
                        let entering = is_entering(s0, s1, c0, c1);
                        intersections.push(IntersectionPoint {
                            point: pt,
                            subj_edge: i,
                            clip_edge: j,
                            subj_t: t_s,
                            clip_t: t_c,
                            entering,
                        });
                    }
                }
            }
        }
        
        intersections
    }
    
    fn handle_no_intersections(&self, op: BoolOp) -> BoolResult {
        let subj_in_clip = self.clip.contains(self.subject.vertices[0]);
        let clip_in_subj = self.subject.contains(self.clip.vertices[0]);
        
        let mut result = BoolResult::default();
        
        match op {
            BoolOp::Intersection => {
                if subj_in_clip {
                    result.contours.push(self.subject.clone());
                } else if clip_in_subj {
                    result.contours.push(self.clip.clone());
                }
            }
            BoolOp::Union => {
                if subj_in_clip {
                    result.contours.push(self.clip.clone());
                } else if clip_in_subj {
                    result.contours.push(self.subject.clone());
                } else {
                    result.contours.push(self.subject.clone());
                    result.contours.push(self.clip.clone());
                }
            }
            BoolOp::Difference => {
                if !subj_in_clip && !clip_in_subj {
                    result.contours.push(self.subject.clone());
                } else if clip_in_subj {
                    // Subject with clip as hole
                    result.contours.push(self.subject.clone());
                    let mut hole = self.clip.clone();
                    hole.is_hole = true;
                    hole.normalize();
                    result.contours.push(hole);
                }
                // If subj_in_clip, result is empty
            }
            BoolOp::Xor => {
                if subj_in_clip || clip_in_subj {
                    // One contains the other - create ring
                    let (outer, inner) = if clip_in_subj {
                        (self.subject.clone(), self.clip.clone())
                    } else {
                        (self.clip.clone(), self.subject.clone())
                    };
                    result.contours.push(outer);
                    let mut hole = inner;
                    hole.is_hole = true;
                    hole.normalize();
                    result.contours.push(hole);
                } else {
                    result.contours.push(self.subject.clone());
                    result.contours.push(self.clip.clone());
                }
            }
        }
        
        result
    }
    
    fn trace_contours(&self, intersections: &[IntersectionPoint], op: BoolOp) -> BoolResult {
        // Build vertex lists with intersections inserted
        let mut subj_verts = self.build_vertex_list(&self.subject, intersections, true);
        let mut clip_verts = self.build_vertex_list(&self.clip, intersections, false);
        
        // Link intersection vertices between lists
        self.link_intersections(&mut subj_verts, &mut clip_verts, intersections);
        
        // Trace contours based on operation
        let mut result = BoolResult::default();
        let mut visited = vec![false; subj_verts.len()];
        
        for start_idx in 0..subj_verts.len() {
            if visited[start_idx] || !subj_verts[start_idx].is_intersection { continue; }
            
            let should_start = match op {
                BoolOp::Union => !subj_verts[start_idx].entering,
                BoolOp::Intersection | BoolOp::Difference => subj_verts[start_idx].entering,
                BoolOp::Xor => true,
            };
            
            if !should_start { continue; }
            
            if let Some(contour) = self.trace_single_contour(
                &subj_verts, &clip_verts, start_idx, op, &mut visited
            ) {
                if contour.vertices.len() >= 3 {
                    result.contours.push(contour);
                }
            }
        }
        
        result
    }
    
    fn build_vertex_list(&self, poly: &Polygon, intersections: &[IntersectionPoint], is_subject: bool) -> Vec<Vertex> {
        let n = poly.vertices.len();
        let mut verts: Vec<Vertex> = Vec::new();
        
        for i in 0..n {
            verts.push(Vertex {
                point: poly.vertices[i],
                is_intersection: false,
                entering: false,
                other_idx: None,
                next: None,
                prev: None,
            });
            
            // Collect intersections on this edge
            let mut edge_ints: Vec<_> = intersections.iter()
                .enumerate()
                .filter(|(_, ip)| {
                    if is_subject { ip.subj_edge == i } else { ip.clip_edge == i }
                })
                .collect();
            
            // Sort by parameter
            edge_ints.sort_by(|(_, a), (_, b)| {
                let t_a = if is_subject { a.subj_t } else { a.clip_t };
                let t_b = if is_subject { b.subj_t } else { b.clip_t };
                t_a.partial_cmp(&t_b).unwrap_or(Ordering::Equal)
            });
            
            for (int_idx, ip) in edge_ints {
                verts.push(Vertex {
                    point: ip.point,
                    is_intersection: true,
                    entering: if is_subject { ip.entering } else { !ip.entering },
                    other_idx: Some(int_idx),
                    next: None,
                    prev: None,
                });
            }
        }
        
        // Link next/prev
        let len = verts.len();
        for i in 0..len {
            verts[i].next = Some((i + 1) % len);
            verts[i].prev = Some((i + len - 1) % len);
        }
        
        verts
    }
    
    fn link_intersections(&self, subj: &mut [Vertex], clip: &mut [Vertex], intersections: &[IntersectionPoint]) {
        for (int_idx, _) in intersections.iter().enumerate() {
            let subj_idx = subj.iter().position(|v| v.other_idx == Some(int_idx));
            let clip_idx = clip.iter().position(|v| v.other_idx == Some(int_idx));
            
            if let (Some(si), Some(ci)) = (subj_idx, clip_idx) {
                subj[si].other_idx = Some(ci);
                clip[ci].other_idx = Some(si);
            }
        }
    }
    
    fn trace_single_contour(
        &self,
        subj: &[Vertex],
        clip: &[Vertex],
        start: usize,
        op: BoolOp,
        visited: &mut [bool],
    ) -> Option<Polygon> {
        let mut contour = Vec::new();
        let mut on_subject = true;
        let mut idx = start;
        let mut iterations = 0;
        let max_iterations = subj.len() + clip.len() + 100;
        
        loop {
            if iterations > max_iterations { break; }
            iterations += 1;
            
            let verts = if on_subject { subj } else { clip };
            if idx >= verts.len() { break; }
            
            let v = &verts[idx];
            contour.push(v.point);
            
            if on_subject && idx < visited.len() {
                visited[idx] = true;
            }
            
            if v.is_intersection {
                // Switch between subject and clip
                let switch = match op {
                    BoolOp::Intersection => v.entering == on_subject,
                    BoolOp::Union => v.entering != on_subject,
                    BoolOp::Difference => on_subject == v.entering,
                    BoolOp::Xor => true,
                };
                
                if switch {
                    if let Some(other) = v.other_idx {
                        on_subject = !on_subject;
                        idx = other;
                        let next_verts = if on_subject { subj } else { clip };
                        if idx < next_verts.len() {
                            if let Some(n) = next_verts[idx].next {
                                idx = n;
                            }
                        }
                        
                        if on_subject && idx == start {
                            break;
                        }
                        continue;
                    }
                }
            }
            
            // Move to next vertex
            if let Some(n) = v.next {
                idx = n;
            } else {
                break;
            }
            
            if on_subject && idx == start {
                break;
            }
        }
        
        if contour.len() >= 3 {
            Some(Polygon::new(contour))
        } else {
            None
        }
    }
}

/// Vertex in the intersection graph
#[derive(Clone, Debug)]
struct Vertex {
    point: Point,
    is_intersection: bool,
    entering: bool,
    other_idx: Option<usize>, // Index in other polygon's vertex list
    next: Option<usize>,
    prev: Option<usize>,
}

/// Intersection point with edge parameters
#[derive(Clone, Debug)]
struct IntersectionPoint {
    point: Point,
    subj_edge: usize,
    clip_edge: usize,
    subj_t: f64,
    clip_t: f64,
    entering: bool,
}

/// Check if point is on left side of edge (CCW)
fn is_left(edge_start: Point, edge_end: Point, p: Point) -> bool {
    let edge = edge_end.sub(edge_start);
    let to_p = p.sub(edge_start);
    edge.cross(to_p) >= 0.0
}

/// Compute line intersection point
fn line_intersection(a0: Point, a1: Point, b0: Point, b1: Point) -> Option<Point> {
    let (pt, t, _) = line_intersection_params(a0, a1, b0, b1)?;
    if t >= 0.0 && t <= 1.0 { Some(pt) } else { None }
}

/// Compute line intersection with parameters
fn line_intersection_params(a0: Point, a1: Point, b0: Point, b1: Point) -> Option<(Point, f64, f64)> {
    let da = a1.sub(a0);
    let db = b1.sub(b0);
    let cross = da.cross(db);
    
    if cross.abs() < EPS { return None; }
    
    let diff = b0.sub(a0);
    let t = diff.cross(db) / cross;
    let u = diff.cross(da) / cross;
    
    Some((a0.add(da.scale(t)), t, u))
}

/// Check if subject edge is entering clip polygon at intersection
fn is_entering(s0: Point, s1: Point, c0: Point, c1: Point) -> bool {
    let clip_edge = c1.sub(c0);
    let subj_dir = s1.sub(s0);
    // Subject enters clip if subject direction points left of clip edge
    clip_edge.cross(subj_dir) > 0.0
}

/// Flatten SVG path data to line segments
pub fn flatten_path(d: &str, tolerance: f64) -> Polygon {
    let mut vertices = Vec::new();
    let (mut cur_x, mut cur_y) = (0.0, 0.0);
    let (mut start_x, mut start_y) = (0.0, 0.0);
    let (mut last_ctrl_x, mut last_ctrl_y) = (0.0, 0.0);
    let mut last_cmd = ' ';
    
    let nums = extract_numbers_f64(d);
    let cmds: Vec<char> = d.chars()
        .filter(|c| matches!(c, 'M'|'m'|'L'|'l'|'H'|'h'|'V'|'v'|'C'|'c'|'S'|'s'|'Q'|'q'|'T'|'t'|'A'|'a'|'Z'|'z'))
        .collect();
    let mut idx = 0;
    
    for cmd in cmds {
        match cmd {
            'M' if idx + 1 < nums.len() => {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                start_x = cur_x; start_y = cur_y;
                vertices.push(Point::new(cur_x, cur_y));
                idx += 2;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'm' if idx + 1 < nums.len() => {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                start_x = cur_x; start_y = cur_y;
                vertices.push(Point::new(cur_x, cur_y));
                idx += 2;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'L' if idx + 1 < nums.len() => {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                vertices.push(Point::new(cur_x, cur_y));
                idx += 2;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'l' if idx + 1 < nums.len() => {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                vertices.push(Point::new(cur_x, cur_y));
                idx += 2;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'H' if idx < nums.len() => {
                cur_x = nums[idx];
                vertices.push(Point::new(cur_x, cur_y));
                idx += 1;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'h' if idx < nums.len() => {
                cur_x += nums[idx];
                vertices.push(Point::new(cur_x, cur_y));
                idx += 1;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'V' if idx < nums.len() => {
                cur_y = nums[idx];
                vertices.push(Point::new(cur_x, cur_y));
                idx += 1;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'v' if idx < nums.len() => {
                cur_y += nums[idx];
                vertices.push(Point::new(cur_x, cur_y));
                idx += 1;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'C' if idx + 5 < nums.len() => {
                let pts = flatten_cubic(
                    Point::new(cur_x, cur_y),
                    Point::new(nums[idx], nums[idx+1]),
                    Point::new(nums[idx+2], nums[idx+3]),
                    Point::new(nums[idx+4], nums[idx+5]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = nums[idx+2]; last_ctrl_y = nums[idx+3];
                cur_x = nums[idx+4]; cur_y = nums[idx+5];
                idx += 6;
            }
            'c' if idx + 5 < nums.len() => {
                let pts = flatten_cubic(
                    Point::new(cur_x, cur_y),
                    Point::new(cur_x + nums[idx], cur_y + nums[idx+1]),
                    Point::new(cur_x + nums[idx+2], cur_y + nums[idx+3]),
                    Point::new(cur_x + nums[idx+4], cur_y + nums[idx+5]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = cur_x + nums[idx+2]; last_ctrl_y = cur_y + nums[idx+3];
                cur_x += nums[idx+4]; cur_y += nums[idx+5];
                idx += 6;
            }
            'S' if idx + 3 < nums.len() => {
                let (x1, y1) = if matches!(last_cmd, 'C'|'c'|'S'|'s') {
                    (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y)
                } else { (cur_x, cur_y) };
                let pts = flatten_cubic(
                    Point::new(cur_x, cur_y),
                    Point::new(x1, y1),
                    Point::new(nums[idx], nums[idx+1]),
                    Point::new(nums[idx+2], nums[idx+3]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = nums[idx]; last_ctrl_y = nums[idx+1];
                cur_x = nums[idx+2]; cur_y = nums[idx+3];
                idx += 4;
            }
            's' if idx + 3 < nums.len() => {
                let (x1, y1) = if matches!(last_cmd, 'C'|'c'|'S'|'s') {
                    (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y)
                } else { (cur_x, cur_y) };
                let pts = flatten_cubic(
                    Point::new(cur_x, cur_y),
                    Point::new(x1, y1),
                    Point::new(cur_x + nums[idx], cur_y + nums[idx+1]),
                    Point::new(cur_x + nums[idx+2], cur_y + nums[idx+3]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = cur_x + nums[idx]; last_ctrl_y = cur_y + nums[idx+1];
                cur_x += nums[idx+2]; cur_y += nums[idx+3];
                idx += 4;
            }
            'Q' if idx + 3 < nums.len() => {
                let pts = flatten_quadratic(
                    Point::new(cur_x, cur_y),
                    Point::new(nums[idx], nums[idx+1]),
                    Point::new(nums[idx+2], nums[idx+3]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = nums[idx]; last_ctrl_y = nums[idx+1];
                cur_x = nums[idx+2]; cur_y = nums[idx+3];
                idx += 4;
            }
            'q' if idx + 3 < nums.len() => {
                let pts = flatten_quadratic(
                    Point::new(cur_x, cur_y),
                    Point::new(cur_x + nums[idx], cur_y + nums[idx+1]),
                    Point::new(cur_x + nums[idx+2], cur_y + nums[idx+3]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = cur_x + nums[idx]; last_ctrl_y = cur_y + nums[idx+1];
                cur_x += nums[idx+2]; cur_y += nums[idx+3];
                idx += 4;
            }
            'T' if idx + 1 < nums.len() => {
                let (x1, y1) = if matches!(last_cmd, 'Q'|'q'|'T'|'t') {
                    (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y)
                } else { (cur_x, cur_y) };
                let pts = flatten_quadratic(
                    Point::new(cur_x, cur_y),
                    Point::new(x1, y1),
                    Point::new(nums[idx], nums[idx+1]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = x1; last_ctrl_y = y1;
                cur_x = nums[idx]; cur_y = nums[idx+1];
                idx += 2;
            }
            't' if idx + 1 < nums.len() => {
                let (x1, y1) = if matches!(last_cmd, 'Q'|'q'|'T'|'t') {
                    (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y)
                } else { (cur_x, cur_y) };
                let pts = flatten_quadratic(
                    Point::new(cur_x, cur_y),
                    Point::new(x1, y1),
                    Point::new(cur_x + nums[idx], cur_y + nums[idx+1]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                last_ctrl_x = x1; last_ctrl_y = y1;
                cur_x += nums[idx]; cur_y += nums[idx+1];
                idx += 2;
            }
            'A' if idx + 6 < nums.len() => {
                let pts = flatten_arc(
                    Point::new(cur_x, cur_y),
                    nums[idx].abs(), nums[idx+1].abs(), nums[idx+2],
                    nums[idx+3] != 0.0, nums[idx+4] != 0.0,
                    Point::new(nums[idx+5], nums[idx+6]),
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                cur_x = nums[idx+5]; cur_y = nums[idx+6];
                idx += 7;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'a' if idx + 6 < nums.len() => {
                let end = Point::new(cur_x + nums[idx+5], cur_y + nums[idx+6]);
                let pts = flatten_arc(
                    Point::new(cur_x, cur_y),
                    nums[idx].abs(), nums[idx+1].abs(), nums[idx+2],
                    nums[idx+3] != 0.0, nums[idx+4] != 0.0,
                    end,
                    tolerance,
                );
                vertices.extend(pts.into_iter().skip(1));
                cur_x = end.x; cur_y = end.y;
                idx += 7;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            'Z' | 'z' => {
                if (cur_x - start_x).abs() > EPS || (cur_y - start_y).abs() > EPS {
                    vertices.push(Point::new(start_x, start_y));
                }
                cur_x = start_x; cur_y = start_y;
                last_ctrl_x = cur_x; last_ctrl_y = cur_y;
            }
            _ => {}
        }
        last_cmd = cmd;
    }
    
    Polygon::new(vertices)
}

/// Flatten cubic bezier to line segments using de Casteljau subdivision
fn flatten_cubic(p0: Point, p1: Point, p2: Point, p3: Point, tolerance: f64) -> Vec<Point> {
    let mut result = vec![p0];
    flatten_cubic_rec(p0, p1, p2, p3, tolerance * tolerance, &mut result);
    result
}

fn flatten_cubic_rec(p0: Point, p1: Point, p2: Point, p3: Point, tol2: f64, out: &mut Vec<Point>) {
    // Check if curve is flat enough
    let d1 = point_line_dist2(p1, p0, p3);
    let d2 = point_line_dist2(p2, p0, p3);
    
    if d1 + d2 <= tol2 {
        out.push(p3);
        return;
    }
    
    // Subdivide at t=0.5
    let p01 = p0.add(p1).scale(0.5);
    let p12 = p1.add(p2).scale(0.5);
    let p23 = p2.add(p3).scale(0.5);
    let p012 = p01.add(p12).scale(0.5);
    let p123 = p12.add(p23).scale(0.5);
    let p0123 = p012.add(p123).scale(0.5);
    
    flatten_cubic_rec(p0, p01, p012, p0123, tol2, out);
    flatten_cubic_rec(p0123, p123, p23, p3, tol2, out);
}

/// Flatten quadratic bezier to line segments
fn flatten_quadratic(p0: Point, p1: Point, p2: Point, tolerance: f64) -> Vec<Point> {
    let mut result = vec![p0];
    flatten_quadratic_rec(p0, p1, p2, tolerance * tolerance, &mut result);
    result
}

fn flatten_quadratic_rec(p0: Point, p1: Point, p2: Point, tol2: f64, out: &mut Vec<Point>) {
    let d = point_line_dist2(p1, p0, p2);
    
    if d <= tol2 {
        out.push(p2);
        return;
    }
    
    let p01 = p0.add(p1).scale(0.5);
    let p12 = p1.add(p2).scale(0.5);
    let p012 = p01.add(p12).scale(0.5);
    
    flatten_quadratic_rec(p0, p01, p012, tol2, out);
    flatten_quadratic_rec(p012, p12, p2, tol2, out);
}

/// Flatten elliptical arc to line segments
fn flatten_arc(p0: Point, mut rx: f64, mut ry: f64, phi_deg: f64, large_arc: bool, sweep: bool, p1: Point, tolerance: f64) -> Vec<Point> {
    if rx < EPS || ry < EPS { return vec![p0, p1]; }
    
    let phi = phi_deg.to_radians();
    let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
    
    // Transform to unit circle
    let dx = (p0.x - p1.x) / 2.0;
    let dy = (p0.y - p1.y) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    
    // Scale radii if necessary
    let lambda = (x1p / rx).powi(2) + (y1p / ry).powi(2);
    if lambda > 1.0 { let s = lambda.sqrt(); rx *= s; ry *= s; }
    
    // Center parameterization
    let sq = ((rx*ry).powi(2) - (rx*y1p).powi(2) - (ry*x1p).powi(2)) / ((rx*y1p).powi(2) + (ry*x1p).powi(2));
    let coef = if large_arc != sweep { sq.max(0.0).sqrt() } else { -sq.max(0.0).sqrt() };
    
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;
    
    let cx = cos_phi * cxp - sin_phi * cyp + (p0.x + p1.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (p0.y + p1.y) / 2.0;
    
    let theta1 = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let mut dtheta = (((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx) - theta1).rem_euclid(std::f64::consts::TAU);
    if !sweep { dtheta -= std::f64::consts::TAU; }
    
    // Approximate arc with line segments
    let n = ((rx.max(ry) * dtheta.abs() / tolerance).ceil() as usize).max(4);
    let mut result = vec![p0];
    
    for i in 1..=n {
        let t = theta1 + dtheta * (i as f64) / (n as f64);
        let x = cx + rx * t.cos() * cos_phi - ry * t.sin() * sin_phi;
        let y = cy + rx * t.cos() * sin_phi + ry * t.sin() * cos_phi;
        result.push(Point::new(x, y));
    }
    
    result
}

/// Squared distance from point to line segment
fn point_line_dist2(p: Point, a: Point, b: Point) -> f64 {
    let ab = b.sub(a);
    let ap = p.sub(a);
    let len2 = ab.len2();
    
    if len2 < EPS { return ap.len2(); }
    
    let t = (ap.dot(ab) / len2).clamp(0.0, 1.0);
    let proj = a.add(ab.scale(t));
    p.sub(proj).len2()
}

fn extract_numbers_f64(d: &str) -> Vec<f64> {
    let mut nums = Vec::new();
    let mut buf = String::new();
    
    for c in d.chars() {
        if c.is_ascii_digit() || c == '.' || (c == '-' && buf.is_empty()) || (c == '-' && buf.ends_with('e')) {
            buf.push(c);
        } else if c == 'e' || c == 'E' {
            buf.push('e');
        } else {
            if !buf.is_empty() {
                if let Ok(n) = buf.parse::<f64>() { nums.push(n); }
                buf.clear();
            }
            if c == '-' { buf.push(c); }
        }
    }
    if !buf.is_empty() {
        if let Ok(n) = buf.parse::<f64>() { nums.push(n); }
    }
    nums
}

/// Perform boolean operation on two SVG paths
pub fn path_boolean(path_a: &str, path_b: &str, op: BoolOp, tolerance: f64) -> String {
    let poly_a = flatten_path(path_a, tolerance);
    let poly_b = flatten_path(path_b, tolerance);
    
    let clipper = PolygonClipper::new(poly_a, poly_b);
    clipper.compute(op).to_path_d()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_point_operations() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(3.0, 4.0);
        
        assert_eq!(p1.add(p2), Point::new(4.0, 6.0));
        assert_eq!(p1.sub(p2), Point::new(-2.0, -2.0));
        assert_eq!(p1.scale(2.0), Point::new(2.0, 4.0));
        assert!((p1.dot(p2) - 11.0).abs() < EPS);
        assert!((p1.cross(p2) - (-2.0)).abs() < EPS);
    }
    
    #[test]
    fn test_segment_intersection() {
        let s1 = Segment::new(Point::new(0.0, 0.0), Point::new(2.0, 2.0), 0, 0);
        let s2 = Segment::new(Point::new(0.0, 2.0), Point::new(2.0, 0.0), 0, 1);
        
        let int = segment_intersection(&s1, &s2);
        assert!(int.is_some());
        let pt = int.unwrap();
        assert!((pt.x - 1.0).abs() < EPS);
        assert!((pt.y - 1.0).abs() < EPS);
    }
    
    #[test]
    fn test_segment_no_intersection() {
        let s1 = Segment::new(Point::new(0.0, 0.0), Point::new(1.0, 0.0), 0, 0);
        let s2 = Segment::new(Point::new(0.0, 1.0), Point::new(1.0, 1.0), 0, 1);
        
        assert!(segment_intersection(&s1, &s2).is_none());
    }
    
    #[test]
    fn test_polygon_area() {
        // CCW square
        let square = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
        ]);
        assert!((square.signed_area() - 1.0).abs() < EPS);
        assert!(square.is_ccw());
    }
    
    #[test]
    fn test_polygon_contains() {
        let square = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(0.0, 2.0),
        ]);
        
        assert!(square.contains(Point::new(1.0, 1.0)));
        assert!(!square.contains(Point::new(3.0, 1.0)));
    }
    
    #[test]
    fn test_intersection_squares() {
        // Two overlapping squares with 1x1 intersection at (1,1)-(2,2)
        let a = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(0.0, 2.0),
        ]);
        let b = Polygon::new(vec![
            Point::new(1.0, 1.0),
            Point::new(3.0, 1.0),
            Point::new(3.0, 3.0),
            Point::new(1.0, 3.0),
        ]);
        
        let clipper = PolygonClipper::new(a, b);
        let result = clipper.compute(BoolOp::Intersection);
        
        // Result should have at least one contour
        assert!(!result.contours.is_empty(), "Intersection should produce contours");
        
        // Check that we got a reasonable result (polygon with >= 3 vertices)
        assert!(result.contours.iter().all(|c| c.vertices.len() >= 3), 
            "All contours should have at least 3 vertices");
        
        // Verify the result is non-empty by checking area > 0
        let area: f64 = result.contours.iter().map(|c| c.signed_area().abs()).sum();
        assert!(area > 0.0, "Intersection area should be positive");
    }
    
    #[test]
    fn test_flatten_cubic() {
        let pts = flatten_cubic(
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 0.0),
            0.1,
        );
        assert!(pts.len() >= 2);
        assert!(pts[0] == Point::new(0.0, 0.0));
        assert!((pts.last().unwrap().x - 1.0).abs() < EPS);
    }
    
    #[test]
    fn test_flatten_path_simple() {
        let poly = flatten_path("M0 0 L10 0 L10 10 L0 10 Z", 1.0);
        assert_eq!(poly.vertices.len(), 5); // 4 vertices + close
    }
    
    #[test]
    fn test_path_boolean_union() {
        let a = "M0 0 L10 0 L10 10 L0 10 Z";
        let b = "M5 5 L15 5 L15 15 L5 15 Z";
        
        let result = path_boolean(a, b, BoolOp::Union, 0.5);
        assert!(!result.is_empty());
        assert!(result.contains('M'));
        assert!(result.contains('Z'));
    }
    
    #[test]
    fn test_sweep_line_basic() {
        let segments = vec![
            Segment::new(Point::new(0.0, 0.0), Point::new(2.0, 2.0), 0, 0),
            Segment::new(Point::new(0.0, 2.0), Point::new(2.0, 0.0), 0, 1),
        ];
        
        let sweep = SweepLine::new(segments);
        let intersections = sweep.find_intersections();
        
        assert_eq!(intersections.len(), 1);
        assert!((intersections[0].2.x - 1.0).abs() < EPS);
        assert!((intersections[0].2.y - 1.0).abs() < EPS);
    }
}

