use pathfinding::prelude::astar;

const CELL: f64 = 8.0;
const TURN_PENALTY: u32 = 30;
const CLEARANCE: f64 = 5.0;
pub const CORNER_R: f64 = 5.0;

#[derive(Clone, Eq, Hash, PartialEq)]
struct GridState {
    c: i32,
    r: i32,
    dc: i8,
    dr: i8,
}

pub struct RoutingGrid {
    blocked: Vec<bool>,
    cols: i32,
    rows: i32,
}

impl RoutingGrid {
    /// Create a grid covering [0..width] × [0..height] in pixels.
    pub fn new(width: f64, height: f64) -> Self {
        let cols = (width / CELL).ceil() as i32 + 4;
        let rows = (height / CELL).ceil() as i32 + 4;
        let size = (cols * rows) as usize;
        RoutingGrid {
            blocked: vec![false; size],
            cols,
            rows,
        }
    }

    /// Mark a block bounding box (in canvas coords) as impassable.
    pub fn mark_obstacle(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let x0 = ((x - CLEARANCE) / CELL).floor() as i32;
        let y0 = ((y - CLEARANCE) / CELL).floor() as i32;
        let x1 = ((x + w + CLEARANCE) / CELL).ceil() as i32;
        let y1 = ((y + h + CLEARANCE) / CELL).ceil() as i32;

        let x0 = x0.max(0);
        let y0 = y0.max(0);
        let x1 = x1.min(self.cols - 1);
        let y1 = y1.min(self.rows - 1);

        for r in y0..=y1 {
            for c in x0..=x1 {
                let idx = (r * self.cols + c) as usize;
                if idx < self.blocked.len() {
                    self.blocked[idx] = true;
                }
            }
        }
    }

    /// Unblock a specific pixel coordinate (used for port anchors in clearance zones).
    pub fn unblock(&mut self, px: f64, py: f64) {
        let (c, r) = self.to_grid(px, py);
        if self.in_bounds(c, r) {
            let idx = (r * self.cols + c) as usize;
            self.blocked[idx] = false;
        }
    }

    fn is_blocked(&self, c: i32, r: i32) -> bool {
        if !self.in_bounds(c, r) {
            return true;
        }
        let idx = (r * self.cols + c) as usize;
        self.blocked[idx]
    }

    fn in_bounds(&self, c: i32, r: i32) -> bool {
        c >= 0 && r >= 0 && c < self.cols && r < self.rows
    }

    fn to_grid(&self, px: f64, py: f64) -> (i32, i32) {
        ((px / CELL).round() as i32, (py / CELL).round() as i32)
    }

    fn to_pixel(&self, c: i32, r: i32) -> (f64, f64) {
        (c as f64 * CELL, r as f64 * CELL)
    }

    /// Run A* from (x1,y1) to (x2,y2) and return simplified pixel waypoints.
    /// Falls back to [(x1,y1),(x2,y2)] if no path found.
    pub fn route(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> Vec<(f64, f64)> {
        let (gc1, gr1) = self.to_grid(x1, y1);
        let (gc2, gr2) = self.to_grid(x2, y2);

        if gc1 == gc2 && gr1 == gr2 {
            return vec![(x1, y1), (x2, y2)];
        }

        // Initial direction: prefer horizontal exit
        let start = GridState { c: gc1, r: gr1, dc: 0, dr: 0 };
        let goal_c = gc2;
        let goal_r = gr2;

        let result = astar(
            &start,
            |s| {
                let dirs: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                let mut successors = Vec::with_capacity(4);
                for (ndc, ndr) in dirs {
                    let nc = s.c + ndc as i32;
                    let nr = s.r + ndr as i32;
                    if self.in_bounds(nc, nr) && !self.is_blocked(nc, nr) {
                        let turn_cost = if s.dc != 0 || s.dr != 0 {
                            if s.dc != ndc || s.dr != ndr { TURN_PENALTY } else { 0 }
                        } else {
                            0
                        };
                        let move_cost: u32 = 10 + turn_cost;
                        successors.push((GridState { c: nc, r: nr, dc: ndc, dr: ndr }, move_cost));
                    }
                }
                successors
            },
            |s| {
                let dc = (s.c - goal_c).unsigned_abs();
                let dr = (s.r - goal_r).unsigned_abs();
                (dc + dr) * 10
            },
            |s| s.c == goal_c && s.r == goal_r,
        );

        match result {
            Some((path, _cost)) => {
                let grid_pts: Vec<(f64, f64)> = path
                    .iter()
                    .map(|s| self.to_pixel(s.c, s.r))
                    .collect();
                let mut pts = simplify_path(grid_pts);
                ortho_snap(&mut pts, &path, x1, y1, x2, y2);
                pts
            }
            None => vec![(x1, y1), (x2, y2)],
        }
    }
}

/// Remove collinear intermediate points from an ordered list of pixel coords.
fn simplify_path(pts: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    if pts.len() <= 2 {
        return pts;
    }
    let mut result = Vec::with_capacity(pts.len());
    result.push(pts[0]);
    for i in 1..pts.len() - 1 {
        let (ax, ay) = pts[i - 1];
        let (bx, by) = pts[i];
        let (cx, cy) = pts[i + 1];
        let dc1 = isign(bx - ax);
        let dr1 = isign(by - ay);
        let dc2 = isign(cx - bx);
        let dr2 = isign(cy - by);
        if dc1 != dc2 || dr1 != dr2 {
            result.push((bx, by));
        }
    }
    result.push(*pts.last().unwrap());
    result
}

/// Snap endpoint pixels to exact coords and re-align adjacent waypoints so
/// the first and last segments remain strictly orthogonal (H or V).
/// Departure/arrival direction comes from the A* path's first and last step.
fn ortho_snap(
    pts: &mut Vec<(f64, f64)>,
    path: &[GridState],
    x1: f64, y1: f64,
    x2: f64, y2: f64,
) {
    let n = pts.len();
    if n == 0 { return; }

    pts[0] = (x1, y1);
    pts[n - 1] = (x2, y2);

    if n <= 2 { return; }

    // Align second point: use the first step's direction from the A* path
    if path.len() >= 2 {
        let dep = &path[1];
        if dep.dr == 0 {
            // Horizontal departure → second waypoint shares y with start
            pts[1].1 = y1;
        } else {
            // Vertical departure → second waypoint shares x with start
            pts[1].0 = x1;
        }
    }

    // Align second-to-last point: use the last step's direction
    if path.len() >= 2 {
        let arr = path.last().unwrap();
        if arr.dr == 0 {
            // Horizontal arrival → second-to-last shares y with end
            pts[n - 2].1 = y2;
        } else {
            // Vertical arrival → second-to-last shares x with end
            pts[n - 2].0 = x2;
        }
    }

    // Re-simplify: removes points that became collinear after adjustment
    let simplified = simplify_path(pts.clone());
    *pts = simplified;
}

fn isign(x: f64) -> i32 {
    if x > 0.1 { 1 } else if x < -0.1 { -1 } else { 0 }
}

/// Render waypoints as an SVG path `d` attribute string with rounded orthogonal corners.
/// Corner radius is CORNER_R (5px), reduced if segment is shorter.
pub fn waypoints_to_svg(pts: &[(f64, f64)]) -> String {
    if pts.is_empty() {
        return String::new();
    }
    if pts.len() == 1 {
        return format!("M {:.1} {:.1}", pts[0].0, pts[0].1);
    }

    let mut d = format!("M {:.1} {:.1}", pts[0].0, pts[0].1);

    for i in 1..pts.len() {
        let (bx, by) = pts[i];
        if i == pts.len() - 1 {
            // Last point: straight line
            d.push_str(&format!(" L {:.1} {:.1}", bx, by));
        } else {
            // Intermediate corner: compute approach and departure with arc
            let (ax, ay) = pts[i - 1];
            let (cx, cy) = pts[i + 1];

            let in_len = ((bx - ax) * (bx - ax) + (by - ay) * (by - ay)).sqrt();
            let out_len = ((cx - bx) * (cx - bx) + (cy - by) * (cy - by)).sqrt();
            let r = CORNER_R.min(in_len / 2.0).min(out_len / 2.0);

            if r < 0.5 {
                // Corner too tight for rounding, use a sharp corner
                d.push_str(&format!(" L {:.1} {:.1}", bx, by));
                continue;
            }

            let d1x = fsign(bx - ax);
            let d1y = fsign(by - ay);
            let d2x = fsign(cx - bx);
            let d2y = fsign(cy - by);

            // Approach point (just before the corner)
            let apx = bx - d1x * r;
            let apy = by - d1y * r;
            // Departure point (just after the corner)
            let dpx = bx + d2x * r;
            let dpy = by + d2y * r;

            // Sweep direction: cross product of incoming and outgoing direction
            let cross = d1x * d2y - d1y * d2x;
            let sweep = if cross > 0.0 { 1 } else { 0 };

            d.push_str(&format!(
                " L {:.1} {:.1} A {:.1} {:.1} 0 0 {} {:.1} {:.1}",
                apx, apy, r, r, sweep, dpx, dpy
            ));
        }
    }

    d
}

fn fsign(x: f64) -> f64 {
    if x > 0.1 { 1.0 } else if x < -0.1 { -1.0 } else { 0.0 }
}
