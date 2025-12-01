//! SVG path parsing utilities
//!
//! Shared path bounds calculation used by both WASM and native renderers.

/// Parse SVG path d attribute and compute bounding box (x, y, width, height)
pub fn parse_path_bounds(d: &str) -> (f32, f32, f32, f32) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let (mut cur_x, mut cur_y, mut start_x, mut start_y) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    let (mut last_ctrl_x, mut last_ctrl_y) = (0.0_f32, 0.0_f32);
    let mut last_cmd = ' ';

    let mut track = |x: f32, y: f32| { min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x); max_y = max_y.max(y); };
    let nums: Vec<f32> = extract_numbers(d);
    let cmds: Vec<char> = d.chars().filter(|c| matches!(c, 'M'|'m'|'L'|'l'|'H'|'h'|'V'|'v'|'C'|'c'|'S'|'s'|'Q'|'q'|'T'|'t'|'A'|'a'|'Z'|'z')).collect();
    let mut idx = 0;

    for cmd in cmds {
        match cmd {
            'M' if idx + 1 < nums.len() => { cur_x = nums[idx]; cur_y = nums[idx + 1]; start_x = cur_x; start_y = cur_y; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'm' if idx + 1 < nums.len() => { cur_x += nums[idx]; cur_y += nums[idx + 1]; start_x = cur_x; start_y = cur_y; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'L' if idx + 1 < nums.len() => { cur_x = nums[idx]; cur_y = nums[idx + 1]; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'l' if idx + 1 < nums.len() => { cur_x += nums[idx]; cur_y += nums[idx + 1]; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'H' if idx < nums.len() => { cur_x = nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'h' if idx < nums.len() => { cur_x += nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'V' if idx < nums.len() => { cur_y = nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'v' if idx < nums.len() => { cur_y += nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'C' if idx + 5 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2, x3, y3) = (nums[idx], nums[idx+1], nums[idx+2], nums[idx+3], nums[idx+4], nums[idx+5]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                cur_x = x3; cur_y = y3; last_ctrl_x = x2; last_ctrl_y = y2; idx += 6;
            }
            'c' if idx + 5 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2, x3, y3) = (cur_x + nums[idx], cur_y + nums[idx+1], cur_x + nums[idx+2], cur_y + nums[idx+3], cur_x + nums[idx+4], cur_y + nums[idx+5]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                last_ctrl_x = x2; last_ctrl_y = y2; cur_x = x3; cur_y = y3; idx += 6;
            }
            'S' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'C'|'c'|'S'|'s') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2, x3, y3) = (nums[idx], nums[idx+1], nums[idx+2], nums[idx+3]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                last_ctrl_x = x2; last_ctrl_y = y2; cur_x = x3; cur_y = y3; idx += 4;
            }
            's' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'C'|'c'|'S'|'s') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2, x3, y3) = (cur_x + nums[idx], cur_y + nums[idx+1], cur_x + nums[idx+2], cur_y + nums[idx+3]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                last_ctrl_x = x2; last_ctrl_y = y2; cur_x = x3; cur_y = y3; idx += 4;
            }
            'Q' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2) = (nums[idx], nums[idx+1], nums[idx+2], nums[idx+3]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 4;
            }
            'q' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2) = (cur_x + nums[idx], cur_y + nums[idx+1], cur_x + nums[idx+2], cur_y + nums[idx+3]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 4;
            }
            'T' if idx + 1 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'Q'|'q'|'T'|'t') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2) = (nums[idx], nums[idx+1]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 2;
            }
            't' if idx + 1 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'Q'|'q'|'T'|'t') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2) = (cur_x + nums[idx], cur_y + nums[idx+1]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 2;
            }
            'A' if idx + 6 < nums.len() => {
                let (rx, ry, phi, large_arc, sweep) = (nums[idx].abs(), nums[idx+1].abs(), nums[idx+2], nums[idx+3] != 0.0, nums[idx+4] != 0.0);
                let (x2, y2) = (nums[idx+5], nums[idx+6]);
                arc_bounds(cur_x, cur_y, rx, ry, phi, large_arc, sweep, x2, y2, &mut track);
                cur_x = x2; cur_y = y2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; idx += 7;
            }
            'a' if idx + 6 < nums.len() => {
                let (rx, ry, phi, large_arc, sweep) = (nums[idx].abs(), nums[idx+1].abs(), nums[idx+2], nums[idx+3] != 0.0, nums[idx+4] != 0.0);
                let (x2, y2) = (cur_x + nums[idx+5], cur_y + nums[idx+6]);
                arc_bounds(cur_x, cur_y, rx, ry, phi, large_arc, sweep, x2, y2, &mut track);
                cur_x = x2; cur_y = y2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; idx += 7;
            }
            'Z' | 'z' => { cur_x = start_x; cur_y = start_y; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            _ => {}
        }
        last_cmd = cmd;
    }
    if min_x == f32::MAX { (0.0, 0.0, 0.0, 0.0) } else { (min_x, min_y, max_x - min_x, max_y - min_y) }
}

/// Compute cubic Bezier bounds by finding extrema
fn cubic_bezier_bounds(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, track: &mut impl FnMut(f32, f32)) {
    track(x0, y0); track(x3, y3);
    for (p0, p1, p2, p3, is_x) in [(x0, x1, x2, x3, true), (y0, y1, y2, y3, false)] {
        let a = -p0 + 3.0*p1 - 3.0*p2 + p3;
        let b = 2.0*(p0 - 2.0*p1 + p2);
        let c = -p0 + p1;
        for t in solve_quadratic(a, b, c) {
            if t > 0.0 && t < 1.0 {
                let val = cubic_at(t, p0, p1, p2, p3);
                if is_x { track(val, cubic_at(t, y0, y1, y2, y3)); }
                else { track(cubic_at(t, x0, x1, x2, x3), val); }
            }
        }
    }
}

/// Compute quadratic Bezier bounds by finding extrema
fn quadratic_bezier_bounds(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, track: &mut impl FnMut(f32, f32)) {
    track(x0, y0); track(x2, y2);
    for (p0, p1, p2, is_x) in [(x0, x1, x2, true), (y0, y1, y2, false)] {
        let denom = p0 - 2.0*p1 + p2;
        if denom.abs() > 1e-10 {
            let t = (p0 - p1) / denom;
            if t > 0.0 && t < 1.0 {
                let val = quadratic_at(t, p0, p1, p2);
                if is_x { track(val, quadratic_at(t, y0, y1, y2)); }
                else { track(quadratic_at(t, x0, x1, x2), val); }
            }
        }
    }
}

/// Compute arc bounds using endpoint parameterization
fn arc_bounds(x1: f32, y1: f32, mut rx: f32, mut ry: f32, phi_deg: f32, large_arc: bool, sweep: bool, x2: f32, y2: f32, track: &mut impl FnMut(f32, f32)) {
    track(x1, y1); track(x2, y2);
    if rx < 1e-10 || ry < 1e-10 { return; }

    let phi = phi_deg.to_radians();
    let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;

    let lambda = (x1p / rx).powi(2) + (y1p / ry).powi(2);
    if lambda > 1.0 { let s = lambda.sqrt(); rx *= s; ry *= s; }

    let sq = ((rx*ry).powi(2) - (rx*y1p).powi(2) - (ry*x1p).powi(2)) / ((rx*y1p).powi(2) + (ry*x1p).powi(2));
    let coef = if large_arc != sweep { sq.max(0.0).sqrt() } else { -sq.max(0.0).sqrt() };
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;

    let theta1 = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let mut dtheta = (((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx) - theta1).rem_euclid(std::f32::consts::TAU);
    if !sweep { dtheta -= std::f32::consts::TAU; }

    for angle in [0.0_f32, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, 3.0 * std::f32::consts::FRAC_PI_2] {
        let t = (angle - theta1).rem_euclid(std::f32::consts::TAU);
        if (sweep && t <= dtheta) || (!sweep && t >= dtheta.abs() - std::f32::consts::TAU) || dtheta.abs() >= std::f32::consts::TAU - 1e-6 {
            let px = cx + rx * angle.cos() * cos_phi - ry * angle.sin() * sin_phi;
            let py = cy + rx * angle.cos() * sin_phi + ry * angle.sin() * cos_phi;
            track(px, py);
        }
    }
}

#[inline] fn cubic_at(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let mt = 1.0 - t;
    mt*mt*mt*p0 + 3.0*mt*mt*t*p1 + 3.0*mt*t*t*p2 + t*t*t*p3
}

#[inline] fn quadratic_at(t: f32, p0: f32, p1: f32, p2: f32) -> f32 {
    let mt = 1.0 - t;
    mt*mt*p0 + 2.0*mt*t*p1 + t*t*p2
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Vec<f32> {
    if a.abs() < 1e-10 { return if b.abs() < 1e-10 { vec![] } else { vec![-c / b] }; }
    let disc = b*b - 4.0*a*c;
    if disc < 0.0 { vec![] }
    else if disc < 1e-10 { vec![-b / (2.0 * a)] }
    else { let sq = disc.sqrt(); vec![(-b - sq) / (2.0 * a), (-b + sq) / (2.0 * a)] }
}

fn extract_numbers(d: &str) -> Vec<f32> {
    let mut nums = Vec::new();
    let mut buf = String::new();
    for c in d.chars() {
        if c.is_ascii_digit() || c == '.' || (c == '-' && buf.is_empty()) || (c == '-' && buf.ends_with('e')) { buf.push(c); }
        else if c == 'e' || c == 'E' { buf.push('e'); }
        else { if !buf.is_empty() { if let Ok(n) = buf.parse::<f32>() { nums.push(n); } buf.clear(); } if c == '-' { buf.push(c); } }
    }
    if !buf.is_empty() { if let Ok(n) = buf.parse::<f32>() { nums.push(n); } }
    nums
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_path_bounds_line() {
        let (x, y, w, h) = parse_path_bounds("M0 0 L100 50");
        assert!((x - 0.0).abs() < 0.01 && (y - 0.0).abs() < 0.01);
        assert!((w - 100.0).abs() < 0.01 && (h - 50.0).abs() < 0.01);
    }

    #[test] fn test_path_bounds_cubic() {
        let (x, y, w, h) = parse_path_bounds("M0 50 C0 0, 100 0, 100 50");
        assert!(y < 50.0);
        assert!(x >= -0.01 && (x + w) <= 100.01);
    }

    #[test] fn test_path_bounds_quadratic() {
        let (x, y, w, h) = parse_path_bounds("M0 0 Q50 100, 100 0");
        assert!(y >= -0.01 && (y + h) >= 45.0);
    }

    #[test] fn test_path_bounds_arc() {
        let (x, y, w, h) = parse_path_bounds("M0 50 A50 50 0 0 1 100 50");
        assert!((w - 100.0).abs() < 1.0);
        assert!(y <= 50.0 && (y + h) >= 50.0);
    }

    #[test] fn test_path_bounds_smooth_cubic() {
        let (x, y, w, h) = parse_path_bounds("M0 0 C10 20 20 20 30 0 S50 -20 60 0");
        assert!(x >= -0.01 && (x + w) <= 60.01);
    }

    #[test] fn test_path_bounds_smooth_quadratic() {
        let (x, y, w, h) = parse_path_bounds("M0 0 Q25 50 50 0 T100 0");
        assert!(x >= -0.01 && (x + w) <= 100.01);
        assert!((y + h) >= 20.0);
    }
}

