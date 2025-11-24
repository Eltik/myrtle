/// Unity MeshTopology enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum MeshTopology {
    /// Triangles variant
    Triangles = 0,
    /// TriangleStrip variant
    TriangleStrip = 1,
    /// Quads variant
    Quads = 2,
    /// Lines variant
    Lines = 3,
    /// LineStrip variant
    LineStrip = 4,
    /// Points variant
    Points = 5,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl MeshTopology {
    /// Creates a MeshTopology from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => MeshTopology::Triangles,
            1 => MeshTopology::TriangleStrip,
            2 => MeshTopology::Quads,
            3 => MeshTopology::Lines,
            4 => MeshTopology::LineStrip,
            5 => MeshTopology::Points,
            _ => MeshTopology::Unknown(value),
        }
    }

    /// Converts the MeshTopology to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            MeshTopology::Triangles => 0,
            MeshTopology::TriangleStrip => 1,
            MeshTopology::Quads => 2,
            MeshTopology::Lines => 3,
            MeshTopology::LineStrip => 4,
            MeshTopology::Points => 5,
            MeshTopology::Unknown(value) => *value,
        }
    }
}

impl From<u32> for MeshTopology {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<MeshTopology> for u32 {
    fn from(value: MeshTopology) -> Self {
        value.to_u32()
    }
}
