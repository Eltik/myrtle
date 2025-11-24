/// Unity ShaderCompilerPlatform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ShaderCompilerPlatform {
    /// kShaderCompPlatformNone variant
    kShaderCompPlatformNone = -1,
    /// kShaderCompPlatformGL variant
    kShaderCompPlatformGL = 0,
    /// kShaderCompPlatformD3D9 variant
    kShaderCompPlatformD3D9 = 1,
    /// kShaderCompPlatformXbox360 variant
    kShaderCompPlatformXbox360 = 2,
    /// kShaderCompPlatformPS3 variant
    kShaderCompPlatformPS3 = 3,
    /// kShaderCompPlatformD3D11 variant
    kShaderCompPlatformD3D11 = 4,
    /// kShaderCompPlatformGLES20 variant
    kShaderCompPlatformGLES20 = 5,
    /// kShaderCompPlatformNaCl variant
    kShaderCompPlatformNaCl = 6,
    /// kShaderCompPlatformFlash variant
    kShaderCompPlatformFlash = 7,
    /// kShaderCompPlatformD3D11_9x variant
    kShaderCompPlatformD3D11_9x = 8,
    /// kShaderCompPlatformGLES3Plus variant
    kShaderCompPlatformGLES3Plus = 9,
    /// kShaderCompPlatformPSP2 variant
    kShaderCompPlatformPSP2 = 10,
    /// kShaderCompPlatformPS4 variant
    kShaderCompPlatformPS4 = 11,
    /// kShaderCompPlatformXboxOne variant
    kShaderCompPlatformXboxOne = 12,
    /// kShaderCompPlatformPSM variant
    kShaderCompPlatformPSM = 13,
    /// kShaderCompPlatformMetal variant
    kShaderCompPlatformMetal = 14,
    /// kShaderCompPlatformOpenGLCore variant
    kShaderCompPlatformOpenGLCore = 15,
    /// kShaderCompPlatformN3DS variant
    kShaderCompPlatformN3DS = 16,
    /// kShaderCompPlatformWiiU variant
    kShaderCompPlatformWiiU = 17,
    /// kShaderCompPlatformVulkan variant
    kShaderCompPlatformVulkan = 18,
    /// kShaderCompPlatformSwitch variant
    kShaderCompPlatformSwitch = 19,
    /// kShaderCompPlatformXboxOneD3D12 variant
    kShaderCompPlatformXboxOneD3D12 = 20,
    /// Unknown or unsupported variant
    Unknown(i32),
}

impl ShaderCompilerPlatform {
    /// Creates a ShaderCompilerPlatform from a i32 value
    pub fn from_i32(value: i32) -> Self {
        match value {
            -1 => ShaderCompilerPlatform::kShaderCompPlatformNone,
            0 => ShaderCompilerPlatform::kShaderCompPlatformGL,
            1 => ShaderCompilerPlatform::kShaderCompPlatformD3D9,
            2 => ShaderCompilerPlatform::kShaderCompPlatformXbox360,
            3 => ShaderCompilerPlatform::kShaderCompPlatformPS3,
            4 => ShaderCompilerPlatform::kShaderCompPlatformD3D11,
            5 => ShaderCompilerPlatform::kShaderCompPlatformGLES20,
            6 => ShaderCompilerPlatform::kShaderCompPlatformNaCl,
            7 => ShaderCompilerPlatform::kShaderCompPlatformFlash,
            8 => ShaderCompilerPlatform::kShaderCompPlatformD3D11_9x,
            9 => ShaderCompilerPlatform::kShaderCompPlatformGLES3Plus,
            10 => ShaderCompilerPlatform::kShaderCompPlatformPSP2,
            11 => ShaderCompilerPlatform::kShaderCompPlatformPS4,
            12 => ShaderCompilerPlatform::kShaderCompPlatformXboxOne,
            13 => ShaderCompilerPlatform::kShaderCompPlatformPSM,
            14 => ShaderCompilerPlatform::kShaderCompPlatformMetal,
            15 => ShaderCompilerPlatform::kShaderCompPlatformOpenGLCore,
            16 => ShaderCompilerPlatform::kShaderCompPlatformN3DS,
            17 => ShaderCompilerPlatform::kShaderCompPlatformWiiU,
            18 => ShaderCompilerPlatform::kShaderCompPlatformVulkan,
            19 => ShaderCompilerPlatform::kShaderCompPlatformSwitch,
            20 => ShaderCompilerPlatform::kShaderCompPlatformXboxOneD3D12,
            _ => ShaderCompilerPlatform::Unknown(value),
        }
    }

    /// Converts the ShaderCompilerPlatform to its i32 representation
    pub fn to_i32(&self) -> i32 {
        match self {
            ShaderCompilerPlatform::kShaderCompPlatformNone => -1,
            ShaderCompilerPlatform::kShaderCompPlatformGL => 0,
            ShaderCompilerPlatform::kShaderCompPlatformD3D9 => 1,
            ShaderCompilerPlatform::kShaderCompPlatformXbox360 => 2,
            ShaderCompilerPlatform::kShaderCompPlatformPS3 => 3,
            ShaderCompilerPlatform::kShaderCompPlatformD3D11 => 4,
            ShaderCompilerPlatform::kShaderCompPlatformGLES20 => 5,
            ShaderCompilerPlatform::kShaderCompPlatformNaCl => 6,
            ShaderCompilerPlatform::kShaderCompPlatformFlash => 7,
            ShaderCompilerPlatform::kShaderCompPlatformD3D11_9x => 8,
            ShaderCompilerPlatform::kShaderCompPlatformGLES3Plus => 9,
            ShaderCompilerPlatform::kShaderCompPlatformPSP2 => 10,
            ShaderCompilerPlatform::kShaderCompPlatformPS4 => 11,
            ShaderCompilerPlatform::kShaderCompPlatformXboxOne => 12,
            ShaderCompilerPlatform::kShaderCompPlatformPSM => 13,
            ShaderCompilerPlatform::kShaderCompPlatformMetal => 14,
            ShaderCompilerPlatform::kShaderCompPlatformOpenGLCore => 15,
            ShaderCompilerPlatform::kShaderCompPlatformN3DS => 16,
            ShaderCompilerPlatform::kShaderCompPlatformWiiU => 17,
            ShaderCompilerPlatform::kShaderCompPlatformVulkan => 18,
            ShaderCompilerPlatform::kShaderCompPlatformSwitch => 19,
            ShaderCompilerPlatform::kShaderCompPlatformXboxOneD3D12 => 20,
            ShaderCompilerPlatform::Unknown(value) => *value,
        }
    }
}

impl From<i32> for ShaderCompilerPlatform {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<ShaderCompilerPlatform> for i32 {
    fn from(value: ShaderCompilerPlatform) -> Self {
        value.to_i32()
    }
}
