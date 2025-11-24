/// Unity ShaderGpuProgramType enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShaderGpuProgramType {
    /// kShaderGpuProgramUnknown variant
    kShaderGpuProgramUnknown = 0,
    /// kShaderGpuProgramGLLegacy variant
    kShaderGpuProgramGLLegacy = 1,
    /// kShaderGpuProgramGLES31AEP variant
    kShaderGpuProgramGLES31AEP = 2,
    /// kShaderGpuProgramGLES31 variant
    kShaderGpuProgramGLES31 = 3,
    /// kShaderGpuProgramGLES3 variant
    kShaderGpuProgramGLES3 = 4,
    /// kShaderGpuProgramGLES variant
    kShaderGpuProgramGLES = 5,
    /// kShaderGpuProgramGLCore32 variant
    kShaderGpuProgramGLCore32 = 6,
    /// kShaderGpuProgramGLCore41 variant
    kShaderGpuProgramGLCore41 = 7,
    /// kShaderGpuProgramGLCore43 variant
    kShaderGpuProgramGLCore43 = 8,
    /// kShaderGpuProgramDX9VertexSM20 variant
    kShaderGpuProgramDX9VertexSM20 = 9,
    /// kShaderGpuProgramDX9VertexSM30 variant
    kShaderGpuProgramDX9VertexSM30 = 10,
    /// kShaderGpuProgramDX9PixelSM20 variant
    kShaderGpuProgramDX9PixelSM20 = 11,
    /// kShaderGpuProgramDX9PixelSM30 variant
    kShaderGpuProgramDX9PixelSM30 = 12,
    /// kShaderGpuProgramDX10Level9Vertex variant
    kShaderGpuProgramDX10Level9Vertex = 13,
    /// kShaderGpuProgramDX10Level9Pixel variant
    kShaderGpuProgramDX10Level9Pixel = 14,
    /// kShaderGpuProgramDX11VertexSM40 variant
    kShaderGpuProgramDX11VertexSM40 = 15,
    /// kShaderGpuProgramDX11VertexSM50 variant
    kShaderGpuProgramDX11VertexSM50 = 16,
    /// kShaderGpuProgramDX11PixelSM40 variant
    kShaderGpuProgramDX11PixelSM40 = 17,
    /// kShaderGpuProgramDX11PixelSM50 variant
    kShaderGpuProgramDX11PixelSM50 = 18,
    /// kShaderGpuProgramDX11GeometrySM40 variant
    kShaderGpuProgramDX11GeometrySM40 = 19,
    /// kShaderGpuProgramDX11GeometrySM50 variant
    kShaderGpuProgramDX11GeometrySM50 = 20,
    /// kShaderGpuProgramDX11HullSM50 variant
    kShaderGpuProgramDX11HullSM50 = 21,
    /// kShaderGpuProgramDX11DomainSM50 variant
    kShaderGpuProgramDX11DomainSM50 = 22,
    /// kShaderGpuProgramMetalVS variant
    kShaderGpuProgramMetalVS = 23,
    /// kShaderGpuProgramMetalFS variant
    kShaderGpuProgramMetalFS = 24,
    /// kShaderGpuProgramSPIRV variant
    kShaderGpuProgramSPIRV = 25,
    /// kShaderGpuProgramConsoleVS variant
    kShaderGpuProgramConsoleVS = 26,
    /// kShaderGpuProgramConsoleFS variant
    kShaderGpuProgramConsoleFS = 27,
    /// kShaderGpuProgramConsoleHS variant
    kShaderGpuProgramConsoleHS = 28,
    /// kShaderGpuProgramConsoleDS variant
    kShaderGpuProgramConsoleDS = 29,
    /// kShaderGpuProgramConsoleGS variant
    kShaderGpuProgramConsoleGS = 30,
    /// kShaderGpuProgramRayTracing variant
    kShaderGpuProgramRayTracing = 31,
    /// Unknown or unsupported variant
    Unknown(u32),
}

impl ShaderGpuProgramType {
    /// Creates a ShaderGpuProgramType from a u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => ShaderGpuProgramType::kShaderGpuProgramUnknown,
            1 => ShaderGpuProgramType::kShaderGpuProgramGLLegacy,
            2 => ShaderGpuProgramType::kShaderGpuProgramGLES31AEP,
            3 => ShaderGpuProgramType::kShaderGpuProgramGLES31,
            4 => ShaderGpuProgramType::kShaderGpuProgramGLES3,
            5 => ShaderGpuProgramType::kShaderGpuProgramGLES,
            6 => ShaderGpuProgramType::kShaderGpuProgramGLCore32,
            7 => ShaderGpuProgramType::kShaderGpuProgramGLCore41,
            8 => ShaderGpuProgramType::kShaderGpuProgramGLCore43,
            9 => ShaderGpuProgramType::kShaderGpuProgramDX9VertexSM20,
            10 => ShaderGpuProgramType::kShaderGpuProgramDX9VertexSM30,
            11 => ShaderGpuProgramType::kShaderGpuProgramDX9PixelSM20,
            12 => ShaderGpuProgramType::kShaderGpuProgramDX9PixelSM30,
            13 => ShaderGpuProgramType::kShaderGpuProgramDX10Level9Vertex,
            14 => ShaderGpuProgramType::kShaderGpuProgramDX10Level9Pixel,
            15 => ShaderGpuProgramType::kShaderGpuProgramDX11VertexSM40,
            16 => ShaderGpuProgramType::kShaderGpuProgramDX11VertexSM50,
            17 => ShaderGpuProgramType::kShaderGpuProgramDX11PixelSM40,
            18 => ShaderGpuProgramType::kShaderGpuProgramDX11PixelSM50,
            19 => ShaderGpuProgramType::kShaderGpuProgramDX11GeometrySM40,
            20 => ShaderGpuProgramType::kShaderGpuProgramDX11GeometrySM50,
            21 => ShaderGpuProgramType::kShaderGpuProgramDX11HullSM50,
            22 => ShaderGpuProgramType::kShaderGpuProgramDX11DomainSM50,
            23 => ShaderGpuProgramType::kShaderGpuProgramMetalVS,
            24 => ShaderGpuProgramType::kShaderGpuProgramMetalFS,
            25 => ShaderGpuProgramType::kShaderGpuProgramSPIRV,
            26 => ShaderGpuProgramType::kShaderGpuProgramConsoleVS,
            27 => ShaderGpuProgramType::kShaderGpuProgramConsoleFS,
            28 => ShaderGpuProgramType::kShaderGpuProgramConsoleHS,
            29 => ShaderGpuProgramType::kShaderGpuProgramConsoleDS,
            30 => ShaderGpuProgramType::kShaderGpuProgramConsoleGS,
            31 => ShaderGpuProgramType::kShaderGpuProgramRayTracing,
            _ => ShaderGpuProgramType::Unknown(value),
        }
    }

    /// Converts the ShaderGpuProgramType to its u32 representation
    pub fn to_u32(&self) -> u32 {
        match self {
            ShaderGpuProgramType::kShaderGpuProgramUnknown => 0,
            ShaderGpuProgramType::kShaderGpuProgramGLLegacy => 1,
            ShaderGpuProgramType::kShaderGpuProgramGLES31AEP => 2,
            ShaderGpuProgramType::kShaderGpuProgramGLES31 => 3,
            ShaderGpuProgramType::kShaderGpuProgramGLES3 => 4,
            ShaderGpuProgramType::kShaderGpuProgramGLES => 5,
            ShaderGpuProgramType::kShaderGpuProgramGLCore32 => 6,
            ShaderGpuProgramType::kShaderGpuProgramGLCore41 => 7,
            ShaderGpuProgramType::kShaderGpuProgramGLCore43 => 8,
            ShaderGpuProgramType::kShaderGpuProgramDX9VertexSM20 => 9,
            ShaderGpuProgramType::kShaderGpuProgramDX9VertexSM30 => 10,
            ShaderGpuProgramType::kShaderGpuProgramDX9PixelSM20 => 11,
            ShaderGpuProgramType::kShaderGpuProgramDX9PixelSM30 => 12,
            ShaderGpuProgramType::kShaderGpuProgramDX10Level9Vertex => 13,
            ShaderGpuProgramType::kShaderGpuProgramDX10Level9Pixel => 14,
            ShaderGpuProgramType::kShaderGpuProgramDX11VertexSM40 => 15,
            ShaderGpuProgramType::kShaderGpuProgramDX11VertexSM50 => 16,
            ShaderGpuProgramType::kShaderGpuProgramDX11PixelSM40 => 17,
            ShaderGpuProgramType::kShaderGpuProgramDX11PixelSM50 => 18,
            ShaderGpuProgramType::kShaderGpuProgramDX11GeometrySM40 => 19,
            ShaderGpuProgramType::kShaderGpuProgramDX11GeometrySM50 => 20,
            ShaderGpuProgramType::kShaderGpuProgramDX11HullSM50 => 21,
            ShaderGpuProgramType::kShaderGpuProgramDX11DomainSM50 => 22,
            ShaderGpuProgramType::kShaderGpuProgramMetalVS => 23,
            ShaderGpuProgramType::kShaderGpuProgramMetalFS => 24,
            ShaderGpuProgramType::kShaderGpuProgramSPIRV => 25,
            ShaderGpuProgramType::kShaderGpuProgramConsoleVS => 26,
            ShaderGpuProgramType::kShaderGpuProgramConsoleFS => 27,
            ShaderGpuProgramType::kShaderGpuProgramConsoleHS => 28,
            ShaderGpuProgramType::kShaderGpuProgramConsoleDS => 29,
            ShaderGpuProgramType::kShaderGpuProgramConsoleGS => 30,
            ShaderGpuProgramType::kShaderGpuProgramRayTracing => 31,
            ShaderGpuProgramType::Unknown(value) => *value,
        }
    }
}

impl From<u32> for ShaderGpuProgramType {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<ShaderGpuProgramType> for u32 {
    fn from(value: ShaderGpuProgramType) -> Self {
        value.to_u32()
    }
}
