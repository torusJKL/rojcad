use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::types::{MeshData, ShapeId};

pub mod app;
pub mod camera;
pub mod gizmo;
pub mod pick;

/// Messages from the viewer thread back to the REPL thread.
#[derive(Debug, Clone)]
pub enum ViewerToRepl {
    ShapeSelected(ShapeId),
    ShapeDeselected,
    ViewerClosed,
}

/// Messages from the REPL thread to the viewer thread.
#[derive(Debug, Clone)]
pub enum ReplToViewer {
    UpdateShapes(Vec<(ShapeId, MeshData, Vec<Vec<[f64; 3]>>)>),
    RemoveShape(ShapeId),
    ClearAll,
}

/// Handle for controlling the viewer thread from the REPL thread.
pub struct ViewerHandle {
    pub repl_tx: Sender<ReplToViewer>,
    pub viewer_rx: Receiver<ViewerToRepl>,
    pub join_handle: Option<JoinHandle<()>>,
    pub running: Arc<AtomicBool>,
}

impl ViewerHandle {
    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawn the viewer on a background thread.
/// Returns a `ViewerHandle` that the REPL thread can use to communicate.
pub fn spawn_viewer() -> ViewerHandle {
    let (repl_tx, repl_rx) = mpsc::channel::<ReplToViewer>();
    let (viewer_tx, viewer_rx) = mpsc::channel::<ViewerToRepl>();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let handle = thread::Builder::new()
        .name("wgpu-viewer".into())
        .spawn(move || {
            app::run_viewer(repl_rx, viewer_tx, running_clone);
        })
        .expect("failed to spawn viewer thread");

    ViewerHandle {
        repl_tx,
        viewer_rx,
        join_handle: Some(handle),
        running,
    }
}
