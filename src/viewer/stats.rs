use std::collections::HashSet;
use std::f64::consts::PI;
use std::sync::atomic::Ordering;

use crate::types::{SHOW_BACK_EDGES, SHOW_STATS_OVERLAY, ShapeId, global_shape_registry};

use super::camera::OrbitCamera;

struct ViewPreset {
    name: &'static str,
    yaw: f64,
    pitch: f64,
}

const VIEW_PRESETS: &[ViewPreset] = &[
    ViewPreset {
        name: "Front",
        yaw: 0.0,
        pitch: 0.0,
    },
    ViewPreset {
        name: "Back",
        yaw: PI,
        pitch: 0.0,
    },
    ViewPreset {
        name: "Top",
        yaw: 0.0,
        pitch: PI / 2.0,
    },
    ViewPreset {
        name: "Bottom",
        yaw: 0.0,
        pitch: -PI / 2.0,
    },
    ViewPreset {
        name: "Right",
        yaw: PI / 2.0,
        pitch: 0.0,
    },
    ViewPreset {
        name: "Left",
        yaw: -PI / 2.0,
        pitch: 0.0,
    },
    ViewPreset {
        name: "Iso",
        yaw: PI / 4.0,
        pitch: 0.615479708670387,
    },
];

const VIEW_TOLERANCE: f64 = 0.15;

pub struct Stats {
    pub frame_times: [f64; 60],
    pub frame_cursor: usize,
    pub frame_count: usize,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            frame_times: [0.0; 60],
            frame_cursor: 0,
            frame_count: 0,
        }
    }

    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        camera: &OrbitCamera,
        selected_ids: &HashSet<ShapeId>,
        dt: f64,
    ) {
        if !SHOW_STATS_OVERLAY.load(Ordering::Relaxed) {
            return;
        }

        self.frame_times[self.frame_cursor] = dt;
        self.frame_cursor = (self.frame_cursor + 1) % 60;
        self.frame_count = self.frame_count.saturating_add(1).min(60);

        let frame_sum: f64 = self.frame_times[..self.frame_count].iter().sum();
        let avg_dt = frame_sum / self.frame_count.max(1) as f64;
        let fps = if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 };
        let frame_ms = avg_dt * 1000.0;

        let yaw_deg = {
            let mut d = camera.yaw.to_degrees() % 360.0;
            if d < 0.0 {
                d += 360.0;
            }
            d
        };
        let pitch_deg = camera.pitch.to_degrees();
        let view_name = detect_view_preset(camera.yaw, camera.pitch);
        let proj_name = if camera.perspective {
            "Perspective"
        } else {
            "Orthographic"
        };

        let registry = global_shape_registry();
        let all = registry.all_shapes();
        let vis_count = all.iter().filter(|e| e.visible).count();
        let hidden = all.len().saturating_sub(vis_count);

        let mut total_tris = 0usize;
        let mut total_verts = 0usize;
        for entry in &all {
            if entry.visible {
                if let Some(ref mesh) = entry.mesh {
                    total_tris += mesh.indices.len() / 3;
                    total_verts += mesh.vertices.len();
                }
            }
        }

        let selected_str = selected_ids
            .iter()
            .next()
            .map_or(String::new(), |id| format!("#{}", id));

        let back_edges = if SHOW_BACK_EDGES.load(Ordering::Relaxed) {
            "ON"
        } else {
            "OFF"
        };

        let proj_indicator = if camera.perspective { "P" } else { "O" };

        egui::Window::new("Stats")
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(14.0, 14.0))
            .resizable(false)
            .min_width(220.0)
            .show(ctx, |ui| {
                egui::Grid::new("stats_grid")
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        ui.strong("VIEW");
                        ui.label(view_name);
                        ui.end_row();

                        ui.label("  Yaw:");
                        ui.label(format!("{:.1}°", yaw_deg));
                        ui.end_row();

                        ui.label("  Pitch:");
                        ui.label(format!("{:.1}°", pitch_deg));
                        ui.end_row();

                        ui.label("  Zoom:");
                        ui.label(format!("{:.2}", camera.radius));
                        ui.end_row();

                        ui.label("  Proj:");
                        ui.label(format!("{}  [{}]", proj_name, proj_indicator));
                        ui.end_row();

                        ui.strong("SHAPES");
                        ui.label("");
                        ui.end_row();

                        ui.label("  Total:");
                        ui.label(format!("{}", all.len()));
                        ui.end_row();

                        ui.label("  Visible:");
                        ui.label(format!("{}", vis_count));
                        ui.end_row();

                        ui.label("  Hidden:");
                        ui.label(format!("{}", hidden));
                        ui.end_row();

                        if !selected_str.is_empty() {
                            ui.label("  Selected:");
                            ui.label(&selected_str);
                            ui.end_row();
                        }

                        ui.strong("GEOMETRY");
                        ui.label("");
                        ui.end_row();

                        ui.label("  Triangles:");
                        ui.label(format_num(total_tris));
                        ui.end_row();

                        ui.label("  Vertices:");
                        ui.label(format_num(total_verts));
                        ui.end_row();

                        ui.strong("TOGGLES");
                        ui.label("");
                        ui.end_row();

                        ui.label("  Back edges:");
                        ui.label(format!("{}  [X]", back_edges));
                        ui.end_row();

                        ui.strong("PERFORMANCE");
                        ui.label("");
                        ui.end_row();

                        ui.label("  FPS:");
                        ui.label(format!("{:.0}", fps));
                        ui.end_row();

                        ui.label("  Frame:");
                        ui.label(format!("{:.2} ms", frame_ms));
                        ui.end_row();
                    });
            });
    }
}

fn detect_view_preset(yaw: f64, pitch: f64) -> &'static str {
    for preset in VIEW_PRESETS {
        let dy = (yaw - preset.yaw).abs();
        let dp = (pitch - preset.pitch).abs();
        if dy < VIEW_TOLERANCE && dp < VIEW_TOLERANCE {
            return preset.name;
        }
    }
    "Custom"
}

fn format_num(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}
