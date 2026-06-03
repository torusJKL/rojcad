//! Shape data type — a Janet abstract type wrapping an OCCT Shape
//! with metadata for visibility, color, etc.

use opencascade::primitives::Shape;

/// A 3D shape with associated metadata.
///
/// This is the core data type wrapped as a Janet abstract value.
/// It carries the OCCT `Shape` plus metadata used by the viewer (visible, color).
pub struct ShapeData {
    pub shape: Shape,
    pub visible: bool,
    #[allow(dead_code)]
    pub color: Option<[f64; 3]>,
}

impl ShapeData {
    /// Create a new shape at the origin with default visibility.
    pub fn new(shape: Shape) -> Self {
        Self {
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
