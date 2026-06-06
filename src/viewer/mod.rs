use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

pub mod app;
pub mod camera;
pub mod gizmo;
pub mod pick;

/// Messages from the viewer thread back to the REPL thread.
#[derive(Debug, Clone)]
pub enum ViewerToRepl {
    ShapeSelected,
    ShapeDeselected,
    ViewerClosed,
}

/// Handle for controlling the viewer thread from the REPL thread.
pub struct ViewerHandle {
    join_handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl ViewerHandle {
    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ViewerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Spawn the viewer on a background thread.
/// Returns a `ViewerHandle` that the REPL thread can use to receive selection events.
pub fn spawn_viewer() -> ViewerHandle {
    let (viewer_tx, _viewer_rx) = mpsc::channel::<ViewerToRepl>();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let handle = thread::Builder::new()
        .name("wgpu-viewer".into())
        .spawn(move || {
            app::run_viewer(viewer_tx, running_clone);
        })
        .expect("failed to spawn viewer thread");

    ViewerHandle {
        join_handle: Some(handle),
        running,
    }
}
