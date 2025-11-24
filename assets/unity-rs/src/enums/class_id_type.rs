/// Unity ClassIDType enumeration
///
/// Represents different Unity class types as defined in the ClassIDReference.
/// This enum can handle unknown types for forward compatibility.
///
/// See: https://docs.unity3d.com/Manual/ClassIDReference.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ClassIDType {
    /// Unknown type
    UnknownType = -1,
    /// Object
    Object = 0,
    /// GameObject
    GameObject = 1,
    /// Component
    Component = 2,
    /// LevelGameManager
    LevelGameManager = 3,
    /// Transform
    Transform = 4,
    /// TimeManager
    TimeManager = 5,
    /// GlobalGameManager
    GlobalGameManager = 6,
    /// Behaviour
    Behaviour = 8,
    /// GameManager
    GameManager = 9,
    /// AudioManager
    AudioManager = 11,
    /// ParticleAnimator
    ParticleAnimator = 12,
    /// InputManager
    InputManager = 13,
    /// EllipsoidParticleEmitter
    EllipsoidParticleEmitter = 15,
    /// Pipeline
    Pipeline = 17,
    /// EditorExtension
    EditorExtension = 18,
    /// Physics2DSettings
    Physics2DSettings = 19,
    /// Camera
    Camera = 20,
    /// Material
    Material = 21,
    /// MeshRenderer
    MeshRenderer = 23,
    /// Renderer
    Renderer = 25,
    /// ParticleRenderer
    ParticleRenderer = 26,
    /// Texture
    Texture = 27,
    /// Texture2D
    Texture2D = 28,
    /// OcclusionCullingSettings
    OcclusionCullingSettings = 29,
    /// GraphicsSettings
    GraphicsSettings = 30,
    /// MeshFilter
    MeshFilter = 33,
    /// OcclusionPortal
    OcclusionPortal = 41,
    /// Mesh
    Mesh = 43,
    /// Skybox
    Skybox = 45,
    /// QualitySettings
    QualitySettings = 47,
    /// Shader
    Shader = 48,
    /// TextAsset
    TextAsset = 49,
    /// Rigidbody2D
    Rigidbody2D = 50,
    /// Physics2DManager
    Physics2DManager = 51,
    /// Collider2D
    Collider2D = 53,
    /// Rigidbody
    Rigidbody = 54,
    /// PhysicsManager
    PhysicsManager = 55,
    /// Collider
    Collider = 56,
    /// Joint
    Joint = 57,
    /// CircleCollider2D
    CircleCollider2D = 58,
    /// HingeJoint
    HingeJoint = 59,
    /// PolygonCollider2D
    PolygonCollider2D = 60,
    /// BoxCollider2D
    BoxCollider2D = 61,
    /// PhysicsMaterial2D
    PhysicsMaterial2D = 62,
    /// MeshCollider
    MeshCollider = 64,
    /// BoxCollider
    BoxCollider = 65,
    /// CompositeCollider2D
    CompositeCollider2D = 66,
    /// EdgeCollider2D
    EdgeCollider2D = 68,
    /// CapsuleCollider2D
    CapsuleCollider2D = 70,
    /// ComputeShader
    ComputeShader = 72,
    /// AnimationClip
    AnimationClip = 74,
    /// ConstantForce
    ConstantForce = 75,
    /// WorldParticleCollider
    WorldParticleCollider = 76,
    /// TagManager
    TagManager = 78,
    /// AudioListener
    AudioListener = 81,
    /// AudioSource
    AudioSource = 82,
    /// AudioClip
    AudioClip = 83,
    /// RenderTexture
    RenderTexture = 84,
    /// CustomRenderTexture
    CustomRenderTexture = 86,
    /// MeshParticleEmitter
    MeshParticleEmitter = 87,
    /// ParticleEmitter
    ParticleEmitter = 88,
    /// Cubemap
    Cubemap = 89,
    /// Avatar
    Avatar = 90,
    /// AnimatorController
    AnimatorController = 91,
    /// GUILayer
    GUILayer = 92,
    /// RuntimeAnimatorController
    RuntimeAnimatorController = 93,
    /// ScriptMapper
    ScriptMapper = 94,
    /// Animator
    Animator = 95,
    /// TrailRenderer
    TrailRenderer = 96,
    /// DelayedCallManager
    DelayedCallManager = 98,
    /// TextMesh
    TextMesh = 102,
    /// RenderSettings
    RenderSettings = 104,
    /// Light
    Light = 108,
    /// CGProgram
    CGProgram = 109,
    /// BaseAnimationTrack
    BaseAnimationTrack = 110,
    /// Animation
    Animation = 111,
    /// MonoBehaviour
    MonoBehaviour = 114,
    /// MonoScript
    MonoScript = 115,
    /// MonoManager
    MonoManager = 116,
    /// Texture3D
    Texture3D = 117,
    /// NewAnimationTrack
    NewAnimationTrack = 118,
    /// Projector
    Projector = 119,
    /// LineRenderer
    LineRenderer = 120,
    /// Flare
    Flare = 121,
    /// Halo
    Halo = 122,
    /// LensFlare
    LensFlare = 123,
    /// FlareLayer
    FlareLayer = 124,
    /// HaloLayer
    HaloLayer = 125,
    /// NavMeshProjectSettings
    NavMeshProjectSettings = 126,
    /// HaloManager
    HaloManager = 127,
    /// Font
    Font = 128,
    /// PlayerSettings
    PlayerSettings = 129,
    /// NamedObject
    NamedObject = 130,
    /// GUITexture
    GUITexture = 131,
    /// GUIText
    GUIText = 132,
    /// GUIElement
    GUIElement = 133,
    /// PhysicMaterial
    PhysicMaterial = 134,
    /// SphereCollider
    SphereCollider = 135,
    /// CapsuleCollider
    CapsuleCollider = 136,
    /// SkinnedMeshRenderer
    SkinnedMeshRenderer = 137,
    /// FixedJoint
    FixedJoint = 138,
    /// RaycastCollider
    RaycastCollider = 140,
    /// BuildSettings
    BuildSettings = 141,
    /// AssetBundle
    AssetBundle = 142,
    /// CharacterController
    CharacterController = 143,
    /// CharacterJoint
    CharacterJoint = 144,
    /// SpringJoint
    SpringJoint = 145,
    /// WheelCollider
    WheelCollider = 146,
    /// ResourceManager
    ResourceManager = 147,
    /// NetworkView
    NetworkView = 148,
    /// NetworkManager
    NetworkManager = 149,
    /// PreloadData
    PreloadData = 150,
    /// MovieTexture
    MovieTexture = 152,
    /// ConfigurableJoint
    ConfigurableJoint = 153,
    /// TerrainCollider
    TerrainCollider = 154,
    /// MasterServerInterface
    MasterServerInterface = 155,
    /// TerrainData
    TerrainData = 156,
    /// LightmapSettings
    LightmapSettings = 157,
    /// WebCamTexture
    WebCamTexture = 158,
    /// EditorSettings
    EditorSettings = 159,
    /// InteractiveCloth
    InteractiveCloth = 160,
    /// ClothRenderer
    ClothRenderer = 161,
    /// EditorUserSettings
    EditorUserSettings = 162,
    /// SkinnedCloth
    SkinnedCloth = 163,
    /// AudioReverbFilter
    AudioReverbFilter = 164,
    /// AudioHighPassFilter
    AudioHighPassFilter = 165,
    /// AudioChorusFilter
    AudioChorusFilter = 166,
    /// AudioReverbZone
    AudioReverbZone = 167,
    /// AudioEchoFilter
    AudioEchoFilter = 168,
    /// AudioLowPassFilter
    AudioLowPassFilter = 169,
    /// AudioDistortionFilter
    AudioDistortionFilter = 170,
    /// SparseTexture
    SparseTexture = 171,
    /// AudioBehaviour
    AudioBehaviour = 180,
    /// AudioFilter
    AudioFilter = 181,
    /// WindZone
    WindZone = 182,
    /// Cloth
    Cloth = 183,
    /// SubstanceArchive
    SubstanceArchive = 184,
    /// ProceduralMaterial
    ProceduralMaterial = 185,
    /// ProceduralTexture
    ProceduralTexture = 186,
    /// Texture2DArray
    Texture2DArray = 187,
    /// CubemapArray
    CubemapArray = 188,
    /// OffMeshLink
    OffMeshLink = 191,
    /// OcclusionArea
    OcclusionArea = 192,
    /// Tree
    Tree = 193,
    /// NavMeshObsolete
    NavMeshObsolete = 194,
    /// NavMeshAgent
    NavMeshAgent = 195,
    /// NavMeshSettings
    NavMeshSettings = 196,
    /// LightProbesLegacy
    LightProbesLegacy = 197,
    /// ParticleSystem
    ParticleSystem = 198,
    /// ParticleSystemRenderer
    ParticleSystemRenderer = 199,
    /// ShaderVariantCollection
    ShaderVariantCollection = 200,
    /// LODGroup
    LODGroup = 205,
    /// BlendTree
    BlendTree = 206,
    /// Motion
    Motion = 207,
    /// NavMeshObstacle
    NavMeshObstacle = 208,
    /// SortingGroup
    SortingGroup = 210,
    /// SpriteRenderer
    SpriteRenderer = 212,
    /// Sprite
    Sprite = 213,
    /// CachedSpriteAtlas
    CachedSpriteAtlas = 214,
    /// ReflectionProbe
    ReflectionProbe = 215,
    /// ReflectionProbes
    ReflectionProbes = 216,
    /// Terrain
    Terrain = 218,
    /// LightProbeGroup
    LightProbeGroup = 220,
    /// AnimatorOverrideController
    AnimatorOverrideController = 221,
    /// CanvasRenderer
    CanvasRenderer = 222,
    /// Canvas
    Canvas = 223,
    /// RectTransform
    RectTransform = 224,
    /// CanvasGroup
    CanvasGroup = 225,
    /// BillboardAsset
    BillboardAsset = 226,
    /// BillboardRenderer
    BillboardRenderer = 227,
    /// SpeedTreeWindAsset
    SpeedTreeWindAsset = 228,
    /// AnchoredJoint2D
    AnchoredJoint2D = 229,
    /// Joint2D
    Joint2D = 230,
    /// SpringJoint2D
    SpringJoint2D = 231,
    /// DistanceJoint2D
    DistanceJoint2D = 232,
    /// HingeJoint2D
    HingeJoint2D = 233,
    /// SliderJoint2D
    SliderJoint2D = 234,
    /// WheelJoint2D
    WheelJoint2D = 235,
    /// ClusterInputManager
    ClusterInputManager = 236,
    /// BaseVideoTexture
    BaseVideoTexture = 237,
    /// NavMeshData
    NavMeshData = 238,
    /// AudioMixer
    AudioMixer = 240,
    /// AudioMixerController
    AudioMixerController = 241,
    /// AudioMixerGroupController
    AudioMixerGroupController = 243,
    /// AudioMixerEffectController
    AudioMixerEffectController = 244,
    /// AudioMixerSnapshotController
    AudioMixerSnapshotController = 245,
    /// PhysicsUpdateBehaviour2D
    PhysicsUpdateBehaviour2D = 246,
    /// ConstantForce2D
    ConstantForce2D = 247,
    /// Effector2D
    Effector2D = 248,
    /// AreaEffector2D
    AreaEffector2D = 249,
    /// PointEffector2D
    PointEffector2D = 250,
    /// PlatformEffector2D
    PlatformEffector2D = 251,
    /// SurfaceEffector2D
    SurfaceEffector2D = 252,
    /// BuoyancyEffector2D
    BuoyancyEffector2D = 253,
    /// RelativeJoint2D
    RelativeJoint2D = 254,
    /// FixedJoint2D
    FixedJoint2D = 255,
    /// FrictionJoint2D
    FrictionJoint2D = 256,
    /// TargetJoint2D
    TargetJoint2D = 257,
    /// LightProbes
    LightProbes = 258,
    /// LightProbeProxyVolume
    LightProbeProxyVolume = 259,
    /// SampleClip
    SampleClip = 271,
    /// AudioMixerSnapshot
    AudioMixerSnapshot = 272,
    /// AudioMixerGroup
    AudioMixerGroup = 273,
    /// NScreenBridge
    NScreenBridge = 280,
    /// AssetBundleManifest
    AssetBundleManifest = 290,
    /// UnityAdsManager
    UnityAdsManager = 292,
    /// RuntimeInitializeOnLoadManager
    RuntimeInitializeOnLoadManager = 300,
    /// CloudWebServicesManager
    CloudWebServicesManager = 301,
    /// UnityAnalyticsManager
    UnityAnalyticsManager = 303,
    /// CrashReportManager
    CrashReportManager = 304,
    /// PerformanceReportingManager
    PerformanceReportingManager = 305,
    /// UnityConnectSettings
    UnityConnectSettings = 310,
    /// AvatarMask
    AvatarMask = 319,
    /// PlayableDirector
    PlayableDirector = 320,
    /// VideoPlayer
    VideoPlayer = 328,
    /// VideoClip
    VideoClip = 329,
    /// ParticleSystemForceField
    ParticleSystemForceField = 330,
    /// SpriteMask
    SpriteMask = 331,
    /// WorldAnchor
    WorldAnchor = 362,
    /// OcclusionCullingData
    OcclusionCullingData = 363,
    /// SmallestEditorClassID
    SmallestEditorClassID = 1000,
    /// PrefabInstance
    PrefabInstance = 1001,
    /// EditorExtensionImpl
    EditorExtensionImpl = 1002,
    /// AssetImporter
    AssetImporter = 1003,
    /// AssetDatabaseV1
    AssetDatabaseV1 = 1004,
    /// Mesh3DSImporter
    Mesh3DSImporter = 1005,
    /// TextureImporter
    TextureImporter = 1006,
    /// ShaderImporter
    ShaderImporter = 1007,
    /// ComputeShaderImporter
    ComputeShaderImporter = 1008,
    /// AudioImporter
    AudioImporter = 1020,
    /// HierarchyState
    HierarchyState = 1026,
    /// GUIDSerializer
    GUIDSerializer = 1027,
    /// AssetMetaData
    AssetMetaData = 1028,
    /// DefaultAsset
    DefaultAsset = 1029,
    /// DefaultImporter
    DefaultImporter = 1030,
    /// TextScriptImporter
    TextScriptImporter = 1031,
    /// SceneAsset
    SceneAsset = 1032,
    /// NativeFormatImporter
    NativeFormatImporter = 1034,
    /// MonoImporter
    MonoImporter = 1035,
    /// AssetServerCache
    AssetServerCache = 1037,
    /// LibraryAssetImporter
    LibraryAssetImporter = 1038,
    /// ModelImporter
    ModelImporter = 1040,
    /// FBXImporter
    FBXImporter = 1041,
    /// TrueTypeFontImporter
    TrueTypeFontImporter = 1042,
    /// MovieImporter
    MovieImporter = 1044,
    /// EditorBuildSettings
    EditorBuildSettings = 1045,
    /// DDSImporter
    DDSImporter = 1046,
    /// InspectorExpandedState
    InspectorExpandedState = 1048,
    /// AnnotationManager
    AnnotationManager = 1049,
    /// PluginImporter
    PluginImporter = 1050,
    /// EditorUserBuildSettings
    EditorUserBuildSettings = 1051,
    /// PVRImporter
    PVRImporter = 1052,
    /// ASTCImporter
    ASTCImporter = 1053,
    /// KTXImporter
    KTXImporter = 1054,
    /// IHVImageFormatImporter
    IHVImageFormatImporter = 1055,
    /// AnimatorStateTransition
    AnimatorStateTransition = 1101,
    /// AnimatorState
    AnimatorState = 1102,
    /// HumanTemplate
    HumanTemplate = 1105,
    /// AnimatorStateMachine
    AnimatorStateMachine = 1107,
    /// PreviewAnimationClip
    PreviewAnimationClip = 1108,
    /// AnimatorTransition
    AnimatorTransition = 1109,
    /// SpeedTreeImporter
    SpeedTreeImporter = 1110,
    /// AnimatorTransitionBase
    AnimatorTransitionBase = 1111,
    /// SubstanceImporter
    SubstanceImporter = 1112,
    /// LightmapParameters
    LightmapParameters = 1113,
    /// LightingDataAsset
    LightingDataAsset = 1120,
    /// GISRaster
    GISRaster = 1121,
    /// GISRasterImporter
    GISRasterImporter = 1122,
    /// CadImporter
    CadImporter = 1123,
    /// SketchUpImporter
    SketchUpImporter = 1124,
    /// BuildReport
    BuildReport = 1125,
    /// PackedAssets
    PackedAssets = 1126,
    /// VideoClipImporter
    VideoClipImporter = 1127,
    /// ActivationLogComponent
    ActivationLogComponent = 2000,
    /// Int
    r#Int = 100000,
    /// Bool
    r#Bool = 100001,
    /// Float
    r#Float = 100002,
    /// MonoObject
    MonoObject = 100003,
    /// Collision
    Collision = 100004,
    /// Vector3f
    Vector3f = 100005,
    /// RootMotionData
    RootMotionData = 100006,
    /// Collision2D
    Collision2D = 100007,
    /// AudioMixerLiveUpdateFloat
    AudioMixerLiveUpdateFloat = 100008,
    /// AudioMixerLiveUpdateBool
    AudioMixerLiveUpdateBool = 100009,
    /// Polygon2D
    Polygon2D = 100010,
    /// Void
    r#Void = 100011,
    /// TilemapCollider2D
    TilemapCollider2D = 19719996,
    /// AssetImporterLog
    AssetImporterLog = 41386430,
    /// VFXRenderer
    VFXRenderer = 73398921,
    /// SerializableManagedRefTestClass
    SerializableManagedRefTestClass = 76251197,
    /// Grid
    Grid = 156049354,
    /// ScenesUsingAssets
    ScenesUsingAssets = 156483287,
    /// ArticulationBody
    ArticulationBody = 171741748,
    /// Preset
    Preset = 181963792,
    /// EmptyObject
    EmptyObject = 277625683,
    /// IConstraint
    IConstraint = 285090594,
    /// TestObjectWithSpecialLayoutOne
    TestObjectWithSpecialLayoutOne = 293259124,
    /// AssemblyDefinitionReferenceImporter
    AssemblyDefinitionReferenceImporter = 294290339,
    /// SiblingDerived
    SiblingDerived = 334799969,
    /// TestObjectWithSerializedMapStringNonAlignedStruct
    TestObjectWithSerializedMapStringNonAlignedStruct = 342846651,
    /// SubDerived
    SubDerived = 367388927,
    /// AssetImportInProgressProxy
    AssetImportInProgressProxy = 369655926,
    /// PluginBuildInfo
    PluginBuildInfo = 382020655,
    /// EditorProjectAccess
    EditorProjectAccess = 426301858,
    /// PrefabImporter
    PrefabImporter = 468431735,
    /// TestObjectWithSerializedArray
    TestObjectWithSerializedArray = 478637458,
    /// TestObjectWithSerializedAnimationCurve
    TestObjectWithSerializedAnimationCurve = 478637459,
    /// TilemapRenderer
    TilemapRenderer = 483693784,
    /// ScriptableCamera
    ScriptableCamera = 488575907,
    /// SpriteAtlasAsset
    SpriteAtlasAsset = 612988286,
    /// SpriteAtlasDatabase
    SpriteAtlasDatabase = 638013454,
    /// AudioBuildInfo
    AudioBuildInfo = 641289076,
    /// CachedSpriteAtlasRuntimeData
    CachedSpriteAtlasRuntimeData = 644342135,
    /// RendererFake
    RendererFake = 646504946,
    /// AssemblyDefinitionReferenceAsset
    AssemblyDefinitionReferenceAsset = 662584278,
    /// BuiltAssetBundleInfoSet
    BuiltAssetBundleInfoSet = 668709126,
    /// SpriteAtlas
    SpriteAtlas = 687078895,
    /// RayTracingShaderImporter
    RayTracingShaderImporter = 747330370,
    /// RayTracingShader
    RayTracingShader = 825902497,
    /// LightingSettings
    LightingSettings = 850595691,
    /// PlatformModuleSetup
    PlatformModuleSetup = 877146078,
    /// VersionControlSettings
    VersionControlSettings = 890905787,
    /// AimConstraint
    AimConstraint = 895512359,
    /// VFXManager
    VFXManager = 937362698,
    /// VisualEffectSubgraph
    VisualEffectSubgraph = 994735392,
    /// VisualEffectSubgraphOperator
    VisualEffectSubgraphOperator = 994735403,
    /// VisualEffectSubgraphBlock
    VisualEffectSubgraphBlock = 994735404,
    /// LocalizationImporter
    LocalizationImporter = 1027052791,
    /// Derived
    Derived = 1091556383,
    /// PropertyModificationsTargetTestObject
    PropertyModificationsTargetTestObject = 1111377672,
    /// ReferencesArtifactGenerator
    ReferencesArtifactGenerator = 1114811875,
    /// AssemblyDefinitionAsset
    AssemblyDefinitionAsset = 1152215463,
    /// SceneVisibilityState
    SceneVisibilityState = 1154873562,
    /// LookAtConstraint
    LookAtConstraint = 1183024399,
    /// SpriteAtlasImporter
    SpriteAtlasImporter = 1210832254,
    /// MultiArtifactTestImporter
    MultiArtifactTestImporter = 1223240404,
    /// GameObjectRecorder
    GameObjectRecorder = 1268269756,
    /// LightingDataAssetParent
    LightingDataAssetParent = 1325145578,
    /// PresetManager
    PresetManager = 1386491679,
    /// TestObjectWithSpecialLayoutTwo
    TestObjectWithSpecialLayoutTwo = 1392443030,
    /// StreamingManager
    StreamingManager = 1403656975,
    /// LowerResBlitTexture
    LowerResBlitTexture = 1480428607,
    /// StreamingController
    StreamingController = 1542919678,
    /// RenderPassAttachment
    RenderPassAttachment = 1571458007,
    /// TestObjectVectorPairStringBool
    TestObjectVectorPairStringBool = 1628831178,
    /// GridLayout
    GridLayout = 1742807556,
    /// AssemblyDefinitionImporter
    AssemblyDefinitionImporter = 1766753193,
    /// ParentConstraint
    ParentConstraint = 1773428102,
    /// FakeComponent
    FakeComponent = 1803986026,
    /// PositionConstraint
    PositionConstraint = 1818360608,
    /// RotationConstraint
    RotationConstraint = 1818360609,
    /// ScaleConstraint
    ScaleConstraint = 1818360610,
    /// Tilemap
    Tilemap = 1839735485,
    /// PackageManifest
    PackageManifest = 1896753125,
    /// PackageManifestImporter
    PackageManifestImporter = 1896753126,
    /// TerrainLayer
    TerrainLayer = 1953259897,
    /// SpriteShapeRenderer
    SpriteShapeRenderer = 1971053207,
    /// NativeObjectType
    NativeObjectType = 1977754360,
    /// TestObjectWithSerializedMapStringBool
    TestObjectWithSerializedMapStringBool = 1981279845,
    /// SerializableManagedHost
    SerializableManagedHost = 1995898324,
    /// VisualEffectAsset
    VisualEffectAsset = 2058629509,
    /// VisualEffectImporter
    VisualEffectImporter = 2058629510,
    /// VisualEffectResource
    VisualEffectResource = 2058629511,
    /// VisualEffectObject
    VisualEffectObject = 2059678085,
    /// VisualEffect
    VisualEffect = 2083052967,
    /// LocalizationAsset
    LocalizationAsset = 2083778819,
    /// ScriptedImporter
    ScriptedImporter = 2089858483,
}

impl ClassIDType {
    /// Creates a ClassIDType from an i32 value
    ///
    /// Returns a ClassIDType variant or UnknownType for unknown values.
    pub fn from_i32(value: i32) -> Self {
        match value {
            -1 => ClassIDType::UnknownType,
            0 => ClassIDType::Object,
            1 => ClassIDType::GameObject,
            2 => ClassIDType::Component,
            3 => ClassIDType::LevelGameManager,
            4 => ClassIDType::Transform,
            5 => ClassIDType::TimeManager,
            6 => ClassIDType::GlobalGameManager,
            8 => ClassIDType::Behaviour,
            9 => ClassIDType::GameManager,
            11 => ClassIDType::AudioManager,
            12 => ClassIDType::ParticleAnimator,
            13 => ClassIDType::InputManager,
            15 => ClassIDType::EllipsoidParticleEmitter,
            17 => ClassIDType::Pipeline,
            18 => ClassIDType::EditorExtension,
            19 => ClassIDType::Physics2DSettings,
            20 => ClassIDType::Camera,
            21 => ClassIDType::Material,
            23 => ClassIDType::MeshRenderer,
            25 => ClassIDType::Renderer,
            26 => ClassIDType::ParticleRenderer,
            27 => ClassIDType::Texture,
            28 => ClassIDType::Texture2D,
            29 => ClassIDType::OcclusionCullingSettings,
            30 => ClassIDType::GraphicsSettings,
            33 => ClassIDType::MeshFilter,
            41 => ClassIDType::OcclusionPortal,
            43 => ClassIDType::Mesh,
            45 => ClassIDType::Skybox,
            47 => ClassIDType::QualitySettings,
            48 => ClassIDType::Shader,
            49 => ClassIDType::TextAsset,
            50 => ClassIDType::Rigidbody2D,
            51 => ClassIDType::Physics2DManager,
            53 => ClassIDType::Collider2D,
            54 => ClassIDType::Rigidbody,
            55 => ClassIDType::PhysicsManager,
            56 => ClassIDType::Collider,
            57 => ClassIDType::Joint,
            58 => ClassIDType::CircleCollider2D,
            59 => ClassIDType::HingeJoint,
            60 => ClassIDType::PolygonCollider2D,
            61 => ClassIDType::BoxCollider2D,
            62 => ClassIDType::PhysicsMaterial2D,
            64 => ClassIDType::MeshCollider,
            65 => ClassIDType::BoxCollider,
            66 => ClassIDType::CompositeCollider2D,
            68 => ClassIDType::EdgeCollider2D,
            70 => ClassIDType::CapsuleCollider2D,
            72 => ClassIDType::ComputeShader,
            74 => ClassIDType::AnimationClip,
            75 => ClassIDType::ConstantForce,
            76 => ClassIDType::WorldParticleCollider,
            78 => ClassIDType::TagManager,
            81 => ClassIDType::AudioListener,
            82 => ClassIDType::AudioSource,
            83 => ClassIDType::AudioClip,
            84 => ClassIDType::RenderTexture,
            86 => ClassIDType::CustomRenderTexture,
            87 => ClassIDType::MeshParticleEmitter,
            88 => ClassIDType::ParticleEmitter,
            89 => ClassIDType::Cubemap,
            90 => ClassIDType::Avatar,
            91 => ClassIDType::AnimatorController,
            92 => ClassIDType::GUILayer,
            93 => ClassIDType::RuntimeAnimatorController,
            94 => ClassIDType::ScriptMapper,
            95 => ClassIDType::Animator,
            96 => ClassIDType::TrailRenderer,
            98 => ClassIDType::DelayedCallManager,
            102 => ClassIDType::TextMesh,
            104 => ClassIDType::RenderSettings,
            108 => ClassIDType::Light,
            109 => ClassIDType::CGProgram,
            110 => ClassIDType::BaseAnimationTrack,
            111 => ClassIDType::Animation,
            114 => ClassIDType::MonoBehaviour,
            115 => ClassIDType::MonoScript,
            116 => ClassIDType::MonoManager,
            117 => ClassIDType::Texture3D,
            118 => ClassIDType::NewAnimationTrack,
            119 => ClassIDType::Projector,
            120 => ClassIDType::LineRenderer,
            121 => ClassIDType::Flare,
            122 => ClassIDType::Halo,
            123 => ClassIDType::LensFlare,
            124 => ClassIDType::FlareLayer,
            125 => ClassIDType::HaloLayer,
            126 => ClassIDType::NavMeshProjectSettings,
            127 => ClassIDType::HaloManager,
            128 => ClassIDType::Font,
            129 => ClassIDType::PlayerSettings,
            130 => ClassIDType::NamedObject,
            131 => ClassIDType::GUITexture,
            132 => ClassIDType::GUIText,
            133 => ClassIDType::GUIElement,
            134 => ClassIDType::PhysicMaterial,
            135 => ClassIDType::SphereCollider,
            136 => ClassIDType::CapsuleCollider,
            137 => ClassIDType::SkinnedMeshRenderer,
            138 => ClassIDType::FixedJoint,
            140 => ClassIDType::RaycastCollider,
            141 => ClassIDType::BuildSettings,
            142 => ClassIDType::AssetBundle,
            143 => ClassIDType::CharacterController,
            144 => ClassIDType::CharacterJoint,
            145 => ClassIDType::SpringJoint,
            146 => ClassIDType::WheelCollider,
            147 => ClassIDType::ResourceManager,
            148 => ClassIDType::NetworkView,
            149 => ClassIDType::NetworkManager,
            150 => ClassIDType::PreloadData,
            152 => ClassIDType::MovieTexture,
            153 => ClassIDType::ConfigurableJoint,
            154 => ClassIDType::TerrainCollider,
            155 => ClassIDType::MasterServerInterface,
            156 => ClassIDType::TerrainData,
            157 => ClassIDType::LightmapSettings,
            158 => ClassIDType::WebCamTexture,
            159 => ClassIDType::EditorSettings,
            160 => ClassIDType::InteractiveCloth,
            161 => ClassIDType::ClothRenderer,
            162 => ClassIDType::EditorUserSettings,
            163 => ClassIDType::SkinnedCloth,
            164 => ClassIDType::AudioReverbFilter,
            165 => ClassIDType::AudioHighPassFilter,
            166 => ClassIDType::AudioChorusFilter,
            167 => ClassIDType::AudioReverbZone,
            168 => ClassIDType::AudioEchoFilter,
            169 => ClassIDType::AudioLowPassFilter,
            170 => ClassIDType::AudioDistortionFilter,
            171 => ClassIDType::SparseTexture,
            180 => ClassIDType::AudioBehaviour,
            181 => ClassIDType::AudioFilter,
            182 => ClassIDType::WindZone,
            183 => ClassIDType::Cloth,
            184 => ClassIDType::SubstanceArchive,
            185 => ClassIDType::ProceduralMaterial,
            186 => ClassIDType::ProceduralTexture,
            187 => ClassIDType::Texture2DArray,
            188 => ClassIDType::CubemapArray,
            191 => ClassIDType::OffMeshLink,
            192 => ClassIDType::OcclusionArea,
            193 => ClassIDType::Tree,
            194 => ClassIDType::NavMeshObsolete,
            195 => ClassIDType::NavMeshAgent,
            196 => ClassIDType::NavMeshSettings,
            197 => ClassIDType::LightProbesLegacy,
            198 => ClassIDType::ParticleSystem,
            199 => ClassIDType::ParticleSystemRenderer,
            200 => ClassIDType::ShaderVariantCollection,
            205 => ClassIDType::LODGroup,
            206 => ClassIDType::BlendTree,
            207 => ClassIDType::Motion,
            208 => ClassIDType::NavMeshObstacle,
            210 => ClassIDType::SortingGroup,
            212 => ClassIDType::SpriteRenderer,
            213 => ClassIDType::Sprite,
            214 => ClassIDType::CachedSpriteAtlas,
            215 => ClassIDType::ReflectionProbe,
            216 => ClassIDType::ReflectionProbes,
            218 => ClassIDType::Terrain,
            220 => ClassIDType::LightProbeGroup,
            221 => ClassIDType::AnimatorOverrideController,
            222 => ClassIDType::CanvasRenderer,
            223 => ClassIDType::Canvas,
            224 => ClassIDType::RectTransform,
            225 => ClassIDType::CanvasGroup,
            226 => ClassIDType::BillboardAsset,
            227 => ClassIDType::BillboardRenderer,
            228 => ClassIDType::SpeedTreeWindAsset,
            229 => ClassIDType::AnchoredJoint2D,
            230 => ClassIDType::Joint2D,
            231 => ClassIDType::SpringJoint2D,
            232 => ClassIDType::DistanceJoint2D,
            233 => ClassIDType::HingeJoint2D,
            234 => ClassIDType::SliderJoint2D,
            235 => ClassIDType::WheelJoint2D,
            236 => ClassIDType::ClusterInputManager,
            237 => ClassIDType::BaseVideoTexture,
            238 => ClassIDType::NavMeshData,
            240 => ClassIDType::AudioMixer,
            241 => ClassIDType::AudioMixerController,
            243 => ClassIDType::AudioMixerGroupController,
            244 => ClassIDType::AudioMixerEffectController,
            245 => ClassIDType::AudioMixerSnapshotController,
            246 => ClassIDType::PhysicsUpdateBehaviour2D,
            247 => ClassIDType::ConstantForce2D,
            248 => ClassIDType::Effector2D,
            249 => ClassIDType::AreaEffector2D,
            250 => ClassIDType::PointEffector2D,
            251 => ClassIDType::PlatformEffector2D,
            252 => ClassIDType::SurfaceEffector2D,
            253 => ClassIDType::BuoyancyEffector2D,
            254 => ClassIDType::RelativeJoint2D,
            255 => ClassIDType::FixedJoint2D,
            256 => ClassIDType::FrictionJoint2D,
            257 => ClassIDType::TargetJoint2D,
            258 => ClassIDType::LightProbes,
            259 => ClassIDType::LightProbeProxyVolume,
            271 => ClassIDType::SampleClip,
            272 => ClassIDType::AudioMixerSnapshot,
            273 => ClassIDType::AudioMixerGroup,
            280 => ClassIDType::NScreenBridge,
            290 => ClassIDType::AssetBundleManifest,
            292 => ClassIDType::UnityAdsManager,
            300 => ClassIDType::RuntimeInitializeOnLoadManager,
            301 => ClassIDType::CloudWebServicesManager,
            303 => ClassIDType::UnityAnalyticsManager,
            304 => ClassIDType::CrashReportManager,
            305 => ClassIDType::PerformanceReportingManager,
            310 => ClassIDType::UnityConnectSettings,
            319 => ClassIDType::AvatarMask,
            320 => ClassIDType::PlayableDirector,
            328 => ClassIDType::VideoPlayer,
            329 => ClassIDType::VideoClip,
            330 => ClassIDType::ParticleSystemForceField,
            331 => ClassIDType::SpriteMask,
            362 => ClassIDType::WorldAnchor,
            363 => ClassIDType::OcclusionCullingData,
            1000 => ClassIDType::SmallestEditorClassID,
            1001 => ClassIDType::PrefabInstance,
            1002 => ClassIDType::EditorExtensionImpl,
            1003 => ClassIDType::AssetImporter,
            1004 => ClassIDType::AssetDatabaseV1,
            1005 => ClassIDType::Mesh3DSImporter,
            1006 => ClassIDType::TextureImporter,
            1007 => ClassIDType::ShaderImporter,
            1008 => ClassIDType::ComputeShaderImporter,
            1020 => ClassIDType::AudioImporter,
            1026 => ClassIDType::HierarchyState,
            1027 => ClassIDType::GUIDSerializer,
            1028 => ClassIDType::AssetMetaData,
            1029 => ClassIDType::DefaultAsset,
            1030 => ClassIDType::DefaultImporter,
            1031 => ClassIDType::TextScriptImporter,
            1032 => ClassIDType::SceneAsset,
            1034 => ClassIDType::NativeFormatImporter,
            1035 => ClassIDType::MonoImporter,
            1037 => ClassIDType::AssetServerCache,
            1038 => ClassIDType::LibraryAssetImporter,
            1040 => ClassIDType::ModelImporter,
            1041 => ClassIDType::FBXImporter,
            1042 => ClassIDType::TrueTypeFontImporter,
            1044 => ClassIDType::MovieImporter,
            1045 => ClassIDType::EditorBuildSettings,
            1046 => ClassIDType::DDSImporter,
            1048 => ClassIDType::InspectorExpandedState,
            1049 => ClassIDType::AnnotationManager,
            1050 => ClassIDType::PluginImporter,
            1051 => ClassIDType::EditorUserBuildSettings,
            1052 => ClassIDType::PVRImporter,
            1053 => ClassIDType::ASTCImporter,
            1054 => ClassIDType::KTXImporter,
            1055 => ClassIDType::IHVImageFormatImporter,
            1101 => ClassIDType::AnimatorStateTransition,
            1102 => ClassIDType::AnimatorState,
            1105 => ClassIDType::HumanTemplate,
            1107 => ClassIDType::AnimatorStateMachine,
            1108 => ClassIDType::PreviewAnimationClip,
            1109 => ClassIDType::AnimatorTransition,
            1110 => ClassIDType::SpeedTreeImporter,
            1111 => ClassIDType::AnimatorTransitionBase,
            1112 => ClassIDType::SubstanceImporter,
            1113 => ClassIDType::LightmapParameters,
            1120 => ClassIDType::LightingDataAsset,
            1121 => ClassIDType::GISRaster,
            1122 => ClassIDType::GISRasterImporter,
            1123 => ClassIDType::CadImporter,
            1124 => ClassIDType::SketchUpImporter,
            1125 => ClassIDType::BuildReport,
            1126 => ClassIDType::PackedAssets,
            1127 => ClassIDType::VideoClipImporter,
            2000 => ClassIDType::ActivationLogComponent,
            100000 => ClassIDType::r#Int,
            100001 => ClassIDType::r#Bool,
            100002 => ClassIDType::r#Float,
            100003 => ClassIDType::MonoObject,
            100004 => ClassIDType::Collision,
            100005 => ClassIDType::Vector3f,
            100006 => ClassIDType::RootMotionData,
            100007 => ClassIDType::Collision2D,
            100008 => ClassIDType::AudioMixerLiveUpdateFloat,
            100009 => ClassIDType::AudioMixerLiveUpdateBool,
            100010 => ClassIDType::Polygon2D,
            100011 => ClassIDType::r#Void,
            19719996 => ClassIDType::TilemapCollider2D,
            41386430 => ClassIDType::AssetImporterLog,
            73398921 => ClassIDType::VFXRenderer,
            76251197 => ClassIDType::SerializableManagedRefTestClass,
            156049354 => ClassIDType::Grid,
            156483287 => ClassIDType::ScenesUsingAssets,
            171741748 => ClassIDType::ArticulationBody,
            181963792 => ClassIDType::Preset,
            277625683 => ClassIDType::EmptyObject,
            285090594 => ClassIDType::IConstraint,
            293259124 => ClassIDType::TestObjectWithSpecialLayoutOne,
            294290339 => ClassIDType::AssemblyDefinitionReferenceImporter,
            334799969 => ClassIDType::SiblingDerived,
            342846651 => ClassIDType::TestObjectWithSerializedMapStringNonAlignedStruct,
            367388927 => ClassIDType::SubDerived,
            369655926 => ClassIDType::AssetImportInProgressProxy,
            382020655 => ClassIDType::PluginBuildInfo,
            426301858 => ClassIDType::EditorProjectAccess,
            468431735 => ClassIDType::PrefabImporter,
            478637458 => ClassIDType::TestObjectWithSerializedArray,
            478637459 => ClassIDType::TestObjectWithSerializedAnimationCurve,
            483693784 => ClassIDType::TilemapRenderer,
            488575907 => ClassIDType::ScriptableCamera,
            612988286 => ClassIDType::SpriteAtlasAsset,
            638013454 => ClassIDType::SpriteAtlasDatabase,
            641289076 => ClassIDType::AudioBuildInfo,
            644342135 => ClassIDType::CachedSpriteAtlasRuntimeData,
            646504946 => ClassIDType::RendererFake,
            662584278 => ClassIDType::AssemblyDefinitionReferenceAsset,
            668709126 => ClassIDType::BuiltAssetBundleInfoSet,
            687078895 => ClassIDType::SpriteAtlas,
            747330370 => ClassIDType::RayTracingShaderImporter,
            825902497 => ClassIDType::RayTracingShader,
            850595691 => ClassIDType::LightingSettings,
            877146078 => ClassIDType::PlatformModuleSetup,
            890905787 => ClassIDType::VersionControlSettings,
            895512359 => ClassIDType::AimConstraint,
            937362698 => ClassIDType::VFXManager,
            994735392 => ClassIDType::VisualEffectSubgraph,
            994735403 => ClassIDType::VisualEffectSubgraphOperator,
            994735404 => ClassIDType::VisualEffectSubgraphBlock,
            1027052791 => ClassIDType::LocalizationImporter,
            1091556383 => ClassIDType::Derived,
            1111377672 => ClassIDType::PropertyModificationsTargetTestObject,
            1114811875 => ClassIDType::ReferencesArtifactGenerator,
            1152215463 => ClassIDType::AssemblyDefinitionAsset,
            1154873562 => ClassIDType::SceneVisibilityState,
            1183024399 => ClassIDType::LookAtConstraint,
            1210832254 => ClassIDType::SpriteAtlasImporter,
            1223240404 => ClassIDType::MultiArtifactTestImporter,
            1268269756 => ClassIDType::GameObjectRecorder,
            1325145578 => ClassIDType::LightingDataAssetParent,
            1386491679 => ClassIDType::PresetManager,
            1392443030 => ClassIDType::TestObjectWithSpecialLayoutTwo,
            1403656975 => ClassIDType::StreamingManager,
            1480428607 => ClassIDType::LowerResBlitTexture,
            1542919678 => ClassIDType::StreamingController,
            1571458007 => ClassIDType::RenderPassAttachment,
            1628831178 => ClassIDType::TestObjectVectorPairStringBool,
            1742807556 => ClassIDType::GridLayout,
            1766753193 => ClassIDType::AssemblyDefinitionImporter,
            1773428102 => ClassIDType::ParentConstraint,
            1803986026 => ClassIDType::FakeComponent,
            1818360608 => ClassIDType::PositionConstraint,
            1818360609 => ClassIDType::RotationConstraint,
            1818360610 => ClassIDType::ScaleConstraint,
            1839735485 => ClassIDType::Tilemap,
            1896753125 => ClassIDType::PackageManifest,
            1896753126 => ClassIDType::PackageManifestImporter,
            1953259897 => ClassIDType::TerrainLayer,
            1971053207 => ClassIDType::SpriteShapeRenderer,
            1977754360 => ClassIDType::NativeObjectType,
            1981279845 => ClassIDType::TestObjectWithSerializedMapStringBool,
            1995898324 => ClassIDType::SerializableManagedHost,
            2058629509 => ClassIDType::VisualEffectAsset,
            2058629510 => ClassIDType::VisualEffectImporter,
            2058629511 => ClassIDType::VisualEffectResource,
            2059678085 => ClassIDType::VisualEffectObject,
            2083052967 => ClassIDType::VisualEffect,
            2083778819 => ClassIDType::LocalizationAsset,
            2089858483 => ClassIDType::ScriptedImporter,
            _ => ClassIDType::UnknownType,
        }
    }

    /// Converts the ClassIDType to its i32 representation
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

impl From<i32> for ClassIDType {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<ClassIDType> for i32 {
    fn from(value: ClassIDType) -> Self {
        value.to_i32()
    }
}
