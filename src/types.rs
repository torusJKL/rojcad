//! Shape data type — a Janet abstract type wrapping an OCCT Shape
//! with metadata for visibility, color, etc.
//!
//! Also defines the shared `ShapeRegistry` used to synchronize shape
//! state between the REPL thread and the viewer thread.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

/// Global generation counter for change tracking.
/// Incremented on every ShapeRegistry write.
/// The viewer reads this to detect changes since last frame.
pub static REGISTRY_GENERATION: AtomicU64 = AtomicU64::new(0);

use opencascade::primitives::Shape;

/// Last selected shape ID, used to propagate selection events to Janet.
/// 0 = no event pending, u64::MAX = deselected, other = selected shape ID.
pub static LAST_SELECTION: AtomicU64 = AtomicU64::new(0);

/// Monotonically increasing shape ID counter.
fn next_shape_id() -> ShapeId {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Global shape registry shared between the REPL thread and viewer thread.
pub fn global_shape_registry() -> &'static ShapeRegistry {
    static REGISTRY: OnceLock<ShapeRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ShapeRegistry::new)
}

/// Unique identifier for a shape in the registry.
pub type ShapeId = u64;

/// Tessellated mesh data produced by OCCT's `shape.mesh()`.
#[derive(Debug, Clone, Default)]
pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// An entry in the shared shape registry.
///
/// Carries the tessellated mesh data and edge polylines so the viewer
/// thread can build GPU buffers without touching OCCT objects.
#[derive(Debug, Clone)]
pub struct ShapeEntry {
    pub shape_id: ShapeId,
    pub mesh: Option<MeshData>,
    pub edge_polylines: Vec<Vec<[f64; 3]>>,
    pub visible: bool,
    pub color: Option<[f64; 3]>,
}

/// Thread-safe shared shape registry.
///
/// The REPL thread writes to this registry after CAD operations.
/// The viewer thread reads from it each frame to determine what to render.
pub struct ShapeRegistry {
    inner: Arc<RwLock<HashMap<ShapeId, ShapeEntry>>>,
}

impl ShapeRegistry {
    /// Create a new empty shape registry.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new shape entry.
    pub fn register(&self, entry: ShapeEntry) {
        let mut map = self.inner.write().expect("shape registry lock poisoned");
        map.insert(entry.shape_id, entry);
        REGISTRY_GENERATION.fetch_add(1, Ordering::SeqCst);
    }

    /// Update an existing shape's mesh and edge data.
    pub fn update(&self, shape_id: ShapeId, mesh: Option<MeshData>, edge_polylines: Vec<Vec<[f64; 3]>>) {
        let mut map = self.inner.write().expect("shape registry lock poisoned");
        if let Some(entry) = map.get_mut(&shape_id) {
            entry.mesh = mesh;
            entry.edge_polylines = edge_polylines;
            REGISTRY_GENERATION.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Remove a shape from the registry.
    pub fn remove(&self, shape_id: ShapeId) {
        let mut map = self.inner.write().expect("shape registry lock poisoned");
        map.remove(&shape_id);
        REGISTRY_GENERATION.fetch_add(1, Ordering::SeqCst);
    }

    /// Return a snapshot of all currently visible shapes with their data.
    pub fn visible_shapes(&self) -> Vec<ShapeEntry> {
        let map = self.inner.read().expect("shape registry lock poisoned");
        map.values()
            .filter(|e| e.visible)
            .map(|e| ShapeEntry {
                shape_id: e.shape_id,
                mesh: e.mesh.clone(),
                edge_polylines: e.edge_polylines.clone(),
                visible: e.visible,
                color: e.color,
            })
            .collect()
    }

    /// Update the visible flag on a shape.
    pub fn set_visible(&self, shape_id: ShapeId, visible: bool) {
        let mut map = self.inner.write().expect("shape registry lock poisoned");
        if let Some(entry) = map.get_mut(&shape_id) {
            entry.visible = visible;
            REGISTRY_GENERATION.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Look up a single shape by ID.
    pub fn shape_by_id(&self, shape_id: ShapeId) -> Option<ShapeEntry> {
        let map = self.inner.read().expect("shape registry lock poisoned");
        map.get(&shape_id).map(|e| ShapeEntry {
            shape_id: e.shape_id,
            mesh: e.mesh.clone(),
            edge_polylines: e.edge_polylines.clone(),
            visible: e.visible,
            color: e.color,
        })
    }

    /// Return a clone of the inner Arc for sharing across threads.
    pub fn clone_inner(&self) -> Arc<RwLock<HashMap<ShapeId, ShapeEntry>>> {
        self.inner.clone()
    }
}

impl Clone for ShapeRegistry {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for ShapeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A 3D shape with associated metadata.
///
/// This is the core data type wrapped as a Janet abstract value.
/// It carries the OCCT `Shape` plus metadata used by the viewer (visible, color).
pub struct ShapeData {
    pub shape_id: ShapeId,
    pub shape: Shape,
    pub visible: bool,
    #[allow(dead_code)]
    pub color: Option<[f64; 3]>,
}

impl ShapeData {
    /// Create a new shape at the origin with default visibility.
    /// Automatically assigns a unique ID and registers in the global registry.
    pub fn new(shape: Shape) -> Self {
        let shape_id = next_shape_id();
        let entry = ShapeEntry {
            shape_id,
            mesh: None,
            edge_polylines: Vec::new(),
            visible: true,
            color: None,
        };
        global_shape_registry().register(entry);
        Self {
            shape_id,
            shape,
            visible: true,
            color: None,
        }
    }

    /// Get the shape type as a human-readable uppercase string.
    pub fn type_string(&self) -> &'static str {
        use opencascade::primitives::ShapeType;
        match self.shape.shape_type() {
            ShapeType::Solid => "SOLID",
            ShapeType::Face => "FACE",
            ShapeType::Edge => "EDGE",
            ShapeType::Wire => "WIRE",
            ShapeType::Shell => "SHELL",
            ShapeType::Vertex => "VERTEX",
            ShapeType::Compound => "COMPOUND",
            ShapeType::CompoundSolid => "COMPOUND_SOLID",
            ShapeType::Shape => "SHAPE",
        }
    }
}
