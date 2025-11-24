/// Unity BuildTarget enumeration
///
/// Represents different build target platforms for Unity.
/// This enum can handle unknown platform values for forward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BuildTarget {
    /// No target
    NoTarget = -2,
    /// Dashboard widget
    DashboardWidget = 1,
    /// Standalone macOS
    StandaloneOSX = 2,
    /// Standalone macOS PowerPC (deprecated)
    StandaloneOSXPPC = 3,
    /// Standalone macOS Intel (deprecated)
    StandaloneOSXIntel = 4,
    /// Standalone Windows
    StandaloneWindows = 5,
    /// Web Player (deprecated)
    WebPlayer = 6,
    /// Web Player Streamed (deprecated)
    WebPlayerStreamed = 7,
    /// Wii (deprecated)
    Wii = 8,
    /// iOS
    iOS = 9,
    /// PlayStation 3 (deprecated)
    PS3 = 10,
    /// Xbox 360 (deprecated)
    XBOX360 = 11,
    /// Android
    Android = 13,
    /// Standalone GLES Emulator (deprecated)
    StandaloneGLESEmu = 14,
    /// NaCl (deprecated)
    NaCl = 16,
    /// Standalone Linux
    StandaloneLinux = 17,
    /// Flash Player (deprecated)
    FlashPlayer = 18,
    /// Standalone Windows 64-bit
    StandaloneWindows64 = 19,
    /// WebGL
    WebGL = 20,
    /// WSA Player (Windows Store Apps)
    WSAPlayer = 21,
    /// Standalone Linux 64-bit
    StandaloneLinux64 = 24,
    /// Standalone Linux Universal
    StandaloneLinuxUniversal = 25,
    /// Windows Phone 8 Player (deprecated)
    WP8Player = 26,
    /// Standalone macOS Intel 64-bit
    StandaloneOSXIntel64 = 27,
    /// BlackBerry (deprecated)
    BlackBerry = 28,
    /// Tizen (deprecated)
    Tizen = 29,
    /// PlayStation Vita
    PSP2 = 30,
    /// PlayStation 4
    PS4 = 31,
    /// PlayStation Mobile (deprecated)
    PSM = 32,
    /// Xbox One
    XboxOne = 33,
    /// Samsung TV (deprecated)
    SamsungTV = 34,
    /// Nintendo 3DS
    N3DS = 35,
    /// Wii U
    WiiU = 36,
    /// Apple TV
    tvOS = 37,
    /// Nintendo Switch
    Switch = 38,
    /// Unknown platform
    UnknownPlatform = 3716,
}

impl BuildTarget {
    /// Creates a BuildTarget from an i32 value
    ///
    /// Returns a BuildTarget variant or UnknownPlatform for unknown values.
    pub fn from_i32(value: i32) -> Self {
        match value {
            -2 => BuildTarget::NoTarget,
            1 => BuildTarget::DashboardWidget,
            2 => BuildTarget::StandaloneOSX,
            3 => BuildTarget::StandaloneOSXPPC,
            4 => BuildTarget::StandaloneOSXIntel,
            5 => BuildTarget::StandaloneWindows,
            6 => BuildTarget::WebPlayer,
            7 => BuildTarget::WebPlayerStreamed,
            8 => BuildTarget::Wii,
            9 => BuildTarget::iOS,
            10 => BuildTarget::PS3,
            11 => BuildTarget::XBOX360,
            13 => BuildTarget::Android,
            14 => BuildTarget::StandaloneGLESEmu,
            16 => BuildTarget::NaCl,
            17 => BuildTarget::StandaloneLinux,
            18 => BuildTarget::FlashPlayer,
            19 => BuildTarget::StandaloneWindows64,
            20 => BuildTarget::WebGL,
            21 => BuildTarget::WSAPlayer,
            24 => BuildTarget::StandaloneLinux64,
            25 => BuildTarget::StandaloneLinuxUniversal,
            26 => BuildTarget::WP8Player,
            27 => BuildTarget::StandaloneOSXIntel64,
            28 => BuildTarget::BlackBerry,
            29 => BuildTarget::Tizen,
            30 => BuildTarget::PSP2,
            31 => BuildTarget::PS4,
            32 => BuildTarget::PSM,
            33 => BuildTarget::XboxOne,
            34 => BuildTarget::SamsungTV,
            35 => BuildTarget::N3DS,
            36 => BuildTarget::WiiU,
            37 => BuildTarget::tvOS,
            38 => BuildTarget::Switch,
            _ => BuildTarget::UnknownPlatform,
        }
    }

    /// Converts the BuildTarget to its i32 representation
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

impl From<i32> for BuildTarget {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<BuildTarget> for i32 {
    fn from(value: BuildTarget) -> Self {
        value.to_i32()
    }
}
