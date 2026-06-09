use std::sync::atomic::Ordering;

use crate::types::SHOW_HELP_OVERLAY;

pub struct Help;

impl Help {
    pub fn new() -> Self {
        Self
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        let mut visible = SHOW_HELP_OVERLAY.load(Ordering::Relaxed);
        if !visible {
            return;
        }

        egui::Window::new("Help")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
            .resizable(true)
            .default_width(460.0)
            .open(&mut visible)
            .show(ctx, |ui| {
                ui.strong("Keyboard Shortcuts");
                ui.separator();
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([16.0, 2.0])
                    .show(ui, |ui| {
                        ui.monospace("Esc");
                        ui.label("Close this help");
                        ui.end_row();
                        ui.monospace("h");
                        ui.label("Toggle this help");
                        ui.end_row();
                        ui.monospace("p / o");
                        ui.label("Toggle perspective / orthographic");
                        ui.end_row();
                        ui.monospace("x");
                        ui.label("Toggle back edges");
                        ui.end_row();
                        ui.monospace("Ctrl+1");
                        ui.label("Front / Back view");
                        ui.end_row();
                        ui.monospace("Ctrl+3");
                        ui.label("Right / Left view");
                        ui.end_row();
                        ui.monospace("Ctrl+7");
                        ui.label("Top / Bottom view");
                        ui.end_row();
                        ui.monospace("Shift+Scroll");
                        ui.label("Dolly forward / backward");
                        ui.end_row();
                        ui.monospace("Shift+RMB drag");
                        ui.label("Dolly forward / backward");
                        ui.end_row();
                        ui.monospace("Ctrl+Shift+");
                        ui.label("Toggle stats overlay");
                        ui.end_row();
                        ui.monospace("  Alt+S");
                        ui.label("");
                        ui.end_row();
                        ui.monospace("Ctrl+Q");
                        ui.label("Quit application");
                        ui.end_row();
                    });

                ui.add_space(8.0);

                ui.strong("REPL Documentation");
                ui.separator();
                egui::Grid::new("repl_grid")
                    .num_columns(2)
                    .spacing([16.0, 2.0])
                    .show(ui, |ui| {
                        ui.monospace("(doc 'sym)");
                        ui.label("Show docs for a function");
                        ui.end_row();
                        ui.monospace("(apropos p)");
                        ui.label("Search functions by pattern");
                        ui.end_row();
                        ui.monospace("(group)");
                        ui.label("List all function groups");
                        ui.end_row();
                        ui.monospace("(group \"n\")");
                        ui.label("List functions in a group");
                        ui.end_row();
                        ui.monospace("(cad-fns)");
                        ui.label("List all rojcad functions");
                        ui.end_row();
                        ui.monospace("(all-fns)");
                        ui.label("List all available functions");
                        ui.end_row();
                    });
                ui.label("Full API docs generated with (dump-docs).");

                ui.add_space(8.0);

                ui.strong("Connecting to REPL");
                ui.separator();
                ui.label("The TCP REPL listens on port 9365 (default).");
                ui.label("Connect with:");
                ui.monospace("  nc 127.0.0.1 9365");

                ui.add_space(8.0);

                ui.strong("Command Line Arguments");
                ui.separator();
                egui::Grid::new("cli_grid")
                    .num_columns(2)
                    .spacing([16.0, 2.0])
                    .show(ui, |ui| {
                        ui.monospace("--headless");
                        ui.label("Run without 3D viewer");
                        ui.end_row();
                        ui.monospace("--port <N>");
                        ui.label("Set REPL port (default 9365)");
                        ui.end_row();
                        ui.monospace("--eval <E>");
                        ui.label("Run Janet code on startup then exit");
                        ui.end_row();
                        ui.monospace("--width <PX>");
                        ui.label("Window width (implies windowed, default maximized)");
                        ui.end_row();
                        ui.monospace("--height <PX>");
                        ui.label("Window height (implies windowed, default maximized)");
                        ui.end_row();
                    });
            });

        SHOW_HELP_OVERLAY.store(visible, Ordering::Relaxed);
    }
}
