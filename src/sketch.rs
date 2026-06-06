use glam::DVec3;
use opencascade::primitives::{Edge, Wire};
use opencascade::workplane::Workplane;

/// A drawing command in workplane-local 2D coordinates.
#[derive(Clone, Debug)]
pub enum SketchCommand {
    Move(f64, f64),
    Line(f64, f64),
    Arc(f64, f64, f64, f64),
}

/// Functional 2D sketch builder.
///
/// Each operation consumes `self` and returns a new `SketchData` —
/// no mutation. Edges are built only when `close()` or `build_wire()` is called.
pub struct SketchData {
    pub cursor: (f64, f64),
    pub first_point: Option<(f64, f64)>,
    pub first_point_set: bool,
    pub commands: Vec<SketchCommand>,
    pub workplane: Workplane,
}

impl SketchData {
    pub fn new(workplane: Workplane) -> Self {
        Self {
            cursor: (0.0, 0.0),
            first_point: None,
            first_point_set: false,
            commands: Vec::new(),
            workplane,
        }
    }

    fn add_command(&self, cmd: SketchCommand) -> Self {
        let mut commands = self.commands.clone();
        let mut first_point = self.first_point;
        let mut first_point_set = self.first_point_set;
        let cursor = match &cmd {
            SketchCommand::Line(x, y) => {
                if !first_point_set {
                    first_point = Some(self.cursor);
                    first_point_set = true;
                }
                (*x, *y)
            }
            SketchCommand::Arc(_, _, x3, y3) => {
                if !first_point_set {
                    first_point = Some(self.cursor);
                    first_point_set = true;
                }
                (*x3, *y3)
            }
            SketchCommand::Move(x, y) => (*x, *y),
        };
        commands.push(cmd);
        Self {
            cursor,
            first_point,
            first_point_set,
            commands,
            workplane: self.workplane.clone(),
        }
    }

    pub fn move_to(&self, x: f64, y: f64) -> Self {
        self.add_command(SketchCommand::Move(x, y))
    }

    pub fn line_to(&self, x: f64, y: f64) -> Self {
        self.add_command(SketchCommand::Line(x, y))
    }

    pub fn line_dx(&self, dx: f64) -> Self {
        self.add_command(SketchCommand::Line(self.cursor.0 + dx, self.cursor.1))
    }

    pub fn line_dy(&self, dy: f64) -> Self {
        self.add_command(SketchCommand::Line(self.cursor.0, self.cursor.1 + dy))
    }

    pub fn line_dx_dy(&self, dx: f64, dy: f64) -> Self {
        self.add_command(SketchCommand::Line(self.cursor.0 + dx, self.cursor.1 + dy))
    }

    pub fn arc_to(&self, x2: f64, y2: f64, x3: f64, y3: f64) -> Self {
        self.add_command(SketchCommand::Arc(x2, y2, x3, y3))
    }

    fn to_world(&self, x: f64, y: f64) -> DVec3 {
        self.workplane.to_world_pos(DVec3::new(x, y, 0.0))
    }

    fn build_edges(&self) -> Vec<Edge> {
        let mut edges: Vec<Edge> = Vec::new();
        let mut cursor: Option<(f64, f64)> = None;
        for cmd in &self.commands {
            match cmd {
                SketchCommand::Move(x, y) => {
                    cursor = Some((*x, *y));
                }
                SketchCommand::Line(x, y) => {
                    if let Some((cx, cy)) = cursor {
                        let p1 = self.to_world(cx, cy);
                        let p2 = self.to_world(*x, *y);
                        edges.push(Edge::segment(p1, p2));
                    }
                    cursor = Some((*x, *y));
                }
                SketchCommand::Arc(x2, y2, x3, y3) => {
                    if let Some((cx, cy)) = cursor {
                        let p1 = self.to_world(cx, cy);
                        let p2 = self.to_world(*x2, *y2);
                        let p3 = self.to_world(*x3, *y3);
                        edges.push(Edge::arc(p1, p2, p3));
                    }
                    cursor = Some((*x3, *y3));
                }
            }
        }
        edges
    }

    pub fn close(&self) -> Wire {
        let mut edges = self.build_edges();
        if let (Some(fp), Some(last_pt)) = (self.first_point, edges.last().map(|e| e.end_point())) {
            let fp_world = self.to_world(fp.0, fp.1);
            if (last_pt - fp_world).length() > 1e-10 {
                edges.push(Edge::segment(last_pt, fp_world));
            }
        }
        Wire::from_edges(&edges)
    }

    pub fn build_wire(&self) -> Wire {
        let edges = self.build_edges();
        Wire::from_edges(&edges)
    }
}
