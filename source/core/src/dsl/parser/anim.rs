//! Animation primitives: keyframes, transitions, temporal interpolation
//!
//! CSS-based animation system for smooth, hardware-accelerated motion.
//! Generates inline `<style>` blocks with @keyframes and CSS transitions.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg(feature = "python")]
use pyo3::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Easing Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Timing function for animations/transitions
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Easing {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// Custom cubic-bezier(x1, y1, x2, y2)
    CubicBezier(f64, f64, f64, f64),
    /// Step function: steps(n, jump-start|jump-end|jump-both|jump-none)
    Steps(u32, StepPosition),
}

impl Default for Easing {
    fn default() -> Self { Self::Ease }
}

impl Easing {
    pub fn to_css(&self) -> String {
        match self {
            Self::Linear => "linear".into(),
            Self::Ease => "ease".into(),
            Self::EaseIn => "ease-in".into(),
            Self::EaseOut => "ease-out".into(),
            Self::EaseInOut => "ease-in-out".into(),
            Self::CubicBezier(x1, y1, x2, y2) => format!("cubic-bezier({},{},{},{})", x1, y1, x2, y2),
            Self::Steps(n, pos) => format!("steps({},{})", n, pos.to_css()),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "linear" => Some(Self::Linear),
            "ease" => Some(Self::Ease),
            "ease-in" => Some(Self::EaseIn),
            "ease-out" => Some(Self::EaseOut),
            "ease-in-out" => Some(Self::EaseInOut),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum StepPosition { Start, End, Both, None }

impl Default for StepPosition {
    fn default() -> Self { Self::End }
}

impl StepPosition {
    fn to_css(&self) -> &'static str {
        match self {
            Self::Start => "jump-start",
            Self::End => "jump-end",
            Self::Both => "jump-both",
            Self::None => "jump-none",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation Direction & Fill Mode
// ─────────────────────────────────────────────────────────────────────────────

/// Animation playback direction
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Direction { #[default] Normal, Reverse, Alternate, AlternateReverse }

impl Direction {
    pub fn to_css(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Reverse => "reverse",
            Self::Alternate => "alternate",
            Self::AlternateReverse => "alternate-reverse",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "reverse" => Some(Self::Reverse),
            "alternate" => Some(Self::Alternate),
            "alternate-reverse" => Some(Self::AlternateReverse),
            _ => None,
        }
    }
}

/// Animation fill mode (how styles apply before/after)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FillMode { #[default] None, Forwards, Backwards, Both }

impl FillMode {
    pub fn to_css(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Forwards => "forwards",
            Self::Backwards => "backwards",
            Self::Both => "both",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "forwards" => Some(Self::Forwards),
            "backwards" => Some(Self::Backwards),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// Animation play state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PlayState { #[default] Running, Paused }

impl PlayState {
    pub fn to_css(&self) -> &'static str {
        match self { Self::Running => "running", Self::Paused => "paused" }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Iteration Count
// ─────────────────────────────────────────────────────────────────────────────

/// Number of animation iterations
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Iteration {
    Count(f64),
    Infinite,
}

impl Default for Iteration {
    fn default() -> Self { Self::Count(1.0) }
}

impl Iteration {
    pub fn to_css(&self) -> String {
        match self {
            Self::Count(n) => format!("{}", n),
            Self::Infinite => "infinite".into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration (normalized to milliseconds)
// ─────────────────────────────────────────────────────────────────────────────

/// Duration in milliseconds (internal representation)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Duration(pub f64);

impl Default for Duration {
    fn default() -> Self { Self(0.0) }
}

impl Duration {
    pub fn ms(v: f64) -> Self { Self(v) }
    pub fn secs(v: f64) -> Self { Self(v * 1000.0) }
    
    pub fn to_css(&self) -> String {
        if self.0 >= 1000.0 && self.0 % 1000.0 == 0.0 {
            format!("{}s", self.0 / 1000.0)
        } else {
            format!("{}ms", self.0)
        }
    }

    pub fn as_ms(&self) -> f64 { self.0 }
    pub fn as_secs(&self) -> f64 { self.0 / 1000.0 }
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyframe Step (single frame in animation sequence)
// ─────────────────────────────────────────────────────────────────────────────

/// Single keyframe in an animation sequence
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct KeyframeStep {
    /// Offset within animation (0-100%)
    pub offset: f64,
    /// Style properties to animate
    pub properties: Vec<AnimatableProperty>,
}

impl KeyframeStep {
    pub fn new(offset: f64) -> Self {
        Self { offset, properties: Vec::new() }
    }

    pub fn with_property(mut self, prop: AnimatableProperty) -> Self {
        self.properties.push(prop);
        self
    }

    pub fn to_css(&self) -> String {
        let props: Vec<String> = self.properties.iter().map(|p| p.to_css()).collect();
        format!("{}% {{ {} }}", self.offset, props.join(" "))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Animatable Properties
// ─────────────────────────────────────────────────────────────────────────────

/// Properties that can be animated
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AnimatableProperty {
    Opacity(f64),
    Fill(String),
    Stroke(String),
    StrokeWidth(f64),
    Transform(String),
    Translate(f64, f64),
    Rotate(f64),
    Scale(f64, f64),
    // Path morphing (d attribute)
    PathD(String),
    // Positional
    X(f64),
    Y(f64),
    Cx(f64),
    Cy(f64),
    R(f64),
    Width(f64),
    Height(f64),
}

impl AnimatableProperty {
    pub fn to_css(&self) -> String {
        match self {
            Self::Opacity(v) => format!("opacity: {};", v),
            Self::Fill(c) => format!("fill: {};", c),
            Self::Stroke(c) => format!("stroke: {};", c),
            Self::StrokeWidth(w) => format!("stroke-width: {};", w),
            Self::Transform(t) => format!("transform: {};", t),
            Self::Translate(x, y) => format!("transform: translate({}px, {}px);", x, y),
            Self::Rotate(deg) => format!("transform: rotate({}deg);", deg),
            Self::Scale(x, y) => format!("transform: scale({}, {});", x, y),
            Self::PathD(d) => format!("d: path('{}');", d),
            Self::X(v) | Self::Cx(v) => format!("cx: {};", v),
            Self::Y(v) | Self::Cy(v) => format!("cy: {};", v),
            Self::R(v) => format!("r: {};", v),
            Self::Width(v) => format!("width: {};", v),
            Self::Height(v) => format!("height: {};", v),
        }
    }

    pub fn property_name(&self) -> &'static str {
        match self {
            Self::Opacity(_) => "opacity",
            Self::Fill(_) => "fill",
            Self::Stroke(_) => "stroke",
            Self::StrokeWidth(_) => "stroke-width",
            Self::Transform(_) | Self::Translate(_, _) | Self::Rotate(_) | Self::Scale(_, _) => "transform",
            Self::PathD(_) => "d",
            Self::X(_) | Self::Cx(_) => "cx",
            Self::Y(_) | Self::Cy(_) => "cy",
            Self::R(_) => "r",
            Self::Width(_) => "width",
            Self::Height(_) => "height",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyframes Definition
// ─────────────────────────────────────────────────────────────────────────────

/// Named keyframes animation definition (CSS @keyframes)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Keyframes {
    pub name: String,
    pub steps: Vec<KeyframeStep>,
}

impl Default for Keyframes {
    fn default() -> Self {
        Self { name: String::new(), steps: Vec::new() }
    }
}

impl Keyframes {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), steps: Vec::new() }
    }

    pub fn with_step(mut self, step: KeyframeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Generate CSS @keyframes block
    pub fn to_css(&self) -> String {
        let frames: Vec<String> = self.steps.iter().map(|s| s.to_css()).collect();
        format!("@keyframes {} {{ {} }}", self.name, frames.join(" "))
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Keyframes {
    #[new]
    fn py_new(name: String) -> Self { Self::new(name) }
    
    #[getter] fn get_name(&self) -> String { self.name.clone() }
    #[getter] fn step_count(&self) -> usize { self.steps.len() }
    fn css(&self) -> String { self.to_css() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation (reference to keyframes + timing)
// ─────────────────────────────────────────────────────────────────────────────

/// Animation applied to an element (references a Keyframes definition)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Animation {
    /// Name of @keyframes to use
    pub name: String,
    /// Duration
    pub duration: Duration,
    /// Timing function
    pub easing: Easing,
    /// Delay before start
    pub delay: Duration,
    /// Number of iterations
    pub iteration: Iteration,
    /// Playback direction
    pub direction: Direction,
    /// Fill mode
    pub fill_mode: FillMode,
    /// Play state
    pub play_state: PlayState,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration: Duration::ms(300.0),
            easing: Easing::Ease,
            delay: Duration::ms(0.0),
            iteration: Iteration::Count(1.0),
            direction: Direction::Normal,
            fill_mode: FillMode::None,
            play_state: PlayState::Running,
        }
    }
}

impl Animation {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Default::default() }
    }

    pub fn with_duration(mut self, dur: Duration) -> Self { self.duration = dur; self }
    pub fn with_easing(mut self, e: Easing) -> Self { self.easing = e; self }
    pub fn with_delay(mut self, d: Duration) -> Self { self.delay = d; self }
    pub fn with_iteration(mut self, i: Iteration) -> Self { self.iteration = i; self }
    pub fn with_direction(mut self, d: Direction) -> Self { self.direction = d; self }
    pub fn with_fill(mut self, f: FillMode) -> Self { self.fill_mode = f; self }
    pub fn infinite(mut self) -> Self { self.iteration = Iteration::Infinite; self }

    /// Generate CSS animation property value
    pub fn to_css(&self) -> String {
        format!(
            "{} {} {} {} {} {} {}",
            self.name,
            self.duration.to_css(),
            self.easing.to_css(),
            self.delay.to_css(),
            self.iteration.to_css(),
            self.direction.to_css(),
            self.fill_mode.to_css(),
        )
    }

    /// Generate full CSS style attribute
    pub fn to_style(&self) -> String {
        format!("animation: {};", self.to_css())
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Animation {
    #[new]
    #[pyo3(signature = (name, duration_ms=300.0))]
    fn py_new(name: String, duration_ms: f64) -> Self {
        Self::new(name).with_duration(Duration::ms(duration_ms))
    }
    
    #[getter] fn get_name(&self) -> String { self.name.clone() }
    #[getter] fn get_duration_ms(&self) -> f64 { self.duration.as_ms() }
    #[getter] fn get_delay_ms(&self) -> f64 { self.delay.as_ms() }
    fn css(&self) -> String { self.to_css() }
    fn style(&self) -> String { self.to_style() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transition (CSS transitions for property changes)
// ─────────────────────────────────────────────────────────────────────────────

/// CSS transition for property changes
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Transition {
    /// Property to transition ("all" for any)
    pub property: String,
    /// Duration
    pub duration: Duration,
    /// Timing function
    pub easing: Easing,
    /// Delay before transition starts
    pub delay: Duration,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            property: "all".into(),
            duration: Duration::ms(300.0),
            easing: Easing::Ease,
            delay: Duration::ms(0.0),
        }
    }
}

impl Transition {
    pub fn new(property: impl Into<String>) -> Self {
        Self { property: property.into(), ..Default::default() }
    }

    pub fn all() -> Self { Self::new("all") }
    
    pub fn with_duration(mut self, d: Duration) -> Self { self.duration = d; self }
    pub fn with_easing(mut self, e: Easing) -> Self { self.easing = e; self }
    pub fn with_delay(mut self, d: Duration) -> Self { self.delay = d; self }

    /// Generate CSS transition value
    pub fn to_css(&self) -> String {
        format!(
            "{} {} {} {}",
            self.property,
            self.duration.to_css(),
            self.easing.to_css(),
            self.delay.to_css(),
        )
    }

    /// Generate full CSS style attribute
    pub fn to_style(&self) -> String {
        format!("transition: {};", self.to_css())
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Transition {
    #[new]
    #[pyo3(signature = (property="all".to_string(), duration_ms=300.0))]
    fn py_new(property: String, duration_ms: f64) -> Self {
        Self::new(property).with_duration(Duration::ms(duration_ms))
    }
    
    #[getter] fn get_property(&self) -> String { self.property.clone() }
    #[getter] fn get_duration_ms(&self) -> f64 { self.duration.as_ms() }
    #[getter] fn get_delay_ms(&self) -> f64 { self.delay.as_ms() }
    fn css(&self) -> String { self.to_css() }
    fn style(&self) -> String { self.to_style() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Temporal Interpolation (frame-based animation)
// ─────────────────────────────────────────────────────────────────────────────

/// Temporal interpolation for frame-based animation
/// Used for programmatic animation control (e.g., in real-time updates)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Interpolation {
    /// Start time (ms from animation start)
    pub start: f64,
    /// End time (ms)
    pub end: f64,
    /// From value
    pub from: f64,
    /// To value  
    pub to: f64,
    /// Easing function
    pub easing: Easing,
}

impl Interpolation {
    pub fn new(start: f64, end: f64, from: f64, to: f64) -> Self {
        Self { start, end, from, to, easing: Easing::Linear }
    }

    pub fn with_easing(mut self, e: Easing) -> Self { self.easing = e; self }

    /// Compute interpolated value at time t (ms)
    pub fn at(&self, t: f64) -> f64 {
        if t <= self.start { return self.from; }
        if t >= self.end { return self.to; }
        
        let progress = (t - self.start) / (self.end - self.start);
        let eased = self.ease(progress);
        self.from + (self.to - self.from) * eased
    }

    /// Apply easing function to linear progress [0,1]
    fn ease(&self, t: f64) -> f64 {
        match &self.easing {
            Easing::Linear => t,
            Easing::Ease => cubic_bezier(t, 0.25, 0.1, 0.25, 1.0),
            Easing::EaseIn => cubic_bezier(t, 0.42, 0.0, 1.0, 1.0),
            Easing::EaseOut => cubic_bezier(t, 0.0, 0.0, 0.58, 1.0),
            Easing::EaseInOut => cubic_bezier(t, 0.42, 0.0, 0.58, 1.0),
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier(t, *x1, *y1, *x2, *y2),
            Easing::Steps(n, pos) => step(*n, *pos, t),
        }
    }
}

/// Approximate cubic bezier curve (Newton-Raphson method)
fn cubic_bezier(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // Find x parameter for given t using Newton-Raphson
    let mut x = t;
    for _ in 0..8 {
        let x_t = bezier_x(x, x1, x2) - t;
        if x_t.abs() < 1e-6 { break; }
        let dx = bezier_dx(x, x1, x2);
        if dx.abs() < 1e-6 { break; }
        x -= x_t / dx;
    }
    bezier_y(x, y1, y2)
}

#[inline]
fn bezier_x(t: f64, x1: f64, x2: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    3.0 * x1 * t * (1.0 - t).powi(2) + 3.0 * x2 * t2 * (1.0 - t) + t3
}

#[inline]
fn bezier_y(t: f64, y1: f64, y2: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    3.0 * y1 * t * (1.0 - t).powi(2) + 3.0 * y2 * t2 * (1.0 - t) + t3
}

#[inline]
fn bezier_dx(t: f64, x1: f64, x2: f64) -> f64 {
    let t2 = t * t;
    3.0 * x1 * (1.0 - t).powi(2) - 6.0 * x1 * t * (1.0 - t) + 6.0 * x2 * t * (1.0 - t) - 3.0 * x2 * t2 + 3.0 * t2
}

fn step(n: u32, pos: StepPosition, t: f64) -> f64 {
    let steps = n as f64;
    match pos {
        StepPosition::Start => (t * steps).ceil() / steps,
        StepPosition::End => (t * steps).floor() / steps,
        StepPosition::Both => ((t * steps).floor() + 1.0).min(steps) / steps,
        StepPosition::None => {
            let s = (t * steps).floor();
            if s >= steps { 1.0 } else { s / (steps - 1.0).max(1.0) }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation State Container (for shapes)
// ─────────────────────────────────────────────────────────────────────────────

/// Animation state attached to a shape
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnimationState {
    /// Animation (references keyframes)
    pub animation: Option<Animation>,
    /// Transitions for property changes
    pub transitions: Vec<Transition>,
}

impl AnimationState {
    pub fn with_animation(animation: Animation) -> Self {
        Self { animation: Some(animation), transitions: Vec::new() }
    }

    pub fn with_transition(transition: Transition) -> Self {
        Self { animation: None, transitions: vec![transition] }
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.transitions.push(t);
    }

    pub fn has_animation(&self) -> bool {
        self.animation.is_some() || !self.transitions.is_empty()
    }

    /// Generate CSS style string for this animation state
    pub fn to_style(&self) -> String {
        let mut styles = Vec::new();
        
        if let Some(anim) = &self.animation {
            styles.push(anim.to_style());
        }
        
        if !self.transitions.is_empty() {
            let trans: Vec<String> = self.transitions.iter().map(|t| t.to_css()).collect();
            styles.push(format!("transition: {};", trans.join(", ")));
        }
        
        styles.join(" ")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_css() {
        assert_eq!(Duration::ms(500.0).to_css(), "500ms");
        assert_eq!(Duration::secs(1.0).to_css(), "1s");
        assert_eq!(Duration::secs(2.5).to_css(), "2500ms");
    }

    #[test]
    fn test_easing_css() {
        assert_eq!(Easing::Linear.to_css(), "linear");
        assert_eq!(Easing::CubicBezier(0.4, 0.0, 0.2, 1.0).to_css(), "cubic-bezier(0.4,0,0.2,1)");
    }

    #[test]
    fn test_keyframes_css() {
        let kf = Keyframes::new("fade-in")
            .with_step(KeyframeStep::new(0.0).with_property(AnimatableProperty::Opacity(0.0)))
            .with_step(KeyframeStep::new(100.0).with_property(AnimatableProperty::Opacity(1.0)));
        
        let css = kf.to_css();
        assert!(css.contains("@keyframes fade-in"));
        assert!(css.contains("0%"));
        assert!(css.contains("100%"));
        assert!(css.contains("opacity"));
    }

    #[test]
    fn test_animation_css() {
        let anim = Animation::new("pulse")
            .with_duration(Duration::secs(1.0))
            .with_easing(Easing::EaseInOut)
            .infinite();
        
        let css = anim.to_css();
        assert!(css.contains("pulse"));
        assert!(css.contains("1s"));
        assert!(css.contains("ease-in-out"));
        assert!(css.contains("infinite"));
    }

    #[test]
    fn test_transition_css() {
        let trans = Transition::new("opacity")
            .with_duration(Duration::ms(200.0))
            .with_easing(Easing::EaseOut);
        
        assert!(trans.to_css().contains("opacity"));
        assert!(trans.to_css().contains("200ms"));
    }

    #[test]
    fn test_interpolation() {
        let interp = Interpolation::new(0.0, 1000.0, 0.0, 100.0);
        
        assert!((interp.at(0.0) - 0.0).abs() < 0.01);
        assert!((interp.at(500.0) - 50.0).abs() < 0.01);
        assert!((interp.at(1000.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_interpolation_with_easing() {
        let interp = Interpolation::new(0.0, 1000.0, 0.0, 100.0)
            .with_easing(Easing::EaseInOut);
        
        // Should start slow, speed up, then slow down
        let early = interp.at(100.0);
        let mid = interp.at(500.0);
        let late = interp.at(900.0);
        
        // Mid should be close to 50
        assert!((mid - 50.0).abs() < 5.0);
        // Early progress should be less than linear
        assert!(early < 10.0);
        // Late progress should be more than linear
        assert!(late > 90.0);
    }

    #[test]
    fn test_animation_state() {
        let state = AnimationState {
            animation: Some(Animation::new("spin").with_duration(Duration::secs(2.0))),
            transitions: vec![Transition::new("opacity").with_duration(Duration::ms(150.0))],
        };
        
        let style = state.to_style();
        assert!(style.contains("animation:"));
        assert!(style.contains("transition:"));
    }
}

