//! Shape data type — a Janet abstract type wrapping an OCCT Shape
//! with metadata for visibility, color, etc.
//!
//! Also defines the shared `ShapeRegistry` used to synchronize shape
//! state between the REPL thread and the viewer thread.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

/// Global generation counter for change tracking.
/// Incremented on every ShapeRegistry write.
/// The viewer reads this to detect changes since last frame.
pub static REGISTRY_GENERATION: AtomicU64 = AtomicU64::new(0);

use opencascade::primitives::Shape;

/// Last selected shape ID, used to propagate selection events to Janet.
/// 0 = no event pending, u64::MAX = deselected, other = selected shape ID.
pub static LAST_SELECTION: AtomicU64 = AtomicU64::new(0);

/// Edge visibility toggles, controlled from the Janet REPL.
pub static SHOW_INACTIVE_EDGES: AtomicBool = AtomicBool::new(true);
pub static SHOW_ACTIVE_EDGES: AtomicBool = AtomicBool::new(true);

/// Edge thickness in NDC units (controlled from Janet).
pub static EDGE_THICKNESS: AtomicU64 = AtomicU64::new(f64::to_bits(0.001));

/// Pack 3 f64 RGB values into a single u64 for atomic storage.
pub fn pack_color(r: f64, g: f64, b: f64) -> u64 {
    let ri = (r.clamp(0.0, 1.0) * 65535.0) as u64;
    let gi = (g.clamp(0.0, 1.0) * 65535.0) as u64;
    let bi = (b.clamp(0.0, 1.0) * 65535.0) as u64;
    (ri << 32) | (gi << 16) | bi
}
/// Unpack a u64 into [r, g, b] f64 values in [0, 1].
pub fn unpack_color(packed: u64) -> [f64; 3] {
    let r = ((packed >> 32) & 0xFFFF) as f64 / 65535.0;
    let g = ((packed >> 16) & 0xFFFF) as f64 / 65535.0;
    let b = (packed & 0xFFFF) as f64 / 65535.0;
    [r, g, b]
}

/// Inactive edge color: light grey (0.7, 0.7, 0.7) packed as u64.
pub static INACTIVE_EDGE_COLOR: AtomicU64 = AtomicU64::new(0);
/// Active (selected) edge color: light blue (0.4, 0.6, 1.0) packed as u64.
pub static ACTIVE_EDGE_COLOR: AtomicU64 = AtomicU64::new(0);

/// Set edge color defaults (called at startup, after statics are initialized).
pub fn init_edge_color_defaults() {
    INACTIVE_EDGE_COLOR.store(pack_color(0.7, 0.7, 0.7), Ordering::SeqCst);
    ACTIVE_EDGE_COLOR.store(pack_color(0.4, 0.6, 1.0), Ordering::SeqCst);
}

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
    pub fn update(
        &self,
        shape_id: ShapeId,
        mesh: Option<MeshData>,
        edge_polylines: Vec<Vec<[f64; 3]>>,
    ) {
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
///
/// Shapes are NOT automatically registered in the viewer registry on creation.
/// Registration happens only when `show` is explicitly called.
pub struct ShapeData {
    pub shape_id: ShapeId,
    pub shape: Shape,
    pub visible: bool,
    pub color: Option<[f64; 3]>,
    pub mesh: Option<MeshData>,
    pub edge_polylines: Vec<Vec<[f64; 3]>>,
    pub registered: bool,
    pub purged: bool,
}

impl ShapeData {
    /// Create a new shape with a unique ID.
    ///
    /// The shape is NOT registered in the viewer registry and NOT tessellated.
    /// Call `show` to register and optionally `:eager` at creation to tessellate early.
    pub fn new(shape: Shape) -> Self {
        let shape_id = next_shape_id();
        Self {
            shape_id,
            shape,
            visible: true,
            color: None,
            mesh: None,
            edge_polylines: Vec::new(),
            registered: false,
            purged: false,
        }
    }

    /// Tessellate the shape if not already tessellated.
    /// Extracts mesh and edge polylines from the OCCT shape.
    pub fn tessellate_if_needed(&mut self) {
        if self.mesh.is_some() {
            return;
        }
        let mesh = crate::cad::extract_mesh(&self.shape);
        let mut edge_polylines = crate::cad::extract_edge_polylines(&self.shape);
        if edge_polylines.len() < crate::cad::SYNTHETIC_WIREFRAME_THRESHOLD
            && let Some(ref m) = mesh
        {
            edge_polylines.extend(crate::cad::generate_synthetic_wireframe(m));
        }
        self.mesh = mesh;
        self.edge_polylines = edge_polylines;
    }

    /// Register this shape in the viewer registry, making it visible.
    /// Tessellates first if needed.
    pub fn show(&mut self) {
        if self.purged {
            panic!("shape has been purged");
        }
        self.tessellate_if_needed();
        if !self.registered {
            let entry = ShapeEntry {
                shape_id: self.shape_id,
                mesh: self.mesh.clone(),
                edge_polylines: self.edge_polylines.clone(),
                visible: true,
                color: self.color,
            };
            global_shape_registry().register(entry);
            self.registered = true;
        } else {
            global_shape_registry().set_visible(self.shape_id, true);
        }
        self.visible = true;
    }

    /// Hide the shape in the viewer. Stays registered.
    pub fn hide(&mut self) {
        if self.registered {
            global_shape_registry().set_visible(self.shape_id, false);
        }
        self.visible = false;
    }

    /// Remove the shape from the viewer registry immediately and mark as purged.
    pub fn remove_from_registry(&mut self) {
        if self.registered {
            global_shape_registry().remove(self.shape_id);
            self.registered = false;
        }
        self.purged = true;
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

impl Drop for ShapeData {
    fn drop(&mut self) {
        if self.registered {
            global_shape_registry().remove(self.shape_id);
        }
    }
}
