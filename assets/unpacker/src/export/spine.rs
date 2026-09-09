use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

use base64::Engine;
use serde_json::Value;

use super::alpha_merge;
use super::texture::decode_texture_object;

/// Spine animation category based on asset naming/content
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineCategory {
    BattleFront,
    BattleBack,
    /// The third battle facing: a skeleton hung off a GameObject the game names `Down`, a
    /// sibling of `Front` and `Back` under the prefab's `FaceSwitcher`, with its own muzzle
    /// and special points (three token skins carry one: Ironmn's pile3, Radian's tower2 and
    /// tower3, EN census 2026-09-09 over 917 bundles). Without this category it fell to the
    /// atlas heuristic, landed in `BattleFront` and raced the real Front for one path.
    BattleDown,
    Building,
    DynIllust,
    Enemy,
}

impl fmt::Display for SpineCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BattleFront => write!(f, "BattleFront"),
            Self::BattleBack => write!(f, "BattleBack"),
            Self::BattleDown => write!(f, "BattleDown"),
            Self::Building => write!(f, "Building"),
            Self::DynIllust => write!(f, "DynIllust"),
            Self::Enemy => write!(f, "Enemy"),
        }
    }
}

/// A grouped spine asset with properly-paired skel + atlas + textures
pub struct SpineAsset {
    /// Base name (e.g., "`char_002_amiya`" or "`build_char_002_amiya`")
    pub name: String,
    /// Raw skel binary data (decoded from base64)
    pub skel_data: Vec<u8>,
    /// Atlas text content
    pub atlas_text: String,
    /// Resolved texture objects (`path_id` → Value), keyed by texture name
    pub textures: Vec<(String, Value)>,
    /// Classification
    pub category: SpineCategory,
    /// Painted background scene for `DynIllust` bundles: every non-character
    /// mesh quad (painted layers + static effect meshes), rasterized and
    /// composited into one flat image. Empty for every other category and for
    /// dynchars whose background is baked into the spine atlas.
    pub bg_quads: Vec<BgQuad>,
    /// `SkeletonDataAsset.scale` (spine px → Unity units, typically `0.01`).
    pub bg_skel_scale: Option<f64>,
    /// The clip the game binds on this skeleton at settle: the `SkeletonMecanim`'s serialized
    /// `_animationName`. "Idle" on 86 of 104 dynchar bindings, but AUTHORED per skin: cel
    /// (`char_245_cello_sale#12`) binds "Interact" and `char_391_rosmon_2` binds "Special" on
    /// their MAIN `dyn_illust` skeletons, and a client that hardcodes "Idle" plays a different
    /// animation than the game on exactly those settled surfaces. None when the field is
    /// empty or names the entrance ("Start").
    pub settle_animation: Option<String>,
    /// Orthographic camera half-height, if a display controller exposes it.
    pub bg_camera_size: Option<f64>,
    /// Render-target aspect (`maxSize.x / maxSize.y`) for the camera frame.
    pub bg_max_aspect: Option<f64>,
    /// The display controller's `_maxSize` itself, in pixels, so the viewer's idle
    /// render-target size is data rather than a constant asserted from a client dump.
    pub bg_max_size: Option<(f64, f64)>,
    /// Authored display-frame CENTRE in spine-authored px, from the display
    /// controller's `_adjustes[0].offset` (the full-illustration adjust). The
    /// camera isn't centred on the skeleton root, so this is needed to frame the
    /// scene as the game does.
    pub bg_camera_offset: Option<(f64, f64)>,
    /// Authored display-frame full extent in spine-authored px, from
    /// `_adjustes[0].size` (square). Used as the framed view size.
    pub bg_camera_view: Option<f64>,
    /// Authored TIGHT display-frame CENTRE in spine-authored px, from the display
    /// controller's `_adjustes[1].offset` (the zoomed-in adjust the in-game viewer
    /// dollies out FROM at open). `None` when the controller has only one adjust.
    pub bg_camera_offset2: Option<(f64, f64)>,
    /// Authored TIGHT display-frame full extent in spine-authored px, from
    /// `_adjustes[1].size` (square) — the inner/zoomed camera endpoint.
    pub bg_camera_view2: Option<f64>,
    /// The character spine's `m_SortingOrder`, so the live scene renderer can
    /// place the animated character among the mesh layers at the right depth.
    pub bg_character_sort: Option<i64>,
    /// Spine-Unity `SkeletonRenderer.separatorSlotNames`: slots at which the game SPLITS this
    /// skeleton's draw into separate submeshes (`SkeletonRenderSeparator` + N
    /// `SkeletonPartsRenderer`s), so other renderers can be interleaved BETWEEN the parts.
    ///
    /// This is the authoritative background/character boundary, and nothing else expresses it —
    /// every one of Virtuosa's 328 slots shares the single root bone `root`, so no bone
    /// hierarchy can supply it. She splits at `B_C_R_Wing_E`, draw index 7, i.e. exactly after
    /// her seven `*_Door_*` architecture slots. Her `bg_*` haze sheets are authored ABOVE
    /// `characterSort` and belong in that gap: over the architecture, under the character.
    /// Empty on skins that do not use the feature (`char_245_cello_2`), which must stay
    /// byte-identical.
    pub separator_slots: Vec<String>,
    /// Sorting orders of this skeleton's separator PARTS, ascending — the depths the game gives
    /// the submeshes. Virtuosa: [0, 20]. A `bg_*` sheet whose own sort lands between two parts
    /// belongs in that gap (`bg_ref` 3, `bg_tint_01` 10, `air_01` 12); one above the last part is
    /// genuinely in front of the character (`bg_rain_01` 25, her rain). The blanket
    /// `isBackdropParticle` demotion gets BOTH of those wrong.
    pub separator_part_sorts: Vec<i64>,
    /// ENTRANCE (`_Start`) cinematic total length in seconds, from the entrance
    /// director `MonoBehaviour`'s `_params.duration` (the client plays the whole
    /// `_Start` set over this span, then hands off to the settled idle). `None`
    /// for the main set or a skin without an entrance director.
    pub bg_entrance_duration: Option<f64>,
    /// `m_StopTime` of the entrance CAMERA clip — the authored end of the cinematic's content,
    /// which can precede the director's nominal `duration`. See `anim::entrance_clip_stop`.
    pub bg_entrance_clip_stop: Option<f32>,
    /// ENTRANCE post-process: `(effect_name, intensity, weight_curve)` from the `pp`
    /// `PostProcessVolume` whose weight the `_Start` clip animates. `None` when the skin ships no
    /// volume or no clip drives it. See `anim::entrance_post_fx`.
    pub bg_entrance_post_fx: Option<super::anim::EntrancePostFx>,
    /// ENTRANCE screen-fade colour, from the director's `_params.fadeColor` (premultiplied
    /// straight RGBA, 0..1). The client fades the whole view to this colour as the entrance
    /// ends and then cuts to the settled idle — the recordings show Virtuosa reaching pure
    /// white by `duration - 0.2s`. Authored white at full alpha on every skin measured, but
    /// read from the data rather than assumed. `None` without an entrance director.
    pub bg_entrance_fade: Option<[f64; 4]>,
    /// The ENTRANCE camera's authored CLEAR colour (`m_BackGroundColor` rgb, 0..1) when it
    /// clears to a solid colour (`m_ClearFlags` 2). This is what the game shows wherever the
    /// art does not reach during the cinematic. Census 2026-08-30: 15 entrance cameras across
    /// the 87 dynchar bundles, all `m_ClearFlags` 2; 0.3382 grey on nine, white on cello and
    /// Muelsyse, 0.963 on Wiš'adel, warm on Ling, and BLACK on chyue, whose pillarbox is the
    /// one margin the viewer's fixed 77 grey was wrong for. `None` without an entrance camera.
    pub bg_entrance_clear: Option<[f64; 3]>,
    /// ENTRANCE transform beat in seconds — the time of the dominant late cluster
    /// of per-object `_delayTime`s (the reform/gala burst; Virtuosa: 12.0s). The
    /// camera has dollied to the wide stop and the character has reformed by here,
    /// so it's the natural entrance→idle hand-off point. `None` when no entrance.
    pub bg_entrance_transform: Option<f64>,
    /// ENTRANCE camera full view in authored px (`2 · orthographic_size / skeletonScale`)
    /// — the TIGHT close-up the `_Start` opens on (Virtuosa 598px, vs the 1929px display
    /// stop). The client dollies OUT from this to the display frame. `None` when no entrance.
    pub bg_entrance_view: Option<f64>,
    /// Is the entrance camera PERSPECTIVE? Then `entranceOrthoCurve` carries dolly DISTANCES,
    /// not ortho sizes, and `2*curve[0]/skeletonScale` is not a view extent — the client must
    /// use `entranceViewPx` (already the frustum height at the dolly's FIRST keyframe).
    pub bg_entrance_persp: bool,
    /// ENTRANCE camera AIM point relative to the skeleton root, in authored px `(dx, dy)`
    /// (Y-up). The `_Start` close-up is centred HERE, not on the hair-dragged character
    /// centroid — it frames the upper body / halo (Virtuosa dy≈+200). `None` when no entrance.
    pub bg_entrance_cam_offset: Option<(f64, f64)>,
    /// ENTRANCE camera dolly ZOOM, extracted from the `_Start` clip animating the Main
    /// Camera's orthographic size: `(time_s, ortho_size)` keyframes. The game's actual
    /// data-driven camera motion (hold → zoom in on the transform → zoom out to the
    /// standing reveal); there is NO positional pan. `None` when no clip animates it.
    pub bg_entrance_ortho_curve: Option<Vec<(f32, f32)>>,
    /// ENTRANCE camera POSITIONAL dolly (pan): `(time_s, progress 0..1)` normalising the
    /// animated camera-ancestor Transform position (the real game camera-move timing).
    pub bg_entrance_pan_curve: Option<Vec<(f32, f32)>>,
    /// ENTRANCE camera FRAME-CENTRE trajectory in authored px: `(time_s, cxPx, cyPx)`, accumulated
    /// from the full camera rig (pure gamedata). The frame is centred here each frame.
    pub bg_entrance_cam_center: Option<Vec<(f32, f32, f32)>>,
    /// ENTRANCE camera ROLL about the view axis, `[t_seconds, degrees]`. `None` unless the rig is
    /// actually rolled — 12 of the 13 entrance skins have an axis-aligned camera basis.
    pub bg_entrance_cam_roll: Option<Vec<(f32, f32)>>,
    /// ENTRANCE LETTERBOX window `[x0, y0, x1, y1]` in authored px, same space as the frame
    /// centre, paired with the BARS' own `m_SortingOrder`. `None` unless the prefab paints one —
    /// only Civilight Eterna does. The sort matters: her bars tie with the director's full-screen
    /// transition planes, which the game paints OVER them.
    pub bg_entrance_aperture: Option<([f32; 4], i32)>,
    /// ENTRANCE voice-line offset (s) from `_params.charVoiceOffset` — when the reformed
    /// cellist starts talking. The seated→standing hand-off beat (the standing form lives at
    /// a different rig position than the seated form, so the entrance hands off to the idle
    /// here rather than trying to reframe onto it). `None` when no entrance director.
    pub bg_entrance_voice: Option<f64>,
    /// Parsed `ParticleSystem`s for dynillusts (emitted as `[particles].json`).
    /// Empty for every other category.
    pub particles: Vec<super::particles::ParticleData>,
}

/// A scene quad's `Torappu/Particles-L2D/<Family>/…` dissolve + disturb masking
/// (`Ram/`, `Disturb/`, `Dissolve/` — see [`is_l2d_compositor`]). The shader carves
/// the quad's real silhouette out of `_DissolveTex` and
/// warps the lookup by `_DisturbTex`; a scene layer drawn as a plain tinted quad shows
/// the mask's whole bounding rectangle instead — Mlynar's entrance wind sheets cover
/// roughly three times the area the game gives them. The particle exporter has carried
/// this since day one (`particles::resolve_ram`); this is the same data for a mesh quad.
pub struct SceneRam {
    /// `_DissolveTex` `Texture2D` + its `path_id` and `[sx, sy, ox, oy]`.
    pub dissolve_pid: Option<i64>,
    pub dissolve_val: Option<Value>,
    pub dissolve_st: [f64; 4],
    /// `_RamTex` `Texture2D` + its `path_id` and `[sx, sy, ox, oy]` — the RAMP the family is
    /// named for. The decompiled fragment ends `col *= texture(_RamTex, TC1.xy)`, and its UV is a
    /// plain `uv * _RamTex_ST.xy + _RamTex_ST.zw` with NO scroll and NO custom-stream offset.
    ///
    /// This is what gives a Ram layer its SHAPE: the main slot is routinely a flat shared FLOW
    /// map (Executor's `cb_a_4` binds `flow_177` to `_MainTex` AND `_DisturbTex`, luminance mean
    /// 0.671 with p90 also 0.671 — i.e. constant), so without the ramp the layer paints a flat
    /// wash instead of the ramp's artwork.
    pub ram_pid: Option<i64>,
    pub ram_val: Option<Value>,
    pub ram_st: [f64; 4],
    /// `_DisturbTex` `Texture2D` + its `path_id` and `[sx, sy, ox, oy]`.
    pub disturb_pid: Option<i64>,
    pub disturb_val: Option<Value>,
    pub disturb_st: [f64; 4],
    /// SECOND dissolve map. The `Dissolve/` family multiplies TWO masks
    /// (`_DissolveTex_01` × `_DissolveTex_02`), each with its own threshold and border;
    /// `Ram/` and `Disturb/` carry only the first. None = single-map material.
    pub dissolve2_pid: Option<i64>,
    pub dissolve2_val: Option<Value>,
    pub dissolve2_st: [f64; 4],
    /// `_WeightTex` — a per-pixel WEIGHT on the disturb displacement, sampled `.xy`.
    ///
    /// The `Disturb Anchor` family multiplies the anchored disturb offset by this map BEFORE
    /// the intensity, so where the map is black the game displaces NOTHING and where it is
    /// white it displaces fully:
    ///
    /// ```glsl
    /// u_xlat16_8.xy = texture(_WeightTex, vs_TEXCOORD1.zw).xy;
    /// u_xlat2.xy    = u_xlat16_8.xy * u_xlat16_1.xy;              // weight * (sample - anchor)
    /// u_xlat16_1.xy = u_xlat2.xy * vec2(_IntensityU, _IntensityV) + vs_TEXCOORD0.xy;
    /// ```
    ///
    /// Ignoring it applies the FULL displacement everywhere. Skadi binds `mask_16` (mean 0.502,
    /// range 0.0-1.0) on 7 materials, i.e. we over-displace by ~2x on average there; her other
    /// two weight materials bind a uniformly-1.0 texture and are unaffected either way.
    /// 51 materials across 16 skins have the feature live (`_WEIGHT_ON` + a bound map).
    pub weight_pid: Option<i64>,
    pub weight_val: Option<Value>,
    pub weight_st: [f64; 4],
    pub amount2: f32,
    pub border_width2: f32,
    /// `_Edgecolor` + `_pow` — the RIM the `…edge` shader variants composite along the dissolve
    /// boundary: `rim = pow(1 - smoothstep(clamp(mask/_Edgecolor.w)), _pow)`, then
    /// `rgb = mix(rgb, _Edgecolor.rgb, rim)` and `a *= mix(1, _Edgecolor.w, rim)`. Only the
    /// variants whose NAME carries `edge` read it; on the others the property is residue and the
    /// program has no rim term at all (verified against both decompiled fragments). None = no rim.
    pub edge_color: Option<[f32; 4]>,
    pub edge_pow: f32,
    /// `_Amount` — the dissolve threshold; `_BorderWidth` — its edge softness.
    pub amount: f32,
    pub border_width: f32,
    /// `_IntensityU`/`_IntensityV` — how far the disturb sample displaces the lookup.
    pub intensity_u: f32,
    pub intensity_v: f32,
    /// Which lookups the disturb offset is applied to.
    pub disturb_influence_dissolve_uv: f32,
    pub disturb_influence_main_uv: f32,
    /// `_DissolveUSpeed`/`_DissolveVSpeed` and the disturb pair, in UV/second.
    pub dissolve_speed: [f32; 2],
    pub disturb_speed: [f32; 2],
    /// `Particles-L2D/Disturb/Disturb2` only: the SECOND noise lookup of `_DisturTex`, channel y,
    /// as `[scale, uSpeed, intensityU, intensityV]` derived from `_Noise2Param` exactly as the
    /// first lookup's fields are derived from `_Noise1Param`. See `disturb2_family`.
    pub disturb2: Option<[f32; 4]>,
    /// `_AnchorU`/`_AnchorV` — the ZERO POINT the disturb sample is measured against.
    ///
    /// The `Particles-L2D/Disturb/Disturb Anchor` family computes its displacement as
    /// `(sample - anchor) * intensity`, not `sample * intensity`:
    ///
    /// ```glsl
    /// u_xlat16_8   = texture(_DisturTex, uv).x;
    /// u_xlat16_1.xy = vec2(u_xlat16_8) + (-vec2(_AnchorU, _AnchorV));
    /// u_xlat16_1.xy = u_xlat16_1.xy * vec2(_IntensityU, _IntensityV) + vs_TEXCOORD0.xy;
    /// ```
    ///
    /// Without the subtraction a mid-grey map (sample ~0.5) displaces by a CONSTANT half an
    /// intensity instead of oscillating around zero, i.e. a static shift rather than a warp.
    /// Defaults to 0.0, which reproduces the previous `sample * intensity` exactly, so every
    /// already-shipping `Ram/` layer is unchanged.
    pub anchor_u: f32,
    pub anchor_v: f32,
    /// `_Rotation0..3` — a per-LOOKUP UV rotation about the (0.5, 0.5) texel centre, in DEGREES,
    /// ordered `[main, dissolve, ram, disturb]`.
    ///
    /// The shader variant is real and live: `_HG_UV_ROTATION` is enabled on **100 materials**
    /// across the corpus, and the decompiled `Ram/Disturb(CustomData)` vertex program rotates
    /// each lookup between its ST and its scroll —
    ///
    /// ```glsl
    /// u_xlat0.xy   = in_TEXCOORD0.xy * _DissolveTex_ST.xy + _DissolveTex_ST.zw;
    /// u_xlat16_2.xy = u_xlat0.xy + vec2(-0.5, -0.5);
    /// u_xlat6.x    = dot(u_xlat16_2.xy, _Rotation1.xz);
    /// u_xlat6.y    = dot(u_xlat16_2.xy, _Rotation1.yw);
    /// u_xlat0.xy   = u_xlat6.xy + vec2(0.5, 0.5);
    /// ```
    ///
    /// mapping `_Rotation0 -> _MainTex`, `1 -> _DissolveTex`, `2 -> _RamTex`, `3 -> _DisturbTex`.
    ///
    /// ⚠️ The MATERIALS DO NOT SERIALIZE `_Rotation0..3` — a UV-rotation `MonoBehaviour` (script
    /// `7163214010000414217`, `_propertyName` is the PREFIX) writes them at runtime from
    /// `_rotateTex1..4` + `angle1`/`_angle2..4`. Reading the material alone finds nothing, which
    /// is why this looked inert. Cello carries 57 of them (her stairs rotate the DISSOLVE lookup
    /// by ±38.5 degrees) and all 35 of her ram layers bind a dissolve mask with a real
    /// `amount` (0.10-0.53), so the carve is genuinely mis-oriented without this.
    pub uv_rot: [f32; 4],
}

/// One textured mesh quad of the background scene, resolved to world geometry
/// (in the spine root's frame) plus its draw state, ready to rasterize.
pub struct BgQuad {
    /// Triangle geometry, mesh-local space.
    pub mesh: super::mesh::MeshData,
    /// `_MainTex` `Texture2D` object.
    pub tex_val: Value,
    /// Optional `_AlphaTex` `Texture2D` object (merged on decode).
    pub alpha_val: Option<Value>,
    /// `_Color`/`_TintColor` multiply (RGBA, 0..1).
    pub tint: [f32; 4],
    /// Additive blend (glow/fx) vs alpha-over (painted layers).
    pub additive: bool,
    /// World matrix in the spine root's local frame.
    pub world: super::mesh::Mat4,
    /// Renderer draw order.
    pub sort: i64,
    /// World-space Z of the quad origin (draw-order tie-break).
    pub z: f32,
    /// `_MainTex` `Texture2D` `path_id` (for claiming against later phases).
    pub tex_pid: i64,
    /// Static `_MainTex_ST` `[scaleX, scaleY, offsetX, offsetY]` — WHICH SUB-RECT of
    /// `tex_pid` this quad samples. Already baked into `mesh.uvs` (unless an `st_curve`
    /// supersedes it); kept here so the frozen-burst heuristic can tell a quad that
    /// samples the SAME atlas region as a particle system (a baked copy of that burst)
    /// from one that samples a DIFFERENT region of a shared atlas (distinct art).
    pub st: [f64; 4],
    /// Material `_SrcBlend`/`_DstBlend` factors, for blend-class classification
    /// (distortion-drop / glass-additive) in `export_scene`.
    pub src_blend: f64,
    pub dst_blend: f64,
    /// `Particles-L2D/Mask/Erase` only: the material's `_Strength`. The program writes
    /// `rgb 0, alpha (1 - tex.x) * _Strength` under an ordinary SrcAlpha/OneMinusSrcAlpha
    /// pass, so at rest the quad is an authored black polygon carving the composition's
    /// edge (Ch'en the Holungday E2's five, 17 to 350 vertices, statically active, no clip on
    /// them). Emitted as `erase` instead of being dropped as a grab-pass map.
    pub erase: Option<f32>,
    /// ENTRANCE uniform-scale MULTIPLIER keyframes for this layer's own transform, relative
    /// to the prefab pose the static `mesh` was baked at (so 1.0 = unchanged). `None` when the
    /// `_Start` clips do not animate this transform's scale.
    ///
    /// Scene quads used to consume no transform curve at all — the geometry was frozen at the
    /// prefab pose — which is why Executor's scope rim exported as a static quad while the game
    /// animates it. Decoding her curve and scaling the measured aperture by it reproduces the
    /// capture's radius to ~1% at four of five beats, so this is the missing input rather than an
    /// anchoring error. See `entrance_transform_curves`.
    /// `(t, sx, sy)`. BOTH axes: Kal'tsit's `st` scales x by 2.967 against y by 2.659, so a single
    /// uniform factor cannot carry it and averaging the two would be a fitted constant.
    pub scale_curve: Option<Vec<(f32, f32, f32)>>,
    /// Fixed point for `scale_curve`, in authored px (Y-up). `None` alongside an absent curve.
    pub scale_pivot: Option<[f32; 2]>,
    /// ENTRANCE Transform POSITION keyframes for the quad's animated owner, as an OFFSET
    /// from its prefab pose, in UNITY units (scaled to authored px by `inv` at emission,
    /// like `follow.origin`). `entrance_transform_curves` has always decoded these alongside
    /// the scale; nothing consumed them, so every animated scene transform exported frozen.
    pub pos_curve: Option<Vec<(f32, f32, f32)>>,
    /// ENTRANCE reveal time (seconds) — when this layer's `GameObject` (or a nearest
    /// ancestor group) is switched ON by an `m_IsActive` curve in the `_Start` clips.
    /// `None` = always active (visible from t=0). Only `_Start` scenes carry non-None.
    pub active_from: Option<f32>,
    /// ENTRANCE hide time (seconds) — when this layer's `GameObject` (or nearest ancestor)
    /// is switched OFF by an `m_IsActive` curve in the `_Start` clips (the cinematic's
    /// environment SWAP). `None` = never hidden (visible to the end). Only `_Start` scenes.
    pub active_until: Option<f32>,
    /// The FULL visibility schedule when the clip authors MORE THAN ONE window (empty
    /// otherwise, so the common single-window case keeps `active_from`/`active_until` alone).
    /// Emitted as `activeWindows`; the renderer draws the layer inside ANY of them.
    pub active_windows: Vec<super::anim::ActiveWindow>,
    /// CROSS-ROOT reveal (seconds): the layer belongs to ANOTHER skeleton's prefab
    /// root (the idle world inside an entrance scene) and only becomes visible when
    /// the game activates that prefab at the director's transform beat. Kept apart
    /// from `active_from`: it gates VISIBILITY only — the layer is scenery, not a
    /// clip-authored overlay, so the frontend's overlay exemptions must not apply.
    pub root_reveal_from: Option<f32>,
    /// ENTRANCE material-colour animation `(t_seconds, rgba)` — the `_Start` clip's
    /// animated material colour resolved against the static tint (see
    /// [`super::anim::layer_color_curve`]), e.g. Mlynar's white flash fading 0→0.671.
    /// `None` = the material colour isn't animated (the static `tint` stands).
    pub color_curve: Option<Vec<(f32, [f32; 4])>>,
    /// SHADER UV-SCROLL (Capability A): the Ram flowing-light shader family
    /// (`Torappu/Particles-L2D/Ram/{Disturb,VertexDisturb}`, `_shaderName` contains
    /// `"Ram/"`) scrolls `_MainTex` continuously against Unity `_Time` via the STATIC
    /// material floats `_MainUSpeed`/`_MainVSpeed` (UV/sec, no `AnimationClip`). Baking only
    /// the static ST froze it; this carries the per-second UV velocity `[u, v]` (Unity UV
    /// space) so the frontend re-adds `_Time · speed` each frame. `None` unless the
    /// material is a Ram-family shader with a non-zero `_Main*Speed`.
    pub uv_scroll: Option<[f32; 2]>,
    /// CLIP `_MainTex_ST` curve (Capability B): absolute texture Scale/Offset samples
    /// `(t_seconds, [scaleX, scaleY, offsetX, offsetY])` animated by the `_Start` entrance
    /// clip (e.g. Skadi2's `01 (4)`/`01 (5)` offset-Y sweep). When present, the STATIC `_MainTex`
    /// ST is NOT baked into the UVs (the curve carries the full ST, including its static
    /// components); the frontend applies `uv·[sx,sy]+[ox,oy]` per frame during the entrance.
    /// `None` = no animated ST (the static ST is baked as before).
    pub st_curve: Option<Vec<(f32, [f32; 4])>>,
    /// DISSOLVE + DISTURB masking (see [`SceneRam`]). `None` unless the material is a
    /// sub-namespaced `Particles-L2D` compositor that actually binds one of the two masks.
    pub ram: Option<SceneRam>,
    /// spine-unity `BoneFollower` in the quad's ancestry — the quad rides a SPINE BONE
    /// at runtime, so its serialized transform (and therefore the baked `pos` above) is
    /// only an editor pose. `None` = a world-fixed scene quad (the common case).
    pub follow: Option<BgFollow>,
    /// SOURCE `GameObject` `path_id`, for diagnostics only — never emitted to JSON.
    ///
    /// Scene layers export ANONYMOUSLY (`idx`/`pos`/`sort`/`tex`/`tint`/`uv` and nothing else),
    /// so there is no way to ask "which object is layer 23?" from the output. That gap silently
    /// invalidated a corpus scan: matching scroller `GameObjects` against exported layer names
    /// returned 0 hits on all nine references — not because the objects were unexported, but
    /// because layers carry no name to match. `DYNCHAR_LAYERMAP=1` prints the mapping at the
    /// emission site so attribution is possible without guessing at index alignment (which
    /// the drop heuristics break anyway).
    pub go_pid: i64,
    /// Source `GameObject` name, resolved at construction (where the object map is in scope).
    /// Diagnostics only — never emitted.
    pub go_name: String,
    /// Is this quad a DESCENDANT of the entrance camera? Then it rides the camera and holds a
    /// CONSTANT position and size on screen, however far the shot dollies or pans — a film-strip
    /// border, a lens overlay, a full-frame haze sheet. Baking its rest-pose world position (what
    /// every other layer wants) instead nails it to the world and the camera flies off it.
    /// Only two skins in the corpus have any: whitw2 (9, her film strip) and kalts (2, her mist).
    pub cam_locked: bool,
    /// For a {@link `cam_locked`} quad: the camera frustum EXTENT (authored px) at ITS OWN distance
    /// from the camera — the height the viewport maps to where this overlay sits. Under a
    /// PERSPECTIVE rig that is NOT the camera's `entranceViewPx`, and using the latter scales and
    /// positions the overlay by the ratio of the two distances.
    pub cam_lock_view: Option<f64>,
}

/// A scene quad's runtime bone attachment (spine-unity `BoneFollower`). At runtime the
/// follower snaps its `GameObject` onto `bone`, so the quad's true world matrix is
/// `bone(t) · followerWorld⁻¹ · quadWorld` — the frontend replays exactly that delta
/// against the baked geometry.
pub struct BgFollow {
    /// Followed spine bone name.
    pub bone: String,
    /// The follower's `followBoneRotation`: false = translation-only tracking.
    pub rot: bool,
    /// The follower `GameObject`'s world ORIGIN in the spine root's frame (Unity units,
    /// Y-up); scaled to authored px at emit time.
    pub origin: [f32; 2],
    /// The follower `GameObject`'s world 2×2 LINEAR basis (Y-up), row-major
    /// `[m00, m01, m10, m11]`. Scale-invariant, so it needs no unit conversion.
    pub basis: [f32; 4],
}

/// Everything [`collect_dynchar_bg_quads`] resolves from a dynillust prefab's
/// scene graph besides the spine itself: the background quads, the display
/// controller's camera framing, the texture `path_id`s the quads claim, and the
/// character spine's draw order.
#[derive(Default)]
struct BgScene {
    quads: Vec<BgQuad>,
    camera_size: Option<f64>,
    max_aspect: Option<f64>,
    max_size: Option<(f64, f64)>,
    camera_offset: Option<(f64, f64)>,
    camera_view: Option<f64>,
    camera_offset2: Option<(f64, f64)>,
    camera_view2: Option<f64>,
    claimed_tex: Vec<i64>,
    char_sort: Option<i64>,
    separator_slots: Vec<String>,
    separator_part_sorts: Vec<i64>,
}

/// Check if a bundle path is eligible for spine extraction.
#[must_use]
pub fn detect_spine_bundle(bundle_subdir: &Path, input_dir: &Path) -> bool {
    let full_path = input_dir.join(bundle_subdir);
    let path_str = full_path.to_string_lossy();
    let path_lower = path_str.to_lowercase();

    const SPINE_SEGMENTS: &[&str] = &[
        "chararts/",
        "skinpack/",
        "npcpack/",
        "building/vault/characters/",
        "arts/dynchars",
        "arts/dynavatars",
    ];
    SPINE_SEGMENTS.iter().any(|segment| {
        path_lower.contains(&format!("/{segment}")) || path_lower.starts_with(segment)
    })
}

/// Check if a bundle path is an enemy spine art bundle.
///
/// Enemy spine data (`SkeletonData` + .skel/.atlas `TextAssets` + textures) lives
/// in `refs/arts/enm_art_*.ab` pack bundles, ~60 enemies per bundle. The
/// `battle/enm_pfb_*.ab` prefab bundles only hold `GameObject` wiring with
/// external references into these art bundles, so they are not needed here.
#[must_use]
pub fn detect_enemy_spine_bundle(bundle_subdir: &Path, input_dir: &Path) -> bool {
    let full_path = input_dir.join(bundle_subdir);
    let path_lower = full_path.to_string_lossy().to_lowercase();

    path_lower.contains("/refs/arts/enm_art") || path_lower.starts_with("refs/arts/enm_art")
}

/// Extract a `path_id` from a JSON reference like {"`m_FileID"`: 0, "`m_PathID"`: 12345}
pub(crate) fn get_path_id(val: &Value) -> Option<i64> {
    val.get("m_PathID").and_then(serde_json::Value::as_i64)
}

/// Whether a `GameObject` is shown in the IDLE display — i.e. no Transform ancestor
/// is `m_IsActive=0` NOR a state-only effect group. Dynchar prefabs bucket effects
/// into groups ("Start Only Effects" / "Interact Only Effects" / "Special Only
/// Effects" / "General Effects") that the game shows only during that state; the
/// groups stay `m_IsActive=1` in the prefab (gated at runtime by the animator), so
/// we exclude by NAME too. Descendants of a non-idle group must not appear in the
/// idle scene/particles, else every state's effects render at once → noise.
/// Animation states whose `<State> Only Effects` groups the prefab ships switched OFF.
const STATE_ONLY: &[&str] = &[
    "start", "interact", "special", "skill", "attack", "die", "assist",
];

/// Is this `GameObject` inside a `<State> Only Effects` group for a state that is NOT the one
/// being exported? Such a group is switched on only while the game plays that state, so its
/// contents belong to neither the idle scene nor the `_Start` cinematic — and unlike an
/// ordinary inactive object, an animated colour curve must NOT be able to resurrect it
/// (Mlynar's `Special Only Effects` blade glow carries the same colliding alpha curve as his
/// entrance rigs, which admitted a special-attack effect into the entrance).
/// Is this `<State> Only Effects` state on the `DYNCHAR_STATE_ADMIT` allow-list?
///
/// Purely diagnostic: the gate exists because these groups belong to interaction states the
/// entrance never enters, and admitting one draws content the game does not. It is here so
/// Does this `GameObject` name a state clone the `_Start` CINEMATIC must not draw?
///
/// Effect rigs are instantiated once per state as `<skin>_<State>[_NN](Clone)`. Rather than match
/// the skin id (which we do not have here) this reads the name's UNDERSCORE SEGMENTS and asks
/// whether any of them names a non-cinematic state.
///
/// ⚠️ Segment equality, never `contains`. `_Start_Idle` carries BOTH "start" and "idle" and is an
/// IDLE clone — a substring test for "start" admits it, which is exactly the trap that hid the
/// reveal-timeline bug (clip `03`). Presence of a non-cinematic segment decides, and "start" never
/// rescues it. Arknights skin ids (`char_2024_chyue_cfa#1`) contain no such segment, so a name can
/// only match through its state tail.
pub(crate) fn non_cinematic_state_clone(name: &str) -> bool {
    // MEASURED, not assumed. Adding "interact" and "special" here drops 68 more systems across 7
    // entrances (wisdel alone loses 38) and is a net LOSS: mue 10.756 -> 10.857, corpus-8 mean
    // 11.616 -> 11.626. The game does draw some interact/special-clone effects during a cinematic,
    // and the `<State> Only Effects` group gate already withholds the ones that must not appear.
    // Only the IDLE clones are wrong here.
    const NON_CINEMATIC: [&str; 1] = ["idle"];
    let Some(base) = name.strip_suffix("(Clone)") else {
        return false;
    };
    base.trim_end()
        .split('_')
        .any(|seg| NON_CINEMATIC.contains(&seg.to_ascii_lowercase().as_str()))
}

/// "what is this gate withholding" is a measurement rather than an argument.
fn state_admitted(state: &str) -> bool {
    static ADMIT: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    ADMIT
        .get_or_init(|| {
            std::env::var("DYNCHAR_STATE_ADMIT")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .iter()
        .any(|a| a == state)
}

pub(crate) fn state_only_blocked(
    all_objects: &HashMap<i64, (i32, Value)>,
    go_pid: i64,
    go_to_transform: &HashMap<i64, i64>,
    start_state_active: bool,
) -> bool {
    let mut cur_tr = match go_to_transform.get(&go_pid) {
        Some(&t) => t,
        None => return false,
    };
    for _ in 0..256 {
        let tf = match all_objects.get(&cur_tr) {
            Some((4, v)) => v,
            _ => return false,
        };
        if let Some(go) = tf.get("m_GameObject").and_then(get_path_id)
            && let Some((1, gv)) = all_objects.get(&go)
        {
            let name = gv
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            if name.contains("only")
                && STATE_ONLY.iter().any(|s| {
                    !(start_state_active && *s == "start")
                        && name.contains(s)
                        // DIAGNOSTIC (`DYNCHAR_STATE_ADMIT=interact,special`, default empty):
                        // stop blocking the named `<State> Only Effects` groups, to measure
                        // what a state gate is actually withholding. Generic — takes state
                        // names, never skin names — so the same run answers the question for
                        // every skin in the corpus.
                        && !state_admitted(s)
                })
            {
                return true;
            }
        }
        match tf.get("m_Father").and_then(get_path_id) {
            Some(f) if f != 0 => cur_tr = f,
            _ => return false,
        }
    }
    false
}

pub(crate) fn go_effectively_active(
    all_objects: &HashMap<i64, (i32, Value)>,
    go_pid: i64,
    go_to_transform: &HashMap<i64, i64>,
    idle_active: &HashMap<i64, bool>,
    start_state_active: bool,
    honor_inactive: bool,
) -> bool {
    let mut cur_tr = match go_to_transform.get(&go_pid) {
        Some(&t) => t,
        None => return true,
    };
    for _ in 0..256 {
        let tf = match all_objects.get(&cur_tr) {
            Some((4, v)) => v,
            _ => return true,
        };
        if let Some(go) = tf.get("m_GameObject").and_then(get_path_id)
            && let Some((1, gv)) = all_objects.get(&go)
        {
            // The idle-loop animation's `m_IsActive` overrides the prefab default:
            // an object the loop switches off is hidden even if statically active,
            // and one it switches on is shown even if statically inactive.
            // NOTE: `m_IsActive` deserialises as a JSON **bool**, so `as_i64()` always yields
            // `None` and this static check is DEAD — every statically-disabled object reads as
            // active. That is a real reader bug, and it is also LOAD-BEARING: do not "fix" it.
            // Entrance rigs legitimately ship `m_IsActive = 0` on their `..._start(Clone)`
            // instances and are activated at runtime, so honouring the flag drops the whole
            // cinematic — measured on cello, the entrance export falls from 58 particle systems
            // to ZERO and 59 scene renderers vanish. Making it live needs a rule that separates
            // "disabled in the prefab, spawned by the game" from "disabled and left that way";
            // the flag alone cannot. See [[dynchar-virtuosa-pink-wings]].
            match idle_active.get(&go) {
                Some(false) => return false,
                Some(true) => {}
                None => {
                    if gv.get("m_IsActive").and_then(Value::as_i64).unwrap_or(1) == 0 {
                        return false;
                    }
                    // DEFAULT ON FOR SCENE QUADS ONLY (2026-09-01, was the
                    // DYNCHAR_HONOR_INACTIVE measurement arm): honour the serialized bool
                    // for NON-cinematic SCENE exports. The dead static check above stays
                    // dead for entrances (rigs ship m_IsActive=0, runtime-activated), and
                    // `honor_inactive` is FALSE on the particle path: the scene A/B that
                    // justified the flip (9 wins, 5 neutral, 0 deletes-art; texas2_2
                    // -70.19 -> -4.11) never covered particles, and the shipped rule
                    // silently dropped 10 of cetsyr epoque#50's 107 settled emitters and
                    // cost her +4.1 MADC (17.701 -> 21.838, re-run clean), on the one key
                    // most dependent on her particles. Particle gating would need its own
                    // A/B before ever turning on. DYNCHAR_KEEP_INACTIVE=1 reverts the
                    // scene rule to the pre-rule export exactly.
                    if honor_inactive
                        && !start_state_active
                        && std::env::var("DYNCHAR_KEEP_INACTIVE").is_err()
                        && gv.get("m_IsActive").and_then(Value::as_bool) == Some(false)
                    {
                        return false;
                    }
                }
            }
            let name = gv
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            // `<State> Only Effects` groups ship `m_IsActive = 0` and are switched on by the
            // game only while that state plays, so they must not leak into the idle scene.
            // The `_Start` cinematic IS the start state, though: gating "start" there drops the
            // very effects the entrance exists to show (Mlynar "Fields of Ruination" loses the
            // `Mlynar_EX2_L_Sword4` blade `glow_01` the game blooms as he raises and lowers the
            // sword). Keep every OTHER state gated — we never render interact/special/skill/…
            if name.contains("only")
                && STATE_ONLY.iter().any(|s| {
                    !(start_state_active && *s == "start")
                        && name.contains(s)
                        // See `state_admitted` — `DYNCHAR_STATE_ADMIT` is the measurement
                        // escape hatch for "what is this gate withholding".
                        && !state_admitted(s)
                })
            {
                return false;
            }
        }
        match tf.get("m_Father").and_then(get_path_id) {
            Some(f) if f != 0 => cur_tr = f,
            _ => return true,
        }
    }
    true
}

/// Does this `GameObject` sit under a `Start Only Effects` group?
///
/// The `<State> Only Effects` groups are authored on the IDLE prefab and switched on by the game
/// only while that state plays. A start-only system therefore falls through BOTH particle gates:
/// the idle export drops it as `inactive-group` (correct — it must not play in the idle loop),
/// and the `_Start` export drops it as `cross-root` (its prefab root is the idle one, not
/// `..._Start`), so it is exported NOWHERE despite being exactly what the entrance exists to show.
///
/// Skadi the Corrupting Heart's pure-red `xiaoyu` emitter (start colour 1.00,0.05,0.05 at full
/// alpha) is the case that surfaced this: the game paints it as a bright red beam at t=7 that we
/// did not draw at all, while every other red line in that frame already matched.
pub(crate) fn has_start_only_ancestor(
    all_objects: &HashMap<i64, (i32, Value)>,
    go_pid: i64,
    go_to_transform: &HashMap<i64, i64>,
) -> bool {
    let Some(&mut_tr) = go_to_transform.get(&go_pid) else {
        return false;
    };
    let mut cur_tr = mut_tr;
    for _ in 0..256 {
        let tf = match all_objects.get(&cur_tr) {
            Some((4, v)) => v,
            _ => return false,
        };
        if let Some(go) = tf.get("m_GameObject").and_then(get_path_id)
            && let Some((1, gv)) = all_objects.get(&go)
        {
            let name = gv
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            if name.contains("only") && name.contains("start") {
                return true;
            }
        }
        match tf.get("m_Father").and_then(get_path_id) {
            Some(f) if f != 0 => cur_tr = f,
            _ => return false,
        }
    }
    false
}

/// DIAGNOSTIC twin of [`go_effectively_active`]: when that returns `false`, report WHICH
/// ancestor blocked it and why.
///
/// The drop logs name the object and its bundle ROOT, which cannot distinguish a legitimate
/// state gate ("Start Only Effects", switched on by the game only during that state) from a
/// wrongly-blocked ordinary group — and those need opposite fixes. Returns
/// `(ancestor_name, reason)`; `None` when the object is active.
pub(crate) fn blocking_ancestor(
    all_objects: &HashMap<i64, (i32, Value)>,
    go_pid: i64,
    go_to_transform: &HashMap<i64, i64>,
    idle_active: &HashMap<i64, bool>,
    start_state_active: bool,
) -> Option<(String, &'static str)> {
    let mut cur_tr = *go_to_transform.get(&go_pid)?;
    for _ in 0..256 {
        let tf = match all_objects.get(&cur_tr) {
            Some((4, v)) => v,
            _ => return None,
        };
        if let Some(go) = tf.get("m_GameObject").and_then(get_path_id)
            && let Some((1, gv)) = all_objects.get(&go)
        {
            let name = gv
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let lower = name.to_ascii_lowercase();
            match idle_active.get(&go) {
                Some(false) => return Some((name, "idle-clip switches it OFF")),
                Some(true) => {}
                None => {
                    if gv.get("m_IsActive").and_then(Value::as_i64).unwrap_or(1) == 0 {
                        return Some((name, "prefab m_IsActive=0, no clip drives it"));
                    }
                }
            }
            if lower.contains("only")
                && STATE_ONLY
                    .iter()
                    .any(|s| !(start_state_active && *s == "start") && lower.contains(s))
            {
                return Some((name, "<State> Only Effects group"));
            }
        }
        match tf.get("m_Father").and_then(get_path_id) {
            Some(f) if f != 0 => cur_tr = f,
            _ => return None,
        }
    }
    None
}

/// Collect spine assets using `MonoBehaviour` reference graph traversal.
///
/// Reference chain:
/// ```text
/// SkeletonMecanim (has skeletonDataAsset + _animationName)
///   → SkeletonData (has skeletonJSON + atlasAssets)
///     → skel: TextAsset (.skel binary)
///     → Atlas MonoBehaviour (has atlasFile + materials)
///       → atlas: TextAsset (.atlas text)
///       → Material (has m_SavedProperties.m_TexEnvs)
///         → _MainTex → Texture2D
///         → _AlphaTex → Texture2D (optional)
/// ```
///
/// Returns (`spine_assets`, `claimed_path_ids`) where `claimed_path_ids` contains
/// `path_ids` of all objects consumed by spine extraction.
pub fn collect_spine_assets(
    all_objects: &HashMap<i64, (i32, Value)>,
    resources: &HashMap<String, Vec<u8>>,
) -> (Vec<SpineAsset>, HashSet<i64>) {
    // EXTERNAL VERTEX BUFFERS, plumbed but OPT-IN. `main.rs` builds `resources`
    // from the bundle's .resS / .resource entries, and `mesh::parse_mesh` needs
    // it: a Mesh whose inline buffer is empty carries an `m_StreamData` pointing
    // into one of those files, which `read_stream_data` resolves by filename.
    // Both dynchar mesh call sites used to pass an EMPTY map, so every such mesh
    // returned None and took its layer with it, silently until `47760c02`.
    //
    // MEASURED, and the default is OFF for it. Ch'en the Holungday is the only
    // skin in the corpus with such a mesh: her entrance's `bg_snow_lens_01`
    // (2410 verts) and `bg_01_lens_01` (432 verts) come back as full-frame
    // ADDITIVE layers at sort -1 and -18, and she goes 16.652 -> 21.583
    // (r .904 -> .853). Per beat, 2/5/8/11/14 all degrade 4.6 to 10.7 and it is
    // Y, not chroma; only t=18.5 improves (9.644 -> 8.646, r .825 -> .854) and
    // t=17 is flat. A layer that helps ONLY at the end while lifting luma
    // everywhere before it is drawn too early, and both export with no reveal
    // (`reveal=-`) so they run the whole cinematic. The geometry is right and
    // the SEQUENCING is missing, which is the same shape as Whislash-alter's
    // `mask_09`. Restore the map with `DYNCHAR_MESHRES=1` when chasing that;
    // the plumbing stays so the next person does not re-derive it.
    let empty_resources: HashMap<String, Vec<u8>> = HashMap::new();
    let resources = if std::env::var("DYNCHAR_MESHRES").as_deref() == Ok("1") {
        resources
    } else {
        &empty_resources
    };
    let mut claimed = HashSet::new();
    let mut assets = Vec::new();

    // Find SkeletonMecanim MonoBehaviours (class_id=114 with skeletonDataAsset field)
    let mut skeleton_mecanims: Vec<(i64, &Value)> = all_objects
        .iter()
        .filter(|(_, (class_id, val))| *class_id == 114 && val.get("skeletonDataAsset").is_some())
        .map(|(pid, (_, val))| (*pid, val))
        .collect();
    // `all_objects` is a HashMap, so its order changes from one process to the next. Sort
    // by path id so two exports of one bundle write the same thing in the same order. This
    // buys reproducibility only: two skeletons that resolve to one output path are still a
    // defect, and one sorted order would just make the same wrong one win every time, which
    // is why `export_spine_assets` also refuses the second write.
    skeleton_mecanims.sort_by_key(|(pid, _)| *pid);

    for (mecanim_pid, mecanim_val) in &skeleton_mecanims {
        // Get _animationName for classification
        let anim_name = mecanim_val
            .get("_animationName")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Get the owning GameObject's name ("Front"/"Back" for battle spines).
        // Front and back skeletons can share one atlas, so the GameObject name
        // is the only reliable front/back discriminator.
        let game_object_name = mecanim_val
            .get("m_GameObject")
            .and_then(get_path_id)
            .and_then(|pid| all_objects.get(&pid))
            .and_then(|(class_id, go_val)| (*class_id == 1).then_some(go_val))
            .and_then(|go_val| go_val.get("m_Name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Follow skeletonDataAsset → SkeletonData MonoBehaviour
        let skel_data_pid = match mecanim_val.get("skeletonDataAsset").and_then(get_path_id) {
            Some(pid) if pid != 0 => pid,
            _ => continue,
        };

        let Some(chain) = follow_skeleton_data(all_objects, skel_data_pid) else {
            continue;
        };

        // Classify the spine asset
        let base_name = chain
            .skel_name
            .strip_suffix(".skel")
            .unwrap_or(&chain.skel_name);
        let category = classify_spine(base_name, anim_name, game_object_name, &chain.atlas_text);

        // Claim all path_ids in this spine instance
        claimed.insert(*mecanim_pid);
        claimed.extend(&chain.claimed);

        // Dynamic illustrations carry a separate painted background scene that
        // is not part of the spine skeleton. Collect its quads so they can be
        // composited into a flat image behind the character.
        // Only the full-body illustration owns the background scene; the
        // co-packed portrait crop (`dyn_portrait_*`) shares the prefab but has
        // no background of its own, so it must not inherit the illust's.
        // Whether this skeleton is the ENTRANCE one has always been read off the FILENAME
        // (`…_Start`). That is a naming convention, not a guarantee, and exactly one skin
        // breaks it: Kalt'sits "boc#6" ships TWO DISTINCT SkeletonData objects (different
        // atlases, different skeletonJSON) BOTH named `dyn_illust_char_003_kalts_boc#6`. So
        // neither pass is recognised as the entrance — her 44 `Start Only Effects` are gated
        // out as if she were idle-only — and both passes write the SAME filenames, the second
        // silently overwriting the first. No `_Start` asset set exists, and the viewer only
        // plays an entrance when it finds one, so her cinematic cannot play at all.
        //
        // Derive it from the prefab ROOT instead, which is the structural fact rather than a
        // convention, and keep the filename test as the fallback. Corpus-wide this changes
        // exactly the one skin: every other entrance skeleton is already `_Start`-named, so
        // `root_is_entrance` and the filename test agree and nothing moves.
        let is_dyn_illust = category == SpineCategory::DynIllust
            && base_name.to_lowercase().starts_with("dyn_illust_");
        let root_is_entrance = is_dyn_illust && {
            let h = BgParticleHost::new(all_objects);
            mecanim_val
                .get("m_GameObject")
                .and_then(get_path_id)
                .and_then(|go| h.prefab_root_of_go(all_objects, go))
                .is_some_and(|r| {
                    h.go_name(all_objects, r)
                        .to_lowercase()
                        .starts_with("dyn_entrance_")
                })
        };
        // The name the asset set is written under. An entrance-root skeleton that is not
        // already `_Start`-named must not collide with its idle twin.
        let export_name: String =
            if root_is_entrance && !base_name.to_lowercase().contains("_start") {
                format!("{base_name}_Start")
            } else {
                base_name.to_string()
            };
        let is_entrance_set = export_name.to_lowercase().contains("_start");

        let (scene, skel_scale, particles) = if is_dyn_illust {
            let skel_scale = all_objects
                .get(&skel_data_pid)
                .and_then(|(_, v)| v.get("scale"))
                .and_then(serde_json::Value::as_f64);
            let spine_tex_pids: HashSet<i64> = chain.claimed.iter().copied().collect();
            // The per-layer ENTRANCE reveal timeline (`m_IsActive`) applies ONLY to the
            // `_Start` cinematic scene; the MAIN scene's idle/interact/special clips also
            // toggle effects, but those layers are idle-visible, not entrance-sequenced.
            let is_entrance = is_entrance_set;
            // Particle systems share the prefab's scene graph. Parse them into the
            // reduced `[particles]` schema; the character sort is the same key the
            // frontend uses to composite particles among the scene layers.
            let host = BgParticleHost::new(all_objects);
            let inv_scale = 1.0 / skel_scale.unwrap_or(0.01);
            // Prefab-instance roots (see the particle RootScope below) — also used to
            // sequence the ENTRANCE scene's cross-root layers at its authored reveal beat.
            let own_root = mecanim_val
                .get("m_GameObject")
                .and_then(get_path_id)
                .and_then(|go| host.prefab_root_of_go(all_objects, go));
            let skeleton_roots: HashSet<i64> = skeleton_mecanims
                .iter()
                .filter_map(|(_, v)| v.get("m_GameObject").and_then(get_path_id))
                .filter_map(|go| host.prefab_root_of_go(all_objects, go))
                .collect();
            // ENTRANCE scenes bundle BOTH prefab roots' layers (the union used to ship
            // unsequenced). The game only activates the idle prefab at the earliest
            // sufficiently-late authored beat — transform completion or voice-line start.
            // This replaces always using transform completion, which was ~1.7s too late
            // for skins with an earlier voice-line beat. Skins with neither late beat
            // (Mlynar) keep the union always-on.
            // A cross-root reveal beat that lands AFTER the cinematic ends can never fire, so the
            // layers it gates would simply never draw. Two skins author exactly that — Civilight
            // Eterna's transform is 20.4 against a 19.0 s entrance (23 of her 28 scene layers) and
            // Eyjafjalla's is 14.5 against 9.77 (3 of 6). For Civilight Eterna the consequence is
            // stark: her own root's last backdrop ends at 12.0, so from there to the end NOTHING
            // draws and the frame is an empty void where the game shows a full landscape. Emitting
            // a gate we know cannot fire is worse than emitting none, so drop it.
            let cross_root_reveal = if is_entrance {
                let t = find_entrance_timing(all_objects);
                t.4.filter(|r| t.0.is_none_or(|dur| *r <= dur))
            } else {
                None
            };
            let scene = collect_dynchar_bg_quads(
                all_objects,
                &spine_tex_pids,
                is_entrance,
                cross_root_reveal,
                own_root,
                &skeleton_roots,
                &host,
                inv_scale,
                resources,
            );
            claimed.extend(scene.claimed_tex.iter().copied());
            // Entrance `m_IsActive` reveal times gate the `_Start` cinematic's particle
            // emitters (apple/glow/wing sparks toggle on mid-cinematic); empty for the
            // main/idle prefab so its particles keep their `_delayTime`-only delays.
            let particle_windows = if is_entrance {
                super::anim::active_windows(all_objects)
            } else {
                HashMap::new()
            };
            // Entrance-clip-animated emission rates (`EmissionModule.rateOverTime`
            // bindings) gate confetti/star emitters whose SERIALIZED rate is a large
            // constant (Mlynar's sword-flourish stars). Entrance-only: the idle
            // prefab's copies are driven by their own state clips, not the cinematic.
            let particle_rate_curves = if is_entrance {
                super::anim::entrance_ps_rate_curves(all_objects)
            } else {
                HashMap::new()
            };
            // MAIN scene only: emitters whose rate is driven by transition/one-shot
            // state clips are quiet at the steady idle (see `event_driven_rate_gos`);
            // their serialized constant is a flourish peak, not an ambient rate. The
            // entrance path instead replays the actual clip curves (`rateCurve`).
            let event_rate_gos = if is_entrance {
                HashSet::new()
            } else {
                super::anim::event_driven_rate_gos(all_objects)
            };
            // Entrance-clip Transform SCALE/position curves on effect hosts. A host the
            // `_Start` clip scales (Virtuosa's crown `ctrl`, 1.0→0.28) is ADMITTED into the
            // entrance export even when it lives under the OTHER (idle) prefab root — the
            // cinematic reaches across roots to drive it — and its baked resting pose is
            // animated by the exported `scaleCurve`/`posCurve`. Entrance-only: the idle
            // scene never plays these clips, so its export stays scoped as before.
            let entrance_transform_curves = if is_entrance {
                super::anim::entrance_transform_curves(all_objects)
            } else {
                HashMap::new()
            };
            // Entrance-clip material-COLOUR curves for particle materials (the scene-quad
            // path's twin, see `EntranceCtx::color_channels`). Entrance-only: the idle
            // prefab's copies hold their serialized colour.
            let particle_color_channels = if is_entrance {
                super::anim::entrance_material_color_channels(all_objects)
            } else {
                HashMap::new()
            };
            // Scope particle membership to THIS skeleton's prefab-instance root. A
            // dynchar bundle ships sibling roots (`dyn_illust_*` idle + `dyn_entrance_*`
            // cinematic), and every ParticleSystem of BOTH used to land in BOTH
            // exports — the idle scene then ran the entrance's sword-confetti/star
            // relays at their big SERIALIZED rates forever (the cinematic clips that
            // gate them don't play in the idle). A system under ANOTHER skeleton's
            // root is dropped; systems in shared/non-skeleton roots are kept.
            let (particles, skipped) = super::particles::collect_dynchar_particles(
                all_objects,
                inv_scale,
                resources,
                &host,
                &super::particles::EntranceCtx {
                    windows: &particle_windows,
                    rate_curves: &particle_rate_curves,
                    event_rate_gos: &event_rate_gos,
                    transform_curves: &entrance_transform_curves,
                    color_channels: &particle_color_channels,
                    is_entrance,
                },
                &super::particles::RootScope {
                    own: own_root,
                    skeleton_roots: &skeleton_roots,
                },
            );
            if !particles.is_empty() || skipped.total() > 0 {
                eprintln!(
                    "  particles: {} exported, {} skipped ({base_name}) [{}]",
                    particles.len(),
                    skipped.total(),
                    skipped
                );
            }
            (scene, skel_scale, particles)
        } else {
            (BgScene::default(), None, Vec::new())
        };

        // ENTRANCE (`_Start`) director timing + camera — only the `_Start` prefab has them.
        let (
            bg_entrance_duration,
            bg_entrance_fade,
            bg_entrance_clear,
            bg_entrance_transform,
            bg_entrance_view,
            bg_entrance_cam_offset,
            bg_entrance_ortho_curve,
            bg_entrance_voice,
            bg_entrance_pan_curve,
            bg_entrance_cam_center,
            bg_entrance_cam_roll,
            bg_entrance_aperture,
            bg_entrance_clip_stop,
            bg_entrance_post_fx,
        ) = if category == SpineCategory::DynIllust && is_entrance_set {
            let (dur, tr, ortho, voice, _) = find_entrance_timing(all_objects);
            let fade = find_entrance_fade(all_objects);
            let clear = find_entrance_clear_color(all_objects);
            // Entrance camera ortho size (world units) → authored-px full view (2·ortho·invScale),
            // the tight close-up the cinematic opens on before dollying out to the display frame.
            let inv = 1.0 / skel_scale.unwrap_or(0.01);
            let cam_off = find_entrance_camera_offset(all_objects).map(|(x, y)| (x * inv, y * inv));
            // For a PERSPECTIVE camera the serialized `orthographic size` is inert — Unity keeps
            // the field but never uses it — so `2·ortho·inv` describes a framing the game never
            // shows. The real opening extent is the frustum height at the subject plane,
            // `2·d(0)·tan(fov/2)`, with `d(0)` the first keyframe of the dolly (or the camera's
            // static Z when it does not move).
            let persp_view = super::anim::entrance_dolly_curve(all_objects).and_then(|c| {
                let d0 = c.first()?.1;
                let fov = entrance_camera_pid(all_objects)
                    .and_then(|p| all_objects.get(&p))
                    .and_then(|(_, v)| v.get("field of view"))
                    .and_then(Value::as_f64)? as f32;
                (d0 > 0.0 && fov > 0.0)
                    .then(|| f64::from(2.0 * d0 * (fov.to_radians() / 2.0).tan()) * inv)
            });
            // The DATA-DRIVEN camera dolly: the animated orthographic-size curve (see
            // `entrance_ortho_curve`). Replaces client-side guesswork about the zoom timing.
            // PERSPECTIVE rigs dolly on the camera's Z instead of animating an ortho size. The
            // dolly curve is `(t, distance)`, and the client's zoom consumer only ever uses the
            // RATIO to the first keyframe — and `d(t)/d(0)` is exactly the perspective scale
            // ratio — so it drops straight into the same field with no client change and no new
            // units. `entrance_dolly_curve` returns None for an orthographic camera, so every
            // existing skin keeps the ortho path by construction. See its doc comment for the
            // measured validation (predicted 0.3637, measured optimum 0.36).
            let post_fx = super::anim::entrance_post_fx(all_objects);
            let ortho_curve = super::anim::entrance_ortho_curve(all_objects)
                .or_else(|| super::anim::entrance_dolly_curve(all_objects));
            let pan_curve = super::anim::entrance_pan_curve(all_objects);
            super::anim::report_unresolved_bindings(all_objects);
            let (cam_center, cam_roll, aperture) =
                super::anim::entrance_camera_track(all_objects, inv, ortho);
            (
                dur,
                fade,
                clear,
                tr,
                persp_view.or_else(|| ortho.map(|o| 2.0 * o * inv)),
                cam_off,
                ortho_curve,
                voice,
                pan_curve,
                cam_center,
                cam_roll,
                aperture,
                super::anim::entrance_clip_stop(all_objects),
                post_fx,
            )
        } else {
            (
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            )
        };

        assets.push(SpineAsset {
            name: export_name,
            skel_data: chain.skel_bytes,
            atlas_text: chain.atlas_text,
            textures: chain.textures,
            category,
            bg_quads: scene.quads,
            bg_skel_scale: skel_scale,
            settle_animation: (!anim_name.is_empty() && anim_name != "Start")
                .then(|| anim_name.to_string()),
            bg_camera_size: scene.camera_size,
            bg_max_aspect: scene.max_aspect,
            bg_max_size: scene.max_size,
            bg_camera_offset: scene.camera_offset,
            bg_camera_view: scene.camera_view,
            bg_camera_offset2: scene.camera_offset2,
            bg_camera_view2: scene.camera_view2,
            bg_character_sort: scene.char_sort,
            separator_slots: scene.separator_slots.clone(),
            separator_part_sorts: scene.separator_part_sorts.clone(),
            bg_entrance_duration,
            bg_entrance_clip_stop,
            bg_entrance_fade,
            bg_entrance_clear,
            bg_entrance_transform,
            bg_entrance_view,
            // `entranceOrthoCurve` falls back to the DOLLY curve for a perspective rig, and a
            // dolly keyframe is a DISTANCE — so the client must not read `2*curve[0]/skelScale`
            // as a view extent there. Recomputed rather than threaded through the tuple above;
            // both calls are pure lookups over `all_objects`.
            bg_entrance_persp: super::anim::entrance_ortho_curve(all_objects).is_none()
                && super::anim::entrance_dolly_curve(all_objects).is_some(),
            bg_entrance_cam_offset,
            bg_entrance_ortho_curve,
            bg_entrance_pan_curve,
            bg_entrance_cam_center,
            bg_entrance_cam_roll,
            bg_entrance_aperture,
            bg_entrance_voice,
            bg_entrance_post_fx,
            particles,
        });
    }

    // Some dynchar bundles ship the same illustration skeleton at two
    // resolutions under one name (e.g. a 1024² low-res variant beside the 2360²
    // master). They collide on output path — last-write-wins would keep whichever
    // was processed last. Keep only the highest-resolution atlas: the low-res
    // variant's atlas omits the large body/background regions, so its meshes
    // sample garbage (SilverAsh "Never-melting Ice" rendered as a stretched blob).
    dedup_keep_highest_res(&mut assets);

    (assets, claimed)
}

/// Page dimensions declared by an atlas text, keyed by page base name (no `.png`).
///
/// libgdx atlas format: a page is a bare filename line followed by `size: W,H`. The frontend
/// scales every region's UVs by these numbers, so a written PNG whose dimensions disagree
/// silently misplaces every attachment.
// `t` here is a line of libgdx `.atlas` text (always emitted lowercase by the Spine exporter),
// not a filesystem path, so this isn't the case-insensitive-extension situation the lint targets.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn atlas_page_sizes(atlas_text: &str) -> HashMap<String, (u32, u32)> {
    let mut out = HashMap::new();
    let mut page: Option<String> = None;
    for line in atlas_text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("size:") {
            if let Some(name) = page.take()
                && let Some((w, h)) = rest.split_once(',')
                && let (Ok(w), Ok(h)) = (w.trim().parse::<u32>(), h.trim().parse::<u32>())
            {
                out.insert(name, (w, h));
            }
        } else if t.ends_with(".png") {
            page = Some(t.trim_end_matches(".png").to_string());
        } else if t.is_empty() {
            page = None;
        }
    }
    out
}

/// Largest page dimension declared in an atlas text (`size: W,H` lines).
fn atlas_max_dim(atlas_text: &str) -> u64 {
    atlas_text
        .lines()
        .filter_map(|l| {
            let (w, h) = l.trim().strip_prefix("size:")?.split_once(',')?;
            Some(
                w.trim()
                    .parse::<u64>()
                    .ok()?
                    .max(h.trim().parse::<u64>().ok()?),
            )
        })
        .max()
        .unwrap_or(0)
}

/// Drop duplicate spine assets that share a `(category, name)` — hence an output
/// path — keeping the one whose atlas has the largest page (highest resolution
/// and, for co-packed resolution variants, the complete region set).
fn dedup_keep_highest_res(assets: &mut Vec<SpineAsset>) {
    let mut best: HashMap<(String, String), usize> = HashMap::new();
    for (i, a) in assets.iter().enumerate() {
        let key = (a.category.to_string(), a.name.clone());
        match best.get(&key) {
            Some(&j) if atlas_max_dim(&assets[j].atlas_text) >= atlas_max_dim(&a.atlas_text) => {}
            _ => {
                best.insert(key, i);
            }
        }
    }
    let keep: HashSet<usize> = best.into_values().collect();
    let mut i = 0;
    assets.retain(|_| {
        let k = keep.contains(&i);
        i += 1;
        k
    });
}

/// Check whether a bundle path is a dynamic-character bundle (`arts/dynchars`).
/// These carry a separate painted background layer alongside the spine.
#[must_use]
pub fn detect_dynchar_bundle(bundle_subdir: &Path, input_dir: &Path) -> bool {
    let path_lower = input_dir
        .join(bundle_subdir)
        .to_string_lossy()
        .to_lowercase();
    path_lower.contains("arts/dynchars")
}

/// ENTRANCE visibility window for a `GameObject`: the window of the nearest ancestor
/// (self first) present in `reveal` — walks `m_Father` up the transform chain.
/// `(None, None)` when neither the object nor any ancestor is toggled (→ always visible).
fn reveal_of_go(
    go_pid: i64,
    reveal: &HashMap<i64, super::anim::ActiveWindowList>,
    go_to_transform: &HashMap<i64, i64>,
    all_objects: &HashMap<i64, (i32, Value)>,
) -> super::anim::ActiveWindowList {
    let mut cur_go = Some(go_pid);
    for _ in 0..256 {
        let Some(g) = cur_go else { break };
        if let Some(w) = reveal.get(&g) {
            return w.clone();
        }
        let Some(tf) = go_to_transform.get(&g) else {
            break;
        };
        let father = all_objects
            .get(tf)
            .and_then(|(_, v)| v.get("m_Father"))
            .and_then(get_path_id)
            .filter(|&p| p != 0);
        let Some(father) = father else { break };
        cur_go = all_objects
            .get(&father)
            .and_then(|(_, v)| v.get("m_GameObject"))
            .and_then(get_path_id);
    }
    Vec::new()
}

/// Collect every background-scene mesh quad of a dynillust prefab.
///
/// Approach (graph-driven, not name-based): walk every `MeshRenderer` (class
/// 23) that does not belong to the character spine, resolve its `MeshFilter`
/// mesh, material `_MainTex`/`_AlphaTex`/tint/blend, and full world transform
/// (relative to the spine root). ALL non-character quads are returned (foreground
/// effects included) — the live scene renderer inserts the animated character
/// among them at `char_sort`.
#[allow(clippy::too_many_arguments)]
fn collect_dynchar_bg_quads(
    all_objects: &HashMap<i64, (i32, Value)>,
    spine_tex_pids: &HashSet<i64>,
    is_entrance: bool,
    cross_root_reveal: Option<f64>,
    own_root: Option<i64>,
    skeleton_roots: &HashSet<i64>,
    host: &BgParticleHost,
    inv_scale: f64,
    resources: &HashMap<String, Vec<u8>>,
) -> BgScene {
    // GameObject path_id → Transform (class 4), and → MeshFilter mesh pid.
    let mut go_to_transform: HashMap<i64, i64> = HashMap::new();
    let mut go_to_mesh: HashMap<i64, i64> = HashMap::new();
    for (pid, (cid, v)) in all_objects {
        match cid {
            4 => {
                if let Some(go) = v.get("m_GameObject").and_then(get_path_id) {
                    go_to_transform.insert(go, *pid);
                }
            }
            33 => {
                if let (Some(go), Some(mesh)) = (
                    v.get("m_GameObject").and_then(get_path_id),
                    v.get("m_Mesh").and_then(get_path_id),
                ) {
                    go_to_mesh.insert(go, mesh);
                }
            }
            _ => {}
        }
    }

    // ENTRANCE reveal timeline: GameObject → time (s) it switches ON (`m_IsActive`).
    // A scene layer inherits the reveal of its nearest such ancestor. Only for the
    // `_Start` cinematic — the main scene's clip toggles are idle/interact state, not
    // an entrance sequence, so its layers stay always-visible.
    // ⚠️ Do NOT lift this `is_entrance` gate to "fix" a missing idle animation. Tried and
    // REFUTED (2026-08-06): the clean skin-preview capture shows Mlynar's IDLE stepping from mean
    // luma ~119 to ~96 at t≈6.5 s (his seated background figures go from lit to dark silhouettes),
    // and the exported idle scene carries 0 windows / 0 colour curves, which looks exactly like an
    // exporter gap. It is not. Running `active_windows` on the idle prefab yields the `_Start`
    // CLIP's curves — the dumped windows are [3.47, 11.67) and [11.53, 12.30) with curves running
    // to t=14.33, i.e. his entrance length, not anything the idle could replay. It also silently
    // changed his layer count 15 → 13. The idle's timed behaviour, whatever drives it, is not in
    // this map.
    let reveal_map = if is_entrance {
        super::anim::active_windows(all_objects)
    } else {
        HashMap::new()
    };
    // ENTRANCE per-layer TRANSFORM animation: GO → scale/position curves from the `_Start`
    // clip(s). Already built for particle hosts (`particles.rs` uses it to drive a rig's
    // scale-in); scene quads never asked for it, so their geometry stayed frozen at the prefab
    // pose. Executor's scope rim is the measurable case — the game animates its scale 0.91..1.66
    // and the aperture follows, which our static quad cannot express.
    let xform_map = if is_entrance {
        super::anim::entrance_transform_curves(all_objects)
    } else {
        HashMap::new()
    };
    // ENTRANCE per-layer material-colour animation: GO → animated colour channels from
    // the `_Start` clip(s) (Mlynar's white flash ramps `_TintColor.a` 0→0.671 over
    // 13→15s — static tint alone would hold it as an opaque white-wash).
    let color_channels = if is_entrance {
        super::anim::entrance_material_color_channels(all_objects)
    } else {
        HashMap::new()
    };
    // ENTRANCE per-layer animated `_MainTex_ST` curves (Capability B): GO → the four ST
    // component curves from the `_Start` clip(s) (Skadi2's entrance seam sweep). Empty for
    // the main scene (its clips don't animate ST).
    let st_channels = if is_entrance {
        super::anim::entrance_st_curves(all_objects)
    } else {
        HashMap::new()
    };
    // GameObjects that own a spine skeleton (SkeletonMecanim/Animation) — the
    // character itself, excluded from the background.
    let spine_gos: HashSet<i64> = all_objects
        .values()
        .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
        .filter_map(|(_, v)| v.get("m_GameObject").and_then(get_path_id))
        .collect();

    // Character draw order = sorting order of a MeshRenderer on a spine GO.
    // Deterministic pick: own prefab root's spine renderer first, then path_id.
    let mut spine_renderers: Vec<(i64, i64, i64)> = all_objects
        .iter()
        .filter_map(|(pid, (cid, v))| {
            if *cid != 23 {
                return None;
            }
            let go = v.get("m_GameObject").and_then(get_path_id)?;
            if !spine_gos.contains(&go) {
                return None;
            }
            let sort = v
                .get("m_SortingOrder")
                .and_then(serde_json::Value::as_i64)?;
            Some((*pid, go, sort))
        })
        .collect();
    spine_renderers.sort_unstable_by_key(|(pid, ..)| *pid);
    if let Some(own) = own_root {
        spine_renderers
            .sort_by_key(|(_, go, _)| host.prefab_root_of_go(all_objects, *go) != Some(own));
    }
    let spine_sort = spine_renderers.first().map_or(0, |(.., sort)| *sort);

    // Spine-Unity's OWN background/character boundary — see `SpineAsset::separator_slots`.
    // Read from the SkeletonRenderer MonoBehaviour on the SAME GameObject whose MeshRenderer
    // supplied `spine_sort`, so a bundle shipping several skeletons (Virtuosa ships three, two
    // with a split and one with the feature disabled) cannot cross-wire one skeleton's split
    // slot onto another's draw order.
    let separator_slots: Vec<String> = spine_renderers
        .first()
        .and_then(|(_, go, _)| {
            all_objects.values().find_map(|(cid, v)| {
                if *cid != 114
                    || v.get("skeletonDataAsset").is_none()
                    || v.get("m_GameObject").and_then(get_path_id) != Some(*go)
                {
                    return None;
                }
                v.get("separatorSlotNames")
                    .and_then(serde_json::Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(str::to_owned))
                            .filter(|x| !x.is_empty())
                            .collect::<Vec<String>>()
                    })
            })
        })
        .unwrap_or_default();

    // The separator PARTS' own depths, taken from the `SkeletonRenderSeparator` on the SAME
    // GameObject as the skeleton — the separator component and the SkeletonRenderer are
    // siblings (verified on Skadi: go -846..302 carries `Skadi_glow` + parts [0,5], go
    // 506..246 carries `Skadi_crown_back` + parts [0,8]).
    //
    // Scanning `partsRenderers` bundle-wide instead unions the parts of EVERY skeleton in the
    // bundle, which breaks the `parts.len() == slots.len() + 1` invariant the consumer needs to
    // map a gap to a split slot — Skadi came out [0, 5, 8] for two 2-part skeletons.
    let separator_part_sorts: Vec<i64> = spine_renderers
        .first()
        .map(|(_, skel_go, _)| {
            let mut sorts: Vec<i64> = Vec::new();
            for (_, v) in all_objects.values() {
                if v.get("m_GameObject").and_then(get_path_id) != Some(*skel_go) {
                    continue;
                }
                let Some(parts) = v
                    .get("partsRenderers")
                    .and_then(serde_json::Value::as_array)
                else {
                    continue;
                };
                for e in parts {
                    let Some(rp) = e.get("m_PathID").and_then(serde_json::Value::as_i64) else {
                        continue;
                    };
                    let Some((_, rv)) = all_objects.get(&rp) else {
                        continue;
                    };
                    let Some(go) = rv.get("m_GameObject").and_then(get_path_id) else {
                        continue;
                    };
                    for (cid2, v2) in all_objects.values() {
                        if *cid2 == 23
                            && v2.get("m_GameObject").and_then(get_path_id) == Some(go)
                            && let Some(o) =
                                v2.get("m_SortingOrder").and_then(serde_json::Value::as_i64)
                        {
                            sorts.push(o);
                        }
                    }
                }
            }
            // Preserve `partsRenderers` ORDER (part 0 first, which is also ascending depth) and
            // do NOT dedup: with N separator slots there are exactly N+1 parts, gap i spans
            // [parts[i], parts[i+1]), and sorting/deduping would silently break that mapping if
            // two parts ever shared a sortingOrder.
            sorts
        })
        .unwrap_or_default();

    // Display-controller MonoBehaviours in a DETERMINISTIC order: the OWN prefab
    // root's controller first, then ascending path_id. A bundle can ship several
    // controllers (sibling prefab roots), and picking via HashMap iteration made
    // the camera fields flip between runs (amiya2's cameraSize 10.0 vs 10.5).
    let mut controller_mbs: Vec<(i64, &Value)> = all_objects
        .iter()
        .filter(|(_, (cid, _))| *cid == 114)
        .map(|(pid, (_, v))| (*pid, v))
        .collect();
    controller_mbs.sort_unstable_by_key(|(pid, _)| *pid);
    if let Some(own) = own_root {
        controller_mbs.sort_by_key(|(_, v)| {
            v.get("m_GameObject")
                .and_then(get_path_id)
                .and_then(|go| host.prefab_root_of_go(all_objects, go))
                != Some(own)
        });
    }
    let camera_size = controller_mbs
        .iter()
        .find_map(|(_, v)| v.get("_cameraSize").and_then(serde_json::Value::as_f64));
    // Render-target aspect from the display controller's _maxSize.
    let max_aspect = controller_mbs.iter().find_map(|(_, v)| {
        let ms = v.get("_maxSize")?;
        let x = ms.get("x").and_then(serde_json::Value::as_f64)?;
        let y = ms.get("y").and_then(serde_json::Value::as_f64)?;
        (y > 0.0).then_some(x / y)
    });
    // The same `_maxSize`, absolute, for the viewer's render-target size.
    let max_size = controller_mbs.iter().find_map(|(_, v)| {
        let ms = v.get("_maxSize")?;
        let x = ms.get("x").and_then(serde_json::Value::as_f64)?;
        let y = ms.get("y").and_then(serde_json::Value::as_f64)?;
        (x > 0.0 && y > 0.0).then_some((x, y))
    });
    // Authored full-illustration display frame from the display controller's
    // `_adjustes[0]` (offset = frame CENTRE, size.x = square full EXTENT, both in
    // spine-authored px). The camera is not centred on the skeleton root, so this
    // is how the game frames the scene.
    // Parse ONE `_adjustes` entry → (offset, size). Used for both the wide adjust[0]
    // (idle framing) and the tight adjust[1] (the zoomed-in open the viewer dollies out from).
    let parse_adjust = |a: &Value| -> (Option<(f64, f64)>, Option<f64>) {
        // An UNSET Unity `Vector2` in these controllers holds `float.MinValue`
        // (-3.4028235e38), not zero, and it must be read as ABSENT. Nearl "Evolved Art"
        // ships exactly that for both of its adjust offsets: passed through as a real frame
        // centre it laid the whole scene ~1500 px above the viewport and the skin rendered as
        // bare environment fill with one corner of cloud — the character never appeared.
        // Reject the sentinel by magnitude against the f32 extremum rather than by a chosen
        // limit: every genuine offset is a few hundred authored px, and nothing legitimate
        // comes within astronomical distance of FLT_MAX.
        let sane = |v: f64| v.is_finite() && v.abs() < f64::from(f32::MAX) * 0.5;
        let off = a.get("offset");
        let ox = off
            .and_then(|o| o.get("x"))
            .and_then(serde_json::Value::as_f64)
            .filter(|v| sane(*v));
        let oy = off
            .and_then(|o| o.get("y"))
            .and_then(serde_json::Value::as_f64)
            .filter(|v| sane(*v));
        let size = a
            .get("size")
            .and_then(|s| s.get("x"))
            .and_then(serde_json::Value::as_f64)
            .filter(|v| sane(*v));
        (ox.zip(oy), size)
    };
    let (camera_offset, camera_view, camera_offset2, camera_view2) = controller_mbs
        .iter()
        .find_map(|(_, v)| {
            let adjustes = v.get("_adjustes")?.as_array()?;
            let (o0, s0) = parse_adjust(adjustes.first()?);
            // The tight endpoint is optional (some controllers ship a single adjust).
            let (o1, s1) = adjustes.get(1).map_or((None, None), parse_adjust);
            Some((o0, s0, o1, s1))
        })
        .unwrap_or((None, None, None, None));

    // Evaluate the scene's idle clip so animated quads (e.g. Nearl "Evolved
    // Art"'s sword/petal shards) export at their settled pose, not the scattered
    // bind pose. Empty when the scene doesn't animate transforms.
    let idle_pose = super::anim::evaluate_idle_pose(all_objects, &go_to_transform);

    let mut quads: Vec<BgQuad> = Vec::new();
    let mut claimed_tex: Vec<i64> = Vec::new();

    // Deterministic order.
    let mut renderers: Vec<(i64, &Value)> = all_objects
        .iter()
        .filter(|(_, (cid, _))| *cid == 23)
        .map(|(pid, (_, v))| (*pid, v))
        .collect();
    renderers.sort_unstable_by_key(|(pid, _)| *pid);

    // GameObjects that RIDE the entrance camera (descendants of its transform). Such a quad is
    // an overlay locked to the viewport — a film-strip border, a lens sheet, a full-frame haze —
    // and its baked world pose is meaningless once the shot dollies. Computed once here; the
    // frontend re-places these against the live frame each tick (`camLocked`).
    // Per camera-locked GO: the frustum EXTENT (authored px) at ITS OWN distance from the camera.
    // A camera child is not framed by the camera's focal-plane extent — under a PERSPECTIVE rig its
    // apparent size is set by the frustum where IT sits. whitw2's film strip lives at z 4.36 while
    // her `entranceViewPx` is the frustum at the dolly's d0 = 3.0, a ratio of exactly
    // 2*4.36*tan(30°) / 2*3.0*tan(30°) = 1.4535 — which is precisely how much too large and too
    // high her strip rendered. Sizing against this instead lands the sprocket bars at screen
    // y -0.2..36.2 and 379.8..416.2, against 0..35 and 380..415 measured in the capture.
    let mut cam_lock_view: HashMap<i64, f64> = HashMap::new();
    let cam_locked_gos: std::collections::HashSet<i64> = {
        let mut out = std::collections::HashSet::new();
        if is_entrance
            && let Some(cam_tr) = entrance_camera_pid(all_objects)
                .and_then(|p| all_objects.get(&p))
                .and_then(|(_, v)| v.get("m_GameObject").and_then(get_path_id))
                .and_then(|g| go_to_transform.get(&g).copied())
        {
            // Camera projection, read once.
            let cam_obj = entrance_camera_pid(all_objects).and_then(|p| all_objects.get(&p));
            let is_ortho = cam_obj
                .and_then(|(_, v)| v.get("orthographic"))
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let ortho_sz = cam_obj
                .and_then(|(_, v)| v.get("orthographic size"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let fov = cam_obj
                .and_then(|(_, v)| v.get("field of view"))
                .and_then(Value::as_f64)
                .unwrap_or(60.0);
            // Authored-px scale, read from the SkeletonData asset the same way the caller does.
            // Ordered by path_id so a bundle with several skeletons cannot flip between runs.
            let mut skel: Vec<(i64, f64)> = all_objects
                .iter()
                .filter(|(_, (_, v))| v.get("skeletonJSON").is_some())
                .filter_map(|(p, (_, v))| Some((*p, v.get("scale").and_then(Value::as_f64)?)))
                .collect();
            skel.sort_unstable_by_key(|(p, _)| *p);
            let inv = 1.0 / skel.first().map_or(0.01, |(_, sc)| *sc);
            let local_of = |tf: i64| -> super::mesh::Mat4 {
                all_objects
                    .get(&tf)
                    .map_or_else(super::mesh::Mat4::identity, |(_, v)| {
                        let g3 = |f: &str, d: f32| {
                            let g = |k: &str| {
                                v.get(f)
                                    .and_then(|x| x.get(k))
                                    .and_then(Value::as_f64)
                                    .unwrap_or(d.into()) as f32
                            };
                            [g("x"), g("y"), g("z")]
                        };
                        let q = {
                            let g = |k: &str, d: f32| {
                                v.get("m_LocalRotation")
                                    .and_then(|x| x.get(k))
                                    .and_then(Value::as_f64)
                                    .unwrap_or(d.into()) as f32
                            };
                            [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
                        };
                        super::mesh::Mat4::trs(
                            g3("m_LocalPosition", 0.0),
                            q,
                            g3("m_LocalScale", 1.0),
                        )
                    })
            };
            for (&go, &tr) in &go_to_transform {
                let mut cur = tr;
                let mut m = super::mesh::Mat4::identity();
                for _ in 0..64 {
                    if cur == cam_tr {
                        out.insert(go);
                        // z of the GO's origin in CAMERA-LOCAL space.
                        let d = f64::from(m.point([0.0, 0.0, 0.0])[2]).abs();
                        let ext = if is_ortho {
                            2.0 * ortho_sz * inv
                        } else {
                            2.0 * d * (fov.to_radians() / 2.0).tan() * inv
                        };
                        if ext > 0.0 {
                            cam_lock_view.insert(go, ext);
                        }
                        break;
                    }
                    m = local_of(cur).mul(&m);
                    match all_objects
                        .get(&cur)
                        .and_then(|(_, v)| v.get("m_Father").and_then(get_path_id))
                        .filter(|&f| f != 0)
                    {
                        Some(f) => cur = f,
                        None => break,
                    }
                }
            }
        }
        out
    };
    let mut skipped_inactive = 0usize;
    // Env-gated attribution: the exported layers are anonymous, so tracing a missing
    // element back to its authored node (and to WHICH prefab root) otherwise means
    // re-deriving the walk by hand. Output-neutral.
    let attrib_dbg = std::env::var("SCENE_ATTRIB").is_ok();
    // DIAGNOSTIC (`SCENE_ATTRIB=1`): describe a renderer the walk is about to DROP by its
    // own data, so a dropped plate can be identified without re-admitting it: each
    // material as name:port:_MainTex, the mesh's world extent in scene px, and the time
    // window of any animated colour channel on the GameObject (chyue's side bands and
    // Lappland's cut fade, 2026-09-03).
    let drop_desc = |go_pid: i64, renderer: &Value| -> String {
        let mats: Vec<String> = renderer
            .get("m_Materials")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| get_path_id(r).filter(|&p| p != 0))
                    .filter_map(|mp| all_objects.get(&mp).map(|(_, m)| m))
                    .map(|m| {
                        let tex = m
                            .get("m_SavedProperties")
                            .and_then(|sp| sp.get("m_TexEnvs"))
                            .and_then(|te| te.as_array())
                            .and_then(|te| {
                                te.iter().find_map(|e| {
                                    let key = e.get("first").and_then(|k| k.as_str())?;
                                    if key != "_MainTex" {
                                        return None;
                                    }
                                    let tp = e
                                        .get("second")
                                        .and_then(|s| s.get("m_Texture"))
                                        .and_then(get_path_id)?;
                                    all_objects
                                        .get(&tp)
                                        .and_then(|(_, t)| t.get("m_Name"))
                                        .and_then(|n| n.as_str())
                                        .map(str::to_string)
                                })
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!(
                            "{}:{}:{}",
                            m.get("m_Name").and_then(|n| n.as_str()).unwrap_or("?"),
                            m.get("_shaderName").and_then(|n| n.as_str()).unwrap_or("?"),
                            tex
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let ext = go_to_mesh
            .get(&go_pid)
            .copied()
            .filter(|&mp| mp != 0)
            .and_then(|mp| all_objects.get(&mp))
            .and_then(|(cid, mv)| {
                (*cid == 43)
                    .then(|| super::mesh::parse_mesh(mv, resources))
                    .flatten()
            })
            .map_or_else(
                || "no in-bundle mesh".to_string(),
                |m| {
                    let world = go_to_transform
                        .get(&go_pid)
                        .map_or_else(super::mesh::Mat4::identity, |&tf| {
                            accumulate_matrix(all_objects, tf, &spine_gos, &idle_pose)
                        });
                    let mut lo = [f32::MAX; 2];
                    let mut hi = [f32::MIN; 2];
                    for p in &m.positions {
                        let w = world.point(*p);
                        lo[0] = lo[0].min(w[0]);
                        lo[1] = lo[1].min(w[1]);
                        hi[0] = hi[0].max(w[0]);
                        hi[1] = hi[1].max(w[1]);
                    }
                    format!("x {:.0}..{:.0} y {:.0}..{:.0}", lo[0], hi[0], lo[1], hi[1])
                },
            );
        let curves = color_channels.get(&go_pid).map_or_else(
            || "none".to_string(),
            |chs| {
                chs.iter()
                    .map(|c| {
                        let t0 = c.curve.first().map_or(0.0, |k| k.0);
                        let t1 = c.curve.last().map_or(0.0, |k| k.0);
                        format!("ch{} {:.2}..{:.2}", c.channel, t0, t1)
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            },
        );
        format!("mats={mats:?} ext={ext} colorcurves={curves}")
    };
    // Tint-source census and switch for the `_MainColor`-without-`_TintColor` population
    // (see `main_color_no_tint_color`). Read once here, not per layer.
    //
    // ⚠️ NOT `DYNCHAR_TINT_DECL`: that variable already gates a DIFFERENT
    // declaration-based tint switch at `reads_main_color`, which is measured worse, so
    // sharing the name would enable both at once and confound the score.
    let tint_census = std::env::var("DYNCHAR_TINTCENSUS").is_ok();
    // `=1` applies the arm to every qualifying layer. A comma-separated GameObject-name
    // list restricts it to those layers, which is how the population is ABLATED: the
    // corpus verdict is dominated by two skins, and cost per layer that runs inverse to
    // layers moved is the signature of an over-broad predicate rather than a uniform one.
    // `=1` for every qualifying layer, or a comma-separated MATERIAL-name list to ablate
    // one at a time. See `dissolve_spelling`.
    let dissolve_spelling_only: Option<Vec<String>> = std::env::var("DYNCHAR_DISSOLVE_SPELLING")
        .ok()
        .map(|v| match v.as_str() {
            "1" => Vec::new(),
            list => list
                .split(',')
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty())
                .collect(),
        });
    // SHIPPED, so the variable is now the OFF switch: absent means ON for every
    // qualifying layer, `=0` restores the pre-ship numbers exactly, and a
    // comma-separated GameObject-name list still restricts it for ablation.
    // Checked against the explicit string, never a falsy coercion.
    // SCENE-SIDE `Dissolve/Dissolve Add UVTween` (2026-09-07). The particle side ports this
    // program (`kind: "uvTween"`, see `particles::resolve_ram`); on the scene side the same
    // materials were vetoed by the `_ToggleUseDissolve` default and drew as plain quads, seven
    // of Ch'en the Holungday E2's layers among them (BirdScarf's HUD counts them as "Dissolve
    // masked 7"). The decompiled program masks UNCONDITIONALLY and pans both lookups by
    // `_Time.y * _UVTween` (main by .xy, dissolve by .zw), so for this family the veto is
    // skipped and both pans come from `_UVTween`. `DYNCHAR_UVTWEEN_SCENE=0` restores the
    // previous export exactly; checked against the explicit string, never a falsy coercion.
    let scene_uvtween_on = std::env::var("DYNCHAR_UVTWEEN_SCENE").as_deref() != Ok("0");
    // ERASE MASKS (2026-09-07): keep `Mask/Erase` quads as `erase` layers (see
    // `BgQuad::erase`). `DYNCHAR_ERASE_MASKS=0` restores the grab-pass drop exactly; checked
    // against the explicit string, never a falsy coercion.
    let erase_masks_on = std::env::var("DYNCHAR_ERASE_MASKS").as_deref() != Ok("0");
    // DISTURB2 (2026-09-07): `Particles-L2D/Disturb/Disturb2 (Add|AlphaBlend)` decompiled
    // (pathIDs -1925973235195993132 and 4827025966035922101): the vertex stage builds two
    // noise UVs from the `_MainTex_ST`-baked UV, `uv * _NoiseN.x + (fract(_Time.x * _NoiseN.y),
    // 0)`, and the fragment offsets the main lookup by `distur(uv1).x * _Noise1.zw * 0.1 +
    // distur(uv2).y * _Noise2.zw * 0.1`, multiplies by `_MainColor` (x1) and, on the Add pass,
    // premultiplies by the texture's alpha. No anchor, no dissolve, no ramp, and
    // `_DisturTex_ST` is never read. `_Time.x` is seconds / 20. The `_DISTURBMODE_DEFAULT`
    // keyword variant only; the other packs a glow channel through `_GlowColor` and stays
    // out. `DYNCHAR_DISTURB2=0` restores the plain-quad export exactly.
    let disturb2_on = std::env::var("DYNCHAR_DISTURB2").as_deref() != Ok("0");
    let main_color_decl: Option<Vec<String>> = match std::env::var("DYNCHAR_MAINCOLOR_DECL") {
        Err(_) => Some(Vec::new()),
        Ok(v) => match v.as_str() {
            "0" => None,
            "1" => Some(Vec::new()),
            list => Some(
                list.split(',')
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect(),
            ),
        },
    };
    for (_, renderer) in renderers {
        let Some(go_pid) = renderer.get("m_GameObject").and_then(get_path_id) else {
            continue;
        };
        if spine_gos.contains(&go_pid)
            || !renderer
                .get("m_Enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
        {
            continue;
        }
        // ⛔ REFUTED: gating entrance SCENE quads the same way is catastrophic.
        //
        // The particle gate in `particles.rs` withholds entrance effects hosted in an IDLE state
        // clone, and the symmetric rule looked obvious here — Ch'en's `sand` quads hang off
        // `..._Idle_SC_CY_Belt_B2_01(Clone)`. It is WRONG: an entrance cinematic legitimately
        // draws the character's idle SCENERY, and dropping it removes the set out from under her.
        // Measured (216 layers over 13 entrances, excu2's whole 56-layer scene among them):
        // exc 6.292 -> 57.932 (r .931 -> .330), ska 9.854 -> 18.498, mly 17.007 -> 25.708,
        // against only mue -0.095 / eyja -0.209 / cet -0.098. Do not re-attempt.
        //
        // 🚨 THE IDLE SCENE MUST NOT DRAW THE ENTRANCE RIG.
        //
        // A dynchar bundle ships TWO prefab roots: the idle world (`dyn_illust_*`) and the
        // entrance cinematic (`dyn_entrance_*`). `own_root` only ever ORDERED them, it never
        // filtered — so the idle scene collected the cinematic's quads too. Those quads are
        // sequenced entirely by the `_Start` clip's reveal timeline and its colour curves, and
        // the idle scene runs NEITHER (`active_windows` is deliberately entrance-only, and the
        // layer colour curves come from the same clip). Any of them that ships prefab-ACTIVE is
        // therefore emitted with no window, no curve and full tint: drawn forever at full
        // strength.
        //
        // Kal'tsit "Remnant" is the case that exposed it. `wenl1` and `wenli` are a PURE WHITE
        // 2341x2181 sheet under `03 > Main Camera > ... > dyn_entrance_char_003_kalts_boc#6`,
        // correctly gated 2.97 -> 4.5 / 5.9 in her entrance and emitted UNGATED into her idle.
        // Her settled shot rendered 48% blown white, mean luma 181 against the game's 97.
        //
        // The converse is NOT symmetric and must stay: an ENTRANCE legitimately draws idle-root
        // layers, sequenced by `cross_root_reveal` (Virtuosa's mirror world). This drops only
        // entrance-root quads from the IDLE scene. `DYNCHAR_IDLE_ENTRANCE=1` reverts.
        if !is_entrance
            && std::env::var("DYNCHAR_IDLE_ENTRANCE").as_deref() != Ok("1")
            && let Some(root) = host.prefab_root_of_go(all_objects, go_pid)
            && Some(root) != own_root
            && all_objects
                .get(&root)
                .and_then(|(_, v)| v.get("m_Name"))
                .and_then(Value::as_str)
                .is_some_and(|n| n.to_ascii_lowercase().starts_with("dyn_entrance"))
        {
            continue;
        }
        // ENTRANCE visibility window (self or nearest toggled ancestor). Resolved
        // BEFORE the active-drop below: entrance-exclusive overlays (the
        // `dyn_entrance_*` subtree, e.g. Mlynar's white-transition flash) ship
        // `m_IsActive=0` in the prefab and are switched ON by the `_Start` clip's
        // reveal timeline, so the static walk alone would drop them.
        let window = reveal_of_go(go_pid, &reveal_map, &go_to_transform, all_objects);
        // Cross-root reveal: in an ENTRANCE scene with an authored transform beat, a
        // layer under ANOTHER skeleton's prefab root (the idle `dyn_illust_*` world)
        // only becomes visible when the game activates that prefab AT the transform.
        // Virtuosa's white mirror-world lives ONLY in the idle root — un-sequenced it
        // washed out the 10.2–12.4s reveal that the game shows as the blue sea.
        // Kept SEPARATE from the clip window: it must not rescue layers the gates
        // below drop (inactive groups / `_meshExtResolved` materials).
        let cross_from: Option<f32> = if is_entrance
            && let Some(tr) = cross_root_reveal
            && let (Some(own), Some(root)) = (own_root, host.prefab_root_of_go(all_objects, go_pid))
            && root != own
            && skeleton_roots.contains(&root)
        {
            Some(tr as f32)
        } else {
            None
        };
        // A/B PROBE (`DYNCHAR_CROSSROOT_UNGATED=0`), kept for the record — the idea it tests is
        // REFUTED. An idle-root layer inside an ENTRANCE with NO authored transform beat has
        // nothing to sequence it: `cross_root_reveal` is None, so `cross_from` above stays None
        // and the layer draws from t=0 for the whole cinematic. That looks wrong — the game
        // activates the idle prefab at the HAND-OFF, not before — and only two skins are affected
        // (Mlynar 5 layers, pasngr 5; every other entrance ships a transform beat).
        //
        // ⛔ But dropping them SCORES WORSE: Mlynar 17.191 -> 17.363. The game does draw them.
        // Keep the default (admit), and do not re-litigate this without new evidence.
        if is_entrance
            && cross_root_reveal.is_none()
            && std::env::var("DYNCHAR_CROSSROOT_UNGATED").as_deref() == Ok("0")
            && let (Some(own), Some(root)) = (own_root, host.prefab_root_of_go(all_objects, go_pid))
            && root != own
            && skeleton_roots.contains(&root)
        {
            continue;
        }
        // Skip renderers under a state-gated (inactive) group. The prefab keeps
        // "Start Only Effects" / "Interact Only Effects" / "Special Only Effects"
        // groups m_IsActive=0 by default (the game activates them only during
        // those states), so their descendants must not appear in the idle scene.
        // EXCEPT: in the `_Start` scene, a GO whose `m_IsActive` the cinematic
        // clip drives (a reveal window exists) is kept — the clip overrides the
        // prefab flag at runtime and the window gates visibility instead. Covers
        // both delayed reveals (Mlynar's white transition, activeFrom 13.0) and
        // shown-from-0-then-hidden overlays (activeUntil only).
        // A GO whose `_Start` clip reveals it via an ANIMATED MATERIAL COLOR/ALPHA
        // curve (rather than an `m_IsActive` toggle) has no `window`, so the static
        // walk + window check alone would drop it (e.g. Mlynar's warm ring `huan`,
        // whose `_MainColor` alpha ramps 0→0.43→0 with NO `m_IsActive` binding).
        // Admit it here when it carries an animated alpha channel that STARTS HIDDEN
        // (alpha ≈ 0 at its first sample) — the same `color_channels` that feed
        // `layer_color_curve` below. Layers admitted ONLY by this exception are
        // re-validated after `color_curve` resolves (see `admitted_by_reveal` below),
        // so a hash-collision channel that doesn't match this material can't resurrect
        // an ungated always-on layer.
        let has_color_reveal = color_channels.get(&go_pid).is_some_and(|chs| {
            chs.iter()
                .any(|c| c.channel == 3 && c.curve.first().is_some_and(|&(_, v)| v.abs() < 0.02))
        });
        let eff_active = go_effectively_active(
            all_objects,
            go_pid,
            &go_to_transform,
            &idle_pose.active,
            is_entrance,
            true,
        );
        // A colour-reveal admission must not override the state gate: a group the game
        // reserves for another state stays out no matter what its curves do.
        let state_blocked = state_only_blocked(all_objects, go_pid, &go_to_transform, is_entrance);
        if !eff_active && window.is_empty() && (!has_color_reveal || state_blocked) {
            skipped_inactive += 1;
            if attrib_dbg {
                let (by, why) = blocking_ancestor(
                    all_objects,
                    go_pid,
                    &go_to_transform,
                    &idle_pose.active,
                    is_entrance,
                )
                .unwrap_or_else(|| ("?".to_string(), "?"));
                eprintln!(
                    "    [scene] DROP inactive-group  {:<26} blocked_by='{by}' ({why}) root={} {}",
                    host.go_name(all_objects, go_pid),
                    host.root_name_of_go(all_objects, go_pid),
                    drop_desc(go_pid, renderer)
                );
            }
            continue;
        }

        let sort = renderer
            .get("m_SortingOrder")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);

        // Material: first with a resolvable, non-spine _MainTex.
        let materials = renderer.get("m_Materials").and_then(|v| v.as_array());
        let Some(materials) = materials else { continue };
        // (tex_val, alpha_val, tint, additive, main_pid, src_blend, dst_blend,
        //  _MainTex ST as [scale_x, scale_y, offset_x, offset_y],
        //  (saved colour props, tint-source prop) for the entrance colour animation,
        //  shader tint scale — 2.0 for the legacy ×2 _TintColor family)
        type ResolvedQuadMaterial = (
            Value,
            Option<Value>,
            [f32; 4],
            bool,
            i64,
            f64,
            f64,
            [f64; 4],
            (Vec<(String, [f32; 4])>, Option<String>),
            f32,
            bool,
            Option<[f32; 2]>,
            Option<SceneRam>,
            Option<f32>,
        );
        let mut resolved: Option<ResolvedQuadMaterial> = None;
        // Whether the chosen material's `_MainTex` came from the ungated external
        // (`_meshExtResolved`) path — used below to re-validate a color-reveal overlay
        // admitted past the meshExt gate.
        let mut resolved_meshext = false;
        // The `_MainColorACtrl` of the material the quad resolved to (0.0 = identity), read
        // inside the material loop because `mat` does not outlive it; applied to the tint
        // and its curve once both are resolved (see `anchor_ctrl`).
        let mut anchor_ctrl_v = 0.0f32;
        // The RAW `_MainColor.a` the fragment forms `k` from. The resolved tint's alpha is
        // not it: on the half-neutral x2 path that alpha is `min(2a, 1)`, which reads 1.0
        // for a = 0.5 and for a = 1.0 alike, and those two give k = 0.5 and k = 1.
        let mut anchor_a_v = 1.0f32;
        for mat_ref in materials {
            let Some(mat_pid) = get_path_id(mat_ref).filter(|&p| p != 0) else {
                continue;
            };
            let Some((21, mat)) = all_objects.get(&mat_pid) else {
                continue;
            };
            // A `_MainTex` resolved BEYOND the baseline particle gates (opaque
            // fill / distortion-shader sprite — see `_meshExtResolved` in the
            // extractor) renders only on entrance-windowed layers: the cinematic
            // deliberately sequences those (Mlynar's white flash), while an
            // always-on frozen fx quad would pollute the idle scene.
            // A color-reveal overlay whose art lives in a shared fx bundle resolves
            // its `_MainTex` via the ungated external path (`_meshExtResolved`), same
            // as the windowed flash plane. Keep it too (gated by its reveal curve),
            // otherwise a warm-ring/god-ray reveal painted on an external texture is
            // lost. Non-reveal meshExt quads without a window stay dropped (they'd be
            // always-on frozen fx polluting the idle scene).
            // ⚠️ The "always-on frozen fx" justification is IDLE-SPECIFIC: an entrance is a
            // finite cinematic, so a windowless always-on backdrop plane there is ordinary
            // scenery, not pollution. `DYNCHAR_MESHEXT_ENT=1` keeps them for entrances only.
            let meshext_keep_entrance =
                is_entrance && std::env::var("DYNCHAR_MESHEXT_ENT").as_deref() != Ok("0");
            // MEASUREMENT ARM (2026-09-06): `DYNCHAR_MESHEXT_IDLE=1` keeps windowless meshExt
            // quads in the IDLE scene too. Nearl Relight's `1a..8a` (the pictures inside her
            // 13 frames, which the game's card shows) and Thorn marthe#9's stage floor fall
            // to this rule; 370 quads across the corpus do. Off by default until gated.
            let meshext_keep_idle =
                !is_entrance && std::env::var("DYNCHAR_MESHEXT_IDLE").as_deref() == Ok("1");
            if mat.get("_meshExtResolved").is_some()
                && window.is_empty()
                && !has_color_reveal
                && !meshext_keep_entrance
                && !meshext_keep_idle
            {
                if attrib_dbg {
                    eprintln!(
                        "    [scene] DROP meshExt-no-window  {:<26}",
                        host.go_name(all_objects, go_pid)
                    );
                }
                continue;
            }
            // _MainTex slot: texture + its Scale/Offset (ST). Unresolvable
            // (cross-bundle) or non-Texture2D refs yield None → skip material.
            let (main_pid, tex_val, st) =
                super::particles::mat_texenv(all_objects, mat, "_MainTex");
            let (Some(main_pid), Some(tex_val)) = (main_pid, tex_val) else {
                if attrib_dbg {
                    eprintln!(
                        "    [scene] DROP no _MainTex       {:<26} mat={mat_pid} {}",
                        host.go_name(all_objects, go_pid),
                        drop_desc(go_pid, renderer)
                    );
                }
                continue;
            };
            if spine_tex_pids.contains(&main_pid) {
                if attrib_dbg {
                    eprintln!(
                        "    [scene] DROP spine-atlas tex   {:<26}",
                        host.go_name(all_objects, go_pid)
                    );
                }
                continue;
            }
            let tex_name = tex_val.get("m_Name").and_then(|v| v.as_str()).unwrap_or("");
            if tex_name.to_lowercase().starts_with("dyn_illust_")
                || tex_name.to_lowercase().starts_with("dyn_portrait_")
            {
                continue;
            }
            let (_, alpha_val, _) = super::particles::mat_texenv(all_objects, mat, "_AlphaTex");
            let blend = |k: &str, d: f64| {
                mat.get("m_SavedProperties")
                    .and_then(|s| s.get("m_Floats"))
                    .and_then(|f| f.get(k))
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(d)
            };
            // Legacy ×2 tint family: the shader samples `2 × _TintColor × tex`, so the
            // effective static tint is the DOUBLED `_TintColor` (clamped like the blend
            // stage) and `_TintColor` — not the unused `_Color` leftover — is the tint
            // source the animated channels replace.
            //
            // Ram-family (`Torappu/Particles-L2D/Ram/*`) scene compositors are the same
            // ×2 convention under a DIFFERENT property: they modulate by `_MainColor`
            // (α≈0.5 = neutral, mirrored by the frontend Ram GLSL's `col += col`), and
            // carry an inert Unity-default `_Color` placeholder that `material_tint`
            // would wrongly pick — rendering a soft/dim `_MainColor` as full white. So a
            // Ram layer uses `(_MainColor × 2).clamp`, with `_MainColor` as the animated
            // channels' tint source and ×2 threaded into `layer_color_curve`.
            // The `Particles-L2D` port is admitted only for a layer the cinematic
            // ANIMATES: there the clip names the colour property the shader actually
            // modulates, so the tint source and its ×2 are established by the data
            // rather than inferred from the shader name alone. Static layers keep the
            // previous `_Color` reading — the corpus evidence for the family is strong
            // (see `legacy_tint_scale`) but unverifiable against a recording, and the
            // idle scenes are the most-viewed surface.
            let animated_color = color_channels.contains_key(&go_pid);
            // Peak of every animated COLOUR (rgb) channel on this GO — the value a ×2 family
            // would have to double without clamping.
            // Does the animated colour RAMP in rgb, or only in ALPHA? Both gates inside
            // `ram_tint_scale` exist to stop the clamp flattening a RAMP; a curve whose rgb is
            // CONSTANT has no ramp to protect, so the clamp costs nothing there. Data-derived —
            // it reads the curve, not the skin.
            let rgb_constant = color_channels.get(&go_pid).is_some_and(|chs| {
                chs.iter().filter(|c| c.channel < 3).all(|c| {
                    let mut it = c.curve.iter().map(|&(_, v)| v);
                    it.next()
                        .is_none_or(|f| it.all(|v| (v - f).abs() <= 0.5 / 255.0))
                })
            });
            let animated_peak = color_channels.get(&go_pid).map(|chs| {
                chs.iter()
                    .filter(|c| c.channel < 3)
                    .flat_map(|c| c.curve.iter().map(|&(_, v)| v))
                    .fold(0.0f32, f32::max)
            });
            // Whether the entrance clip names `_MainColor` on THIS layer — the evidence the
            // shader modulates by it (see the `l2d_main_color_family` branch below).
            let animates_main_color = color_channels
                .get(&go_pid)
                .is_some_and(|chs| super::anim::animates_prop(chs, "_MainColor"));
            let (legacy_scale, legacy_hdr) = legacy_tint_scale(mat, animated_color);
            let (ram_scale, ram_hdr) = ram_tint_scale(mat, animated_peak, rgb_constant);
            let mut cprops = material_color_props(mat);
            let (tint, tint_scale, hdr_color) = if legacy_scale > 1.0 {
                cprops.1 = Some("_TintColor".to_string());
                let tc = cprops
                    .0
                    .iter()
                    .find(|(n, _)| n == "_TintColor")
                    .map_or([1.0; 4], |(_, c)| *c);
                // RGB ceiling is INFINITY only for the newly-admitted sub-namespaced families
                // (see `legacy_tint_scale`); the long-shipped ones keep the 1.0 clamp. Alpha is
                // always clamped: it is a coverage/blend weight, not light — a premultiplied
                // source alpha above 1 makes the destination factor `1 - a` negative.
                let hi = if legacy_hdr { f32::INFINITY } else { 1.0 };
                (
                    [
                        (tc[0] * legacy_scale).clamp(0.0, hi),
                        (tc[1] * legacy_scale).clamp(0.0, hi),
                        (tc[2] * legacy_scale).clamp(0.0, hi),
                        (tc[3] * legacy_scale).clamp(0.0, 1.0),
                    ],
                    legacy_scale,
                    legacy_hdr,
                )
            } else if ram_scale > 1.0 {
                cprops.1 = Some("_MainColor".to_string());
                let mc = cprops
                    .0
                    .iter()
                    .find(|(n, _)| n == "_MainColor")
                    .map_or([1.0; 4], |(_, c)| *c);
                // RGB is NOT clamped to 1: the ×2 is a real over-bright multiply and the
                // frontend renders scene layers into a half-float HDR target
                // (`hdrTonemap.ts`), so a doubled value above 1 is representable and gets
                // tonemapped, not truncated. Clamping here flattened a ramp's baseline and
                // its peak onto the same ceiling, shrinking the very brightening delta the
                // ×2 exists to reproduce. ALPHA stays clamped: it is a coverage/blend
                // weight, not light — a premultiplied source alpha above 1 makes the
                // destination factor `1 - a` negative and corrupts the composite.
                let hi = if ram_hdr { f32::INFINITY } else { 1.0 };
                (
                    [
                        (mc[0] * ram_scale).clamp(0.0, hi),
                        (mc[1] * ram_scale).clamp(0.0, hi),
                        (mc[2] * ram_scale).clamp(0.0, hi),
                        (mc[3] * ram_scale).clamp(0.0, 1.0),
                    ],
                    ram_scale,
                    ram_hdr,
                )
            } else if animates_main_color && l2d_main_color_family(mat) {
                // Same `_MainColor` compositor family as the Ram branch above, but WITHOUT
                // its ×2 convention (`ram_tint_scale` rejects a `_MainColor` that already
                // reaches full scale, which doubling could only clamp). Rejecting the
                // DOUBLING is not a reason to fall back to `material_tint`: that reads the
                // inert Unity `_Color` = white placeholder these materials carry, so the
                // layer renders with `_MainColor`'s ALPHA (the clip animates it, and it is
                // the curve we already export) over `_Color`'s WHITE rgb — one property's
                // opacity wearing another's colour. Mlynar "Fields of Ruination"'s entrance
                // wind sheets are authored `_MainColor` = (0.41, 0.46, 1.0) — a deep blue —
                // and were painting a full-white haze over the whole frame for 13 s.
                // Gated on the clip ANIMATING `_MainColor`, the same data-derived evidence
                // the ×2 port uses: the clip naming the property proves the shader
                // modulates by it, rather than inferring it from the shader name alone.
                cprops.1 = Some("_MainColor".to_string());
                let mc = cprops
                    .0
                    .iter()
                    .find(|(n, _)| n == "_MainColor")
                    .map_or([1.0; 4], |(_, c)| *c);
                // EXPERIMENT (`DYNCHAR_MC_X2=1`, default OFF) — **MEASURED AND REFUTED**,
                // and the most direct test this question will ever get.
                //
                // Rejecting `ram_tint_scale`'s HEURISTIC for the ×2 is not the same as
                // establishing that the shader doesn't double, so this asks the GLSL
                // instead of the authored value (see `main_color_doubles`). For these wind
                // sheets the answer is unambiguously YES — `Disturb(CustomData)` runs
                // `c = tex * _MainColor * vs_COLOR0; c = c + c;` and the frontend's
                // `RAM_SCENE_FRAG` deliberately does not double — so on paper the layer
                // renders at half Unity's amplitude.
                //
                // rgb is deliberately NOT clamped here: the shader writes `SV_Target0.xyz`
                // straight through (only `.w` is clamped) and the frontend composites scene
                // layers into a half-float HDR target, so the blue reaches 2.0 the way
                // Unity has it. That makes this a STRICTLY different test from the earlier
                // `DYNCHAR_RAM_X2_CONST`, which took the clamped path and pinned that 2.0
                // back to 1.0 — a desaturated double. Both are now refuted, in both
                // directions: clamped costs mly 17.405 -> 18.501, this HDR one costs
                // 17.405 -> **23.666**. `tint_scale` rides along onto the animated curve
                // (`anim.rs`), so the port covers the whole timeline, not just the base.
                //
                // What that buys: the long-open blue cast is NOT a missing shader ×2. The
                // doubling provably exists in Unity and porting it is measurably wrong by a
                // wide margin, so something else in this pipeline already carries that
                // factor for this family. It is not `EFFECT_SCENE_GAIN` (0.3, a reduction,
                // and these layers are `fullGain`-exempt for carrying a colour curve).
                // Do not re-derive from the GLSL alone — the shader is right and the
                // conclusion still doesn't follow.
                if std::env::var("DYNCHAR_MC_X2").is_ok() && main_color_doubles(mat) {
                    (
                        [
                            mc[0] * 2.0,
                            mc[1] * 2.0,
                            mc[2] * 2.0,
                            (mc[3] * 2.0).min(1.0),
                        ],
                        2.0,
                        true,
                    )
                } else {
                    (mc, 1.0, false)
                }
            } else if let Some(how) = main_color_no_tint_color(mat) {
                // A shader that reads `_MainColor` on a material with NO `_TintColor`.
                // `material_tint` below would fold `_Color`, the inert Unity white
                // placeholder, so the layer draws at full opacity in a colour the shader
                // was never told to use. The branch above it wants the CLIP to name
                // `_MainColor` as its evidence; that evidence is unavailable to a layer no
                // clip animates, and the shader's own declaration is the stronger source
                // anyway.
                //
                // ⛔ MEASURED CORPUS-WIDE AND WORSE, so the default is OFF.
                // `DYNCHAR_TINTCENSUS=1` prints the population, `DYNCHAR_MAINCOLOR_DECL=1`
                // applies it. Every one of the 370 layers that reach this arm is admitted by
                // the shader's DECLARATION (`via=decl`); the material fallback fires zero
                // times with the shaders staged, and all 370 would move (none is a no-op).
                // With the flag on, 37 exported files change across 31 skins, but only FIVE
                // are entrance scenes, so only five references can move at all:
                //
                //   cel  17.039 -> 17.002   mue    10.756 -> 11.422   exc  6.292 -> 6.292
                //   wis   5.557 ->  5.557   whitw2 33.431 -> 35.216
                //
                // The other seven are bit-identical (ska, mly, eyja, cet, chyue, fugue,
                // kalts). Net +2.414 over the twelve; corpus-8 mean 11.481 -> 11.560.
                // Whislash-alter carries the damage (r .717 -> .686) and Muelsyse most of the
                // rest (r .935 -> .930), so it is not noise: reading `_MainColor` here dims
                // layers the game evidently draws brighter.
                //
                // The reading is not what is in doubt. On Ch'en the Holungday's two lens
                // sheets it is provably right: this arm reproduces the hand-patched
                // (0.6322, 0.6239, 0.7255, 0.1804) and (0.6145, 0.6505, 0.7255, 0.251) and
                // scores 21.583 -> 16.876 with `DYNCHAR_MESHRES=1`, recovering 4.707 of the
                // 4.931 that folding the inert `_Color` costs there. What the corpus says is
                // that something else in this pipeline already compensates for the wrong
                // tint on the OTHER layers, exactly as the ×2 experiments above found.
                if tint_census {
                    eprintln!(
                        "    [tintcensus] {:<26} shader={} via={how} _Color={:?} _MainColor={:?} root={}",
                        host.go_name(all_objects, go_pid),
                        mat.get("_shaderName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?"),
                        material_tint(mat),
                        cprops
                            .0
                            .iter()
                            .find(|(n, _)| n == "_MainColor")
                            .map(|(_, c)| *c),
                        own_root.map_or_else(|| "?".to_string(), |r| host.go_name(all_objects, r)),
                    );
                }
                let decl_on = main_color_decl.as_ref().is_some_and(|only| {
                    only.is_empty() || only.iter().any(|n| *n == host.go_name(all_objects, go_pid))
                });
                if decl_on {
                    cprops.1 = Some("_MainColor".to_string());
                    let mc = cprops
                        .0
                        .iter()
                        .find(|(n, _)| n == "_MainColor")
                        .map_or([1.0; 4], |(_, c)| *c);
                    // THE ×2 BELONGS HERE TOO (`DYNCHAR_MC_X2`, the same flag the branch
                    // above uses, default OFF). That branch applies `main_color_doubles`
                    // and this one did not, which is exactly the asymmetry the corpus
                    // measured. Ch'en the Holungday is `Dissolve(CustomData)`, a verified
                    // NON-doubling family, so folding `_MainColor` whole is complete and
                    // worth -4.707. Whislash-alter's `jiu (1)` is `Disturb Anchor
                    // (AlphaBlend)`, a DOUBLING family: the fragment runs
                    // `tex * _MainColor * vs_COLOR0` and then `c = c + c`, so an authored
                    // 0.5 grey is the NEUTRAL and folding it un-doubled renders the layer
                    // at half. Same clamp discipline as the branch above: rgb is not
                    // clamped because the frontend composites into a half-float HDR
                    // target, alpha is, because it is a coverage weight.
                    //
                    // ⚠️ `_MainColorACtrl` is identity on `jiu (1)` and not for the reason
                    // the note on `main_color_doubles` gives: her `_MainColor.w` is 1.0, so
                    // `mix(1, _MainColor.w, ctrl)` is 1 whatever `ctrl` holds. Do not chase
                    // it.
                    // ⚠️ ITS OWN FLAG, deliberately NOT `DYNCHAR_MC_X2`. That name also
                    // enables the doubling in the branch above, which is separately measured
                    // and refuted, and sharing it confounds the corpus: Mlynar is
                    // bit-identical under this arm alone and goes 17.007 -> 23.805 the moment
                    // the shared flag lights the other branch as well.
                    if std::env::var("DYNCHAR_MC_X2_DECL").as_deref() != Ok("0")
                        && main_color_doubles(mat)
                    {
                        (
                            [
                                mc[0] * 2.0,
                                mc[1] * 2.0,
                                mc[2] * 2.0,
                                (mc[3] * 2.0).min(1.0),
                            ],
                            2.0,
                            true,
                        )
                    } else {
                        (mc, 1.0, false)
                    }
                } else {
                    (material_tint(mat), 1.0, false)
                }
            } else {
                (material_tint(mat), 1.0, false)
            };
            // Capability A — Ram-family shader UV-scroll (static material floats, no clip).
            // Gate STRICTLY: only a Ram-family shader with a non-zero `_Main*Speed` carries
            // scroll; every other layer stays byte-identical.
            let uv_scroll = {
                let shader = mat
                    .get("_shaderName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                // The `Ram/` gate understates the family. Decompiling
                // `Particles-L2D/Disturb/Disturb(CustomData)` shows the SAME main-texture
                // pan in its vertex program:
                //     u_xlat0.xy = _Time.yy * vec2(_MainUSpeed, _MainVSpeed);
                //     vs_TEXCOORD0.xy = u_xlat0.xy + (in_TEXCOORD0.xy * _MainTex_ST.xy + _MainTex_ST.zw);
                // so every L2D compositor scrolls `_MainTex`, not just `Ram/`, and a non-Ram
                // layer with a non-zero `_Main*Speed` renders STATIC where the game pans it.
                // `DYNCHAR_UVSCROLL_ALL=1` widens the gate to the whole family.
                //
                // MEASURED, and kept OFF: it gives 46 of Virtuosa's layers a scroll they
                // currently lack (a sort -4 family at +-0.2 v/s -- her vertical rain -- and a
                // sort 4 family at +-0.1 u/s), and moves her by **+0.008** (15.342 -> 15.350;
                // per beat t=5 +0.078, t=14 -0.023, t=17 -0.008). Mlynar and Skadi export
                // byte-identical. So the gate is shader-CORRECT and parity-NEUTRAL on the only
                // reference it touches, which is not enough to justify a corpus-wide
                // reclassification unmeasured elsewhere -- see `dynchar-veil-frame-coverage`.
                // Turn it on only together with a corpus render of the affected skins.
                let wide =
                    std::env::var("DYNCHAR_UVSCROLL_ALL").is_ok() && is_l2d_compositor(shader);
                // `Dissolve Add UVTween` pans `_MainTex` by `_UVTween.xy` (its `_Main*Speed`
                // floats are undeclared residue); see `scene_uvtween_on`.
                let uvtween_scene = scene_uvtween_on && is_uvtween_program(shader);
                if uvtween_scene {
                    let c = super::particles::mat_color(mat, "_UVTween", [0.0; 4]);
                    let (us, vs) = (c[0] as f32, c[1] as f32);
                    if us != 0.0 || vs != 0.0 {
                        Some([us, vs])
                    } else {
                        None
                    }
                } else if shader.contains("Ram/") || wide {
                    let us = blend("_MainUSpeed", 0.0) as f32;
                    let vs = blend("_MainVSpeed", 0.0) as f32;
                    if us != 0.0 || vs != 0.0 {
                        Some([us, vs])
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            // DISSOLVE/DISTURB masking (see [`SceneRam`]). The mask pair is not a `Ram/`
            // peculiarity: every sub-namespaced `Particles-L2D` compositor carves its
            // quad's real silhouette out of `_DissolveTex` and warps the lookup by
            // `_DisturbTex`, and the shader maths the frontend already runs is the same
            // for all of them. Restricting it to `Ram/` left Mlynar "Fields of
            // Ruination"'s entrance wind sheets (`Disturb/Disturb(CustomData)`,
            // `_Amount` 0.10 / `_BorderWidth` 1.0 ⇒ ~0.37 mean coverage) drawn as full
            // opaque rectangles across the frame.
            //
            // What admits a layer is the MATERIAL, not the shader name: it has to
            // actually bind one of the two maps (the `has(…)` test below). The dissolve
            // slot additionally answers to `_ToggleUseDissolve`, the shader's own keyword
            // switch — masking by a slot the material has switched off would erase the
            // layer, so that drops the dissolve while keeping any disturb. `Ram/` keeps
            // the default-ON reading it shipped with; a newly admitted family has to
            // DECLARE the switch, so the material itself states its dissolve is live.
            //
            // Strictly ADDITIVE to the shipped gate: `Ram/` stays matched wherever it
            // lives, including the two `Torappu/Particles/Ram/…` layers (cgbird, Archetto)
            // that are outside the `Particles-L2D` namespace entirely.
            let ram = {
                let shader = mat
                    .get("_shaderName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                // Does THIS shader carry its UV pan in `_UVTween` rather than the
                // `_Dissolve*Speed`/`_Disturb*Speed` pairs? See the note on `dissolve_speed`
                // below: the two families are mutually exclusive in the decompiled uniform
                // lists, so the shader name selects which property is live and which is
                // serialized residue.
                // SHIPPED 2026-09-07 (`DYNCHAR_UVTWEEN=0` restores the Speed-pair read exactly):
                // gated on the entrance beats of the four skins it touches (kalts boc#6,
                // Executor sale#12, Muelsyse boc#8, Skadi iteration#2), 26 beats, 12
                // bit-identical, no key moving more than 0.16 luma, and neutral on its nine
                // settled rows (5 bit-identical). Ships on the decompiled uniform lists above.
                let uvtween_scene = scene_uvtween_on && is_uvtween_program(shader);
                let uvtween_family = uvtween_scene
                    || (std::env::var("DYNCHAR_UVTWEEN").as_deref() != Ok("0")
                        && (shader.contains("UVTween") || shader.contains("Disturb Anchor")));
                let uvtween = {
                    let c = super::particles::mat_color(mat, "_UVTween", [0.0; 4]);
                    [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32]
                };
                if std::env::var("DYNCHAR_SHADER_DEBUG").is_ok() {
                    eprintln!(
                        "  [shader] {:<52} uvtween={:?} dissSpeed=({:.3},{:.3})",
                        shader,
                        uvtween,
                        blend("_DissolveUSpeed", 0.0),
                        blend("_DissolveVSpeed", 0.0)
                    );
                }
                if shader.contains("Ram/") || is_l2d_compositor(shader) {
                    // TWO-MAP NAMES FIRST. The `Dissolve/Dissolve` family reads
                    // `_DissolveTex_01`/`_02` with `_Amount_01/_02` and `_BorderWidth_01/_02`;
                    // its single-name `_Amount`/`_BorderWidth`/`_DissolveTex` properties are
                    // INERT RESIDUE from the Standard shader the asset was authored against.
                    // Reading the residue picked up meaningless thresholds and resolved NO map,
                    // so no `ram` block was emitted at all and the layer drew as its full
                    // bounding rectangle instead of the carved silhouette — 44 exported scene
                    // layers across the corpus. Prefer the `_01`/`_02` pair wherever it resolves
                    // and fall back to the single names, so `Ram/` and `Disturb/` are untouched.
                    let t01 = super::particles::mat_texenv(all_objects, mat, "_DissolveTex_01");
                    let two_map = t01.0.is_some();
                    let (mut diss_pid, mut diss_val, diss_st) = if two_map {
                        t01
                    } else {
                        super::particles::mat_texenv(all_objects, mat, "_DissolveTex")
                    };
                    // `_WeightTex` — the disturb WEIGHT map (see `SceneRam::weight_*`). Resolved
                    // unconditionally: the slot lookup returns None for any material that does
                    // not bind it, so single-map and Ram-family layers are unchanged.
                    let (weight_pid, weight_val, weight_st) =
                        super::particles::mat_texenv(all_objects, mat, "_WeightTex");
                    let (diss2_pid, diss2_val, diss2_st) = if two_map {
                        super::particles::mat_texenv(all_objects, mat, "_DissolveTex_02")
                    } else {
                        (None, None, [1.0, 1.0, 0.0, 0.0])
                    };
                    // A two-map material states its intent by BINDING the maps; it carries no
                    // `_ToggleUseDissolve` at all, so the switch must not veto it.
                    //
                    // SHIPPED 2026-09-07 (`DYNCHAR_DISSOLVE_NOSWITCH=0` restores the veto exactly):
                    // gated alone on its eight settled rows (luma sum -1.52 with 4 closer, chroma
                    // -1.25 with 5 closer, coverage 6 closer; Gavial E2 +13.62 -> +12.28) and
                    // bit-identical on all 23 entrance beats of Fugue, kalts and Skadi.
                    // A FOURTH SPELLING (was `DYNCHAR_DISSOLVE_NOSWITCH=1`, default OFF): a family
                    // with NO SWITCH AT ALL states its intent the same way the two-map materials
                    // do. `Torappu/Particles-L2D/Dissolve/Dissolve AB` declares exactly
                    // `_TintColor, _MainTex, _DissolveTex, _Amount, _BorderWidth, _ZTest`, and its
                    // fragment applies the mask UNCONDITIONALLY, with no branch to switch:
                    //
                    //     u_xlat16_0 = clamp((tex(_DissolveTex).x - _Amount) / _BorderWidth, 0, 1)
                    //     SV_Target0.w = clamp(u_xlat16_0 * u_xlat1.w, 0, 1)
                    //
                    // So reading `_ToggleUseDissolve` on it defaults to 0 and drops a live mask,
                    // the same shape as the `_UseDissolveTex` third spelling above and as the
                    // `_DisturbTex`/`_DisturTex` pair below.
                    //
                    // Kal'tsit's veil is the case that found it. Her scene layers 47 and 48 are GO
                    // `wenli` on this shader, they author `_Amount` 0.134 and `_BorderWidth`
                    // 0.569/0.453, and they carry NO `_ToggleUseDissolve`, `_UseDissolveTex` or
                    // `_DissolveIntensity` whatsoever. We therefore draw the veil as a full sheet:
                    // ablating `overlay` attributes +42.677 luma to it at her beat 3 against a
                    // total signed error of only +33.368, so the veil alone over-explains the beat.
                    //
                    // ⚠️ The third spelling's caveat does NOT apply here. There the switch alone
                    // was insufficient because 31 of 37 materials carried no `_Amount` and would
                    // have masked on a 0.5 default; `Dissolve AB` authors a real `_Amount`.
                    //
                    // Gated because the two neighbouring shader-correct dissolve changes both
                    // MEASURED WORSE (see the veto note above, and `DYNCHAR_TINT_DECL`): on this
                    // pipeline the GLSL is necessary evidence and never sufficient. Keyed on the
                    // shader positively DECLARING no switch, so `None` (no shader bundle staged)
                    // changes nothing and an under-staged export cannot silently flip families.
                    //
                    // ⚠️ NARROWED, and the wide form is why. Keyed on the SWITCH alone this admits
                    // 332 layers across 45 of 86 skins, and its biggest corpus movers are cel (52)
                    // and mly (14) — precisely the two references the veto note above records as
                    // MEASURED AND REFUTED (cel 15.342 -> 15.780, mly 17.360 -> 18.455). The wide
                    // form IS that refuted change, so it must not ship.
                    //
                    // The narrowing is the third spelling's own caveat, applied here: a mask is
                    // only portable when its THRESHOLD is authored rather than defaulted. So
                    // require the shader to declare the threshold it actually reads (`_Amount`,
                    // which `Dissolve AB` declares and `Dissolve(CustomData)` does not, spelling
                    // it `_DissolveIntensity`) AND the material to carry a value for it.
                    //
                    // That alone was NOT enough: it still admitted 276 layers across 43 skins with
                    // cel at 12 and mly at 14, and NEITHER of those skins has a single `Dissolve
                    // AB` layer, so it was still handing masks to the families the veto note
                    // measured as regressions. Scoped to the family the evidence actually covers,
                    // the same way this file already scopes `Ram/` and `Disturb Anchor`. Widening
                    // beyond it is a separate question that needs its own measurement.
                    let no_switch = std::env::var("DYNCHAR_DISSOLVE_NOSWITCH").as_deref()
                        != Ok("0")
                        && super::shader_map::shader_declares(shader, "_ToggleUseDissolve")
                            == Some(false)
                        && super::shader_map::shader_declares(shader, "_Amount") == Some(true)
                        && has_float_prop(mat, "_Amount")
                        && shader.contains("Dissolve AB");
                    let toggle_default = if shader.contains("Ram/") || two_map || no_switch {
                        1.0
                    } else {
                        0.0
                    };
                    // `DYNCHAR_DISSOLVE_GATE=1`: honour `_ToggleUseDissolve` only on the shaders
                    // that actually DECLARE it. Dumping the property block of every
                    // dissolve-capable `Particles*` shader shows just three do — the
                    // `Disturb Anchor` variants (declared default 0.0, which is what the
                    // fallback below already assumes). On the other 25 the fragment applies the
                    // mask UNCONDITIONALLY (no `if(_ToggleUseDissolve)` branch at all), so a
                    // material's copy of the property is inert residue and vetoing on it drops a
                    // real mask.
                    //
                    // Mlynar's two clouds are the case that found this: `bg02_yun_01` happens to
                    // carry `_ToggleUseDissolve = 1` and keeps its mask, while `bg02_yun_02` —
                    // same shader, same rig — carries no such property, defaults to 0 and is
                    // VETOED, so it draws as its full bounding rectangle instead of a carved
                    // cloud. The existing `Ram/ || two_map` default is a partial approximation
                    // of this rule; asking the shader subsumes it.
                    //
                    // ⛔ **MEASURED AND REFUTED — keep the veto.** The property-block reading is
                    // solid, and applying it is WORSE on two of three references:
                    //
                    //     mly 17.360 -> 18.455      cel 15.342 -> 15.780      ska unchanged
                    //
                    // The blast radius is large (every skin gains masks, several new atlas pages),
                    // so the newly-admitted masks must be wrong in some OTHER respect — a mask
                    // that resolves is not the same as a mask the runtime actually applies with
                    // our `_Amount`/`_BorderWidth`. This is the third shader-correct change this
                    // session to measure worse (see `main_color_doubles` and the L2D x2 gate):
                    // on this pipeline the GLSL is necessary evidence, never sufficient.
                    // THE THIRD SPELLING (`DYNCHAR_DISSOLVE_SPELLING`, default OFF; `=1` for
                    // every qualifying layer, or a comma-separated MATERIAL-name list to
                    // ablate). `Dissolve/Dissolve(CustomData)` calls the switch
                    // `_UseDissolveTex` and the threshold `_DissolveIntensity`; reading
                    // `_ToggleUseDissolve` on it defaults to 0 and drops a live mask. This is
                    // the same shape as the `_DisturbTex`/`_DisturTex` pair below.
                    //
                    // ⚠️ The switch alone is NOT enough. `amount` reads `_Amount` and falls back
                    // to 0.5, and 31 of the 37 affected materials carry no `_Amount` at all, so
                    // admitting them on the switch while masking on a 0.5 default would ship the
                    // right geometry with the wrong threshold. `dissolve_spelling` returns both
                    // names together for that reason.
                    let spelling_on = dissolve_spelling_only.as_ref().is_some_and(|only| {
                        only.is_empty()
                            || mat
                                .get("m_Name")
                                .and_then(Value::as_str)
                                .is_some_and(|n| only.iter().any(|m| m == n))
                    });
                    let (switch_prop, threshold_prop) = if spelling_on {
                        dissolve_spelling(mat, shader)
                    } else {
                        ("_ToggleUseDissolve", "_Amount")
                    };
                    let vetoed = if uvtween_scene {
                        // The program has no switch: `roundEven(_Amount + 0.5)` and the
                        // `_DissolveTex` lookup run on every fragment.
                        false
                    } else if std::env::var("DYNCHAR_DISSOLVE_GATE").is_ok() {
                        shader.contains("Disturb Anchor") && blend("_ToggleUseDissolve", 0.0) < 0.5
                    } else {
                        blend(switch_prop, toggle_default) < 0.5
                    };
                    if vetoed {
                        diss_pid = None;
                        diss_val = None;
                    }
                    // TWO SPELLINGS. The `Ram/` family binds `_DisturbTex`; the
                    // `Particles-L2D/Disturb/*` family spells the SAME slot `_DisturTex`
                    // (no `b`) while still using the `b` spelling for its speed floats — the
                    // mixed naming is in the shipped GLSL, not a transcription slip. Reading
                    // only the long name resolved nothing on that family, so a layer already
                    // admitted by its live dissolve drew with `uHasDisturb = 0` and no warp
                    // at all. Corpus-wide the short spelling is on 444 materials against the
                    // long one's 1881, so this is a second population rather than an edge case.
                    //
                    // ⚠️ This deliberately does NOT change ADMISSION — a disturb-only material
                    // still fails `admit` below. Porting disturb-only layers was measured worse
                    // (Virtuosa 35.322 -> 37.770); this only completes layers that are already in.
                    // `_RamTex` is only SAMPLED by the `Ram/` family. Of the 176 shaders in
                    // `[uc]shaders.ab`, exactly 9 declare the property and EVERY one has `Ram/`
                    // in its name (`Particles-L2D/Ram/*`, `Particles/Ram/*`, `UI/Ram/*`); its
                    // sibling `Particles-L2D/Disturb/Disturb(CustomData)` does not declare it at
                    // all. On any other family a material's `_RamTex` is RESIDUE from whatever
                    // shader it was previously authored against, and binding it multiplies the
                    // layer by a texture the program never reads — Civilight Eterna has 16 such
                    // `Disturb/` layers against 35 genuine `Ram/` ones.
                    //
                    // ⚠️ This is the trap in `dynchar-mlynar-left-half-displaced`: a material's
                    // PROPERTY SET does not identify its shader. Gate on `_shaderName`.
                    let (ram_pid, ram_val, ram_st) = if shader.contains("Ram/") {
                        super::particles::mat_texenv(all_objects, mat, "_RamTex")
                    } else {
                        (None, None, super::particles::ST_IDENTITY)
                    };
                    let (dist_pid, dist_val, dist_st) = {
                        let long = super::particles::mat_texenv(all_objects, mat, "_DisturbTex");
                        if long.0.is_some() && long.1.is_some() {
                            long
                        } else {
                            super::particles::mat_texenv(all_objects, mat, "_DisturTex")
                        }
                    };
                    let has = |p: Option<i64>, v: &Option<Value>| p.is_some() && v.is_some();
                    // What a newly admitted family must show is a LIVE DISSOLVE: the
                    // material binds `_DissolveTex` and leaves `_ToggleUseDissolve` on
                    // (the `diss_*` pair survived the switch above). That is the whole of
                    // the defect — a mask cutting the quad down to its real silhouette —
                    // and the disturb comes along because the shader warps the dissolve
                    // lookup with it (`_DisturbInfluenceDissolveUV`).
                    //
                    // A material binding ONLY `_DisturbTex`, dissolve switched off, is
                    // asking for a `_MainTex` UV WARP and no masking at all. That is a
                    // separate capability, and porting it MEASURED WORSE: it is every one
                    // of Virtuosa's 42 candidate layers (`_IntensityU` up to 0.10, dAlpha
                    // 1.0 throughout), and admitting them cost her 35.322 → 37.770 MADC
                    // while changing no coverage anywhere. `Ram/` keeps the either-map
                    // admission it shipped and was measured with.
                    // Disturb2, see `disturb2_on`: admitted on its disturb map alone, with both
                    // noise lookups derived from `_Noise1Param` / `_Noise2Param`.
                    let mat_keywords: String = mat
                        .get("m_ShaderKeywords")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| {
                            mat.get("m_ValidKeywords")
                                .and_then(Value::as_array)
                                .map(|a| {
                                    a.iter()
                                        .filter_map(Value::as_str)
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                })
                        })
                        .unwrap_or_default();
                    let disturb2_family = disturb2_on
                        && shader.contains("/Disturb/Disturb2")
                        && !mat_keywords
                            .split_whitespace()
                            .any(|k| k.starts_with("_DISTURBMODE_") && k != "_DISTURBMODE_DEFAULT");
                    let noise1 = super::particles::mat_color(mat, "_Noise1Param", [0.0; 4]);
                    let noise2 = super::particles::mat_color(mat, "_Noise2Param", [0.0; 4]);
                    let admit = if shader.contains("Ram/") {
                        has(diss_pid, &diss_val) || has(dist_pid, &dist_val)
                    } else {
                        has(diss_pid, &diss_val) || (disturb2_family && has(dist_pid, &dist_val))
                    };
                    // DIAGNOSTIC (SCENE_DEBUG=1): a `Dissolve/` material's mask lives under the
                    // TWO-MAP names (`_DissolveTex_01/_02`, `_Amount_01/_02`,
                    // `_BorderWidth_01/_02`); the single-name properties this code reads are
                    // inert residue on that family, so no `ram` block is emitted and the layer
                    // draws as its full bounding rectangle instead of the carved silhouette.
                    // Print the layers that actually REACH the export with a live two-map
                    // dissolve — the corpus scan counts materials, and most materials never
                    // reach a kept layer.
                    // CENSUS (`DYNCHAR_DISSOLVE_CENSUS=1`) for the THIRD switch spelling.
                    // `Dissolve/Dissolve(CustomData)` declares its dissolve switch
                    // `_UseDissolveTex` and its threshold `_DissolveIntensity`; its compiled
                    // GLSL names `_ToggleUseDissolve` and `_Amount` ZERO times, while
                    // `Disturb Anchor (AlphaBlend)` declares both of those. So the veto above
                    // reads a property this family does not have, defaults it to 0 and drops a
                    // live mask. Exactly parallel to the `_DisturbTex`/`_DisturTex` two
                    // spellings already handled below. Prints one line per layer that REACHES
                    // this gate so the population can be counted before anything is changed.
                    if std::env::var("DYNCHAR_DISSOLVE_CENSUS").is_ok() {
                        eprintln!(
                            "  [disscensus] admit={admit} two_map={two_map} toggle={:.3} useDiss={:.3} amount={:.3} dissInt={:.3} bw={:.3} mainUV=({:.3},{:.3}) dissTexBound={} shader={shader} mat={}",
                            blend("_ToggleUseDissolve", -1.0),
                            blend("_UseDissolveTex", -1.0),
                            blend("_Amount", -1.0),
                            blend("_DissolveIntensity", -1.0),
                            blend("_BorderWidth", -1.0),
                            blend("_MainUSpeed", 0.0),
                            blend("_MainVSpeed", 0.0),
                            super::particles::mat_texenv(all_objects, mat, "_DissolveTex")
                                .0
                                .is_some(),
                            mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                        );
                    }
                    if !admit && std::env::var("SCENE_DEBUG").is_ok() {
                        let f01 = blend("_Amount_01", -1.0);
                        let t01 = super::particles::mat_texenv(all_objects, mat, "_DissolveTex_01");
                        if f01 > 0.0 && t01.0.is_some() {
                            eprintln!(
                                "  TWOMAP-LOST '{}' amount_01={f01:.3} bw_01={:.3} amount_02={:.3}",
                                mat.get("m_Name").and_then(Value::as_str).unwrap_or("?"),
                                blend("_BorderWidth_01", -1.0),
                                blend("_Amount_02", -1.0)
                            );
                        }
                    }
                    if admit {
                        Some(SceneRam {
                            dissolve_pid: diss_pid,
                            dissolve_val: diss_val,
                            dissolve_st: diss_st,
                            disturb_pid: dist_pid,
                            disturb_val: dist_val,
                            disturb_st: if disturb2_family {
                                [noise1[0], noise1[0], 0.0, 0.0]
                            } else {
                                dist_st
                            },
                            ram_pid,
                            ram_val,
                            ram_st,
                            dissolve2_pid: diss2_pid,
                            dissolve2_val: diss2_val,
                            dissolve2_st: diss2_st,
                            weight_pid,
                            weight_val,
                            weight_st,
                            edge_color: shader.to_ascii_lowercase().contains("edge").then(|| {
                                let c = super::particles::mat_color(
                                    mat,
                                    "_Edgecolor",
                                    [1.0, 1.0, 1.0, 1.0],
                                );
                                [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32]
                            }),
                            edge_pow: blend("_pow", 1.0) as f32,
                            amount2: if two_map {
                                blend("_Amount_02", 0.0) as f32
                            } else {
                                0.0
                            },
                            border_width2: if two_map {
                                blend("_BorderWidth_02", 0.1) as f32
                            } else {
                                0.1
                            },
                            amount: if two_map {
                                blend("_Amount_01", 0.5)
                            } else {
                                blend(threshold_prop, 0.5)
                            } as f32,
                            border_width: if two_map {
                                blend("_BorderWidth_01", 0.1)
                            } else {
                                blend("_BorderWidth", 0.1)
                            } as f32,
                            uv_rot: uv_rotation_of_go(all_objects, go_pid),
                            anchor_u: blend("_AnchorU", 0.0) as f32,
                            anchor_v: blend("_AnchorV", 0.0) as f32,
                            intensity_u: if disturb2_family {
                                (noise1[2] * 0.1) as f32
                            } else {
                                blend("_IntensityU", 0.0) as f32
                            },
                            intensity_v: if disturb2_family {
                                (noise1[3] * 0.1) as f32
                            } else {
                                blend("_IntensityV", 0.0) as f32
                            },
                            disturb_influence_dissolve_uv: blend("_DisturbInfluenceDissolveUV", 0.0)
                                as f32,
                            disturb_influence_main_uv: blend("_DisturbInfluenceMainUV", 1.0) as f32,
                            // Mask pan. Which property carries it is a property of the
                            // SHADER, and the two families are mutually exclusive — verified
                            // by decompiling every `Particles*` shader in `[uc]shaders.ab`
                            // and listing its declared uniforms:
                            //
                            //   `*UVTween`, `Disturb Anchor*`  declare `_UVTween`, and NOT
                            //                                  `_Dissolve*Speed`/`_Disturb*Speed`
                            //   `*(CustomData)`, `Ram/*`       declare the Speed pairs, and
                            //                                  NOT `_UVTween`
                            //
                            // So on a UVTween-family material the Speed pair is INERT RESIDUE
                            // (the same trap as `_DstBlend`, see `particle-blend-shader-name`),
                            // and reading it pans the mask at the wrong rate. Their GLSL is
                            // explicit about what `_UVTween` means:
                            //
                            //   vs_TEXCOORD1 = _Time.y * _UVTween;
                            //   u_xlat1 = vs_TEXCOORD0.zwxy + vs_TEXCOORD1.zwxy;
                            //     -> dissolveUV += t * _UVTween.zw ;  mainUV += t * _UVTween.xy
                            //
                            // Mlynar's clouds (`bg02_yun_01/02`, Disturb Anchor) are the case
                            // that found this: authored `_UVTween` z/w = 0.200/-0.050 and
                            // 0.300/-0.050 while their residual `_DissolveUSpeed` reads
                            // 0.120 / 0.200 — so we evolved their dissolve mask ~1.5-1.7x too
                            // slowly and their silhouette drifted out of step with the game.
                            dissolve_speed: {
                                let base = if uvtween_family {
                                    [uvtween[2], uvtween[3]]
                                } else {
                                    [
                                        blend("_DissolveUSpeed", 0.0) as f32,
                                        blend("_DissolveVSpeed", 0.0) as f32,
                                    ]
                                };
                                let c = uv_scroll_component(all_objects, go_pid);
                                [base[0] + c.dissolve[0], base[1] + c.dissolve[1]]
                            },
                            disturb_speed: if disturb2_family {
                                [(noise1[1] / 20.0) as f32, 0.0]
                            } else {
                                let c = uv_scroll_component(all_objects, go_pid);
                                [
                                    blend("_DisturbUSpeed", 0.0) as f32 + c.disturb[0],
                                    blend("_DisturbVSpeed", 0.0) as f32 + c.disturb[1],
                                ]
                            },
                            disturb2: disturb2_family.then(|| {
                                [
                                    noise2[0] as f32,
                                    (noise2[1] / 20.0) as f32,
                                    (noise2[2] * 0.1) as f32,
                                    (noise2[3] * 0.1) as f32,
                                ]
                            }),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            resolved_meshext = mat.get("_meshExtResolved").is_some();
            anchor_ctrl_v = anchor_ctrl(mat);
            anchor_a_v = mat
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_Colors"))
                .and_then(|c| c.get("_MainColor"))
                .and_then(|c| c.get("a"))
                .and_then(Value::as_f64)
                .unwrap_or(1.0) as f32;
            resolved = Some((
                tex_val,
                alpha_val,
                tint,
                is_additive(mat),
                main_pid,
                blend("_SrcBlend", 5.0),
                blend("_DstBlend", 10.0),
                st,
                cprops,
                tint_scale,
                hdr_color,
                uv_scroll,
                ram,
                (erase_masks_on
                    && mat
                        .get("_shaderName")
                        .and_then(Value::as_str)
                        .is_some_and(|sh| sh.ends_with("/Mask/Erase")))
                .then(|| blend("_Strength", 1.0) as f32),
            ));
            break;
        }
        let Some((
            tex_val,
            alpha_val,
            tint,
            additive,
            main_pid,
            src_blend,
            dst_blend,
            st,
            (color_props, tint_prop),
            tint_scale,
            hdr_color,
            uv_scroll,
            ram,
            erase,
        )) = resolved
        else {
            continue;
        };
        // The layer's animated material colour (entrance scenes only), matched against
        // the material's own colour properties and resolved onto the static tint.
        // DIAGNOSTIC (`DYNCHAR_COLORDBG=1`): why does a layer end up with no animated colour?
        // Prints what the extractor found for this GO against the material colour props the
        // matcher will test it against — the two halves of `layer_color_curve`'s only filter.
        if std::env::var("DYNCHAR_COLORDBG").is_ok() {
            let chs = color_channels.get(&go_pid);
            eprintln!(
                "  [colordbg] go={go_pid} channels={} props=[{}] tint_prop={:?}",
                chs.map_or("NONE".to_string(), |c| c
                    .iter()
                    .map(|x| format!("crc28=0x{:07x}/ch{}", x.prop_crc28, x.channel))
                    .collect::<Vec<_>>()
                    .join(" ")),
                color_props
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                tint_prop,
            );
        }
        let color_curve = color_channels.get(&go_pid).and_then(|chs| {
            super::anim::layer_color_curve(
                chs,
                &color_props,
                tint_prop.as_deref(),
                tint,
                tint_scale,
                hdr_color,
                additive,
            )
        });
        // The Anchor `k` (`DYNCHAR_ANCHOR_K=1`, see `anchor_ctrl`): all four channels scaled by
        // `ctrl * (a - 1) + 1`, on the static tint and on every curve key. `a` is the RAW
        // `_MainColor.a`: the material's value, or the clip's own alpha track sampled at the
        // key's time when the clip animates it. Never the resolved tint's alpha, which the
        // x2 path has already scaled and clamped.
        let (tint, color_curve) = if anchor_ctrl_v == 0.0 {
            (tint, color_curve)
        } else {
            let alpha_track = color_channels
                .get(&go_pid)
                .and_then(|chs| super::anim::prop_channel(chs, "_MainColor", 3));
            let raw_alpha_at = |t: f32| -> f32 {
                let Some(track) = alpha_track else {
                    return anchor_a_v;
                };
                let curve = &track.curve;
                match curve.iter().position(|&(kt, _)| kt >= t) {
                    None => curve.last().map_or(anchor_a_v, |&(_, v)| v),
                    Some(0) => curve[0].1,
                    Some(i) => {
                        let (t0, v0) = curve[i - 1];
                        let (t1, v1) = curve[i];
                        if t1 > t0 {
                            v0 + (v1 - v0) * (t - t0) / (t1 - t0)
                        } else {
                            v1
                        }
                    }
                }
            };
            let scale = |v: [f32; 4], a: f32| {
                let k = anchor_k(anchor_ctrl_v, a);
                [v[0] * k, v[1] * k, v[2] * k, v[3] * k]
            };
            (
                scale(tint, anchor_a_v),
                color_curve.map(|c| {
                    c.into_iter()
                        .map(|(t, v)| (t, scale(v, raw_alpha_at(t))))
                        .collect()
                }),
            )
        };
        // The layer's animated `_MainTex_ST` curve (entrance scenes only). When present it
        // supersedes the static ST bake below (the curve carries the full ST).
        let st_curve = st_channels.get(&go_pid).and_then(|chs| {
            super::anim::layer_st_curve(
                chs,
                [st[0] as f32, st[1] as f32, st[2] as f32, st[3] as f32],
            )
        });

        // A GO that reached here ONLY because of the color-reveal exception — i.e. it
        // has no `m_IsActive` window and would otherwise have been dropped, either as
        // inactive (`!eff_active`) or as an unwindowed external-texture quad
        // (`resolved_meshext`) — must actually export a reveal `color_curve` that STARTS
        // HIDDEN (alpha ≈ 0 at t0). Otherwise a hash-collision alpha channel that didn't
        // match this material (yielding no `color_curve`) would render as an ungated
        // always-on layer. Layers kept for any OTHER reason (a real window, or a normal
        // in-bundle statically-active layer) are untouched — this only re-checks the two
        // paths the color-reveal exception newly opened.
        let admitted_by_reveal =
            has_color_reveal && window.is_empty() && (!eff_active || resolved_meshext);
        if admitted_by_reveal {
            let reveals = color_curve
                .as_ref()
                .is_some_and(|c| c.first().is_some_and(|&(_, rgba)| rgba[3].abs() < 0.02));
            if !reveals {
                if attrib_dbg {
                    // Previously SILENT. A layer the colour-reveal exception admitted, then
                    // rejected because no reveal curve resolved, vanishes with no trace — which
                    // is indistinguishable from "the prefab never had it".
                    eprintln!(
                        "    [scene] DROP reveal-unproven {:<22} curve={} first_a={:?}",
                        host.go_name(all_objects, go_pid),
                        color_curve.as_ref().map_or(0, Vec::len),
                        color_curve
                            .as_ref()
                            .and_then(|c| c.first().map(|&(_, rgba)| rgba[3])),
                    );
                }
                skipped_inactive += 1;
                continue;
            }
        }

        // Geometry from the GameObject's MeshFilter. An in-bundle Mesh (class
        // 43) is parsed; a null (`m_Mesh == 0`) or external/built-in reference
        // (e.g. Unity's Quad primitive, path_id 10210, in default resources)
        // falls back to the unit quad — dynchar scene layers are flat quads.
        //
        // NOTE (2026-08-01): skipping the NULL case instead — on the theory that a null
        // `m_Mesh` renders nothing in Unity, so substituting a quad paints a rectangle the
        // game never draws — was built, exported and MEASURED. It is a NO-OP here: Ch'en's
        // scene came out byte-identical at 26 layers, because the one null-mesh object
        // ("wave") is already excluded by a later gate. Her rectangular sky is NOT this: her
        // bundle has ZERO external mesh refs and genuine 4-vertex meshes in-bundle, so that
        // quad is authored geometry, and her only `_AlphaTex` belongs to the spine atlas, not
        // a scene quad. Do not re-attempt without a skin where the null case actually reaches
        // the output.
        let mut mesh = match go_to_mesh.get(&go_pid).copied() {
            Some(mp) if mp != 0 => match all_objects.get(&mp) {
                Some((43, mesh_val)) => {
                    if let Some(m) = super::mesh::parse_mesh(mesh_val, resources) {
                        m
                    } else {
                        if attrib_dbg {
                            // Previously SILENT. An in-bundle Mesh that fails to parse takes its
                            // layer with it and leaves no trace, which reads as "the prefab never
                            // had it" — the hardest kind of gap to notice.
                            // ⚠️ Print the ROOT. SCENE_ATTRIB output is INTERLEAVED across
                            // bundles, so a line without one cannot be attributed to a skin —
                            // neighbouring lines routinely belong to five different skins.
                            eprintln!(
                                "    [scene] DROP mesh-parse-failed {:<20} mesh_pid={mp} root={}",
                                host.go_name(all_objects, go_pid),
                                own_root.map_or_else(
                                    || "?".to_string(),
                                    |r| host.go_name(all_objects, r)
                                ),
                            );
                        }
                        continue;
                    }
                }
                _ => super::mesh::unit_quad(), // built-in / external quad
            },
            _ => super::mesh::unit_quad(),
        };

        // Bake the material's _MainTex Scale/Offset into the UVs (Unity UV
        // space, pre-flip — the frontend flips V). Layers that reference a
        // sub-rect of an atlas (e.g. Mlynar's bg_02: city / clouds / rainbow
        // packed in one texture) would otherwise show the whole atlas squashed.
        if st_curve.is_none() && st != super::particles::ST_IDENTITY {
            for uv in &mut mesh.uvs {
                uv[0] = uv[0] * st[0] as f32 + st[2] as f32;
                uv[1] = uv[1] * st[1] as f32 + st[3] as f32;
            }
        }

        // Full world transform relative to the spine root, with idle-pose
        // overrides applied to any animated transforms in the chain.
        let world = go_to_transform
            .get(&go_pid)
            .map_or_else(super::mesh::Mat4::identity, |&tf| {
                accumulate_matrix(all_objects, tf, &spine_gos, &idle_pose)
            });
        let z = world.point([0.0, 0.0, 0.0])[2];

        // spine-unity `BoneFollower` in the ancestry: at runtime the follower SNAPS its
        // GameObject onto the named spine bone, so everything below it (this quad) rides
        // the bone and the transform baked into `world` above is only an editor pose.
        // Mlynar "Fields of Ruination" is the proof: his sword flare (`glow_01` under
        // `..._Start_Mlynar_L_Sword2(Clone)`, `followBoneRotation` on) exports as a
        // horizontal streak parked in the lower-left instead of a warm halo running along
        // the blade. The particle exporter has honoured this since Virtuosa's falling
        // apple; scene MESH quads never did. Capture the follower's world frame so the
        // frontend can replay `bone(t) · followerWorld⁻¹` against the baked geometry.
        let follow = host
            .follower_of_go(all_objects, go_pid)
            .map(|(bone, rot, follower_go)| {
                let fw = host.world_of_go(all_objects, follower_go);
                let o = fw.point([0.0, 0.0, 0.0]);
                BgFollow {
                    bone,
                    rot,
                    origin: [o[0], o[1]],
                    basis: [fw.0[0][0], fw.0[0][1], fw.0[1][0], fw.0[1][1]],
                }
            });

        if attrib_dbg {
            eprintln!(
                "    [scene] KEEP  sort={sort:<4} z={z:<10.4} reveal={:<6} {:<28} root={}",
                cross_from.map_or_else(|| "-".to_string(), |t| format!("{t:.2}")),
                host.go_name(all_objects, go_pid),
                host.root_name_of_go(all_objects, go_pid)
            );
        }
        // ENTRANCE scale animation for THIS quad's own transform, normalised against the prefab
        // pose the mesh above was baked at, so the renderer applies a pure multiplier and a skin
        // with no animation is bit-identical. Dividing by the prefab scale is what makes it a
        // multiplier: the curve is absolute local scale, and `world` already carries the prefab
        // value, so emitting the raw curve would double-apply it.
        // Walk SELF-THEN-ANCESTORS, exactly as `reveal_of_go` does for the visibility window: a
        // parent's scale animation carries its whole subtree, and the animated transform is
        // routinely NOT the one holding the mesh. Executor's scope is the case — the annulus quad
        // belongs to `zhunx_02` (the crosshair) while the scale curve sits on its parent
        // `heip_01 (1)`, so a self-only lookup finds nothing and the aperture stays frozen.
        let xform_owner = {
            let mut cur = Some(go_pid);
            let mut found = None;
            for _ in 0..256 {
                let Some(g) = cur else { break };
                if xform_map.contains_key(&g) {
                    found = Some(g);
                    break;
                }
                let Some(tf) = go_to_transform.get(&g) else {
                    break;
                };
                let father = all_objects
                    .get(tf)
                    .and_then(|(_, v)| v.get("m_Father"))
                    .and_then(get_path_id)
                    .filter(|&p| p != 0);
                let Some(father) = father else { break };
                cur = all_objects
                    .get(&father)
                    .and_then(|(_, v)| v.get("m_GameObject"))
                    .and_then(get_path_id);
            }
            found
        };
        let scale_curve = xform_owner
            .and_then(|owner| xform_map.get(&owner))
            .and_then(|et| {
                if et.scale.len() < 2 {
                    return None;
                }
                // Normalise against the OWNER's prefab scale — the transform the curve belongs to,
                // not the quad's own, or the multiplier is divided by the wrong number. EACH AXIS
                // against ITS OWN component: dividing y by `m_LocalScale.x` was the second half of
                // the uniform-scale defect, and it silently skews any non-square host.
                let local = xform_owner
                    .and_then(|owner| go_to_transform.get(&owner))
                    .and_then(|tf| all_objects.get(tf))
                    .and_then(|(_, tv)| tv.get("m_LocalScale"));
                let axis = |k: &str| {
                    local
                        .and_then(|s| s.get(k))
                        .and_then(Value::as_f64)
                        .unwrap_or(1.0) as f32
                };
                let (bx, by) = (axis("x"), axis("y"));
                if bx.abs() < 1e-6 || by.abs() < 1e-6 {
                    return None;
                }
                // Pair y by index, which is safe because both come from the SAME binding and so
                // share their key times. A clip that binds only x leaves `scale_y` empty; there the
                // host really is uniform and the x multiplier stands in for both.
                let uniform = et.scale_y.len() != et.scale.len();
                let c: Vec<(f32, f32, f32)> = et
                    .scale
                    .iter()
                    .enumerate()
                    .map(|(i, &(t, v))| {
                        let sx = v / bx;
                        (t, sx, if uniform { sx } else { et.scale_y[i].1 / by })
                    })
                    .collect();
                // All-1.0 curves carry no information and would only bloat every scene JSON. BOTH
                // axes have to be flat before a curve is discarded.
                c.iter()
                    .any(|&(_, x, y)| (x - 1.0).abs() > 1e-3 || (y - 1.0).abs() > 1e-3)
                    .then_some(c)
            });
        let scale_pivot = scale_curve.as_ref().and_then(|_| {
            let owner = xform_owner?;
            let tf = *go_to_transform.get(&owner)?;
            // Match particles.rs's pivot: scaling a baked mesh must stay fixed at the animated
            // owner's world origin, not at this child quad's origin or its father's.
            let p =
                accumulate_matrix(all_objects, tf, &spine_gos, &idle_pose).point([0.0, 0.0, 0.0]);
            // The particle precedent emits authored px, so carry this scene pivot through the
            // same skeleton-scale conversion used by the layer geometry and position curve.
            Some([
                (f64::from(p[0]) * inv_scale) as f32,
                (f64::from(p[1]) * inv_scale) as f32,
            ])
        });
        // The OWNER's animated POSITION, resolved into the same space as `pos`. The curve is
        // authored in the owner's PARENT frame, so a parent-frame delta has to be carried
        // through the parent's world LINEAR part (rotation/scale) before it means anything in
        // authored px — taking the raw local delta would be right only for an unrotated,
        // unscaled ancestry. Emitted as a delta, not an absolute, so the baked `pos` stays the
        // single source of the layer's rest pose.
        let pos_curve = xform_owner
            .and_then(|owner| xform_map.get(&owner).map(|et| (owner, et)))
            .and_then(|(owner, et)| {
                if et.pos_x.len() < 2 && et.pos_y.len() < 2 {
                    return None;
                }
                let tf = *go_to_transform.get(&owner)?;
                let tv = &all_objects.get(&tf)?.1;
                let lp = tv.get("m_LocalPosition")?;
                let p0x = lp.get("x").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let p0y = lp.get("y").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let father = tv.get("m_Father").and_then(get_path_id).filter(|&p| p != 0);
                let pw = father.map_or_else(super::mesh::Mat4::identity, |f| {
                    accumulate_matrix(all_objects, f, &spine_gos, &idle_pose)
                });
                let o = pw.point([0.0, 0.0, 0.0]);
                let mut ts: Vec<f32> = et
                    .pos_x
                    .iter()
                    .chain(et.pos_y.iter())
                    .map(|&(t, _)| t)
                    .collect();
                ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                ts.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
                let c: Vec<(f32, f32, f32)> = ts
                    .iter()
                    .map(|&t| {
                        let dx = sample_kf(&et.pos_x, t, p0x) - p0x;
                        let dy = sample_kf(&et.pos_y, t, p0y) - p0y;
                        let w = pw.point([dx, dy, 0.0]);
                        (t, w[0] - o[0], w[1] - o[1])
                    })
                    .collect();
                c.iter()
                    .any(|&(_, dx, dy)| dx.abs() > 1e-3 || dy.abs() > 1e-3)
                    .then_some(c)
            });
        // DIAGNOSTIC (`DYNCHAR_XFORM_DEBUG=1`): which GO owns each quad, and did it find a
        // curve? The curve map is keyed by the clip's DISAMBIGUATED ctrl, which need not be the
        // clone the quad was collected from — that mismatch is why Executor's scope rim exports
        // without its scale animation while `band_01` gets one.
        if std::env::var("DYNCHAR_XFORM_DEBUG").is_ok() {
            eprintln!(
                "  [xform] go={go_pid} name={:?} root={:?} curve={} sort={sort}",
                host.go_name(all_objects, go_pid),
                host.root_name_of_go(all_objects, go_pid),
                scale_curve.as_ref().map_or(0, Vec::len),
            );
        }
        // EFFECT-ROOT DELAY (`DYNCHAR_SCENE_DELAY=0` reverts, `=2` is a diagnostic). A scene
        // layer under a director
        // `_effects[]` sub-prefab that carries a `_delayTime` activator plays its clip in that
        // sub-prefab's LOCAL time: the game enables the root at `_delayTime` and the clip's
        // windows and curves run from there. The particle path already compounds this through
        // `delay_of_go`; the scene path exported clip-local times and drew every such layer early.
        //
        // Kal'tsit is the case that found it. `wenl1`/`wenli` (her veil pair, scene 47/48) sit
        // under `Kalts_boc#6_Start_05`, delayed 6.20 s: the game's frame is a full white-out at
        // scene 9.9 (luma 251.4, sd 1.3), exactly `wenl1`'s curve peak 3.7 + 6.2, while we drew the
        // pulse at 3.7 and measured the game flat there. Her shards `st`/`st2`/`st3` (44/43/45)
        // sit under `Start_04`, delayed 4.50 s, and were drawn full-size from 1.07 s where the
        // game shows them only from 5.57 s. A layer with no window of its own under a delayed root
        // starts at the delay.
        //
        // MEASURED 2026-08-31 on the five skins whose `_Start` scene it changes (the other nine
        // corpus keys are bit-identical by construction): kalts 31.357 -> 30.339 with r .592 ->
        // .650, fugue 9.759 -> 9.567 with r up, mue +0.035, exc/excunew bit-identical, but
        // whitw2 32.296 -> 35.847 with r .720 -> .729: her six moved layers (`yan`, `music_02`
        // and siblings under `start_langawei_Wolf_B_All` at 12.30 s and the two 13.00 s wolf
        // roots) improve every beat through 12.0 and then blow beat 13.5 from 21.593 to 51.269
        // (DC -16.8 -> +50.9). Attributed 2026-08-31: NONE of the six carries it. The delay
        // ADMITS a 28th layer, `baizhuanchang_02` under `start_04(Clone)` (`_delayTime` 13.60),
        // a full-frame white sheet whose alpha curve runs 0 -> 1 over 13.60..14.43, previously
        // dropped as never-active. The game whites out on that schedule (101 at scene 13.75 ->
        // 253 by 14.65, cut at 15.05) and the ON arm reproduces the ramp within ~0.15 s; beat
        // 13.5 (scene 14.05) samples the middle of that ramp, where the phase costs ~40 luma,
        // and the director's own fade sprite still plays on top (+15..+25) because
        // `sceneDrivesEntranceFade` does not admit this sheet. Both are residuals of a layer the
        // game draws, not of the clock. Ships ON: the schedule is authored, r improves on every
        // moved skin, and OFF leaves kalts's veil six seconds early.
        // `DYNCHAR_SCENE_DELAY=2`: WINDOWS ONLY on the root's clock, curves left clip-local.
        // Measured WORSE (whitw2 43.523, fugue 30.305 with her sort-100 sheet's curve left
        // clip-local), so the curves belong on the clock too. Kept as the diagnostic.
        let (window, scale_curve, pos_curve, color_curve) = {
            let mode = std::env::var("DYNCHAR_SCENE_DELAY").unwrap_or_else(|_| "1".to_string());
            let dly = if is_entrance && (mode == "1" || mode == "2") {
                host.delay_of_go(all_objects, go_pid)
            } else {
                0.0
            };
            if dly > 1e-6 {
                let d = dly as f32;
                let curves_too = mode == "1";
                let w: super::anim::ActiveWindowList = if window.is_empty() {
                    vec![(Some(d), None)]
                } else {
                    window
                        .iter()
                        .map(|&(a, b)| (Some(a.map_or(d, |t| t + d)), b.map(|t| t + d)))
                        .collect()
                };
                let sh = |t: f32| if curves_too { t + d } else { t };
                (
                    w,
                    scale_curve.map(|c| c.into_iter().map(|(t, x, y)| (sh(t), x, y)).collect()),
                    pos_curve.map(|c| c.into_iter().map(|(t, x, y)| (sh(t), x, y)).collect()),
                    color_curve.map(|c| c.into_iter().map(|(t, v)| (sh(t), v)).collect()),
                )
            } else {
                (window, scale_curve, pos_curve, color_curve)
            }
        };
        quads.push(BgQuad {
            mesh,
            tex_val,
            alpha_val,
            tint,
            additive,
            world,
            sort,
            z,
            tex_pid: main_pid,
            st,
            src_blend,
            dst_blend,
            erase,
            scale_curve,
            scale_pivot,
            pos_curve,
            active_from: window.first().and_then(|w| w.0),
            active_until: window.first().and_then(|w| w.1),
            active_windows: if window.len() > 1 {
                window.clone()
            } else {
                Vec::new()
            },
            root_reveal_from: cross_from,
            color_curve,
            uv_scroll,
            st_curve,
            ram,
            follow,
            go_pid,
            go_name: host.go_name(all_objects, go_pid),
            cam_locked: cam_locked_gos.contains(&go_pid),
            cam_lock_view: cam_lock_view.get(&go_pid).copied(),
        });
    }

    if skipped_inactive > 0 {
        eprintln!(
            "  scene: skipped {skipped_inactive} mesh renderer(s) under state-gated inactive groups"
        );
    }
    let mut scene = BgScene {
        camera_size,
        max_aspect,
        max_size,
        camera_offset,
        camera_view,
        camera_offset2,
        camera_view2,
        separator_slots,
        separator_part_sorts,
        ..BgScene::default()
    };
    if quads.is_empty() {
        return scene;
    }
    for q in &quads {
        claimed_tex.push(q.tex_pid);
    }
    scene.quads = quads;
    scene.claimed_tex = claimed_tex;
    scene.char_sort = Some(spine_sort);
    scene
}

/// Shader-family tint multiplier for a scene-layer material. Torappu's ports of
/// Unity's LEGACY particle shaders (`Torappu/Particles/AlphaBlend`, `…/Additive`, … —
/// the plain `…/Particles/…` namespace, NOT the `Particles-L2D`/Ram compositors)
/// sample `2 × _TintColor × tex`, so authors key `_TintColor` rgb 0.5 as neutral
/// white and the doubling applies to ALPHA too. Mlynar "Fields of Ruination" proves
/// the convention in data: its white-flash plane (`Torappu/Particles/AlphaBlend`)
/// keys `_TintColor` rgb to a CONSTANT 0.5 while ramping `.a` to 0.671 — in-game a
/// pure-white FULL white-out (2 × 0.671 clamps to 1), where a plain multiply reads
/// half-grey. (The Ram GLSL port in particles.ts mirrors the same ×2 as `col += col`.)
/// Returns 2.0 for that family when the material carries `_TintColor`, else 1.0.
fn legacy_tint_scale(mat: &Value, animated_color: bool) -> (f32, bool) {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // `Particles-L2D` is the L2D port of the same legacy shaders and shares the
    // convention: across the dynchar corpus 244 of 283 `Particles-L2D/AlphaBlend`
    // materials author `_TintColor` rgb at exactly 0.5 — meaningless as a literal
    // 50% grey veil, canonical as ×2 neutral white — matching the proven
    // `Particles/AlphaBlend` family (16 of 17 at 0.5). Match only the PLAIN blend
    // modes (`…/Particles-L2D/<Mode>` with nothing below it): the sub-namespaced
    // families (`Ram/`, `Disturb/`, `Dissolve/`, `Mask/`) composite extra maps and
    // are handled separately (`ram_tint_scale`). Purely additive — no shader the
    // original test matched stops matching.
    let l2d_rest = shader
        .rfind("Particles-L2D/")
        .map(|i| &shader[i + "Particles-L2D/".len()..]);
    let plain_l2d = l2d_rest.is_some_and(|rest| !rest.is_empty() && !rest.contains('/'));
    // SUB-NAMESPACED L2D families (`Dissolve/`, `Disturb/`, `Mask/`, `3D/`) carry the SAME
    // convention, and the exclusion above was an accident of the name test rather than a
    // finding. Two independent lines of evidence:
    //
    //  1. The PROGRAM. `Torappu/Particles-L2D/Dissolve/Dissolve Add UVTween`, decompressed out
    //     of `[uc]shaders.ab`, builds `vs_COLOR0 = in_COLOR0 * _TintColor` in the vertex stage
    //     and then `u_xlat1 = vs_COLOR0 + vs_COLOR0; SV_Target0.xyz = (tex * u_xlat1).xyz` in
    //     the fragment — a literal ×2 on `_TintColor`, identical to the plain ports. Every
    //     Torappu particle shader read so far (AlphaBlend, Additive, Dissolve, Dissolve Add
    //     UVTween, Disturb, Disturb(CustomData), Ram/Disturb) doubles the same way.
    //  2. The NON-L2D TWIN ALREADY MATCHES. `Torappu/Particles/Dissolve/*` and
    //     `Torappu/Particles/Disturb/*` are admitted today by the `/Particles/` test above —
    //     the very same sub-namespaces, doubled. Only the L2D port was excluded, purely
    //     because its name has a slash below `Particles-L2D/`.
    //
    // `Ram/` stays out: `ram_tint_scale` handles that family through `_MainColor`, and letting
    // it match here would double it twice (this branch is tested first).
    //
    // Gated on `_TintColor` being the modulator this shader ACTUALLY reads, evidenced by the
    // material carrying no `_MainColor`. In these families the two are mutually exclusive by
    // construction: `Dissolve Add UVTween` declares only `_TintColor` (its vertex stage is
    // `vs_COLOR0 = in_COLOR0 * _TintColor`) and its materials carry no `_MainColor` at all,
    // while `Disturb(CustomData)` modulates by `_MainColor` (`2 * tex * _MainColor *
    // vs_COLOR0`) and leaves `_TintColor` as inert residue — pinned at exactly 0.500 on
    // material after material of Mlynar's backdrop (`bg_02`, `bg_03`, `bg_04`, `yun_01`,
    // `bg_tree_01..03`, …), the signature of a never-authored default. Doubling that residue
    // would be meaningless, and worse, this branch is tested BEFORE the `_MainColor` ones, so
    // matching it would steal those layers away from the tint they are actually drawn with.
    //
    // Deliberately NOT gated on the half-neutral bound `ram_tint_scale` applies. That bound
    // exists to avoid CLAMPING damage — "a ×2 material cannot author above its neutral because
    // doubling would blow past white" only holds where white is a ceiling. This branch emits
    // HDR, so it is not: Mlynar's blade is authored `_TintColor` (1.000, 0.753, 0.489), which
    // doubles to (2.0, 1.51, 0.98) — an over-bright amber that saturates white-hot at the core
    // and bleeds gold at the edges, which is precisely the flare the game draws. The value
    // being above the neutral is the AUTHOR ASKING for over-bright, not evidence against the
    // convention.
    //
    // 🔑 THE EVIDENCE IS THE SHADER'S DECLARATION, NOT THE MATERIAL'S SLOT LIST.
    // The guard below originally read `!has_color_prop(mat, "_MainColor")` — "the
    // material carries no `_MainColor`, so the shader must modulate by `_TintColor`".
    // That inference is invalid in the other direction: Unity keeps every property a
    // material ever carried, so a `_MainColor` left over from the shader the asset was
    // authored against survives as residue on a shader that cannot read it. Measured
    // over all 87 dynchar bundles, **630 materials carry a `_MainColor` their shader
    // never declares** — and because this branch is tested BEFORE the `_MainColor`
    // ones, every one of them was being handed to the wrong tint.
    //
    // Kal'tsit is the clean case: `wenli` and `wenl1` are the SAME quad drawn twice
    // with the SAME authored colours (`_TintColor` 0.5/0.484, `_MainColor` 0.3456),
    // differing only in shader. `Particles-L2D/AlphaBlend` took `_TintColor`x2 =
    // (1,1,1,0.968) while `Dissolve/Dissolve AB` was pushed onto `_MainColor`x2 =
    // (0.691,0.691,0.691,1.0) — two different tints for one sheet. `Dissolve AB`'s
    // whole declared block is `_TintColor, _MainTex, _DissolveTex, _Amount,
    // _BorderWidth, _ZTest`, with `_TintColor` DEFAULTING to (0.5,0.5,0.5,0.5) — the
    // x2 neutral, from the shader's own mouth. `Disturb(CustomData)` in the same
    // bundle does declare `_MainColor`, which is the positive control proving the
    // scan finds the property when it is really there.
    //
    // ⛔ PARKED (`DYNCHAR_TINT_DECL=1` enables) — CORRECT AND MEASURED WORSE.
    // Switching the guard to the declaration costs **+7.45 on Kal'tsit** (48.614 ->
    // 56.053) and is corpus-neutral to 3dp; clamping the newly-admitted families
    // instead of giving them the HDR ceiling does not help (56.067), so the damage is
    // the tint SOURCE, not the ceiling.
    //
    // The reading itself is not in doubt. A raw-byte scan of every `Particles-L2D`
    // shader shows `_MainColor` and `_TintColor` are strictly MUTUALLY EXCLUSIVE —
    // `Dissolve AB` contains the string `_MainColor` zero times — so the residue really
    // is unreadable by the shader that draws it.
    //
    // The best explanation for the conflict is that on these materials `_TintColor` is
    // the STALE half: `kalts_wenli` carries `_TintColor` (0.5,0.5,0.5,0.484), which is
    // the shader's own declared DEFAULT (0.5,0.5,0.5,0.5), against a hand-authored
    // `_MainColor` (0.3456,0.3456,0.3456,0.509). That is the signature of a material
    // authored against a `_MainColor` shader and later re-pointed at this one, leaving
    // the value the artist chose in a slot the new shader cannot read. Unity would draw
    // the default; our renderer scores much better drawing what was authored. Until
    // that is resolved against a capture, the measured behaviour wins.
    //
    // Falls back to the old material test when the shader is unknown (no shader
    // bundle staged), so an under-staged export cannot silently flip families.
    let reads_main_color = if std::env::var("DYNCHAR_TINT_DECL").is_ok() {
        super::shader_map::shader_declares(shader, "_MainColor")
            .unwrap_or_else(|| has_color_prop(mat, "_MainColor"))
    } else {
        has_color_prop(mat, "_MainColor")
    };
    let sub_l2d = l2d_rest.is_some_and(|rest| rest.contains('/') && !rest.starts_with("Ram/"))
        && !reads_main_color;
    // STATIC plain-L2D layers are admitted too (`DYNCHAR_STATIC_L2D_TINT=0` reverts).
    // The animated-only caution above dated from when the idle surface had no recording to
    // verify against. It now does: Executor's idle `glow` layers (`Particles-L2D/Additive`,
    // static, `_TintColor` (0.368,0.314,0.257, a 0.378) and (0.360,0.629,1.0, a 0.309))
    // exported tint white 1.0 under the old rule and rendered a blown white disc over the
    // face, 55% of the authored eye boxes above luma 250 where the 60 fps capture shows 0%
    // and eye mean 225.4 against our 237.8. The family convention is the same one already
    // proven for animated layers and for every Torappu particle program read so far
    // (vs_COLOR0 = in_COLOR0 * _TintColor; col = vs_COLOR0 + vs_COLOR0).
    let static_l2d_tint = std::env::var("DYNCHAR_STATIC_L2D_TINT").as_deref() != Ok("0");
    let legacy = shader.contains("/Particles/")
        || shader.starts_with("Particles/")
        || ((animated_color || static_l2d_tint) && plain_l2d)
        || sub_l2d;
    let has_tint_color = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .is_some_and(|c| c.contains_key("_TintColor"));
    // HDR only for the newly-admitted sub-namespaced families. Their gate PROVED the authored
    // tint sits at or below the ×2 neutral, so the doubled colour is a real over-bright value
    // the frontend's half-float scene target can carry — and on an animated layer that is the
    // whole point: Mlynar's blade ramps `_TintColor` r to 1.0, which doubles to 2.0 and blooms
    // past `BLOOM_THRESHOLD`, drawing the broad white flare the game shows and we do not.
    // Clamping it to 1.0 collapses that flare onto flat white with no bloom headroom, the same
    // baseline-and-peak-share-a-ceiling failure documented on the Ram branch. The families
    // matched BEFORE this change keep their clamped behaviour untouched: they are shipped and
    // measured, and widening them is a separate question from admitting a family at all.
    // HDR (the INFINITY rgb ceiling) stays exactly where it was measured: on materials that
    // carry no `_MainColor` slot at all. The layers the declaration guard newly admits are a
    // different population — their `_TintColor` rgb is 1.0, DOUBLE the 0.5 x2-neutral, so the
    // "authored at or below neutral, therefore a real over-bright request" argument that
    // justified the uncapped ceiling does not hold for them. Uncapped they render (2,2,2),
    // which measured +7.4 on Kal'tsit (48.614 -> 56.053).
    let hdr = sub_l2d && !has_color_prop(mat, "_MainColor");
    if legacy && has_tint_color {
        (2.0, hdr)
    } else {
        (1.0, false)
    }
}

/// Does this material declare the given colour property at all?
///
/// Presence is the discriminator between the two modulation conventions in the
/// `Particles-L2D` sub-families: a shader reads either `_TintColor` or `_MainColor`, never
/// both, and a material only carries the one its shader declares. See `legacy_tint_scale`.
fn has_color_prop(mat: &Value, key: &str) -> bool {
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .is_some_and(|c| c.contains_key(key))
}

/// Ram-family (`Torappu/Particles-L2D/Ram/*`) scene-layer tint multiplier. These
/// compositors sample `2 × _MainColor × tex` (the frontend Ram GLSL mirrors it as
/// `col += col`), so authors key `_MainColor` α≈0.5 as neutral and dim/colour-shift it
/// below that for a soft wash — while `material_tint` would read the inert Unity
/// `_Color` default and render full white. Returns 2.0 when the shader is Ram-family
/// AND the material carries a `_MainColor` colour property, else 1.0.
fn ram_tint_scale(mat: &Value, animated_peak: Option<f32>, rgb_constant: bool) -> (f32, bool) {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let has_main_color = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .is_some_and(|c| c.contains_key("_MainColor"));
    // `Ram/` has always been recognised. Its sibling `Particles-L2D` compositors
    // (`Disturb/`, `Dissolve/`) modulate by the SAME ×2 `_MainColor`: across the dynchar
    // corpus they author it at exactly 0.502 — the ×2 neutral — in bulk (413 of 773
    // `Disturb(CustomData)`, 80 of 167 `Dissolve(CustomData)`), the identical convention
    // that proves the Ram family.
    //
    // Admitted ONLY for layers the entrance clip ANIMATES, where the clip names the
    // property outright. That restriction is not caution for its own sake: extending the
    // ×2 to a STATIC `Disturb(CustomData)` backdrop was tried before on Mlynar and
    // MEASURED WORSE, because clamping a static baseline and its peak to the same ceiling
    // shrinks the brightening delta instead of growing it. An animated curve has no such
    // problem — Virtuosa's shaft ramps `_MainColor` 0.502 → 0.196, which is a full-white
    // 1.004 falling to 0.392 once doubled, and rendering it undoubled left the whole frame
    // at 0.545× the game's brightness through the entire plunge.
    // For the newly-admitted families, require the half-neutral convention to ACTUALLY hold:
    // a ×2 material cannot author `_MainColor` above its own neutral, because doubling would
    // blow past white. Where the authored value is already full-scale the doubling only
    // clamps — flattening the very ramp we are trying to reproduce, which is exactly how the
    // earlier Mlynar backdrop attempt measured worse. Virtuosa's shaft sits at 0.502 and
    // doubles cleanly to 1.004; Mlynar's layers sit at 0.588–1.0 and are left alone.
    // `Ram/` keeps its unconditional doubling — shipped and verified.
    // The neutral is authored as an 8-bit colour, so it arrives as 128/255 = 0.50196 — the
    // bound has to admit that quantization, not a bare 0.5.
    const NEUTRAL_MAIN_COLOR: f64 = 0.5 + 1.0 / 255.0;
    let half_neutral = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.get("_MainColor"))
        .is_some_and(|c| {
            ["r", "g", "b"].iter().all(|k| {
                c.get(*k)
                    .and_then(serde_json::Value::as_f64)
                    .is_none_or(|v| v <= NEUTRAL_MAIN_COLOR)
            })
        });
    // Gate on the ANIMATED PEAK too, not just the serialized value: what gets doubled is the
    // curve. The bound is "the curve never reaches FULL SCALE", not "never exceeds the
    // neutral": a material authored at the ×2 neutral cannot ramp `_MainColor` to 1.0,
    // because doubling would reach 2.0 — so a curve that touches exactly 1.000 is authored
    // in the DIRECT (undoubled) convention and must be left alone, while anything strictly
    // below full scale is consistent with the ×2 convention the static neutral declares.
    // (The `0.5/255` margin is 8-bit quantization.) This admits Mlynar's bg01 backdrop
    // (crest 0.588 → 1.176) and still rejects every layer that reaches 1.000: Skadi2 14/15,
    // Virtuosa 0/78, excu2 ×5, mlyss 22/23.
    // Re-measured after the environment-background fill was corrected (which removed ~+11.6
    // of frame mean and could have made this doubling redundant): still ahead —
    // Mlynar mean MAD 33.903 with vs 34.256 without, Virtuosa and Skadi2 unmoved. Be honest
    // about WHAT it buys, though: on bg01 it is a near-constant +10.5 level lift, not the
    // step the game shows at t≈12.8 (ours +3.3, the game +29.3). With the DC removed both
    // sides have the same beat SHAPE, so the step is still missing somewhere else.
    // EXPERIMENT (`DYNCHAR_MC_LAW=1`, default OFF) — **MEASURED AND REFUTED**.
    //
    // The `Particles-L2D/` prefix is a NAMESPACE, not a tint law, and decompiling the family
    // shows the two do not coincide: `Dissolve/Dissolve(CustomData)` applies `_MainColor.xyz`
    // in its VERTEX stage and doubles NOWHERE, while its `Disturb/` and `Ram/` siblings run
    // `c = c + c`. So this prefix admits layers whose shader has no ×2 to port — which looks
    // like a plain bug until it is measured.
    //
    // ⚠️ CORRECTION 2026-08-29: read that as a statement about `Dissolve(CustomData)`, NOT about
    // `Dissolve/`. "The Dissolve family doubles nowhere" is FALSE as a family claim, and reading
    // it that way is how this investigation went wrong for a full run. Two Dissolve shaders were
    // dumped out of `[uc]shaders.ab` and both DO double:
    //
    //     Dissolve/Dissolve AB          vertex `vs_COLOR0 = in_COLOR0 * _TintColor`
    //                                   fragment `u_xlat1 = vs_COLOR0 + vs_COLOR0`
    //     Dissolve/Dissolve Add UVTween same pair (already recorded in `legacy_tint_scale`)
    //
    // The discriminator is not the sub-namespace, it is WHICH PROPERTY the shader declares.
    // `Dissolve(CustomData)` declares `_MainColor` and does not double; `Dissolve AB` declares
    // `_TintColor`, contains the string `_MainColor` zero times, and doubles. Both branches
    // already key on the declaration (`reads_main_color`, `l2d_main_color_family`); only this
    // comment generalised from one shader to its namespace. Read the shader that is bound.
    //
    // Narrowing the gate to the shaders that actually double changes exactly 2 layers across
    // the three references (cello `_Start` L0 sort −15 and L115, both 1.0039/α1.0 → 0.502/
    // α0.502) and costs **cel 17.167 -> 30.677**; mly and ska are bit-identical.
    //
    // Taken with the `DYNCHAR_MC_X2` result in the third tint branch, the two measurements
    // INVERT: where the shader DOES double (Mlynar's wind sheets) we must not, and where it
    // does NOT (cello's backdrop) we must. So this ×2 is not a port of `c = c + c` at all —
    // it is an amplitude correction that merely happens to be 2, and the shader's own
    // doubling is absorbed somewhere else in the texture/premultiply path. **The existing
    // prefix gate produces the right answer on all three references; do not "fix" it to
    // match the GLSL.**
    let doubling_shader = std::env::var("DYNCHAR_MC_LAW").is_err() || main_color_doubles(mat);
    let l2d_animated = shader.contains("Particles-L2D/")
        && doubling_shader
        && half_neutral
        && animated_peak.is_some_and(|p| f64::from(p) < 1.0 - 0.5 / 255.0);
    if !has_main_color {
        (1.0, false)
    } else if l2d_animated {
        // HDR: the gate above proved the curve stays strictly below full scale, so the
        // doubled colour is a meaningful over-bright RAMP (Mlynar's `bg01`: 0.824 →
        // 1.176) that the frontend's half-float target can carry. Clamping it collapses
        // baseline and peak onto the same ceiling and erases the brightening.
        (2.0, true)
    } else if std::env::var("DYNCHAR_DISSOLVE_CD_X2").is_ok()
        && shader.contains("Dissolve/Dissolve(CustomData)")
    {
        // THE THIRD TRIAL (`DYNCHAR_DISSOLVE_CD_X2=1`, default OFF). `Dissolve(CustomData)`
        // (pathID -4346129731114749491, and the plain `Particles/` twin) doubles on the
        // TEXTURE FETCH rather than after the property multiply:
        //
        // ```glsl
        // tex = texture(_MainTex, uv); tex = tex.wxyz + tex.wxyz;        // x2, rgb and alpha
        // alpha = tex.x * _MainColor.w;  rgb = tex.yzw * vs_COLOR0.xyz;  // vs = in_COLOR0 * _MainColor.xyz
        // ```
        //
        // so the layer's colour and coverage are both twice a plain `_MainColor` multiply, the
        // same net factor the Disturb x2 and the Anchor k stated for their families. Both of
        // those were read correctly, gated correctly, and refuted by the clips (register,
        // twenty-second and twenty-fourth runs): the frames did not carry what the program
        // states. This is the last unmeasured member of the family and is gated as the third
        // trial of that pattern, not as a fix. Clamped, since the two earlier HDR variants
        // measured worse than their clamped ones. `main_color_doubles`'s comment saying this
        // family "doubles nowhere" is wrong about the program and is left as the pre-trial
        // reading; the measurement decides what it becomes.
        (2.0, false)
    } else if std::env::var("DYNCHAR_RAM_X2_CONST").is_ok()
        && shader.contains("Particles-L2D/")
        && rgb_constant
    {
        // EXPERIMENT (`DYNCHAR_RAM_X2_CONST=1`, default OFF) — **MEASURED AND REFUTED**.
        //
        // The idea: both gates above exist to protect a RAMP from the clamp, so a curve that
        // animates only its ALPHA has no rgb ramp and could be admitted with a CLAMPED doubling
        // for free. Mlynar's three sort-10 layers are that shape, and scaling their rgb by
        // 2:2:1 is worth 0.349 MADC — the x2-with-blue-clamped ratio to 3 significant figures.
        //
        // Enabling it costs mly 17.405 -> 18.501 (cel/ska unmoved), because the x2 doubles the
        // ALPHA too (0.294 -> 0.588) and that layer's coverage is already right. Which is the
        // discriminator the whole investigation was missing: **a x2 material authors its values
        // AT the half-neutral, alpha included** — Virtuosa's admitted layers carry
        // `_MainColor` alpha ~0.502, Mlynar's carries 0.294. His layer is not in this
        // convention at all, so the gates above are RIGHT to exclude it and the 0.349 is a
        // fitted hue change that the x2 merely approximates in rgb.
        //
        // Kept inert as the evidence. Do not re-derive: the tempting "no ramp, so clamping is
        // free" argument is sound and still gives the wrong answer, because alpha is the tell.
        (2.0, false)
    } else if shader.contains("Ram/") {
        // EXPERIMENT (`DYNCHAR_RAM_NEUTRAL=1`, default OFF) -- **MEASURED AND REFUTED**.
        //
        // The idea: gate the `Ram/` doubling on the SAME half-neutral convention the sibling
        // families use. A x2 material cannot author `_MainColor` above its own neutral,
        // because doubling would blow past white -- so a `Ram/` material at 0.75 (excu2's
        // `cb_a_4`, the sheet behind her scope aperture) would be authored DIRECT, and
        // doubling it only clamps 1.5 -> 1.0. That layer IS measurably over-bright: ablating
        // it takes the lit region from +17.7 to +3.6 luma against the game at her t=5 beat.
        //
        // It is nonetheless the wrong correction. Gating changes 60 layer tints across three
        // references and costs **exc 8.918 -> 9.534**, ska 10.427 -> 10.451, cet 17.908 ->
        // 17.914; the other five are bit-identical. So the over-brightness is NOT a tint-scale
        // error -- scaling these sheets by 0.75 makes every other beat worse than it makes
        // t=5 better. This is a THIRD independent refutation of touching the `Ram/` x2 (the
        // two below unclamp it, this one withholds it), which is why the doubling stays
        // unconditional: it is an amplitude correction, not a port of `c = c + c`.
        //
        // Kept inert as the evidence. Do not re-derive.
        if std::env::var("DYNCHAR_RAM_NEUTRAL").is_ok() && !half_neutral {
            return (1.0, false);
        }
        // `Ram/` keeps the CLAMPED doubling it shipped with. Its materials sit at or near
        // full scale (Skadi2's layers double to 1.4-2.0), so letting them through
        // unclamped is not a ramp but a wholesale brightening — MEASURED worse
        // (Skadi2 mean MAD 18.615 -> 20.093 over the recorded beats).
        // RE-MEASURED 2026-08-03 on the current baseline and REFUTED AGAIN, harder: unclamping
        // costs Skadi 10.505 -> 26.953 and does NOT move Mlynar at all (17.525 either way), so it
        // is not the reason his city lights fail to glare. Do not retry.
        (2.0, false)
    } else {
        (1.0, false)
    }
}

/// A SUB-NAMESPACED `Torappu/Particles-L2D/<Family>/…` shader — the compositors that
/// sample extra maps on top of `_MainTex` (`Ram/`, `Disturb/`, `Dissolve/`, `Mask/`), as
/// opposed to the plain blend modes (`Particles-L2D/AlphaBlend`, `…/Additive`) that have
/// nothing below the namespace. Membership alone implies nothing about WHICH maps a given
/// material binds — every caller pairs this with a test on the material's own properties.
/// Script pathIDs of the two UV-SCROLL `MonoBehaviours`. They share the field layout
/// (`xspeed`/`yspeed` for the MAIN map, `useSecondMap` + `secondMapName` +
/// `secondXSpeed`/`secondYSpeed` for a NAMED second map); the second variant only adds
/// `protectMainUV`/`protectSecondUV`.
const UV_SCROLL_SCRIPTS: [i64; 2] = [1_369_540_917_083_035_942, 568_963_171_123_803_239];

/// The scroll a UV-scroll component adds to this `GameObject`'s lookups, in UV/second.
///
/// ⚠️ ADDITIVE to the material's own `_DissolveUSpeed`/`_DisturbUSpeed`. They are two different
/// mechanisms — the SHADER pans the lookup against `_Time`, while the component drives the
/// material's ST offset every frame — so a layer carrying both scrolls at the sum. Kal'tsit's
/// mist sheets (`ql*`) are the case that found this: their materials read `_DissolveUSpeed` 0.0
/// while the component scrolls `_DissolveTex` at -0.01..-0.15 u/s, so we evolved their masks
/// not at all.
///
/// Only the SECOND map is folded in here, because that is the one with a working renderer path
/// (`uDissolveScroll` / `uDisturbScroll`). The component's MAIN-map scroll is deliberately NOT
/// applied: `uvScroll` is a measured NO-OP on a ram-shader layer, so routing it there would be
/// inert and would read as a false null. `_RamTex` is skipped for the same reason — the ramp
/// has no scroll uniform at all.
#[derive(Default)]
struct UvScrollAdd {
    dissolve: [f32; 2],
    disturb: [f32; 2],
}

fn uv_scroll_component(all_objects: &HashMap<i64, (i32, Value)>, go_pid: i64) -> UvScrollAdd {
    let mut out = UvScrollAdd::default();
    for (cid, v) in all_objects.values() {
        if *cid != 114 {
            continue;
        }
        let Some(script) = v.get("m_Script").and_then(get_path_id) else {
            continue;
        };
        if !UV_SCROLL_SCRIPTS.contains(&script) {
            continue;
        }
        if v.get("m_GameObject").and_then(get_path_id) != Some(go_pid) {
            continue;
        }
        // The component applies its second map ONLY when the flag is set; a non-zero speed with
        // `useSecondMap` off is authored residue (Skadi's `pattern_01` is one).
        let use2 = v
            .get("useSecondMap")
            .and_then(|b| b.as_bool().map(u64::from).or_else(|| b.as_u64()))
            .unwrap_or(0)
            != 0;
        if !use2 {
            continue;
        }
        let name = v
            .get("secondMapName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let sx = v.get("secondXSpeed").and_then(Value::as_f64).unwrap_or(0.0) as f32;
        let sy = v.get("secondYSpeed").and_then(Value::as_f64).unwrap_or(0.0) as f32;
        if name.starts_with("_Dissolve") {
            out.dissolve[0] += sx;
            out.dissolve[1] += sy;
        } else if name.starts_with("_Distur") {
            out.disturb[0] += sx;
            out.disturb[1] += sy;
        }
    }
    out
}

/// Script pathID of the UV-ROTATION `MonoBehaviour` (see `SceneRam::uv_rot`).
const UV_ROTATION_SCRIPT: i64 = 7_163_214_010_000_414_217;

/// `[main, dissolve, ram, disturb]` UV rotation in DEGREES for this `GameObject`, from the
/// UV-rotation component attached to it. All zero when it carries none.
///
/// The component names its slots 1-based (`_rotateTex1..4`) against the shader's 0-based
/// `_Rotation0..3`, and spells the first angle `angle1` while the rest are `_angle2..4`.
/// A slot whose `_rotateTexN` flag is off keeps 0, so an authored angle that the component
/// does not actually apply cannot leak into the export.
pub(super) fn uv_rotation_of_go(all_objects: &HashMap<i64, (i32, Value)>, go_pid: i64) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    for (cid, v) in all_objects.values() {
        if *cid != 114 {
            continue;
        }
        if v.get("m_Script").and_then(get_path_id) != Some(UV_ROTATION_SCRIPT) {
            continue;
        }
        if v.get("m_GameObject").and_then(get_path_id) != Some(go_pid) {
            continue;
        }
        for (i, slot) in out.iter_mut().enumerate() {
            let on = v
                .get(format!("_rotateTex{}", i + 1).as_str())
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                != 0.0;
            if !on {
                continue;
            }
            let key = if i == 0 {
                "angle1".to_string()
            } else {
                format!("_angle{}", i + 1)
            };
            *slot = v.get(key.as_str()).and_then(Value::as_f64).unwrap_or(0.0) as f32;
        }
    }
    out
}

#[must_use]
/// The `Dissolve Add UVTween` program, under either of its two pass spellings.
///
/// `Torappu/Particles-L2D/Dissolve/Dissolve AB UVTween` is BYTE-IDENTICAL to the `Add`
/// spelling (blob 3209 bytes, decompiled GLSL differing only in the name line); only the pass
/// blend differs, `SrcAlpha One` against `SrcAlpha OneMinusSrcAlpha` in `rtBlend0`, which
/// `is_additive` already reads per material. Twenty-eight scene materials sit on the AB
/// spelling (Virtuosa sale#12 13, Kal'tsit sale#14 5, Rosmontis epoque#17 4 and sale#16 2,
/// Siege epoque#50 4) and draw as plain quads with no mask and no pan.
///
/// SHIPPED 2026-09-07 (`DYNCHAR_UVTWEEN_AB=0` narrows the family back to the `Add` spelling
/// exactly). The particle side first released Skadi boc#4's ten rain systems as a grey fog,
/// which was the render-mode-none routing fault (fixed in the viewer, e7cfe63d), not this
/// admission: with that fixed she reads -7.30 against -7.35 off. Scene side over 13 settled
/// rows against the static art: null to closer (luma sum -0.41 with 8 closer, Phantom sale#4
/// -2.11 -> -1.86, Blacknight wild#7 +3.08 -> +2.69, Dusk E2 +4.30 -> +5.74 the one loss);
/// entrance 15 beats, 7 bit-identical, kalts +0.48 summed over 8. Same program as the Add
/// spelling; checked against the explicit string.
pub fn is_uvtween_program(shader: &str) -> bool {
    shader.ends_with("/Dissolve/Dissolve Add UVTween")
        || (std::env::var("DYNCHAR_UVTWEEN_AB").as_deref() != Ok("0")
            && shader.ends_with("/Dissolve/Dissolve AB UVTween"))
}

#[must_use]
pub fn is_l2d_compositor(shader: &str) -> bool {
    // `Torappu/Particles/<sub>/…` is the SAME compositor family as
    // `Torappu/Particles-L2D/<sub>/…` — the namespace is a packaging split, not a shader
    // difference. `Ram/` was already matched wherever it lives (the cgbird / Archetto layers
    // outside `Particles-L2D` entirely); `Dissolve/` and `Disturb/` were not, so a material on
    // `Torappu/Particles/Dissolve/Dissolve AB Double` bound `_DissolveTex_01`/`_02` that nothing
    // ever read and the layer drew as a FULL OPAQUE RECTANGLE — the exact failure already
    // recorded for Mlynar's `Disturb/` wind sheets, one namespace over.
    //
    // Scope, measured across ALL 87 dynchar bundles: this widening changes the export of
    // exactly ONE skin (fugue). Every other skin is byte-identical with the gate wide or narrow,
    // so it is not a corpus-wide reclassification. `DYNCHAR_PARTICLES_NS=0` restores the
    // narrow gate.
    let tags: &[&str] = if std::env::var("DYNCHAR_PARTICLES_NS").as_deref() == Ok("0") {
        &["Particles-L2D/"]
    } else {
        &["Particles-L2D/", "Particles/"]
    };
    tags.iter().any(|tag| {
        shader
            .rfind(tag)
            .map(|i| &shader[i + tag.len()..])
            .is_some_and(|rest| rest.contains('/'))
    })
}

/// The `_MainColorACtrl` a material's shader actually READS, or 0.0 (the identity) when it
/// does not, or when the gate is off.
///
/// EXPERIMENT (`DYNCHAR_ANCHOR_K=1`, default OFF). The three `Disturb Anchor` fragments
/// (`Particles-L2D/Disturb/Disturb Anchor (AlphaBlend)` 7695302872418600095, `(Add)`, and
/// `Particles/Disturb/Disturb Anchor`) end their colour path with
///
/// ```glsl
/// u_xlat16_0 = texture(_MainTex, uv) * _MainColor * vs_COLOR0;
/// u_xlat16_0 = u_xlat16_0 + u_xlat16_0;
/// u_xlat16_9.x = _MainColorACtrl * (_MainColor.w - 1.0) + 1.0;
/// u_xlat16_0 = u_xlat16_0.wxyz * u_xlat16_9.xxxx;       // all four channels
/// ```
///
/// so with the control at 1 the layer's colour AND alpha are scaled by `_MainColor.a` once
/// more than a plain `_MainColor` multiply gives. The property is serialized on 709 dynchar
/// materials, but only these three programs declare it; on `Disturb(CustomData)` sheets it is
/// residue (see the note on `main_color_doubles`). Census 2026-09-08 (`probe_anchorctrl`):
/// 323 Anchor materials, 250 with the control set, 163 live (control set and alpha below
/// 1) on 32 skins, median k 0.578.
///
/// Kept SEPARATE from the program's own `x + x`: that doubling is on the refuted footing of
/// the Disturb x2 gate (register, twenty-second run), and this term is evaluated after it
/// in the fragment, so the two are independent multiplies and are gated independently.
pub(crate) fn anchor_ctrl(mat: &Value) -> f32 {
    if std::env::var("DYNCHAR_ANCHOR_K").is_err() {
        return 0.0;
    }
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !shader.contains("Disturb Anchor") {
        return 0.0;
    }
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Floats"))
        .and_then(|f| f.get("_MainColorACtrl"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0) as f32
}

/// `k = ctrl * (alpha - 1) + 1`, the Anchor scale for one evaluation of `_MainColor.a`. The
/// fragment evaluates it per frame against the property the clip animates, so a colour curve
/// applies it per key on that key's own alpha.
pub(crate) fn anchor_k(ctrl: f32, alpha: f32) -> f32 {
    ctrl * (alpha - 1.0) + 1.0
}

/// Whether this material's shader multiplies by `_MainColor` and then DOUBLES the result,
/// read out of the shader's own GLSL rather than inferred from how the authored value
/// looks. `Torappu/Particles-L2D/Disturb/Disturb(CustomData)`:
///
/// ```glsl
/// u_xlat16_1 = texture(_MainTex, uv) * _MainColor * vs_COLOR0;
/// u_xlat16_1 = u_xlat16_1 + u_xlat16_1;                 // ×2, all FOUR channels
/// SV_Target0.xyz = u_xlat16_1.xyz;                      // rgb NOT clamped
/// SV_Target0.w   = clamp(mask * u_xlat16_1.w * _Opacity, 0.0, 1.0);
/// ```
///
/// The frontend's `RAM_SCENE_FRAG` deliberately does not double ("already baked into the
/// exported tint"), so a layer that reaches the branch below with no scale is rendered at
/// HALF the amplitude the shader gives it — the doubling happens nowhere at all.
///
/// Scanning every `Particles*` shader in `[uc]shaders.ab` for this self-add splits the
/// families cleanly, and the split does NOT follow the namespace: the `Dissolve(CustomData)`
/// siblings apply `_MainColor.xyz` in their VERTEX stage with no doubling anywhere, which is
/// why this asks the shader instead of the `Particles-L2D/` prefix.
///
/// `_MainColorACtrl` is a red herring here — only the three `Disturb Anchor` variants
/// declare it (`k = mix(1, _MainColor.a, ctrl)`), and the wind sheets that carry the
/// property on the material do not use those shaders, so it is serialized residue.
fn main_color_doubles(mat: &Value) -> bool {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // Verified by decompiling each one's fragment program. Doubling families first; the
    // `Dissolve/` and `Disturb2` siblings are the verified NON-doubling ones and must not
    // be folded in by a looser prefix match.
    shader.contains("Particles-L2D/Disturb/Disturb(CustomData)")
        || shader.contains("Particles-L2D/Disturb/Disturb Anchor")
        || shader.contains("Particles-L2D/Disturb/Disturb (")
        || shader.contains("Particles-L2D/Ram/")
}

/// Whether a material belongs to the `Particles-L2D` compositor family that modulates by
/// `_MainColor` (`Ram/`, `Disturb/`, `Dissolve/`, … — the sub-namespaced shaders) AND
/// carries that property. Says nothing about the ×2 convention: that is
/// [`ram_tint_scale`]'s question, and a material can be in the family while being authored
/// in the DIRECT convention (`_MainColor` reaching full scale).
fn l2d_main_color_family(mat: &Value) -> bool {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    is_l2d_compositor(shader)
        && mat
            .get("m_SavedProperties")
            .and_then(|sp| sp.get("m_Colors"))
            .and_then(|c| c.as_object())
            .is_some_and(|c| c.contains_key("_MainColor"))
}

/// Which property spells this shader's dissolve SWITCH and THRESHOLD.
///
/// Three families ship three vocabularies for one piece of maths. Decompiled, the
/// dissolve term is identical in all of them, only the threshold's NAME changes:
///
/// ```text
/// Ram/Disturb(CustomData)      X = vs_TEXCOORD2.x + _Amount      (no switch, always on)
/// Disturb Anchor (AlphaBlend)  X = _Amount                       switch _ToggleUseDissolve
/// Dissolve/Dissolve(CustomData) X = _DissolveIntensity           switch _UseDissolveTex
///
/// t = tex - X;  k = 1 - roundEven(X + 0.5);
/// t = _BorderWidth * k + t;  t = t / _BorderWidth;  clamp(t, 0, 1)
/// ```
///
/// `_BorderWidth` is the same name and the same term in all three, so only the switch
/// and the threshold need naming. `Dissolve(CustomData)` names `_ToggleUseDissolve` and
/// `_Amount` ZERO times in its compiled GLSL, and `Ram/Disturb(CustomData)` names
/// `_DissolveIntensity` and `_UseDissolveTex` zero times, so the two vocabularies are
/// disjoint in the shipped programs.
///
/// Decided by the shader's DECLARATION, with the material's slot list as the fallback for
/// an under-staged export. A shader declaring BOTH is ambiguous and keeps today's names,
/// which is what guarantees no already-admitted layer moves.
fn dissolve_spelling(mat: &Value, shader: &str) -> (&'static str, &'static str) {
    let declares = |p: &str| {
        super::shader_map::shader_declares(shader, p).unwrap_or_else(|| has_float_prop(mat, p))
    };
    if declares("_UseDissolveTex") && !declares("_Amount") {
        ("_UseDissolveTex", "_DissolveIntensity")
    } else {
        ("_ToggleUseDissolve", "_Amount")
    }
}

/// Does the material carry `key` among its saved FLOAT properties?
fn has_float_prop(mat: &Value, key: &str) -> bool {
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Floats"))
        .and_then(|f| f.as_object())
        .is_some_and(|f| f.contains_key(key))
}

/// Does the material's own shader DECLARE `_MainColor`, and which test answered?
///
/// Declaration first: Unity keeps every property a material ever carried, so a
/// `_MainColor` left over from the shader an asset was authored against survives as
/// residue on a shader that cannot read it (630 such materials across the 87 dynchar
/// bundles, counted where `reads_main_color` is derived). `shader_declares` returns
/// `None` only when no shader bundle is staged, and then the material's slot list is
/// the fallback so an under-staged export cannot silently flip families.
fn main_color_declared(mat: &Value) -> (bool, &'static str) {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    super::shader_map::shader_declares(shader, "_MainColor").map_or_else(
        || (has_color_prop(mat, "_MainColor"), "mat"),
        |d| (d, "decl"),
    )
}

/// A layer whose shader reads `_MainColor` and whose material carries NO `_TintColor`.
///
/// This is the population `material_tint` mishandles: with no `_TintColor` to fall back
/// on it folds `_Color`, the inert Unity white placeholder, so the layer draws at full
/// opacity in whatever colour the shader was never told to use. Ch'en the Holungday's
/// two lens sheets are the measured case, authored `_MainColor` alpha 0.180 and 0.251
/// and drawn at 1.0 over an ADDITIVE blend.
///
/// The `_TintColor`-absent requirement is what keeps this DISJOINT from the parked
/// experiment at `reads_main_color`: that one is about materials carrying BOTH and
/// picking the wrong one, and it is measured worse (Kal'tsit's `wenli`/`wenl1` carry
/// both). Nothing here touches those.
fn main_color_no_tint_color(mat: &Value) -> Option<&'static str> {
    let (declared, how) = main_color_declared(mat);
    (declared && !has_color_prop(mat, "_TintColor") && has_color_prop(mat, "_MainColor"))
        .then_some(how)
}

/// Material colour multiply: `_Color` if present, else `_TintColor`, else white.
pub(crate) fn material_tint(mat: &Value) -> [f32; 4] {
    let colors = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object());
    let read = |key: &str| -> Option<[f32; 4]> {
        let c = colors?.get(key)?;
        Some([
            c.get("r")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0) as f32,
            c.get("g")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0) as f32,
            c.get("b")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0) as f32,
            c.get("a")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0) as f32,
        ])
    };
    read("_Color")
        .or_else(|| read("_TintColor"))
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

/// A material's saved colour properties `(name, rgba)` plus the name of the one
/// [`material_tint`] folded into the static tint (`_Color` first, else `_TintColor`).
/// Feeds [`super::anim::layer_color_curve`], which matches the `_Start` clip's animated
/// colour-channel bindings against these names.
pub(crate) fn material_color_props(mat: &Value) -> (Vec<(String, [f32; 4])>, Option<String>) {
    let comp =
        |c: &Value, k: &str| c.get(k).and_then(serde_json::Value::as_f64).unwrap_or(1.0) as f32;
    let props: Vec<(String, [f32; 4])> = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .map(|colors| {
            colors
                .iter()
                .map(|(k, c)| {
                    (
                        k.clone(),
                        [comp(c, "r"), comp(c, "g"), comp(c, "b"), comp(c, "a")],
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let tint_prop = ["_Color", "_TintColor"]
        .into_iter()
        .find(|k| props.iter().any(|(n, _)| n == k))
        .map(String::from);
    (props, tint_prop)
}

/// Detect additive blending from the material's `_DstBlend` factor (`One` = 1).
pub(crate) fn is_additive(mat: &Value) -> bool {
    // `_DstBlend` is only a LIVE input for the shader families that declare it (the
    // `(CustomData)` Disturb/Ram ports). The `Particles-L2D/Additive`, `AlphaBlend` and
    // `Dissolve Add` ports fix their blend inside the shader pass and declare no blend
    // property at all — there the material's `_DstBlend` float is inert residue from the
    // Unity Standard shader the asset was authored against (the same block also carries
    // `_Glossiness`, `_Metallic`, `_Parallax`… none of which the port reads).
    //
    // Trusting it unconditionally mis-blends 1648 of 3097 particle systems. Mlynar's
    // ground-impact flash (`bao 1`, `Particles-L2D/Additive`, stale `_DstBlend` 0) is a
    // near-white 2116px quad: additive it brightens the frame, which is what the game
    // shows at t≈12.8; composited `normal` it lays a translucent grey veil that DARKENS
    // it instead. So the shader NAME wins wherever the name fixes the blend, and
    // `_DstBlend` is consulted only for the property-driven families.
    let shader = mat
        .get("_shaderName")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    // Match the last path segment so `Ram/Add` reads as additive while
    // `Ram/Disturb(CustomData)` falls through to its declared `_DstBlend`.
    let port = shader.rsplit('/').next().unwrap_or("");
    if port.contains("alphablend") {
        return false;
    }
    if port.contains("add") {
        return true;
    }
    mat.get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Floats"))
        .and_then(|f| f.get("_DstBlend"))
        .and_then(serde_json::Value::as_f64)
        .is_some_and(|dst| (dst - 1.0).abs() < 0.01)
}

/// `find_entrance_timing`'s return: `(duration, transform, camera_ortho, voice,
/// background_reveal)`, all in seconds / world units — see that function's doc comment.
type EntranceTiming = (
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<f64>,
);

/// Read the entrance (`_Start`) cinematic timing from the prefab's director
/// `MonoBehaviour`. The director is the class-114 that owns `_mainCamera` + `_params`
/// (`{ duration, charVoiceOffset, fadeColor }`). Returns
/// `(duration, transform, camera_ortho, voice, background_reveal)`:
/// - `duration`  = `_params.duration` — the whole entrance span.
/// - `transform` = the reform beat: the per-object `_delayTime` value shared by the
///   MOST objects in the LATE half of the timeline (> duration·0.4). The `_Start`
///   sequences its sub-animations/effects by `_delayTime`; the big late cluster
///   (Virtuosa: 12.0s ×5) is the gala transformation burst — the camera has reached
///   the wide stop and the character has reformed by then, so it's the hand-off.
/// - `background_reveal` = the earliest of `transform` and `voice` that is later than
///   `duration · 0.4`, or `None` when neither is a sufficiently-late authored beat.
///
/// Also returns the entrance CAMERA's orthographic size (world units) — the tight
/// close-up the `_Start` opens on (Virtuosa: 2.99, vs the display `_cameraSize` 10.5;
/// 2.99/10.5 ≈ 0.28, a real head-to-torso close-up on the seated cellist). The
/// `_mainCamera` is fixed at this size; the client dollies OUT from it to the display
/// stop as she reforms. All `None` when the prefab has no entrance director.
/// Scan `all_objects` in ascending `path_id` order.
///
/// `HashMap::values()` yields an ARBITRARY order, so a `find_map` over it picks a random
/// winner whenever more than one object matches. Cheng Wanjing "Serene Whisper" ships two
/// class-20 Cameras in its `_Start` prefab, and the unordered pick flipped its
/// `entranceViewPx` between 1000 and 600 across runs of identical code — a 1.67x framing
/// swing decided by hash iteration order. Ordering by `path_id` makes the choice stable,
/// matching the convention `build_hash_to_go` already uses for collisions.
/// Deterministic object order for sibling modules (see `anim::entrance_post_fx`, which must not
/// walk hash order — the probe that found the post-process volume printed a different clip on
/// every run because it did).
pub(crate) fn objects_by_path_id_pub(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> Vec<(&i64, &(i32, Value))> {
    objects_by_path_id(all_objects)
}

fn objects_by_path_id(all_objects: &HashMap<i64, (i32, Value)>) -> Vec<(&i64, &(i32, Value))> {
    let mut v: Vec<_> = all_objects.iter().collect();
    v.sort_unstable_by_key(|(pid, _)| **pid);
    v
}

/// The ENTRANCE camera's `path_id`. The director names it outright (`_mainCamera.camera`),
/// which is the authored answer whenever a prefab ships more than one class-20 Camera;
/// the lowest-`path_id` camera is only a deterministic last resort.
pub(crate) fn entrance_camera_pid(all_objects: &HashMap<i64, (i32, Value)>) -> Option<i64> {
    let ordered = objects_by_path_id(all_objects);
    let named = ordered.iter().find_map(|(_, (cid, v))| {
        (*cid == 114)
            .then(|| {
                v.get("_mainCamera")
                    .and_then(|m| m.get("camera"))
                    .and_then(get_path_id)
                    .filter(|&p| p != 0)
            })
            .flatten()
    });
    named
        .filter(|p| matches!(all_objects.get(p), Some((20, _))))
        .or_else(|| {
            ordered
                .iter()
                .find_map(|(pid, (cid, _))| (*cid == 20).then_some(**pid))
        })
}

/// The entrance director's `_params.fadeColor` as straight RGBA in 0..1.
///
/// Documented in [`find_entrance_timing`]'s doc comment for a long time but never read: the
/// screen fade that ends every entrance was simply absent from our render. Virtuosa's recording
/// fades to pure white over roughly the last second and holds it until `duration`.
fn find_entrance_fade(all_objects: &HashMap<i64, (i32, Value)>) -> Option<[f64; 4]> {
    let ordered = objects_by_path_id(all_objects);
    let c = ordered.iter().find_map(|(_, (cid, v))| {
        (*cid == 114 && v.get("_mainCamera").is_some())
            .then(|| v.get("_params").and_then(|p| p.get("fadeColor")))
            .flatten()
    })?;
    let g = |k: &str| c.get(k).and_then(Value::as_f64);
    Some([g("r")?, g("g")?, g("b")?, g("a")?])
}

/// The entrance camera's solid clear colour (see `SpineAsset::bg_entrance_clear`). Read from
/// the class-20 Camera the director names, and only when `m_ClearFlags` is 2 (`SolidColor`);
/// any other clear mode leaves nothing authored to show.
fn find_entrance_clear_color(all_objects: &HashMap<i64, (i32, Value)>) -> Option<[f64; 3]> {
    let pid = entrance_camera_pid(all_objects)?;
    let (cid, cam) = all_objects.get(&pid)?;
    if *cid != 20 || cam.get("m_ClearFlags").and_then(Value::as_i64) != Some(2) {
        return None;
    }
    let c = cam.get("m_BackGroundColor")?;
    let g = |k: &str| c.get(k).and_then(Value::as_f64);
    Some([g("r")?, g("g")?, g("b")?])
}

fn find_entrance_timing(all_objects: &HashMap<i64, (i32, Value)>) -> EntranceTiming {
    let ordered = objects_by_path_id(all_objects);
    let params = ordered.iter().find_map(|(_, (cid, v))| {
        (*cid == 114 && v.get("_mainCamera").is_some())
            .then(|| v.get("_params"))
            .flatten()
    });
    let duration = params
        .and_then(|p| p.get("duration"))
        .and_then(Value::as_f64);
    // `charVoiceOffset` — when the reformed cellist starts her voice line (Virtuosa 10.3s).
    // She is STANDING and talking by then, so it's the true entrance→standing-idle hand-off
    // beat (the seated form has dissolved; the tall standing form lives elsewhere in the rig).
    let voice = params
        .and_then(|p| p.get("charVoiceOffset"))
        .and_then(Value::as_f64);
    let Some(dur) = duration else {
        return (None, None, None, None, None);
    };
    // Orthographic size of the entrance camera (class 20). One per `_Start` prefab.
    let cam_ortho = entrance_camera_pid(all_objects)
        .and_then(|p| all_objects.get(&p))
        .and_then(|(_, v)| v.get("orthographic size"))
        .and_then(Value::as_f64);
    // DIAGNOSTIC (`DYNCHAR_TFCHAIN=1`): is the entrance camera actually ORTHOGRAPHIC? A
    // perspective camera would make each layer's Z a real parallax term, which flat 2D
    // compositing cannot express; an orthographic one makes Z pure sort order.
    if std::env::var("DYNCHAR_TFCHAIN").is_ok()
        && let Some(cam) = entrance_camera_pid(all_objects).and_then(|p| all_objects.get(&p))
    {
        eprintln!(
            "  [camera] orthographic={:?} ortho_size={:?} fov={:?} near={:?} far={:?}",
            cam.1.get("orthographic"),
            cam_ortho,
            cam.1.get("field of view"),
            cam.1.get("near clip plane"),
            cam.1.get("far clip plane"),
        );
    }
    // Tally `_delayTime`s (rounded to 0.05s) and pick the most-shared LATE beat.
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for (cid, v) in all_objects.values() {
        if *cid != 114 {
            continue;
        }
        if let Some(dt) = v.get("_delayTime").and_then(Value::as_f64) {
            *counts.entry((dt * 20.0).round() as i64).or_insert(0) += 1;
        }
    }
    let transform = counts
        .iter()
        .map(|(k, c)| (*k as f64 / 20.0, *c))
        .filter(|(t, _)| *t > dur * 0.4)
        .max_by(|a, b| a.1.cmp(&b.1).then(a.0.partial_cmp(&b.0).unwrap()))
        .map(|(t, _)| t);
    let background_reveal = [transform, voice]
        .into_iter()
        .flatten()
        .filter(|&time| time > dur * 0.4)
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    (Some(dur), transform, cam_ortho, voice, background_reveal)
}

/// Accumulate a `GameObject`'s world TRANSLATION by summing `m_LocalPosition` up the
/// transform `m_Father` chain (rotation/scale ignored — the camera/root chain in these
/// prefabs is identity-rotated at unit scale). Used to find the entrance camera's aim
/// point relative to the skeleton root.
fn go_world_translation(all_objects: &HashMap<i64, (i32, Value)>, go_pid: i64) -> [f64; 3] {
    let start_tf = all_objects.iter().find_map(|(pid, (cid, v))| {
        (*cid == 4 && v.get("m_GameObject").and_then(get_path_id) == Some(go_pid)).then_some(*pid)
    });
    let mut acc = [0.0, 0.0, 0.0];
    let mut cur = start_tf;
    for _ in 0..256 {
        let Some(p) = cur else { break };
        let Some((4, tf)) = all_objects.get(&p) else {
            break;
        };
        if let Some(lp) = tf.get("m_LocalPosition") {
            acc[0] += lp.get("x").and_then(Value::as_f64).unwrap_or(0.0);
            acc[1] += lp.get("y").and_then(Value::as_f64).unwrap_or(0.0);
            acc[2] += lp.get("z").and_then(Value::as_f64).unwrap_or(0.0);
        }
        cur = tf.get("m_Father").and_then(get_path_id).filter(|&p| p != 0);
    }
    acc
}

/// Entrance camera AIM point relative to the skeleton root, in world units `(dx, dy)`.
/// The `_Start` camera is fixed; its XY is where the orthographic view is centred. The
/// difference from the skeleton root is how far ABOVE (dy>0) the character's origin the
/// game frames — the close-up sits on her upper body / the halo, not the hair-dragged
/// centroid. `None` when the prefab has no entrance camera + skeleton.
fn find_entrance_camera_offset(all_objects: &HashMap<i64, (i32, Value)>) -> Option<(f64, f64)> {
    // Ordered scan: two cameras / two skeletons in one prefab would otherwise be picked
    // by hash iteration order, making `entranceCamOffsetPx` differ between runs.
    let ordered = objects_by_path_id(all_objects);
    let cam_go = entrance_camera_pid(all_objects)
        .and_then(|p| all_objects.get(&p))
        .and_then(|(_, v)| v.get("m_GameObject").and_then(get_path_id))?;
    let skel_go = ordered.iter().find_map(|(_, (cid, v))| {
        (*cid == 114 && v.get("skeletonDataAsset").is_some())
            .then(|| v.get("m_GameObject").and_then(get_path_id))
            .flatten()
    })?;
    let c = go_world_translation(all_objects, cam_go);
    let s = go_world_translation(all_objects, skel_go);
    Some((c[0] - s[0], c[1] - s[1]))
}

/// Shared scene-graph context for placing non-spine objects (background quads,
/// particle emitters) in the spine root's local frame: the GameObject→Transform
/// map, the set of spine-root `GameObjects` (the walk stops before them), and the
/// evaluated idle pose. Built once per dynchar prefab.
pub(crate) struct BgParticleHost {
    go_to_transform: HashMap<i64, i64>,
    spine_gos: HashSet<i64>,
    /// `GameObject` `path_id` → the `_delayTime` (seconds) of a delay/activator
    /// `MonoBehaviour` on it. The `_Start` cinematic sequences its effect GROUPS by
    /// activating them after a delay; a particle system's own start delay is its
    /// nearest such ancestor's value (see {@link `delay_of_go`}).
    go_delay: HashMap<i64, f64>,
    /// `GameObject` `path_id` → spine-unity `BoneFollower` on it: `(boneName,
    /// followBoneRotation)`. Effect-prefab CLONE roots (the director `_effects`,
    /// e.g. Virtuosa's `start_apple_01(Clone)`) ride a SPINE BONE at runtime via
    /// this `MonoBehaviour` — their serialized transform is only an editor pose the
    /// follower overrides. Identified by field shape (`boneName` +
    /// `followXYPosition`), no script whitelist.
    go_follower: HashMap<i64, (String, bool)>,
    idle: super::anim::IdlePose,
}

impl BgParticleHost {
    /// Build the context from the prefab's object graph.
    pub(crate) fn new(all_objects: &HashMap<i64, (i32, Value)>) -> Self {
        let mut go_to_transform: HashMap<i64, i64> = HashMap::new();
        for (pid, (cid, v)) in all_objects {
            if *cid == 4
                && let Some(go) = v.get("m_GameObject").and_then(get_path_id)
            {
                go_to_transform.insert(go, *pid);
            }
        }
        let spine_gos: HashSet<i64> = all_objects
            .values()
            .filter(|(cid, v)| *cid == 114 && v.get("skeletonDataAsset").is_some())
            .filter_map(|(_, v)| v.get("m_GameObject").and_then(get_path_id))
            .collect();
        // Map each GameObject carrying a `_delayTime` activator MonoBehaviour to its delay.
        let mut go_delay: HashMap<i64, f64> = HashMap::new();
        for (cid, v) in all_objects.values() {
            if *cid == 114
                && let Some(dt) = v.get("_delayTime").and_then(Value::as_f64)
                && dt > 0.0
                && let Some(go) = v.get("m_GameObject").and_then(get_path_id)
            {
                // Keep the largest if a GO somehow has more than one.
                let e = go_delay.entry(go).or_insert(0.0);
                *e = e.max(dt);
            }
        }
        // spine-unity BoneFollower components (field-shape identified).
        let mut go_follower: HashMap<i64, (String, bool)> = HashMap::new();
        for (cid, v) in all_objects.values() {
            if *cid == 114
                && let Some(bone) = v.get("boneName").and_then(Value::as_str)
                && !bone.is_empty()
                && v.get("followXYPosition").is_some()
                && let Some(go) = v.get("m_GameObject").and_then(get_path_id)
            {
                let rot = v
                    .get("followBoneRotation")
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    != 0;
                go_follower.insert(go, (bone.to_string(), rot));
            }
        }
        let idle = super::anim::evaluate_idle_pose(all_objects, &go_to_transform);
        Self {
            go_to_transform,
            spine_gos,
            go_delay,
            go_follower,
            idle,
        }
    }

    /// Nearest `BoneFollower` in the `GameObject`'s transform ancestry (self first,
    /// stopping at the spine root): `(boneName, followBoneRotation, follower GO)`.
    /// The follower's GO is returned so the caller can bake the emitter's local
    /// offset WITHIN the followed rig (the follower snaps that GO onto the bone).
    pub(crate) fn follower_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> Option<(String, bool, i64)> {
        let mut cur_go = Some(go_pid);
        for _ in 0..256 {
            let g = cur_go?;
            if let Some((bone, rot)) = self.go_follower.get(&g) {
                return Some((bone.clone(), *rot, g));
            }
            if self.spine_gos.contains(&g) {
                return None;
            }
            let tf = self.go_to_transform.get(&g)?;
            let father = all_objects
                .get(tf)
                .and_then(|(_, v)| v.get("m_Father"))
                .and_then(get_path_id)
                .filter(|&p| p != 0)?;
            cur_go = all_objects
                .get(&father)
                .and_then(|(_, v)| v.get("m_GameObject"))
                .and_then(get_path_id);
        }
        None
    }

    /// Topmost ancestor `GameObject` of `go_pid` — the prefab-INSTANCE root its
    /// subtree belongs to. Dynchar bundles ship SEVERAL sibling prefab roots
    /// (`dyn_illust_*` idle + `dyn_entrance_*` cinematic), each with its own
    /// skeleton and effect rigs; membership decisions must compare these roots.
    pub(crate) fn prefab_root_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> Option<i64> {
        let mut cur_tr = *self.go_to_transform.get(&go_pid)?;
        for _ in 0..256 {
            let Some((4, tf)) = all_objects.get(&cur_tr) else {
                break;
            };
            match tf
                .get("m_Father")
                .and_then(get_path_id)
                .filter(|&f| f != 0 && all_objects.contains_key(&f))
            {
                Some(f) => cur_tr = f,
                None => break,
            }
        }
        all_objects
            .get(&cur_tr)
            .and_then(|(_, tf)| tf.get("m_GameObject"))
            .and_then(get_path_id)
    }

    /// Whether the `GameObject` and every ancestor are active (see `go_effectively_active`).
    pub(crate) fn effectively_active(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
        start_state_active: bool,
    ) -> bool {
        go_effectively_active(
            all_objects,
            go_pid,
            &self.go_to_transform,
            &self.idle.active,
            start_state_active,
            false,
        )
    }

    /// Is this `GameObject` under a `Start Only Effects` group? See [`has_start_only_ancestor`].
    pub(crate) fn has_start_only_ancestor(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> bool {
        has_start_only_ancestor(all_objects, go_pid, &self.go_to_transform)
    }

    /// DIAGNOSTIC: which ancestor blocked [`Self::effectively_active`], and why.
    pub(crate) fn blocking_ancestor(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
        start_state_active: bool,
    ) -> Option<(String, &'static str)> {
        blocking_ancestor(
            all_objects,
            go_pid,
            &self.go_to_transform,
            &self.idle.active,
            start_state_active,
        )
    }

    /// World matrix of a `GameObject`'s Transform, in the spine root's local frame.
    pub(crate) fn world_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> super::mesh::Mat4 {
        self.go_to_transform
            .get(&go_pid)
            .map_or_else(super::mesh::Mat4::identity, |&tf| {
                accumulate_matrix(all_objects, tf, &self.spine_gos, &self.idle)
            })
    }

    /// The `GameObject`'s transform accumulated to the TOP of the prefab — the spine-root stop
    /// is NOT applied, so a root's own transform is included and two different roots become
    /// directly comparable.
    ///
    /// `world_of_go` stops at whichever skeleton root it reaches, which means a system living
    /// under the IDLE root comes back in the IDLE root's frame while the entrance composite
    /// draws in the `_Start` root's frame. Re-basing one into the other needs both expressed in
    /// a common frame, which is what this provides. See the cross-root re-basing in
    /// `collect_dynchar_particles`.
    pub(crate) fn world_full_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> super::mesh::Mat4 {
        self.go_to_transform
            .get(&go_pid)
            .map_or_else(super::mesh::Mat4::identity, |&tf| {
                accumulate_matrix(all_objects, tf, &HashSet::new(), &self.idle)
            })
    }

    /// The world matrix of the `GameObject`'s Transform PARENT (its `m_Father` chain,
    /// up to but excluding the spine root). Used to project a child's animated LOCAL
    /// position (in the parent's frame) into the spine-root world frame — the pivot
    /// offset of an entrance-driven effect host (see `entrance_transform_curves`).
    /// `None` when the `GameObject` has no transform or its transform has no parent.
    pub(crate) fn parent_world_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> Option<super::mesh::Mat4> {
        let tf = *self.go_to_transform.get(&go_pid)?;
        let father = all_objects
            .get(&tf)
            .and_then(|(_, v)| v.get("m_Father"))
            .and_then(get_path_id)
            .filter(|&p| p != 0)?;
        Some(accumulate_matrix(
            all_objects,
            father,
            &self.spine_gos,
            &self.idle,
        ))
    }

    /// The `GameObject`'s serialized local `(scale, position)` from its Transform. Used
    /// to normalise an entrance SCALE curve into a multiplier of the baked (resting)
    /// pose and to reference the animated POSITION delta.
    pub(crate) fn local_scale_pos_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> Option<([f32; 3], [f32; 3])> {
        let tf = *self.go_to_transform.get(&go_pid)?;
        let (_, v) = all_objects.get(&tf)?;
        let vec3 = |field: &str, d: f32| {
            let g = |k: &str| {
                v.get(field)
                    .and_then(|x| x.get(k))
                    .and_then(Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x"), g("y"), g("z")]
        };
        Some((vec3("m_LocalScale", 1.0), vec3("m_LocalPosition", 0.0)))
    }

    /// The nearest ancestor `GameObject` of `go_pid` (self first, up to the spine root)
    /// that the `_Start` cinematic drives with a Transform-scale curve (present in
    /// `curves`). This is the effect-host whose animated scale/position the emitter
    /// rides — Virtuosa's crown `ctrl` above the `spark_small` glow host. `None` when
    /// no ancestor is entrance-transform-driven.
    pub(crate) fn entrance_transform_host_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
        curves: &HashMap<i64, super::anim::EntranceTransform>,
    ) -> Option<i64> {
        let mut cur_go = Some(go_pid);
        for _ in 0..256 {
            let g = cur_go?;
            if curves.contains_key(&g) {
                return Some(g);
            }
            let tf = self.go_to_transform.get(&g)?;
            let father = all_objects
                .get(tf)
                .and_then(|(_, v)| v.get("m_Father"))
                .and_then(get_path_id)
                .filter(|&p| p != 0)?;
            cur_go = all_objects
                .get(&father)
                .and_then(|(_, v)| v.get("m_GameObject"))
                .and_then(get_path_id);
        }
        None
    }

    /// The `GameObject`'s own `m_Name`, or `"?"`. Diagnostics only: the exported JSONs
    /// are anonymous, so attributing a layer/system back to an authored node needs this.
    // Kept as a method (rather than an associated fn) for call-site symmetry with its several
    // `host.go_name(...)` / `self.go_name(...)` call sites elsewhere in this impl.
    #[allow(clippy::unused_self)]
    pub(crate) fn go_name(&self, all_objects: &HashMap<i64, (i32, Value)>, go_pid: i64) -> String {
        all_objects
            .get(&go_pid)
            .and_then(|(_, v)| v.get("m_Name"))
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string()
    }

    /// `go_name` of the `GameObject`'s prefab-INSTANCE root — which of a dynchar bundle's
    /// sibling roots (`dyn_illust_*` idle / `dyn_entrance_*` cinematic) it belongs to.
    pub(crate) fn root_name_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> String {
        self.prefab_root_of_go(all_objects, go_pid)
            .map_or_else(|| "?".to_string(), |r| self.go_name(all_objects, r))
    }

    /// The `GameObject` `m_Name` of every ancestor of `go_pid`, nearest-first,
    /// walking `m_Father` up to (but excluding) the spine root. In-game, a
    /// particle emitter that drifts with the character is parented under a
    /// spine-driven bone `GameObject` (spine-unity's `SkeletonUtilityBone`, named
    /// after the bone it mirrors); its name therefore appears in this chain. The
    /// frontend matches these names against the loaded skeleton's bone names to
    /// decide which emitters follow a bone (and which stay world-fixed). The
    /// emitter's own `GameObject` name is included first so a directly-on-bone
    /// emitter still resolves.
    pub(crate) fn ancestor_go_names(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> Vec<String> {
        let mut names = Vec::new();
        let Some(&start) = self.go_to_transform.get(&go_pid) else {
            return names;
        };
        let mut cur = Some(start);
        for _ in 0..256 {
            let Some(tf_pid) = cur else { break };
            let Some((4, tf)) = all_objects.get(&tf_pid) else {
                break;
            };
            let go = tf.get("m_GameObject").and_then(get_path_id);
            if go.is_some_and(|g| self.spine_gos.contains(&g)) {
                break; // reached the spine root; its own name is not a bone
            }
            if let Some((1, gv)) = go.and_then(|g| all_objects.get(&g))
                && let Some(n) = gv.get("m_Name").and_then(Value::as_str)
                && !n.is_empty()
            {
                names.push(n.to_string());
            }
            cur = tf.get("m_Father").and_then(get_path_id).filter(|&p| p != 0);
        }
        names
    }

    /// Start delay (seconds) of the `GameObject`'s effect, from `_delayTime` activators
    /// in its transform ancestry (self included, up to but excluding the spine root).
    /// Nested activators COMPOUND — a group enabled at 2s that in turn enables a child
    /// delayed 1s fires at 3s — so the delays are SUMMED. `0.0` when none apply.
    pub(crate) fn delay_of_go(&self, all_objects: &HashMap<i64, (i32, Value)>, go_pid: i64) -> f64 {
        let mut total = 0.0;
        if let Some(d) = self.go_delay.get(&go_pid) {
            total += *d;
        }
        let Some(&start) = self.go_to_transform.get(&go_pid) else {
            return total;
        };
        let mut cur = all_objects
            .get(&start)
            .and_then(|(_, tf)| tf.get("m_Father"))
            .and_then(get_path_id)
            .filter(|&p| p != 0);
        for _ in 0..256 {
            let Some(tf_pid) = cur else { break };
            let Some((4, tf)) = all_objects.get(&tf_pid) else {
                break;
            };
            let go = tf.get("m_GameObject").and_then(get_path_id);
            if go.is_some_and(|g| self.spine_gos.contains(&g)) {
                break; // reached the spine root
            }
            if let Some(g) = go
                && let Some(d) = self.go_delay.get(&g)
            {
                total += *d;
            }
            cur = tf.get("m_Father").and_then(get_path_id).filter(|&p| p != 0);
        }
        total
    }

    /// ENTRANCE reveal time (s) of a particle emitter: the `activeFrom` of the nearest
    /// ancestor (self first) that the `_Start` cinematic's `m_IsActive` curves switch ON
    /// (`windows`, from {@link `super::anim::active_windows`}). The apple/glow/wing spark
    /// emitters start OFF and are toggled on mid-cinematic (Virtuosa ~4.8–9.8s); without
    /// this their `_delayTime` is 0 so they'd wrongly emit from t=0 (during the seated
    /// intro) and be spent before the apple falls. `None` when no ancestor is toggled.
    /// The FULL `m_IsActive` window `(from, until)` of the nearest gated ancestor — the
    /// counterpart of {@link `entrance_reveal_of_go`}, which returns only the reveal. Diagnostic
    /// for the "particles never switch off" gap.
    pub(crate) fn entrance_window_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
        windows: &HashMap<i64, super::anim::ActiveWindowList>,
    ) -> Option<(Option<f32>, Option<f32>)> {
        let mut cur_go = Some(go_pid);
        for _ in 0..256 {
            let g = cur_go?;
            // First window only — see `entrance_reveal_of_go`.
            if let Some(&w) = windows.get(&g).and_then(|w| w.first()) {
                return Some(w);
            }
            let tf = self.go_to_transform.get(&g)?;
            let father = all_objects
                .get(tf)
                .and_then(|(_, v)| v.get("m_Father"))
                .and_then(get_path_id)
                .filter(|&p| p != 0)?;
            cur_go = all_objects
                .get(&father)
                .and_then(|(_, v)| v.get("m_GameObject"))
                .and_then(get_path_id);
        }
        None
    }

    pub(crate) fn entrance_reveal_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
        windows: &HashMap<i64, super::anim::ActiveWindowList>,
    ) -> Option<f32> {
        let mut cur_go = Some(go_pid);
        for _ in 0..256 {
            let g = cur_go?;
            // FIRST window only: an emitter's schedule is its delay + lifetime, and driving a
            // particle system through a multi-window flash is a separate question from the
            // scene layer this generalisation was built for. Single-window hosts (all of them
            // today, bar the scene overlays) are unaffected.
            if let Some(Some(from)) = windows.get(&g).and_then(|w| w.first()).map(|w| w.0) {
                return Some(from);
            }
            let tf = self.go_to_transform.get(&g)?;
            let father = all_objects
                .get(tf)
                .and_then(|(_, v)| v.get("m_Father"))
                .and_then(get_path_id)
                .filter(|&p| p != 0)?;
            cur_go = all_objects
                .get(&father)
                .and_then(|(_, v)| v.get("m_GameObject"))
                .and_then(get_path_id);
        }
        None
    }
}

/// Accumulate the full world matrix (TRS with rotation) by walking `m_Father`
/// from `start_tf_pid` upward, stopping *before* any spine-root transform so the
/// result is expressed in the spine root's local frame (the character's frame).
/// Sample a keyframe list at `t` with linear interpolation, holding the end values.
/// Returns `dflt` for an empty curve (an axis the clip does not animate).
fn sample_kf(c: &[(f32, f32)], t: f32, dflt: f32) -> f32 {
    if c.is_empty() {
        return dflt;
    }
    if t <= c[0].0 {
        return c[0].1;
    }
    if t >= c[c.len() - 1].0 {
        return c[c.len() - 1].1;
    }
    for w in c.windows(2) {
        let (t0, v0) = w[0];
        let (t1, v1) = w[1];
        if t <= t1 {
            let d = t1 - t0;
            return if d.abs() < 1e-6 {
                v1
            } else {
                v0 + (v1 - v0) * (t - t0) / d
            };
        }
    }
    c[c.len() - 1].1
}

fn accumulate_matrix(
    all_objects: &HashMap<i64, (i32, Value)>,
    start_tf_pid: i64,
    stop_gos: &HashSet<i64>,
    idle: &super::anim::IdlePose,
) -> super::mesh::Mat4 {
    use super::mesh::Mat4;
    let mut acc = Mat4::identity();
    let mut cur = Some(start_tf_pid);
    let mut guard = 0;

    while let Some(tf_pid) = cur {
        guard += 1;
        if guard > 256 {
            break;
        }
        // DIAGNOSTIC (`DYNCHAR_TFCHAIN=1`): this walk TERMINATES on any transform that is not
        // class 4, which silently drops every ancestor above it. RectTransform is class 224 and
        // these prefabs contain them, so a chain that passes through one would be accumulated
        // only partially — a constant position error for exactly those layers.
        if std::env::var("DYNCHAR_TFCHAIN").is_ok() && all_objects.get(&tf_pid).is_none() {
            eprintln!(
                "  [tfchain] MISSING transform pid {tf_pid} at depth={guard} — chain truncated"
            );
        }
        if std::env::var("DYNCHAR_TFCHAIN").is_ok()
            && let Some((cid, v)) = all_objects.get(&tf_pid)
            && *cid != 4
        {
            let name = v
                .get("m_GameObject")
                .and_then(get_path_id)
                .and_then(|g| all_objects.get(&g))
                .and_then(|(_, o)| o.get("m_Name"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            eprintln!(
                "  [tfchain] TRUNCATED at class {cid} (pid {tf_pid}) go='{name}' depth={guard}"
            );
        }
        let Some((4, tf)) = all_objects.get(&tf_pid) else {
            break;
        };
        if tf
            .get("m_GameObject")
            .and_then(get_path_id)
            .is_some_and(|go| stop_gos.contains(&go))
        {
            break; // reached the spine root; don't include its transform
        }

        let vec3 = |field: &str, d: f32| -> [f32; 3] {
            let g = |k: &str| {
                tf.get(field)
                    .and_then(|v| v.get(k))
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x"), g("y"), g("z")]
        };
        // Position/rotation come from the idle-pose override for this transform
        // when present, else the prefab bind pose.
        let prefab_pos = vec3("m_LocalPosition", 0.0);
        let pos = idle.pos.get(&tf_pid).copied().unwrap_or(prefab_pos);
        // DIAGNOSTIC (`DYNCHAR_TFCHAIN=1`): the IDLE-pose override is applied when building
        // EVERY scene's quads, including the `_Start` entrance scene. Where the idle pose puts
        // a transform somewhere other than the prefab bind pose, the entrance quad is baked at
        // the IDLE position and stays there for the whole entrance.
        if std::env::var("DYNCHAR_TFCHAIN").is_ok() {
            let d = [
                pos[0] - prefab_pos[0],
                pos[1] - prefab_pos[1],
                pos[2] - prefab_pos[2],
            ];
            if d[0].abs() > 1e-6 || d[1].abs() > 1e-6 || d[2].abs() > 1e-6 {
                let name = tf
                    .get("m_GameObject")
                    .and_then(get_path_id)
                    .and_then(|g| all_objects.get(&g))
                    .and_then(|(_, o)| o.get("m_Name"))
                    .and_then(Value::as_str)
                    .unwrap_or("?");
                eprintln!(
                    "  [idlepose] go='{name}' prefab=({:.4},{:.4}) idle=({:.4},{:.4}) delta=({:.4},{:.4}) units",
                    prefab_pos[0], prefab_pos[1], pos[0], pos[1], d[0], d[1]
                );
            }
        }
        let quat = if let Some(&e) = idle.euler.get(&tf_pid) {
            super::anim::euler_deg_to_quat(e)
        } else {
            let g = |k: &str, d: f32| {
                tf.get("m_LocalRotation")
                    .and_then(|v| v.get(k))
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(d.into()) as f32
            };
            [g("x", 0.0), g("y", 0.0), g("z", 0.0), g("w", 1.0)]
        };
        let local = Mat4::trs(pos, quat, vec3("m_LocalScale", 1.0));
        acc = local.mul(&acc); // parent transforms child: multiply on the left

        cur = tf.get("m_Father").and_then(get_path_id).filter(|&p| p != 0);
    }

    acc
}

/// Collect enemy spine assets from a `refs/arts/enm_art_*` pack bundle.
///
/// Unlike operator bundles, enemy art bundles carry no `SkeletonMecanim`
/// components (those live in the `battle/enm_pfb_*` prefab bundles and point
/// here via external references), so traversal starts directly at each
/// `SkeletonData` `MonoBehaviour` (class 114 with `skeletonJSON` + `atlasAssets`).
/// All assets are categorized as [`SpineCategory::Enemy`].
#[must_use]
pub fn collect_enemy_spine_assets(
    all_objects: &HashMap<i64, (i32, Value)>,
) -> (Vec<SpineAsset>, HashSet<i64>) {
    let mut claimed = HashSet::new();
    let mut assets = Vec::new();

    let mut skel_data_pids: Vec<i64> = all_objects
        .iter()
        .filter(|(_, (class_id, val))| {
            *class_id == 114
                && val.get("skeletonJSON").is_some()
                && val.get("atlasAssets").is_some()
        })
        .map(|(pid, _)| *pid)
        .collect();
    // Deterministic processing order across runs
    skel_data_pids.sort_unstable();

    for skel_data_pid in skel_data_pids {
        let Some(chain) = follow_skeleton_data(all_objects, skel_data_pid) else {
            continue;
        };

        let base_name = chain
            .skel_name
            .strip_suffix(".skel")
            .unwrap_or(&chain.skel_name);

        claimed.extend(&chain.claimed);

        assets.push(SpineAsset {
            name: base_name.to_string(),
            skel_data: chain.skel_bytes,
            atlas_text: chain.atlas_text,
            textures: chain.textures,
            category: SpineCategory::Enemy,
            bg_quads: Vec::new(),
            bg_skel_scale: None,
            settle_animation: None,
            bg_camera_size: None,
            bg_max_aspect: None,
            bg_max_size: None,
            bg_camera_offset: None,
            bg_camera_view: None,
            bg_camera_offset2: None,
            bg_camera_view2: None,
            bg_character_sort: None,
            separator_slots: Vec::new(),
            separator_part_sorts: Vec::new(),
            bg_entrance_duration: None,
            bg_entrance_clip_stop: None,
            bg_entrance_fade: None,
            bg_entrance_clear: None,
            bg_entrance_transform: None,
            bg_entrance_view: None,
            bg_entrance_persp: false,
            bg_entrance_cam_offset: None,
            bg_entrance_ortho_curve: None,
            bg_entrance_post_fx: None,
            bg_entrance_pan_curve: None,
            bg_entrance_cam_center: None,
            bg_entrance_cam_roll: None,
            bg_entrance_aperture: None,
            bg_entrance_voice: None,
            particles: Vec::new(),
        });
    }

    (assets, claimed)
}

/// Everything resolved by following a `SkeletonData` `MonoBehaviour`'s references.
struct SkeletonDataChain {
    skel_name: String,
    skel_bytes: Vec<u8>,
    atlas_text: String,
    textures: Vec<(String, Value)>,
    /// All `path_ids` consumed while walking the chain (including `skel_data_pid`)
    claimed: Vec<i64>,
}

/// Follow a `SkeletonData` `MonoBehaviour`'s reference chain:
/// ```text
/// SkeletonData (skeletonJSON + atlasAssets)
///   → skel: TextAsset (.skel binary, base64)
///   → Atlas MonoBehaviour (atlasFile + materials)
///     → atlas: TextAsset (.atlas text)
///     → Material → _MainTex/_AlphaTex → Texture2D
/// ```
/// Returns `None` when any required link is missing or malformed.
fn follow_skeleton_data(
    all_objects: &HashMap<i64, (i32, Value)>,
    skel_data_pid: i64,
) -> Option<SkeletonDataChain> {
    let skel_data_val = match all_objects.get(&skel_data_pid) {
        Some((114, val)) => val,
        _ => return None,
    };

    // Follow skeletonJSON → TextAsset (.skel)
    let skel_text_pid = match skel_data_val.get("skeletonJSON").and_then(get_path_id) {
        Some(pid) if pid != 0 => pid,
        _ => return None,
    };

    let skel_text_val = match all_objects.get(&skel_text_pid) {
        Some((49, val)) => val,
        _ => return None,
    };

    let skel_name = skel_text_val["m_Name"].as_str().unwrap_or("").to_string();
    let skel_script = skel_text_val["m_Script"].as_str().unwrap_or("");

    // `m_Script` (the .skel binary) is stored by the object reader as
    // `base64:<...>` when the bytes aren't valid UTF-8, or as a raw string when
    // they happen to be (a large `.skel` can be — e.g. Eyjafjalla "A Picnic
    // Before A Long Trip"'s idle illustration). Recover the bytes from whichever
    // encoding is present; a raw UTF-8 string's bytes ARE the skel losslessly.
    let skel_bytes = match skel_script.strip_prefix("base64:") {
        Some(b64) => base64::engine::general_purpose::STANDARD.decode(b64).ok()?,
        None => skel_script.as_bytes().to_vec(),
    };

    // Follow atlasAssets[0] → Atlas MonoBehaviour
    let atlas_mono_pid = match skel_data_val
        .get("atlasAssets")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(get_path_id)
    {
        Some(pid) if pid != 0 => pid,
        _ => return None,
    };

    let atlas_mono_val = match all_objects.get(&atlas_mono_pid) {
        Some((114, val)) => val,
        _ => return None,
    };

    // Follow atlasFile → TextAsset (.atlas)
    let atlas_text_pid = match atlas_mono_val.get("atlasFile").and_then(get_path_id) {
        Some(pid) if pid != 0 => pid,
        _ => return None,
    };

    let atlas_text_val = match all_objects.get(&atlas_text_pid) {
        Some((49, val)) => val,
        _ => return None,
    };

    let atlas_script = atlas_text_val["m_Script"].as_str().unwrap_or("");
    if atlas_script.starts_with("base64:") || !atlas_script.contains(".png") {
        return None;
    }
    let atlas_text = atlas_script.to_string();

    let mut claimed = vec![skel_data_pid, skel_text_pid, atlas_mono_pid, atlas_text_pid];

    // Follow materials → Material → textures
    let mut textures: Vec<(String, Value)> = Vec::new();
    let mut texture_pids: Vec<i64> = Vec::new();

    if let Some(materials) = atlas_mono_val.get("materials").and_then(|v| v.as_array()) {
        for mat_ref in materials {
            let mat_pid = match get_path_id(mat_ref) {
                Some(pid) if pid != 0 => pid,
                _ => continue,
            };

            let mat_val = match all_objects.get(&mat_pid) {
                Some((21, val)) => val,
                _ => continue,
            };

            // Extract textures from m_SavedProperties.m_TexEnvs
            // m_TexEnvs is a JSON object: {"_MainTex": {m_Texture: ...}, "_AlphaTex": ...}
            if let Some(tex_envs) = mat_val
                .get("m_SavedProperties")
                .and_then(|sp| sp.get("m_TexEnvs"))
                .and_then(|te| te.as_object())
            {
                for (tex_type, tex_data) in tex_envs {
                    if tex_type != "_MainTex" && tex_type != "_AlphaTex" {
                        continue;
                    }
                    let tex_pid = match tex_data.get("m_Texture").and_then(get_path_id) {
                        Some(pid) if pid != 0 => pid,
                        _ => continue,
                    };

                    if let Some((28, tex_val)) = all_objects.get(&tex_pid) {
                        let tex_name = tex_val["m_Name"].as_str().unwrap_or("unnamed").to_string();
                        if !texture_pids.contains(&tex_pid) {
                            texture_pids.push(tex_pid);
                            textures.push((tex_name, tex_val.clone()));
                        }
                    }
                }
            }

            claimed.push(mat_pid);
        }
    }

    claimed.extend(&texture_pids);

    Some(SkeletonDataChain {
        skel_name,
        skel_bytes,
        atlas_text,
        textures,
        claimed,
    })
}

/// Classify a spine asset into a category.
///   1. skel name starts with "dyn_" → `DynIllust`
///   2. _animationName == "Relax" OR skel name starts with "build_" → Building
///   3. owning `GameObject` named "Front"/"Back"/"Down" → `BattleFront`/`BattleBack`/`BattleDown`
///   4. fallback: atlas front count (f_, c_) >= back count (b_) → `BattleFront`, else `BattleBack`
///
/// Step 3 is required for correctness: front and back battle skeletons often
/// share a single atlas (e.g. `char_1048_orchd2`), so the atlas heuristic
/// classifies both the same way and one overwrites the other on export. `Down` is the
/// same failure with a third name: the EN census of owning GameObject names (2026-09-09,
/// 917 bundles, 2947 behaviours) reads Front, Back, Down, `Spine` (building, caught by
/// step 2) and `res_holder` plus the dynchar roots (caught by step 1), nothing else, so
/// step 3 now names the whole battle set. A name step 3 does not know still falls to
/// step 4, and `export_spine_assets` refuses to overwrite a path written earlier in the
/// run, so the next unknown name is reported rather than silently winning.
fn classify_spine(
    skel_name: &str,
    anim_name: &str,
    game_object_name: &str,
    atlas_text: &str,
) -> SpineCategory {
    let name_lower = skel_name.to_lowercase();

    // 1. Dynamic illustration
    if name_lower.starts_with("dyn_") {
        return SpineCategory::DynIllust;
    }

    // 2. Building: "Relax" animation or build_ prefix
    if anim_name == "Relax" || name_lower.starts_with("build_") {
        return SpineCategory::Building;
    }

    // 3. Battle spines hang off GameObjects literally named "Front"/"Back"
    if game_object_name.eq_ignore_ascii_case("front") {
        return SpineCategory::BattleFront;
    }
    if game_object_name.eq_ignore_ascii_case("back") {
        return SpineCategory::BattleBack;
    }
    if game_object_name.eq_ignore_ascii_case("down") {
        return SpineCategory::BattleDown;
    }

    // 4. Fallback: front vs back based on atlas region prefixes
    let atlas_lower = atlas_text.to_lowercase();
    let front_count = atlas_lower.matches("\nf_").count() + atlas_lower.matches("\nc_").count();
    let back_count = atlas_lower.matches("\nb_").count();

    if front_count >= back_count {
        SpineCategory::BattleFront
    } else {
        SpineCategory::BattleBack
    }
}

/// Export organized spine assets. Returns count of exported files.
///
/// `char_name` is the destination directory for every asset (operator bundles
/// hold one character). Pass `None` for multi-enemy pack bundles: each asset
/// then derives its own directory from its skel name via [`enemy_dir_name`],
/// grouping form variants (`enemy_1000_gopro_2`) under the base enemy id.
#[must_use]
/// Every skeleton path this process has written, with a hash of its bytes, so a second
/// asset resolving to the same path is detected instead of silently overwriting the first
/// (in one bundle through iteration order, or across two bundles through the parallel walk).
static WRITTEN_SKELS: std::sync::LazyLock<std::sync::Mutex<HashMap<std::path::PathBuf, (u64, usize)>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static COLLISIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static DUPLICATES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

enum OutputClaim {
    /// First write of this path in the run.
    New,
    /// The path was written with these exact bytes already.
    Duplicate,
    /// The path was written with different bytes; carries the kept file's byte count.
    Conflict(usize),
}

fn claim_output_path(path: &Path, bytes: &[u8]) -> OutputClaim {
    let hash = fnv1a64(bytes);
    let mut written = WRITTEN_SKELS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    match written.get(path) {
        Some((h, len)) if *h == hash && *len == bytes.len() => OutputClaim::Duplicate,
        Some((_, len)) => OutputClaim::Conflict(*len),
        None => {
            written.insert(path.to_path_buf(), (hash, bytes.len()));
            OutputClaim::New
        }
    }
}

/// Skeleton paths a second asset tried to overwrite with different bytes this run.
#[must_use]
pub fn collision_count() -> usize {
    COLLISIONS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Skeleton paths written twice with identical bytes this run.
#[must_use]
pub fn duplicate_count() -> usize {
    DUPLICATES.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn export_spine_assets(
    spine_assets: &[SpineAsset],
    output_dir: &Path,
    char_name: Option<&str>,
    resources: &HashMap<String, Vec<u8>>,
) -> usize {
    let mut count = 0;

    for asset in spine_assets {
        let dir_name = char_name.map_or_else(
            || enemy_dir_name(&asset.name),
            std::string::ToString::to_string,
        );
        let spine_dir = output_dir
            .join("spine")
            .join(asset.category.to_string())
            .join(dir_name);

        if std::fs::create_dir_all(&spine_dir).is_err() {
            continue;
        }

        // Write skel. A path already written in this run is never overwritten: a second
        // skeleton resolving to it means the category rules did not tell two assets apart,
        // and letting iteration order pick the survivor is how three token skins shipped a
        // random facing (the `Down` race, 2026-09-09). Identical bytes are a harmless
        // duplicate and are counted; different bytes are a collision, reported, skipped, and
        // fatal to the run's exit status (main.rs reads `collision_count`).
        let skel_path = spine_dir.join(format!("{}.skel", asset.name));
        match claim_output_path(&skel_path, &asset.skel_data) {
            OutputClaim::Conflict(kept) => {
                COLLISIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                eprintln!(
                    "COLLISION {}: a second skeleton ({} bytes) resolves to a path already written this run ({kept} bytes, kept); not written. classify_spine must name what tells them apart.",
                    skel_path.display(),
                    asset.skel_data.len()
                );
                continue;
            }
            OutputClaim::Duplicate => {
                DUPLICATES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            OutputClaim::New => {}
        }
        if std::fs::write(&skel_path, &asset.skel_data).is_ok() {
            count += 1;
        }

        // Write atlas.
        //
        // A skin's ENTRANCE (`_Start`) and IDLE assets are separate spine assets that land in the
        // SAME output directory, and their atlases can name the same page file at DIFFERENT sizes:
        // Kal'tsit `boc#6` declares `dyn_illust_char_003_kalts_boc#6.png` as 2348x2348 for the idle
        // and 2336x2336 for the entrance. Whichever is written last wins, and the loser's regions
        // are all scaled by the size ratio — the character renders as disconnected fragments.
        //
        // Namespace a page under the asset that declares it whenever the two names differ, and
        // rewrite the atlas text to match, so the two assets can never clobber each other.
        let mut atlas_text = asset.atlas_text.clone();
        let mut page_renames: HashMap<String, String> = HashMap::new();
        // ⚠️ Namespace as `{asset}__{page}`, NOT as `{asset}`. A MULTI-PAGE atlas names its
        // extra pages `{base}2`, `{base}3`, … (6 skins in the corpus do), and collapsing them
        // all onto the asset name would make page 2 clobber page 1 — trading one collision for
        // a worse one. The prefixed form is unique per (asset, page) by construction.
        for page in atlas_page_sizes(&asset.atlas_text).keys() {
            if *page != asset.name {
                page_renames.insert(page.clone(), format!("{}__{page}", asset.name));
            }
        }
        for (from, to) in &page_renames {
            atlas_text = atlas_text.replace(&format!("{from}.png"), &format!("{to}.png"));
        }
        let atlas_path = spine_dir.join(format!("{}.atlas", asset.name));
        if std::fs::write(&atlas_path, &atlas_text).is_ok() {
            count += 1;
        }

        // Decode textures and apply alpha merging.
        //
        // `decoded` is keyed by texture NAME, so two Texture2D objects sharing one name
        // silently overwrite each other and whichever comes last in `asset.textures` wins.
        // That is not hypothetical: Kal'tsit's `boc#6` bundle carries TWO textures called
        // `dyn_illust_char_003_kalts_boc#6`, 2348x2348 (fmt 49) and 2336x2336 (fmt 50), and
        // the `.atlas` declares 2348. Picking the 2336 one scales every region's UVs by
        // 2348/2336 and shatters the character into disconnected fragments.
        //
        // The atlas itself says which page is correct, so resolve collisions against it
        // rather than by iteration order. Same lesson as the shader map: never key on a
        // bare name when the bundle can repeat one.
        let page_sizes = atlas_page_sizes(&atlas_text);
        let mut decoded: HashMap<String, _> = HashMap::new();
        for (_, tex_val) in &asset.textures {
            match decode_texture_object(tex_val, resources) {
                Ok(Some(tex)) => match decoded.entry(tex.name.clone()) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(tex);
                    }
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        let want = page_sizes.get(&tex.name).copied();
                        let new_fits = want == Some((tex.width, tex.height));
                        let old_fits = want == Some((e.get().width, e.get().height));
                        eprintln!(
                            "  duplicate spine texture name {} — {}x{} vs {}x{}, atlas declares {}",
                            tex.name,
                            e.get().width,
                            e.get().height,
                            tex.width,
                            tex.height,
                            want.map_or("nothing".to_string(), |(w, h)| format!("{w}x{h}"))
                        );
                        if new_fits && !old_fits {
                            e.insert(tex);
                        }
                    }
                },
                Ok(None) => {}
                Err(e) => {
                    let name = tex_val["m_Name"].as_str().unwrap_or("?");
                    eprintln!("  error decoding spine texture {name}: {e}");
                }
            }
        }
        // Apply the page rename to the decoded textures so the PNG lands under the namespaced
        // filename the rewritten atlas now points at.
        //
        // The page's `[alpha]` companion MUST follow it. `merge_and_export` pairs `foo` with
        // `foo[alpha]` by name, so a renamed page whose companion kept the old name is written
        // fully opaque and the companion lands as an orphan: five split-alpha pages in the
        // corpus (Nian E2, Ch'en boc#6, Dusk nian#7, Ling E2, Phantom sale#4) rendered with
        // hard black region borders, opaque shadow rectangles and black eye glows for that
        // reason (the atlas' own `[alpha]` page is the only alpha those regions have).
        for (from, to) in &page_renames {
            if let Some(mut tex) = decoded.remove(from) {
                tex.name.clone_from(to);
                decoded.insert(to.clone(), tex);
            }
            let (from_alpha, to_alpha) = (format!("{from}[alpha]"), format!("{to}[alpha]"));
            if let Some(mut tex) = decoded.remove(&from_alpha) {
                tex.name.clone_from(&to_alpha);
                decoded.insert(to_alpha, tex);
            }
        }
        // ASSERT the invariant the frontend depends on: every page the atlas declares must be
        // written at exactly the declared size. Cheap, and it catches this whole class at export
        // instead of as a shattered skin in a corpus render.
        for (page, (w, h)) in &page_sizes {
            if let Some(tex) = decoded.get(page)
                && (tex.width, tex.height) != (*w, *h)
            {
                eprintln!(
                    "  ATLAS SIZE MISMATCH {}/{page}: atlas declares {w}x{h} but the texture is {}x{} — every region UV will be wrong",
                    asset.name, tex.width, tex.height
                );
            }
        }
        count += alpha_merge::merge_and_export(decoded, &spine_dir);

        // Export the full multi-layer scene for the live renderer. (The legacy
        // flat `[bg]` rasterization is no longer emitted — the frontend renders
        // every scene layer live.)
        count += export_scene(asset, &spine_dir, resources);

        // Export the ParticleSystems parallel to the scene, reusing the same
        // skeletonScale / cameraSize / characterSort the scene computed.
        if !asset.particles.is_empty() {
            count += super::particles::export_particles(
                &asset.name,
                &asset.particles,
                &spine_dir,
                asset.bg_skel_scale.unwrap_or(0.01),
                asset.bg_camera_size,
                asset.bg_character_sort,
                &asset.separator_part_sorts,
                resources,
            );
        }
    }

    count
}

/// Luminance stats over the OPAQUE texels a quad actually covers (per-triangle
/// barycentric grid over its UVs): `(mean, p90, p98, dark_frac)` of `max(r,g,b)`
/// normalized 0..1, where `dark_frac` is the fraction of covered opaque texels
/// that are near-black (a black backdrop authored to drop when the layer blends
/// additively). Sampling the covered texels — not the whole texture — classifies
/// a layer's content (featureless-black distortion vs bright-on-dark glass)
/// independent of how much transparent atlas space its UV bbox spans. Returns
/// `(1, 1, 1, 0)` (treated as "not dark", no correction) when too few opaque
/// texels are sampled.
fn opaque_luma(
    rgba: &[u8],
    w: u32,
    h: u32,
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> (f32, f32, f32, f32) {
    if w == 0 || h == 0 || uvs.len() < 3 || indices.len() < 3 {
        return (1.0, 1.0, 1.0, 0.0);
    }
    let mut lums: Vec<u8> = Vec::new();
    const N: usize = 8;
    let (triangles, _remainder) = indices.as_chunks::<3>();
    for tri in triangles {
        let (Some(&p0), Some(&p1), Some(&p2)) = (
            uvs.get(tri[0] as usize),
            uvs.get(tri[1] as usize),
            uvs.get(tri[2] as usize),
        ) else {
            continue;
        };
        for i in 0..=N {
            for j in 0..=(N - i) {
                let s = i as f32 / N as f32;
                let t = j as f32 / N as f32;
                let r = 1.0 - s - t;
                let u = (r * p0[0] + s * p1[0] + t * p2[0]).clamp(0.0, 1.0);
                let v = (r * p0[1] + s * p1[1] + t * p2[1]).clamp(0.0, 1.0);
                let px = (u * (w as f32 - 1.0)) as u32;
                let py = ((1.0 - v) * (h as f32 - 1.0)) as u32; // PNG top-left origin
                let idx = ((py * w + px) * 4) as usize;
                if let Some(&a) = rgba.get(idx + 3)
                    && a > 128
                {
                    lums.push(rgba[idx].max(rgba[idx + 1]).max(rgba[idx + 2]));
                }
            }
        }
    }
    if lums.len() < 20 {
        return (1.0, 1.0, 1.0, 0.0);
    }
    let sum: u32 = lums.iter().map(|&x| u32::from(x)).sum();
    let mean = sum as f32 / lums.len() as f32 / 255.0;
    let dark_frac = lums.iter().filter(|&&x| x < 45).count() as f32 / lums.len() as f32;
    lums.sort_unstable();
    let p90 = f32::from(lums[(lums.len() * 9 / 10).min(lums.len() - 1)]) / 255.0;
    let p98 = f32::from(lums[(lums.len() * 98 / 100).min(lums.len() - 1)]) / 255.0;
    (mean, p90, p98, dark_frac)
}

/// THE SKIN-SCOPED TEXTURE POOL (`DYNCHAR_TEX_POOL=1`, default OFF).
///
/// Every scene and particle export writes its textures as `<name>[scene]/<i>.png` or
/// `<name>[particles]/<i>.png`, deduplicated by source `path_id` inside that one call and
/// blind across calls. The idle scene and the `_Start` scene of one skin are separate calls
/// on the same skin directory, so a texture reachable from both (and every particle sheet
/// the two rigs share) is decoded and written twice under different numbers: 2026-09-08
/// census on the CN tree, 88.9 MB of within-skin duplicate bytes, 13.7 MB of Virtuosa's
/// 42.4, 12.2 of Cetsyr's 45.1, and every one of them cross-directory. The viewer keys its
/// texture cache on the URL, so each duplicate is also a second decode and a second GPU
/// upload (43 MB of decoded RGBA on Cetsyr).
///
/// Under the pool a decoded texture is encoded once, hashed by its encoded bytes, and
/// written to `<skin>/tex/<hash>.png` only when that file does not exist yet, whichever
/// call reaches it first. The JSON keeps its dense 0-based `tex` indices and `textureCount`
/// unchanged and gains a `textures` table, index -> path relative to the skin directory,
/// which the loaders resolve instead of `<dir>/<i>.png`. `DYNCHAR_TEX_POOL=0` reverts to the
/// per-call `<dir>/<i>.png` layout, byte for byte; the harness's control arm is that export.
/// Default ON since 2026-09-08 (Ian's ruling after the delivery gate: 0.000 on both surfaces).
pub(crate) fn tex_pool_on() -> bool {
    std::env::var("DYNCHAR_TEX_POOL").as_deref() != Ok("0")
}

/// FNV-1a, 64 bit: a stable content key for the pool file names. The exporter carries no
/// hashing crate, and a pool of a few hundred files per skin needs no more than this.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Encode a decoded RGBA texture as PNG bytes with the same encoder and settings
/// `image::save_buffer` uses today (the crate's default, deflate at its fastest level with
/// adaptive filters), so a pooled file is byte-identical to the file it replaces.
pub(crate) fn encode_png(rgba: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    use image::ImageEncoder;
    use image::codecs::png::{FilterType, PngEncoder};
    let mut out = Vec::with_capacity(rgba.len() / 4);
    // The same level every other page takes (`texture::png_compression`); at `Fast` this is
    // `PngEncoder::new`, the encoder `save_buffer` uses, byte for byte.
    PngEncoder::new_with_quality(
        &mut out,
        super::texture::png_compression(),
        FilterType::Adaptive,
    )
    .write_image(rgba, w, h, image::ExtendedColorType::Rgba8)
    .ok()?;
    Some(out)
}

/// Write one texture into the skin's pool and return its path relative to the skin
/// directory. A file that already exists under the same content hash is not rewritten.
pub(crate) fn pool_write_png(spine_dir: &Path, rgba: &[u8], w: u32, h: u32) -> Option<String> {
    let bytes = encode_png(rgba, w, h)?;
    let rel = format!("tex/{:016x}.png", fnv1a64(&bytes));
    let path = spine_dir.join(&rel);
    if !path.exists() {
        std::fs::create_dir_all(path.parent()?).ok()?;
        std::fs::write(&path, &bytes).ok()?;
    }
    Some(rel)
}

/// The path index `idx` has always had, `<tex_dir>/<idx>.png`, relative to the skin dir.
fn legacy_tex_rel(tex_dir: &Path, spine_dir: &Path, idx: usize) -> String {
    let dir = tex_dir.strip_prefix(spine_dir).unwrap_or(tex_dir);
    format!("{}/{idx}.png", dir.display())
}

/// Save one decoded texture: through the pool when it is on, else as `<tex_dir>/<idx>.png`
/// exactly as before. Returns the relative path the `textures` table records for `idx` and
/// whether a file landed.
pub(crate) fn save_tex(
    tex_dir: &Path,
    spine_dir: &Path,
    idx: usize,
    rgba: &[u8],
    w: u32,
    h: u32,
) -> (String, bool) {
    if tex_pool_on()
        && let Some(rel) = pool_write_png(spine_dir, rgba, w, h)
    {
        return (rel, true);
    }
    let ok = super::texture::write_png(&tex_dir.join(format!("{idx}.png")), rgba, w, h).is_ok();
    (legacy_tex_rel(tex_dir, spine_dir, idx), ok)
}

/// Decode a Ram MASK (`_DissolveTex`/`_DisturbTex`) into the scene's shared texture
/// list, returning its index. Deduped by source `path_id` alongside the drawn artwork, so
/// a mask that IS the layer's own `_MainTex` costs no extra slot. Unlike the artwork it
/// is never alpha-merged and never enters `tex_px`: the luminance classifier reads that
/// map to judge what a layer LOOKS like, and a noise mask is shader input, not paint.
// Nine arguments: the pool added the skin directory and the index -> path table beside
// the per-call index map they extend; bundling them would be a struct for one caller.
#[allow(clippy::too_many_arguments)]
fn resolve_scene_mask(
    pid: Option<i64>,
    val: Option<&Value>,
    resources: &HashMap<String, Vec<u8>>,
    tex_dir: &Path,
    spine_dir: &Path,
    tex_index: &mut HashMap<i64, usize>,
    tex_names: &mut Vec<String>,
    next_idx: &mut usize,
    saved: &mut usize,
) -> Option<usize> {
    let (pid, val) = (pid?, val?);
    if let Some(&i) = tex_index.get(&pid) {
        return Some(i);
    }
    let Ok(Some(tex)) = decode_texture_object(val, resources) else {
        return None;
    };
    let idx = *next_idx;
    let (rel, ok) = save_tex(tex_dir, spine_dir, idx, &tex.rgba, tex.width, tex.height);
    if ok {
        *saved += 1;
    }
    tex_names.push(rel);
    tex_index.insert(pid, idx);
    *next_idx += 1;
    Some(idx)
}

/// Two `_MainTex_ST` tuples select the same atlas sub-rect (tolerant of float noise).
fn st_eq(a: [f64; 4], b: [f64; 4]) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-4)
}

/// CARD BACKDROP TRANSFORM (2026-09-01). The windowed card composites the STATIC
/// illustration behind the animated scene, and needs the exact art-to-scene placement.
/// The scene layers carry authored uv (a rect of an exported scene texture) and pos
/// (a rect in scene px), so atlas-to-scene is authored; the one missing link is the
/// correspondence between those texture crops and the skinpack/chararts PNG, which are
/// different images. That correspondence is COMPUTED here, at export time, per key:
/// masked normalized cross-correlation of the largest opaque layers' texture crops
/// against the illustration over a geometric scale sweep, refined, and taken by
/// consensus. The result is a similarity (uniform scale + offset), written into the
/// scene JSON as `backdropScale` (scene px per art px) and `backdropOffsetPx` (the
/// art CENTRE in authored Y-up scene coords). Derivation, not choice: every input is
/// an authored asset or authored geometry, and no per-skin value exists in code.
/// Absent fields (no art file, no confident consensus) leave the frontend on its
/// camera-extent fallback. Art root: `DYNCHAR_ART_ROOT` env, else `../../../textures`
/// relative to the spine dir (the installed output tree).
fn derive_backdrop_transform(
    layers: &[serde_json::Value],
    tex_dir: &Path,
    spine_dir: &Path,
    tex_names: &[String],
) -> Option<(f64, [f64; 2])> {
    use image::imageops::{FilterType, resize};
    // PARKED (2026-09-01): the instrument FAILED its known-answer control. Three matcher
    // variants (luma NCC, luma NCC + refine, gradient NCC + derived scale band) recover the
    // truth on shu_nian#11 (1.1506 vs 1.10 measured) but not on chen2_2 or dusk_nian#12,
    // whose scene textures are DEFOCUSED REPAINTS of the illustration rather than crops of
    // it, so texture-vs-art matching cannot anchor their scale. A failed control means the
    // output must not ship: OFF unless DYNCHAR_BD_DERIVE=1 (measurement runs only), so every
    // default export stays byte-identical. The refutation and the surviving path (a
    // character-anchored correspondence, which needs feature matching the exporter does not
    // have) are recorded in docs/DYNCHAR_GATES.md.
    std::env::var("DYNCHAR_BD_DERIVE").ok()?;
    let key = spine_dir.file_name()?.to_str()?.to_string();
    let op = key.rsplit_once('_').map(|(a, _)| a)?.to_string();
    let art_root = std::env::var("DYNCHAR_ART_ROOT").map_or_else(
        |_| spine_dir.join("../../../textures"),
        std::path::PathBuf::from,
    );
    let art_path = ["chararts", "skinpack"]
        .iter()
        .map(|d| art_root.join(d).join(&op).join(format!("{key}.png")))
        .find(|p| p.exists())?;
    let art = image::open(&art_path).ok()?.to_rgba8();
    let (aw, ah) = (f64::from(art.width()), f64::from(art.height()));
    // Luma pyramid of the illustration at two working widths.
    let mk = |w: u32| -> (Vec<f32>, u32, u32) {
        let h = ((ah * f64::from(w) / aw).round() as u32).max(1);
        let im = resize(&art, w, h, FilterType::Triangle);
        let v = im
            .pixels()
            .map(|p| {
                let a = f32::from(p[3]) / 255.0;
                (0.299 * f32::from(p[0]) + 0.587 * f32::from(p[1]) + 0.114 * f32::from(p[2])) * a
            })
            .collect();
        (v, w, h)
    };
    let (art320, a320w, a320h) = mk(320);
    let (art640, a640w, a640h) = mk(640);
    // Candidate layers: the largest opaque non-additive quads.
    struct Cand {
        idx: u64,
        u0: f64,
        v0: f64,
        u1: f64,
        v1: f64,
        px: f64,
        py: f64,
        pw: f64,
        ph: f64,
    }
    let mut cands: Vec<Cand> = Vec::new();
    for l in layers {
        if l.get("additive").and_then(serde_json::Value::as_bool) == Some(true) {
            continue;
        }
        if let Some(t) = l.get("tint").and_then(serde_json::Value::as_array)
            && t.len() == 4
            && t[3].as_f64().unwrap_or(1.0) < 0.98
        {
            continue;
        }
        let idx = l.get("tex").and_then(serde_json::Value::as_u64)?;
        let nums = |k: &str, step0: usize| -> Vec<f64> {
            l.get(k)
                .and_then(serde_json::Value::as_array)
                .map(|a| {
                    a.iter()
                        .skip(step0)
                        .step_by(2)
                        .filter_map(serde_json::Value::as_f64)
                        .collect()
                })
                .unwrap_or_default()
        };
        let (xs, ys) = (nums("pos", 0), nums("pos", 1));
        let (us, vs) = (nums("uv", 0), nums("uv", 1));
        if xs.len() < 3 || us.len() < 3 {
            continue;
        }
        let mm = |v: &[f64]| {
            (
                v.iter().copied().fold(f64::INFINITY, f64::min),
                v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        };
        let ((x0, x1), (y0, y1)) = (mm(&xs), mm(&ys));
        let ((u0, u1), (v0, v1)) = (mm(&us), mm(&vs));
        if !(x1 - x0).is_finite() || (x1 - x0) <= 1.0 || (u1 - u0) <= 0.001 {
            continue;
        }
        cands.push(Cand {
            idx,
            u0,
            v0,
            u1,
            v1,
            px: x0,
            py: y0,
            pw: x1 - x0,
            ph: y1 - y0,
        });
    }
    cands.sort_by(|a, b| (b.pw * b.ph).total_cmp(&(a.pw * a.ph)));
    cands.truncate(5);
    // Masked NCC of a template over a luma image, best position at a fixed step.
    let ncc_best = |img: &[f32],
                    iw: u32,
                    ih: u32,
                    tpl: &[f32],
                    tw: u32,
                    th: u32,
                    step: usize,
                    x0: i64,
                    y0: i64,
                    x1: i64,
                    y1: i64|
     -> (f32, i64, i64) {
        let (iw, ih) = (i64::from(iw), i64::from(ih));
        let (twi, thi) = (i64::from(tw), i64::from(th));
        let n = (tw * th) as f32;
        let tmean = tpl.iter().sum::<f32>() / n;
        let tvar: f32 = tpl.iter().map(|v| (v - tmean) * (v - tmean)).sum();
        if tvar < 1e-3 {
            return (-2.0, 0, 0);
        }
        let mut best = (-2.0f32, 0i64, 0i64);
        let mut y = y0.max(0);
        while y + thi <= ih.min(y1 + thi) {
            let mut x = x0.max(0);
            while x + twi <= iw.min(x1 + twi) {
                let mut s = 0.0f32;
                let mut ss = 0.0f32;
                let mut sc = 0.0f32;
                for ty in 0..thi {
                    let row = ((y + ty) * iw + x) as usize;
                    let trow = (ty * twi) as usize;
                    for tx in 0..twi as usize {
                        let a = img[row + tx];
                        let b = tpl[trow + tx];
                        s += a;
                        ss += a * a;
                        sc += a * b;
                    }
                }
                let mean = s / n;
                let var = ss - s * mean;
                if var > 1e-3 {
                    let score = (sc - s * tmean) / (var * tvar).sqrt();
                    if score > best.0 {
                        best = (score, x, y);
                    }
                }
                x += step as i64;
            }
            y += step as i64;
        }
        best
    };
    let mut accepted: Vec<(f32, f64, f64, f64)> = Vec::new(); // score, scene_per_art, ox, oy
    for c in &cands {
        // Read the page back through the `textures` table when the pool named it, else
        // from its legacy `<tex_dir>/<idx>.png` slot.
        let page = usize::try_from(c.idx)
            .ok()
            .and_then(|i| tex_names.get(i))
            .map_or_else(
                || tex_dir.join(format!("{}.png", c.idx)),
                |rel| spine_dir.join(rel),
            );
        let Ok(tex) = image::open(page) else {
            continue;
        };
        let tex = tex.to_rgba8();
        let (tw_full, th_full) = (f64::from(tex.width()), f64::from(tex.height()));
        // Two vertical readings: exported UVs are GL-style (frontend samples 1-v), but
        // the safe move is to score both orientations and keep the better one.
        for flip in [true, false] {
            let (cy0, cy1) = if flip {
                ((1.0 - c.v1) * th_full, (1.0 - c.v0) * th_full)
            } else {
                (c.v0 * th_full, c.v1 * th_full)
            };
            let (cx0, cx1) = (c.u0 * tw_full, c.u1 * tw_full);
            let (cw, ch) = (cx1 - cx0, cy1 - cy0);
            if cw < 24.0 || ch < 24.0 {
                continue;
            }
            let crop = image::imageops::crop_imm(
                &tex,
                cx0.max(0.0) as u32,
                cy0.max(0.0) as u32,
                (cw as u32).min(tex.width()),
                (ch as u32).min(tex.height()),
            )
            .to_image();
            let mut best: Option<(f32, f64, i64, i64)> = None; // score, s_art_per_croppx(at320), x, y
            let mut twf = 14.0f64;
            #[allow(clippy::while_float)]
            while twf <= 300.0 {
                let tw = twf as u32;
                let th = ((ch * twf / cw).round() as u32).max(4);
                if th <= a320h.saturating_sub(1) && tw < a320w {
                    let tpl = resize(&crop, tw, th, FilterType::Triangle);
                    let tl: Vec<f32> = tpl
                        .pixels()
                        .map(|p| {
                            let a = f32::from(p[3]) / 255.0;
                            (0.299 * f32::from(p[0])
                                + 0.587 * f32::from(p[1])
                                + 0.114 * f32::from(p[2]))
                                * a
                        })
                        .collect();
                    let (sc, bx, by) = ncc_best(
                        &art320,
                        a320w,
                        a320h,
                        &tl,
                        tw,
                        th,
                        2,
                        0,
                        0,
                        i64::from(a320w),
                        i64::from(a320h),
                    );
                    if best.is_none() || sc > best.unwrap().0 {
                        best = Some((sc, twf / cw, bx, by));
                    }
                }
                twf *= 1.09;
            }
            let Some((sc320, s320, bx, by)) = best else {
                continue;
            };
            if sc320 < 0.45 {
                continue;
            }
            // Refine at 640: same parametrization, double coords.
            let mut fine: Option<(f32, f64, i64, i64)> = None;
            let mut fs = s320 * 0.94;
            #[allow(clippy::while_float)]
            while fs <= s320 * 1.06 {
                let tw = ((cw * fs * 2.0).round() as u32).max(8);
                let th = ((ch * fs * 2.0).round() as u32).max(8);
                if tw < a640w && th < a640h {
                    let tpl = resize(&crop, tw, th, FilterType::Triangle);
                    let tl: Vec<f32> = tpl
                        .pixels()
                        .map(|p| {
                            let a = f32::from(p[3]) / 255.0;
                            (0.299 * f32::from(p[0])
                                + 0.587 * f32::from(p[1])
                                + 0.114 * f32::from(p[2]))
                                * a
                        })
                        .collect();
                    let (sc, fx, fy) = ncc_best(
                        &art640,
                        a640w,
                        a640h,
                        &tl,
                        tw,
                        th,
                        1,
                        bx * 2 - 8,
                        by * 2 - 8,
                        bx * 2 + 8,
                        by * 2 + 8,
                    );
                    if fine.is_none() || sc > fine.unwrap().0 {
                        fine = Some((sc, fs, fx, fy));
                    }
                }
                fs *= 1.015;
            }
            let Some((score, s_art, fx, fy)) = fine else {
                continue;
            };
            if score < 0.55 {
                continue;
            }
            // s_art: art-320-pyramid px per crop px. Native art px per crop px scales by
            // the 640 template being 2 x s_art wide and native being aw/640 per 640 px.
            let art_per_crop_native = s_art * 2.0 * (aw / f64::from(a640w));
            let scene_per_crop = c.pw / cw;
            let scene_per_art = scene_per_crop / art_per_crop_native;
            // Matched region centre: fx is art-640 px, template width there is cw*s_art*2.
            let up = aw / f64::from(a640w);
            let mx = (fx as f64 + cw * s_art) * up;
            let my = (fy as f64 + ch * s_art) * up;
            // Scene coords (Y-up) of the ART CENTRE.
            let (cxs, cys) = (c.px + c.pw / 2.0, c.py + c.ph / 2.0);
            let ox = cxs + (aw / 2.0 - mx) * scene_per_art;
            let oy = cys - (ah / 2.0 - my) * scene_per_art;
            accepted.push((score, scene_per_art, ox, oy));
        }
    }
    if accepted.is_empty() {
        return None;
    }
    accepted.sort_by(|a, b| a.1.total_cmp(&b.1));
    let med_scale = accepted[accepted.len() / 2].1;
    let close: Vec<&(f32, f64, f64, f64)> = accepted
        .iter()
        .filter(|a| (a.1 / med_scale - 1.0).abs() < 0.04)
        .collect();
    let pick = |sel: fn(&(f32, f64, f64, f64)) -> f64, from: &[&(f32, f64, f64, f64)]| -> f64 {
        let mut v: Vec<f64> = from.iter().map(|a| sel(a)).collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    if close.len() >= 2 {
        return Some((
            pick(|a| a.1, &close),
            [pick(|a| a.2, &close), pick(|a| a.3, &close)],
        ));
    }
    let best = accepted.iter().max_by(|a, b| a.0.total_cmp(&b.0))?;
    (best.0 >= 0.70).then_some((best.1, [best.2, best.3]))
}

/// Export the full multi-layer scene for the live renderer. Every non-character
/// mesh quad becomes a textured 2D mesh in spine-authored pixels (Y-up, origin at
/// the skeleton root), geometry inlined in `{name}[scene].json`, textures written
/// to a `{name}[scene]/` folder (deduped by source `path_id`). The frontend draws
/// these as Pixi meshes in `sort` order and inserts the animated character spine
/// at `characterSort`. Returns the number of files written.
fn export_scene(
    asset: &SpineAsset,
    spine_dir: &Path,
    resources: &HashMap<String, Vec<u8>>,
) -> usize {
    // A skin with no scene meshes can still need a bare `[scene].json` carrying
    // just the authored camera frame, so the frontend can align a static-art
    // backdrop for it (e.g. Siege, whose whole illustration is in the spine).
    let has_frame = asset.bg_camera_offset.is_some() && asset.bg_camera_view.is_some();
    if asset.bg_quads.is_empty() && !has_frame {
        return 0;
    }
    let skel_scale = asset.bg_skel_scale.unwrap_or(0.01) as f32;
    if skel_scale == 0.0 {
        return 0;
    }
    let inv = 1.0 / skel_scale;
    let dbg = std::env::var("SCENE_DEBUG").is_ok();
    if dbg {
        eprintln!(
            "[scene] {} : {} collected quad(s)",
            asset.name,
            asset.bg_quads.len()
        );
    }

    let tex_dir = spine_dir.join(format!("{}[scene]", asset.name));
    if !asset.bg_quads.is_empty() {
        std::fs::create_dir_all(&tex_dir).ok();
    }

    // Draw order: ascending sort, farther Z first within ties (matches raster).
    let mut order: Vec<&BgQuad> = asset.bg_quads.iter().collect();
    order.sort_by(|a, b| a.sort.cmp(&b.sort).then(b.z.total_cmp(&a.z)));

    let mut tex_index: HashMap<i64, usize> = HashMap::new();
    // Decoded pixels per texture (RGBA, width, height), kept for the per-quad
    // blend-class analysis below; freed when the scene ends.
    let mut tex_px: HashMap<i64, (Vec<u8>, u32, u32)> = HashMap::new();
    let mut layers: Vec<serde_json::Value> = Vec::new();
    let mut next_idx = 0usize;
    let mut saved = 0usize;
    // Index -> path relative to the skin directory (see `tex_pool_on`).
    let mut tex_names: Vec<String> = Vec::new();
    // Signatures of already-emitted layers, to drop exact duplicates (some scenes
    // stack identical quad GameObjects, e.g. Skadi "Red Countess" — frozen they
    // just overdraw, and double-brighten when additive).
    let mut seen_sigs: std::collections::HashSet<u64> = std::collections::HashSet::new();
    // Same artwork, minus everything time-varying: texture, depth, blend, geometry,
    // tint. An entrance scene bundles BOTH prefab roots, so the same painted quad
    // routinely appears twice — but only the copy the `_Start` clip actually drives
    // carries a colour curve or a reveal window. The undriven twin would then paint
    // at full static tint for the whole cinematic, double-exposing the frame
    // (Mlynar's backdrop and his white `bg_glow_01` each gained such a twin once
    // clip bindings were correctly scoped to their owning Animator). Map the bare
    // signature to the emitted layer so an unanimated twin can yield to an animated
    // one — in EITHER arrival order.
    let mut bare_sigs: HashMap<u64, (usize, bool)> = HashMap::new();

    // Camera-frame extent in authored px (2 * cameraSize half-height), for the
    // oversize test below.
    let frame_extent = asset.bg_camera_size.map(|c| 2.0 * c as f32 * inv);

    // Textures actually emitted by this skin's PARTICLE systems (main + trail).
    // These are the dynamic effects (snow, sparks, glow flecks) the live
    // `[particles]` renderer draws; the scene mesh may ALSO carry a frozen
    // keyframe copy of them (baked particle quads) that must be dropped so they
    // don't stamp static garbage over the animated version.
    // Keyed by texture, the set of `_MainTex_ST` sub-rects the particle systems actually
    // SAMPLE. Two systems (or a system and a scene quad) routinely share one atlas while
    // reading different cells of it, so the texture alone doesn't identify the artwork.
    let particle_tex_st: HashMap<i64, Vec<[f64; 4]>> = {
        let mut m: HashMap<i64, Vec<[f64; 4]>> = HashMap::new();
        for p in &asset.particles {
            for (pid, st) in [(p.tex_pid, p.tex_st), (p.trail_tex_pid, p.trail_tex_st)] {
                if let Some(pid) = pid {
                    let e = m.entry(pid).or_default();
                    if !e.iter().any(|s| st_eq(*s, st)) {
                        e.push(st);
                    }
                }
            }
        }
        m
    };

    // Detect frozen particle-burst quads. A texture instanced as many small quads
    // COULD be an effect burst (snow flecks, sparks, an explosion's frames — e.g.
    // SilverAsh the Reignfrost's `explosive`/`snow`) baked into the mesh; frozen,
    // the copies stamp opaque garbage over the live particle version. BUT the same
    // "many small repeated quads" shape ALSO describes legitimate STRUCTURAL
    // geometry — a tiled surface (Virtuosa's piano-key staircase, crystal steps,
    // pedestals: 200+ small quads of ~9 textures). Size/count alone can't tell them
    // apart and wrongly dropped Virtuosa's entire throne. The reliable signal: a
    // frozen burst's texture is ALSO emitted by a PARTICLE system (that's what makes
    // it a duplicate); a structural tile is a mesh-only texture. So gate the drop on
    // `particle_tex_pids` — only ever remove a small-repeated texture that a live
    // particle system re-draws. Structural tiles (no particle counterpart) are kept.
    // A genuine painted backdrop is 1-2 large quads, and the `< half the camera
    // frame` size bound protects large parallax copies (Ines "Under the Flaming
    // Dome"'s full-frame `bg_01`) regardless.
    let burst_tex: std::collections::HashSet<i64> = {
        let mut stats: HashMap<i64, (u32, f32)> = HashMap::new();
        for quad in &order {
            let (mut xmn, mut ymn, mut xmx, mut ymx) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for &p in &quad.mesh.positions {
                let w = quad.world.point(p);
                let (x, y) = (w[0] * inv, w[1] * inv);
                xmn = xmn.min(x);
                xmx = xmx.max(x);
                ymn = ymn.min(y);
                ymx = ymx.max(y);
            }
            let ext = (xmx - xmn).max(ymx - ymn);
            let e = stats.entry(quad.tex_pid).or_insert((0, 0.0));
            e.0 += 1;
            e.1 = e.1.max(ext);
        }
        stats
            .iter()
            .filter(|(pid, (count, max_ext))| {
                particle_tex_st.contains_key(pid)
                    && *count >= 4
                    && frame_extent.is_some_and(|fe| fe > 0.0 && *max_ext < fe * 0.5)
            })
            .map(|(pid, _)| *pid)
            .collect()
    };

    // CROSS-ROOT DUPLICATES. An entrance scene bundles BOTH prefab roots' layers, and for
    // several skins the idle root is a near-copy of the entrance root's scenery: Muelsyse ships
    // `bg_sky_01`, `bg_cloud_01`, `bg_water_01`, `bg_tree_01`, every `water_line_*` and every
    // `ribbon_*` TWICE — once from her own root (always drawn) and once from the idle root
    // (gated by `root_reveal_from`). The gate reveals the second copy without retiring the
    // first, so from the transform beat onward every one of those layers draws twice, and an
    // alpha-blended layer composited over itself is markedly heavier than the game's single
    // draw. That is why her error STEPS at `entranceTransform` (13.0) and never recovers:
    // t=13 scores 13.2 and t=14..19 sit at 23-30.
    //
    // The game swaps worlds rather than adding one, so the redundant copy must not be emitted.
    // Keep the OWN-root layer (it is on screen for the whole cinematic) and drop the cross-root
    // twin. Measured mue **17.220 -> 11.755**, every other reference bit-identical.
    //
    // The key is the source GameObject NAME alone, and that was measured against tighter keys
    // rather than assumed — the two roots re-author the same object with different draw data:
    //
    //     (name, sort, tex)  -> mue 12.458      (name, sort) -> 12.435      (name) -> 11.755
    //
    // `ribbon_19` is the case that shows why: both roots carry it at sort -5, but the idle copy
    // binds a different `_MainTex` (4 vs 12), so any key including the texture leaves it doubled.
    //
    // ⚠️ Only ever drops a TWIN. A cross-root layer with NO own-root counterpart is genuine extra
    // scenery and is untouched — Cello's cross-root layers are distinct content (removing their
    // gate moves t=2/t=5 and leaves her worst beat bit-identical), and she scores 18.006 either
    // way. Nothing here fires for a skin whose two roots hold different scenery.
    let own_root_keys: std::collections::HashSet<String> = order
        .iter()
        .filter(|q| q.root_reveal_from.is_none())
        .map(|q| q.go_name.clone())
        .collect();
    let mut dropped_twins = 0usize;
    // (reported after the loop so a corpus export shows where it fires)

    for quad in order {
        // Cross-root duplicate of a layer we already draw — see `own_root_keys` above.
        if quad.root_reveal_from.is_some() && own_root_keys.contains(&quad.go_name) {
            dropped_twins += 1;
            continue;
        }
        // A frozen burst copy samples the SAME atlas sub-rect as the particle system it
        // duplicates. A quad reading a DIFFERENT `_MainTex_ST` rect of a shared atlas is
        // distinct art and must survive: Virtuosa "Diversity in Oneness"'s `window/lan_01`
        // + `lan_add` (the white rhombus and its blue rain-streak overlay) share texture
        // `l2d_cello_46` with a starburst emitter, but the emitter crops to the top quarter
        // (ST y=0.25) while the meshes read the whole sheet (ST identity).
        if burst_tex.contains(&quad.tex_pid)
            && particle_tex_st
                .get(&quad.tex_pid)
                .is_some_and(|sts| sts.iter().any(|s| st_eq(*s, quad.st)))
        {
            if dbg {
                eprintln!(
                    "  DROP[burst-sprite] '{}' sort={}",
                    quad.tex_val["m_Name"].as_str().unwrap_or(""),
                    quad.sort
                );
            }
            continue;
        }
        // Skip non-visual helper layers (coverage masks, distortion maps) —
        // UNLESS the entrance cinematic sequences the layer: a clip-driven layer is a
        // deliberate visual beat, not a helper (Mlynar's white-transition flash samples
        // `mask_09`).
        //
        // ⚠️ "Sequenced" deliberately means an active WINDOW ONLY, not a colour curve.
        // Extending it to `quad.color_curve.is_none()` was tried and is CATASTROPHIC:
        // Whislash-alter's `baizhuanchang_02` is a sort-100 full-frame plane sampling
        // `mask_09` with a 26-key `_MainColor` alpha curve and NO window, and admitting it
        // takes her 33.431 -> 80.603 (r .736 -> .377).
        //
        // The reason is that its curve's times are CLIP-LOCAL: they come from the 0.83s
        // sub-clip `char_1038_whitw2_start_04`, run 0.00 -> 0.83 with alpha 0 -> 1, and
        // nothing tells us when that sub-clip PLAYS. Applied on the entrance timeline the
        // plane reaches full opacity at 0.83s and, unwindowed, holds it for the rest of the
        // cinematic. Kal'tsit's equivalent plane survives the filter only because her curve
        // comes from the cinematic clip itself, so its times are already entrance-global
        // (13.8 -> 14.67) AND it carries a window.
        //
        // Fixing this needs sub-clip PLACEMENT, which we do not currently read — not a
        // looser filter.
        let name_l = quad.tex_val["m_Name"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase();
        if (name_l.contains("mask") || name_l.ends_with("_dm"))
            && quad.active_from.is_none()
            && quad.active_until.is_none()
        {
            if dbg {
                eprintln!("  DROP[mask/_dm] '{name_l}' sort={}", quad.sort);
            }
            continue;
        }

        // 2D vertices in authored pixels (Y-up), flattened [x0,y0,x1,y1,...].
        let mut pos: Vec<f32> = Vec::with_capacity(quad.mesh.positions.len() * 2);
        let (mut xmn, mut ymn, mut xmx, mut ymx) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &p in &quad.mesh.positions {
            let w = quad.world.point(p);
            let (x, y) = (w[0] * inv, w[1] * inv);
            pos.push(x);
            pos.push(y);
            xmn = xmn.min(x);
            xmx = xmx.max(x);
            ymn = ymn.min(y);
            ymx = ymx.max(y);
        }
        // Drop degenerate quads collapsed to a line/point (zero-area) — they carry
        // no image but a frozen animation keyframe can leave them as stray black
        // slivers (e.g. Ines "Melodic Flutter").
        if (xmx - xmn) < 1.0 || (ymx - ymn) < 1.0 {
            if dbg {
                eprintln!(
                    "  DROP[degenerate] '{name_l}' sort={} size={:.0}x{:.0}",
                    quad.sort,
                    xmx - xmn,
                    ymx - ymn
                );
            }
            continue;
        }
        // Drop absurdly-oversized fill/fx quads (full-screen colour fills, camera-
        // panning haze) authored many times the camera frame: frozen they blanket
        // the scene (e.g. Ines "Melodic Flutter"'s giant black + additive fills).
        // Legitimate backdrops are authored only a few× the frame.
        if let Some(fe) = frame_extent {
            let extent = (xmx - xmn).max(ymx - ymn);
            if fe > 0.0 && extent > fe * 10.0 {
                if dbg {
                    // The full bbox is an OUTLIER-SENSITIVE statistic: a mesh whose art
                    // sits in frame but that carries a few stray/degenerate vertices
                    // parked far off-origin measures as "62x the frame" and is dropped
                    // as a fill. Report the 2nd..98th percentile extent alongside it —
                    // if the core is small, the bbox is lying and this drop is a bug.
                    let pct = |mut v: Vec<f32>| -> (f32, f32) {
                        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let n = v.len();
                        if n == 0 {
                            return (0.0, 0.0);
                        }
                        (
                            v[(n as f32 * 0.02) as usize],
                            v[((n - 1) as f32 * 0.98) as usize],
                        )
                    };
                    let (xlo, xhi) = pct(pos.iter().step_by(2).copied().collect());
                    let (ylo, yhi) = pct(pos.iter().skip(1).step_by(2).copied().collect());
                    let core = (xhi - xlo).max(yhi - ylo);
                    eprintln!(
                        "  DROP[oversize] '{name_l}' sort={} extent={extent:.0} frame={fe:.0} ({:.1}x)  \
                         verts={} core_p2_p98={core:.0} ({:.1}x frame)",
                        quad.sort,
                        extent / fe,
                        pos.len() / 2,
                        core / fe
                    );
                }
                continue;
            }
        }
        // Drop exact-duplicate layers: same texture, depth, blend, and projected
        // geometry. Hash a rounded signature so tiny float noise still collapses.
        //
        // The signature must cover everything that makes two co-located quads render
        // DIFFERENTLY, not just their geometry — otherwise a pair of stacked
        // full-screen planes that share one texture but carry different tints,
        // colour animations or reveal windows collapses into whichever the object
        // order happened to visit first. Skadi the Corrupting Heart stacks exactly
        // that: `heip_01` (黑屏, the fade-to-BLACK) and `baise_01` (白色, the WHITE
        // handoff flash) are the same `mask_09` quad at the same sort, differing only
        // in material tint and their animated alpha windows — so the black fade was
        // silently swallowed by the white flash.
        {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            let mut q = |v: f32| (f64::from(v) * 1000.0).round() as i64;
            quad.tex_pid.hash(&mut h);
            quad.sort.hash(&mut h);
            quad.additive.hash(&mut h);
            for &v in &pos {
                (v.round() as i64).hash(&mut h);
            }
            quad.mesh.indices.hash(&mut h);
            // Visual identity: static tint, blend factors, and the alpha texture.
            for &c in &quad.tint {
                q(c).hash(&mut h);
            }
            q(quad.src_blend as f32).hash(&mut h);
            q(quad.dst_blend as f32).hash(&mut h);
            // Timing identity: the clip reveal window. `root_reveal_from` is
            // deliberately EXCLUDED — an entrance scene bundles both prefab roots, so
            // the same painted quad routinely appears twice, once ungated (entrance
            // root) and once cross-root-gated (idle root). Those are the same artwork
            // and collapsing them is the whole point of this pass; keeping both would
            // draw the layer twice from the transform beat onward.
            quad.active_from.map(&mut q).hash(&mut h);
            quad.active_until.map(&mut q).hash(&mut h);
            // Animation identity: the colour and UV curves drive what the layer shows
            // over time, so two quads running different curves are never duplicates.
            if let Some(cc) = &quad.color_curve {
                cc.len().hash(&mut h);
                for &(t, rgba) in cc {
                    q(t).hash(&mut h);
                    for &c in &rgba {
                        q(c).hash(&mut h);
                    }
                }
            }
            if let Some(sc) = &quad.st_curve {
                sc.len().hash(&mut h);
                for &(t, st) in sc {
                    q(t).hash(&mut h);
                    for &c in &st {
                        q(c).hash(&mut h);
                    }
                }
            }
            if let Some([u, v]) = quad.uv_scroll {
                q(u).hash(&mut h);
                q(v).hash(&mut h);
            }
            // EXPERIMENT (`DYNCHAR_KEEP_DUPLICATES=1`, default OFF): keep exact-duplicate quads.
            //
            // Unity draws BOTH copies, so for a semi-transparent or ADDITIVE plane the pair
            // accumulates — collapsing them to one halves the wash. Mlynar stacks a pure-white
            // additive `bg_02` at sort 2 with a dropped twin, and the game's frame at t=13 is
            // heavily washed where ours is clean. This gate makes that measurable without
            // rebuilding.
            // Exact-duplicate quads are dropped only when ADDITIVE.
            //
            // Unity draws every copy, so a duplicate is not free: an ADDITIVE quad drawn twice
            // doubles its light, which is visibly wrong, while an alpha-blend quad drawn twice is
            // genuinely stacked art (two passes at alpha a give 1-(1-a)^2 coverage, not a).
            // Collapsing both was costing real stacking.
            //
            // The corpus splits exactly along that line: Virtuosa's 66 duplicates and Skadi's 22
            // are ALL alpha-blend, while Mlynar's 12 are half additive. Measured against the
            // (re-aligned) oracle, keeping ALL duplicates gave cel -0.448 but mly +2.165 — the
            // gain is entirely on the alpha-blend side and the loss entirely on the additive side.
            //
            // `DYNCHAR_KEEP_DUPLICATES=1` keeps every duplicate (the blanket variant, for A/B);
            // `DYNCHAR_DROP_ALL_DUPLICATES=1` restores the old drop-everything behaviour.
            let keep_dupes = std::env::var("DYNCHAR_KEEP_DUPLICATES").is_ok();
            let drop_all_dupes = std::env::var("DYNCHAR_DROP_ALL_DUPLICATES").is_ok();
            let dedup_this = if keep_dupes {
                false
            } else if drop_all_dupes {
                true
            } else {
                quad.additive
            };
            if !seen_sigs.insert(h.finish()) && dedup_this {
                if dbg {
                    eprintln!("  DROP[duplicate] '{name_l}' sort={}", quad.sort);
                }
                continue;
            }
        }
        // Cross-root twin resolution (see `bare_sigs`). The bare signature stops one
        // step short of the full one: it omits the reveal window and the colour/ST
        // curves, so an animated layer and its undriven copy collide here even though
        // they differ above. Two ANIMATED quads never collide this way in a harmful
        // sense — they keep their own entries and both survive, which is what keeps
        // Mlynar's two `xuewen24_dm_new` reveals (different windows, same artwork)
        // distinct.
        let bare_animated = quad.color_curve.is_some()
            || quad.st_curve.is_some()
            || quad.active_from.is_some()
            || quad.active_until.is_some();
        let bare_sig = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            let q = |v: f32| (f64::from(v) * 1000.0).round() as i64;
            quad.tex_pid.hash(&mut h);
            quad.sort.hash(&mut h);
            quad.additive.hash(&mut h);
            for &v in &pos {
                (v.round() as i64).hash(&mut h);
            }
            quad.mesh.indices.hash(&mut h);
            for &c in &quad.tint {
                q(c).hash(&mut h);
            }
            q(quad.src_blend as f32).hash(&mut h);
            q(quad.dst_blend as f32).hash(&mut h);
            h.finish()
        };
        // An unanimated twin of an already-emitted animated layer is the undriven
        // prefab-root copy: skip it outright.
        let mut replace_at = None;
        // ⛔ Do NOT add an "expiry handover" here — i.e. keeping the undriven twin and revealing
        // it at the driven copy's `activeUntil`. It was built and MEASURED (2026-08-09) on the
        // case that motivated it, Civilight Eterna's three full-frame lavender fields expiring at
        // t=12.0: the re-export changed **0 files** (no dynchar has a twin in that state), and the
        // equivalent renderer-side probe `?statictail=1` — let every expired layer keep painting
        // at its static idle tint — measured 32.138 → 32.216, i.e. inert-to-worse. Her t=12 cut is
        // `Transition_black_01`'s SECOND `m_IsActive` window, which the exporter was truncating;
        // see `ActiveWindowList` in anim.rs.
        match bare_sigs.get(&bare_sig) {
            Some(&(_, true)) if !bare_animated => {
                if dbg {
                    eprintln!("  DROP[undriven-twin] '{name_l}' sort={}", quad.sort);
                }
                continue;
            }
            // The undriven copy was visited FIRST: overwrite it in place rather than
            // stacking a second draw. Same texture, sort and geometry, so the slot's
            // position in draw order is already correct.
            Some(&(idx, false)) if bare_animated => replace_at = Some(idx),
            _ => {}
        }

        // Decode + save texture once per source path_id.
        let tex_idx = if let Some(&i) = tex_index.get(&quad.tex_pid) {
            i
        } else {
            let Ok(Some(mut tex)) = decode_texture_object(&quad.tex_val, resources) else {
                continue;
            };
            if let Some(alpha_val) = &quad.alpha_val
                && let Ok(Some(alpha)) = decode_texture_object(alpha_val, resources)
            {
                tex = alpha_merge::combine_with_alpha(&tex, &alpha);
            }
            let idx = next_idx;
            let (rel, ok) = save_tex(&tex_dir, spine_dir, idx, &tex.rgba, tex.width, tex.height);
            if ok {
                saved += 1;
            }
            tex_names.push(rel);
            tex_px.insert(quad.tex_pid, (tex.rgba.clone(), tex.width, tex.height));
            tex_index.insert(quad.tex_pid, idx);
            next_idx += 1;
            idx
        };

        let uv: Vec<f32> = quad.mesh.uvs.iter().flat_map(|u| [u[0], u[1]]).collect();
        // Per-vertex colours (RGBA, straight): Unity fades these scene quads at
        // their edges via vertex alpha (e.g. Nearl "Evolved Art"'s translucent
        // spirit-horse sheets). Emit them so the frontend can modulate; omit the
        // array entirely when every vertex is opaque white (the common case) to
        // keep the JSON small.
        let col: Vec<f32> = quad
            .mesh
            .colors
            .iter()
            .flat_map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        let has_vcol = quad.mesh.colors.iter().any(|c| {
            (c[0] - 1.0).abs() > 0.004
                || (c[1] - 1.0).abs() > 0.004
                || (c[2] - 1.0).abs() > 0.004
                || (c[3] - 1.0).abs() > 0.004
        });

        // Classify the layer by blend class + the luminance of the texels it
        // actually covers (mesh-sampled opaque texels: mean + 90th percentile).
        // Two scalable corrections for effect layers that flat-compositing can't
        // otherwise reproduce (blend factors: 1=One, 0=Zero, 10=OneMinusSrcAlpha):
        //   • One/Zero opaque + featureless near-black = a grab-pass distortion /
        //     refraction map (no colour of its own; in-engine it warps the frame
        //     behind it). We can't refract, so drop it rather than stamp a black
        //     bar (e.g. Ines "Under the Flaming Dome", Mlynar, Rosmontis).
        //   • premultiplied-alpha + dark but high-contrast (bright detail on
        //     black) = a glass/filigree glow overlay authored to blend additively;
        //     normally it stamps a black blob (e.g. Ines "Melodic Flutter"'s
        //     gramophone). Force additive so the black drops and detail glows.
        let (lum_mean, lum_p90, lum_p98, dark_frac) = tex_px
            .get(&quad.tex_pid)
            .map_or((1.0, 1.0, 1.0, 0.0), |(px, w, h)| {
                opaque_luma(px, *w, *h, &quad.mesh.uvs, &quad.mesh.indices)
            });
        let is_opaque = (quad.src_blend - 1.0).abs() < 0.5 && quad.dst_blend < 0.5;
        let is_premul_alpha =
            (quad.src_blend - 1.0).abs() < 0.5 && (quad.dst_blend - 10.0).abs() < 0.5;
        // `p98` guards the claim the mean and p90 cannot make alone: FEATURELESS. A
        // refraction map is dark all the way up its distribution (the four real ones in
        // the corpus peak at 0.000-0.133 and are 100% pure black); a compact light streak
        // on a black field is dark on average but keeps a bright top few percent
        // (Mlynar's sword glow: mean 0.041, p90 0.059, p98 0.420 over 8.9% lit texels).
        if quad.erase.is_none() && is_opaque && lum_mean < 0.06 && lum_p90 < 0.12 && lum_p98 < 0.25
        {
            if dbg {
                eprintln!(
                    "  DROP[grabpass] '{name_l}' sort={} lum_mean={lum_mean:.3} p90={lum_p90:.3} p98={lum_p98:.3} dark={dark_frac:.3}",
                    quad.sort
                );
            }
            continue; // grab-pass distortion map → drop
        }
        // Glass/glow overlays authored on a black field to blend additively —
        // frozen and composited straight they stamp a black bar/block. Two shapes:
        //   • premultiplied-alpha with spread bright detail (p90 high): the
        //     original gramophone/filigree case.
        //   • a compact bright peak sitting on a near-black-OPAQUE field (p98 high,
        //     majority pure-black): soft light-blooms and light-streaks — often
        //     authored with One/Zero (opaque) blend, so they'd otherwise paint the
        //     black surround as a solid block (Phantom "Focus"'s bloom, Wiš'adel's
        //     light arc). A <0.25 mean already excludes painted scenes (a lit room
        //     + window sits well above it), and requiring a >0.6 pure-black
        //     majority separates a compact glow from a merely-dark backdrop (whose
        //     colour isn't pure black). Force additive so the black drops.
        // The `p98 > 0.9` peak was too strict — it excluded dimmer streaks such as
        // Mlynar's sword glow, which peaks at 0.420. Relax it to the same 0.25
        // "carries light of its own" floor the grab-pass drop uses. The floor must
        // STAY: a layer with no bright texels at all is not a glow drawn on black, it
        // is a black CURTAIN whose whole job is to paint black (Skadi the Corrupting
        // Heart's fade-to-black plane, pure black at dark_frac 1.000). Forcing that
        // additive composites it to nothing and silently deletes the fade.
        let is_black_field_glow = quad.erase.is_none()
            && ((is_premul_alpha && lum_mean < 0.25 && lum_p90 > 0.75)
                || ((is_premul_alpha || is_opaque)
                    && lum_mean < 0.25
                    && lum_p98 > 0.25
                    && dark_frac > 0.6));
        let additive = quad.additive || is_black_field_glow;

        let mut layer = serde_json::json!({
            "sort": quad.sort,
            "additive": additive,
            "tint": quad.tint,
            "tex": tex_idx,
            "pos": pos,
            "uv": uv,
            "idx": quad.mesh.indices,
        });
        if has_vcol {
            layer["col"] = serde_json::json!(col);
        }
        // LAYER META (2026-09-07): the quad's GameObject name, so a viewer dump or a register
        // row can name a layer by what the artist called it instead of by a texture index that
        // renumbers on every export. `DYNCHAR_LAYER_META=0` omits it (and `maxSize` below), which
        // keeps the JSON byte-identical to the export before this field.
        if std::env::var("DYNCHAR_LAYER_META").as_deref() != Ok("0") {
            layer["name"] = serde_json::json!(quad.go_name);
        }
        // ERASE MASK: the material's `_Strength` (see `BgQuad::erase`); the frontend draws the
        // polygon black at this alpha, skipping every classification rule.
        if let Some(strength) = quad.erase {
            layer["erase"] = serde_json::json!(strength);
        }
        // Camera-riding overlay (see `BgQuad::cam_locked`). Emitted only when true, so every
        // skin without one stays byte-identical.
        if quad.cam_locked {
            layer["camLocked"] = serde_json::json!(true);
            if let Some(v) = quad.cam_lock_view {
                layer["camLockViewPx"] = serde_json::json!(v as f32);
            }
        }
        // ENTRANCE uniform-scale multiplier over the baked pose (1.0 = unchanged). Emitted only
        // when the `_Start` clips actually animate this transform, so an unanimated corpus stays
        // byte-identical.
        if let Some(sc) = &quad.scale_curve {
            layer["scaleCurve"] =
                serde_json::json!(sc.iter().map(|&p| <[f32; 3]>::from(p)).collect::<Vec<_>>());
        }
        // Match particles.rs's fixed scale pivot; omit it with the curve so static exports stay
        // byte-identical and the frontend never has a transform to replay.
        if let Some(p) = &quad.scale_pivot {
            layer["scalePivot"] = serde_json::json!(p);
        }
        // ENTRANCE reveal time (s) — the layer is hidden until its `m_IsActive` switches
        // ON in the `_Start` cinematic (cathedral first, mirror-world + throne later).
        // Omitted (always visible) for main scenes and always-active `_Start` layers.
        if let Some(t) = quad.active_from {
            layer["activeFrom"] = serde_json::json!(t);
        }
        if let Some(t) = quad.active_until {
            layer["activeUntil"] = serde_json::json!(t);
        }
        // FULL schedule when the clip flashes this layer more than once. `activeFrom`/
        // `activeUntil` above still carry the FIRST window, so a renderer that does not know
        // about `activeWindows` degrades to exactly the previous behaviour rather than to
        // always-on. Each entry is `[from, until]` with `null` for an open bound.
        if quad.active_windows.len() > 1 {
            layer["activeWindows"] = serde_json::json!(
                quad.active_windows
                    .iter()
                    .map(|&(f, u)| serde_json::json!([f, u]))
                    .collect::<Vec<_>>()
            );
        }
        // CROSS-ROOT reveal (idle-world layers inside an entrance scene): visibility
        // gate only — deliberately NOT `activeFrom`, so the frontend's reveal-overlay
        // exemptions (mis-sort/veil demotion bypasses) don't treat scenery as a
        // cinematic overlay.
        if let Some(t) = quad.root_reveal_from {
            layer["rootRevealFrom"] = serde_json::json!(t);
        }
        // ENTRANCE material-colour animation: `[t, r, g, b, a]` samples replacing the
        // static tint while the `_Start` cinematic plays (e.g. Mlynar's white flash
        // fading 0→0.671 instead of holding an opaque white-wash). Omitted when the
        // material colour isn't animated.
        if let Some(cc) = &quad.color_curve {
            layer["colorCurve"] = serde_json::json!(
                cc.iter()
                    .map(|&(t, c)| [t, c[0], c[1], c[2], c[3]])
                    .collect::<Vec<_>>()
            );
        }
        // SHADER UV-SCROLL (Capability A): per-second UV velocity `[u, v]` (Unity UV space)
        // for Ram-family scene layers; the frontend offsets the layer's UVs by `t · [u,v]`
        // each frame. Omitted for every non-scroll layer.
        if let Some([uv_0, uv_1]) = quad.uv_scroll {
            layer["uvScroll"] = serde_json::json!([uv_0, uv_1]);
        }
        // Ram DISSOLVE/DISTURB masking (see [`SceneRam`]): the mask indices join the
        // scene's own texture list, so the frontend loader needs no new plumbing.
        if let Some(r) = &quad.ram {
            let diss = resolve_scene_mask(
                r.dissolve_pid,
                r.dissolve_val.as_ref(),
                resources,
                &tex_dir,
                spine_dir,
                &mut tex_index,
                &mut tex_names,
                &mut next_idx,
                &mut saved,
            );
            let dist = resolve_scene_mask(
                r.disturb_pid,
                r.disturb_val.as_ref(),
                resources,
                &tex_dir,
                spine_dir,
                &mut tex_index,
                &mut tex_names,
                &mut next_idx,
                &mut saved,
            );
            let diss2 = resolve_scene_mask(
                r.dissolve2_pid,
                r.dissolve2_val.as_ref(),
                resources,
                &tex_dir,
                spine_dir,
                &mut tex_index,
                &mut tex_names,
                &mut next_idx,
                &mut saved,
            );
            let weightt = resolve_scene_mask(
                r.weight_pid,
                r.weight_val.as_ref(),
                resources,
                &tex_dir,
                spine_dir,
                &mut tex_index,
                &mut tex_names,
                &mut next_idx,
                &mut saved,
            );
            let ramt = resolve_scene_mask(
                r.ram_pid,
                r.ram_val.as_ref(),
                resources,
                &tex_dir,
                spine_dir,
                &mut tex_index,
                &mut tex_names,
                &mut next_idx,
                &mut saved,
            );
            if diss.is_some() || diss2.is_some() || dist.is_some() || ramt.is_some() {
                layer["ram"] = serde_json::json!({
                    "dissolveTex": diss,
                    "dissolveST": r.dissolve_st,
                    "dissolveTex2": diss2,
                    "dissolveST2": r.dissolve2_st,
                    "weightTex": weightt,
                    "weightST": r.weight_st,
                    "amount2": r.amount2,
                    "borderWidth2": r.border_width2,
                    "edgeColor": r.edge_color,
                    "edgePow": r.edge_pow,
                    "disturbTex": dist,
                    "disturbST": r.disturb_st,
                    "ramTex": ramt,
                    "ramST": r.ram_st,
                    "uvRot": r.uv_rot,
                    "amount": r.amount,
                    "borderWidth": r.border_width,
                    "anchorU": r.anchor_u,
                    "anchorV": r.anchor_v,
                    "intensityU": r.intensity_u,
                    "intensityV": r.intensity_v,
                    "disturbInfluenceDissolveUV": r.disturb_influence_dissolve_uv,
                    "disturbInfluenceMainUV": r.disturb_influence_main_uv,
                    "dissolveSpeed": r.dissolve_speed,
                    "disturbSpeed": r.disturb_speed,
                });
                // Disturb2's second noise, present only on that family so every other ram block
                // serialises exactly as before.
                if let Some(d2) = r.disturb2 {
                    layer["ram"]["disturb2"] = serde_json::json!(d2);
                }
            }
        }
        // ENTRANCE Transform POSITION curve: `[t, dx, dy]` authored-px offsets the frontend
        // adds to this layer's rest pose (Executor's scope rim pans while ours is pinned).
        if let Some(pc) = &quad.pos_curve {
            layer["posCurve"] = serde_json::json!(
                pc.iter()
                    .map(|&(t, x, y)| [t, x * inv, y * inv])
                    .collect::<Vec<_>>()
            );
        }
        // CLIP `_MainTex_ST` curve (Capability B): `[t, sx, sy, ox, oy]` samples the frontend
        // replays during the entrance (Skadi2's seam sweep). Omitted for static-ST layers.
        if let Some(sc) = &quad.st_curve {
            layer["stCurve"] = serde_json::json!(
                sc.iter()
                    .map(|&(t, s)| [t, s[0], s[1], s[2], s[3]])
                    .collect::<Vec<_>>()
            );
        }
        // BONE ATTACHMENT: the quad rides a spine bone (see [`BgFollow`]). `pos` above is
        // the baked EDITOR pose; the frontend re-bases it every frame by
        // `bone(t) · followerWorld⁻¹`, where the follower's world frame is given here in
        // authored px (Y-up) as an origin plus its 2×2 linear basis.
        if let Some(f) = &quad.follow {
            layer["followBone"] = serde_json::json!(f.bone);
            layer["followBoneRot"] = serde_json::json!(f.rot);
            layer["followOrigin"] = serde_json::json!([f.origin[0] * inv, f.origin[1] * inv]);
            layer["followBasis"] = serde_json::json!(f.basis);
        }
        if dbg {
            let ext = (xmx - xmn).max(ymx - ymn);
            eprintln!(
                "  KEEP '{name_l}' tex={tex_idx} sort={} size={:.0}x{:.0} ext={ext:.0} add={additive} src/dst={:.0}/{:.0} lum_mean={lum_mean:.3} p90={lum_p90:.3} p98={lum_p98:.3} darkf={dark_frac:.2}",
                quad.sort,
                xmx - xmn,
                ymx - ymn,
                quad.src_blend,
                quad.dst_blend
            );
        }
        if let Some(idx) = replace_at {
            layers[idx] = layer;
            bare_sigs.insert(bare_sig, (idx, true));
        } else {
            bare_sigs.insert(bare_sig, (layers.len(), bare_animated));
            // DIAGNOSTIC (`DYNCHAR_LAYERMAP=1`): emitted-layer index -> source GameObject.
            // The only way to attribute a scene layer to an object; see `BgQuad::go_pid`.
            if std::env::var("DYNCHAR_LAYERMAP").is_ok() {
                eprintln!(
                    "  [layermap] layer={} go={} name={:?} sort={} tex={} ram={}",
                    layers.len(),
                    quad.go_pid,
                    quad.go_name,
                    quad.sort,
                    tex_idx,
                    quad.ram.is_some(),
                );
            }
            layers.push(layer);
        }
    }

    if dropped_twins > 0 && std::env::var("SCENE_DEBUG").is_ok() {
        eprintln!("  cross-root twins dropped: {dropped_twins}");
    }

    // ENTRANCE only: drop an UNDRIVEN copy of artwork the cinematic drives elsewhere.
    // Effect prefabs are instantiated once per animation state (Mlynar carries four
    // `glow_01` sword flares — one per sword rig, same 128px streak texture at the same
    // sort, each posed differently). The `_Start` clip drives only the rigs it plays;
    // the idle/special copies get no colour curve and no reveal window, so they would
    // paint their streak at full strength for the entire cinematic while the game shows
    // them only during their own state. Keyed on (texture, sort) rather than geometry
    // because the copies differ exactly in pose — which is why the bare-signature twin
    // rule above cannot see them. Requires a driven sibling, so art that is legitimately
    // static everywhere in the scene is untouched. Corpus-wide this removes 8 layers
    // across 4 of the 12 entrance skins.
    if asset.name.to_ascii_lowercase().contains("_start") {
        // The key must include GEOMETRY. Keyed on (tex, sort) alone this drops any static
        // quad that merely SHARES an atlas and a depth with an animated one — which is the
        // common case, not the rare one: Mlynar "Fields of Ruination" has five distinct
        // layers at tex 6 / sort 1, and the rule discarded a 1039x1039 panel at
        // (-933, 44) because four unrelated quads elsewhere in the frame carry a colour
        // curve. A true undriven TWIN is the same artwork in the same place, so requiring
        // the bounds to match keeps every real twin (both of Mlynar's other two drops match
        // their driven copy to well under a pixel) while sparing distinct art. The duplicate
        // rule above already keys on full geometry; this one simply never did.
        let bbox_key = |l: &serde_json::Value| -> Option<(i64, i64, i64, i64)> {
            let p = l.get("pos")?.as_array()?;
            let xs = p.iter().step_by(2).filter_map(serde_json::Value::as_f64);
            let ys = p
                .iter()
                .skip(1)
                .step_by(2)
                .filter_map(serde_json::Value::as_f64);
            let (mut x0, mut x1) = (f64::INFINITY, f64::NEG_INFINITY);
            let (mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY);
            for x in xs {
                x0 = x0.min(x);
                x1 = x1.max(x);
            }
            for y in ys {
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
            x0.is_finite().then(|| {
                (
                    x0.round() as i64,
                    y0.round() as i64,
                    x1.round() as i64,
                    y1.round() as i64,
                )
            })
        };
        type SibKey = (i64, i64, (i64, i64, i64, i64));
        let driven: std::collections::HashSet<SibKey> = layers
            .iter()
            .filter(|l| {
                l.get("colorCurve").is_some()
                    || l.get("activeFrom").is_some()
                    || l.get("activeUntil").is_some()
            })
            .filter_map(|l| {
                Some((
                    l.get("tex")?.as_i64()?,
                    l.get("sort")?.as_i64()?,
                    bbox_key(l)?,
                ))
            })
            .collect();
        layers.retain(|l| {
            if l.get("colorCurve").is_some()
                || l.get("activeFrom").is_some()
                || l.get("activeUntil").is_some()
            {
                return true;
            }
            let key = (|| {
                Some((
                    l.get("tex")?.as_i64()?,
                    l.get("sort")?.as_i64()?,
                    bbox_key(l)?,
                ))
            })();
            let keep = !key.is_some_and(|k| driven.contains(&k));
            if !keep && dbg {
                // Print the GEOMETRY too: the key is only (tex, sort), so this drop cannot
                // tell a genuine undriven twin from a DIFFERENT quad that merely shares an
                // atlas and a depth. The bbox is what distinguishes them.
                let bb = l.get("pos").and_then(serde_json::Value::as_array).map(|p| {
                    let xs: Vec<f64> = p
                        .iter()
                        .step_by(2)
                        .filter_map(serde_json::Value::as_f64)
                        .collect();
                    let ys: Vec<f64> = p
                        .iter()
                        .skip(1)
                        .step_by(2)
                        .filter_map(serde_json::Value::as_f64)
                        .collect();
                    let mn = |v: &Vec<f64>| v.iter().copied().fold(f64::INFINITY, f64::min);
                    let mx = |v: &Vec<f64>| v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    (mn(&xs), mn(&ys), mx(&xs), mx(&ys))
                });
                eprintln!("  DROP[undriven-sibling] tex/sort={key:?} bbox={bb:?}");
            }
            keep
        });
    }

    // No mesh layers AND no camera frame → nothing worth writing. With a frame but
    // no layers, still emit the bare JSON so the frontend can align a static-art
    // backdrop (spine-only skins).
    if layers.is_empty() && !has_frame {
        return 0;
    }

    let meta = serde_json::json!({
        "coordinateSystem": "spine-authored pixels, Y-up, origin at skeleton root; frontend flips Y for Pixi, and V (1-v) for UVs",
        "skeletonScale": skel_scale,
        "cameraSize": asset.bg_camera_size,
        "cameraSizePx": asset.bg_camera_size.map(|c| c as f32 * inv),
        // Authored display frame (spine-authored px): centre + square extent from
        // the display controller's `_adjustes[0]` — how the game frames the scene.
        "cameraOffsetPx": asset.bg_camera_offset.map(|(x, y)| [x as f32, y as f32]),
        // Unity writes an UNINITIALISED `_adjustes` stop as `-FLT_MAX`. Carried through
        // verbatim it is a non-zero number, so a consumer's truthiness check accepts it and
        // builds a ~3e38 px crop — Nearl the Radiant Knight "Epoque" has both stops unset and
        // rendered as an EMPTY frame. Emit null unless the extent is finite and positive, so
        // the consumer falls back to the `cameraSizePx` framing it already has for skins with
        // no authored stop at all.
        "cameraViewPx": asset.bg_camera_view.filter(|v| v.is_finite() && *v > 0.0).map(|v| v as f32),
        // The TIGHT/zoomed-in endpoint from `_adjustes[1]` (present only on special-entry
        // skins whose viewer dollies out from a close-up at open). Null when absent.
        "cameraOffsetPx2": asset.bg_camera_offset2.map(|(x, y)| [x as f32, y as f32]),
        "cameraViewPx2": asset.bg_camera_view2.filter(|v| v.is_finite() && *v > 0.0).map(|v| v as f32),
        "aspect": asset.bg_max_aspect,
        "characterSort": asset.bg_character_sort,
        // See `SpineAsset::separator_slots`. Slot NAMES, in the skeleton's own draw order.
        "separatorSlots": asset.separator_slots,
        "separatorPartSorts": asset.separator_part_sorts,
        // ENTRANCE (`_Start`) cinematic timing (seconds), from the entrance director's
        // `_params.duration` + the reform `_delayTime` cluster. Present only on `_Start`
        // scenes; drives the client entrance camera dolly (tight→wide across `_adjustes`)
        // and the hand-off to the settled idle — so those are gamedata, not guessed.
        // The clip the game binds at settle (`SkeletonMecanim._animationName`, see
        // `SpineAsset::settle_animation`). Null where the binding is "Idle"-equivalent absent.
        "settleAnimation": asset.settle_animation,
        "entranceDuration": asset.bg_entrance_duration.map(|v| v as f32),
        "entranceClipStop": asset.bg_entrance_clip_stop,
        // Straight RGBA of the director's end-of-entrance screen fade (see `bg_entrance_fade`).
        "entranceFade": asset.bg_entrance_fade.map(|c| Value::from(vec![c[0], c[1], c[2], c[3]])),
        // The entrance camera's solid clear colour, rgb 0..1 (see `bg_entrance_clear`).
        "entranceClearColor": asset.bg_entrance_clear.map(|c| Value::from(vec![c[0], c[1], c[2]])),
        "entranceTransform": asset.bg_entrance_transform.map(|v| v as f32),
        // Tight entrance close-up view (authored px) from the entrance camera's ortho size.
        "entranceViewPx": asset.bg_entrance_view.map(|v| v as f32),
        "entrancePerspective": asset.bg_entrance_persp.then_some(true),
        "entranceCamOffsetPx": asset.bg_entrance_cam_offset.map(|(x, y)| [x as f32, y as f32]),
        // Data-driven camera dolly zoom: [[t_seconds, orthographic_size], …] keyframes.
        "entranceOrthoCurve": asset.bg_entrance_ortho_curve.as_ref().map(|c| c.iter().map(|(t, s)| [*t, *s]).collect::<Vec<_>>()),
        "entrancePostFx": asset.bg_entrance_post_fx.as_ref().map(|(name, inten, curve, params)| serde_json::json!({
            "effect": name,
            "intensity": inten,
            "weightCurve": curve.iter().map(|(t, w)| [*t, *w]).collect::<Vec<_>>(),
            // The settings object's own overridden scalars, so an effect that needs a MAGNITUDE
            // reads it from the profile instead of a fitted constant (`HGMobileBlur` ships
            // `blurDegree` / `blurSpread` / `quality` / `resMode`).
            "params": params
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(*v)))
                .collect::<serde_json::Map<String, serde_json::Value>>(),
        })),
        "entrancePanCurve": asset.bg_entrance_pan_curve.as_ref().map(|c| c.iter().map(|(t, s)| [*t, *s]).collect::<Vec<_>>()),
        "entranceCamCenterCurve": asset.bg_entrance_cam_center.as_ref().map(|c| c.iter().map(|(t, x, y)| [*t, *x, *y]).collect::<Vec<_>>()),
        "entranceCamRollCurve": asset.bg_entrance_cam_roll.as_ref().map(|c| c.iter().map(|(t, r)| [*t, *r]).collect::<Vec<_>>()),
        "entranceAperturePx": asset.bg_entrance_aperture.map(|(rect, _)| rect),
        "entranceApertureSort": asset.bg_entrance_aperture.map(|(_, sort)| sort),
        "entranceVoiceOffset": asset.bg_entrance_voice.map(|v| v as f32),
        "textureCount": next_idx,
        "layers": layers,
    });
    // Derived card-backdrop placement (see derive_backdrop_transform). Appended onto the
    // meta afterwards so an absent derivation leaves the JSON byte-identical to before.
    let meta = {
        let mut meta = meta;
        // The pool's index -> path table; only when the pool is on, so the off arm's
        // JSON is byte-identical (see `tex_pool_on`).
        if tex_pool_on()
            && let Some(map) = meta.as_object_mut()
        {
            map.insert("textures".into(), serde_json::json!(tex_names));
        }
        if let Some((s, o)) = derive_backdrop_transform(&layers, &tex_dir, spine_dir, &tex_names)
            && let Some(map) = meta.as_object_mut()
        {
            map.insert("backdropScale".into(), serde_json::json!(s));
            map.insert("backdropOffsetPx".into(), serde_json::json!([o[0], o[1]]));
        }
        // The controller's `_maxSize`, absolute (see `Asset::bg_max_size`); appended the same
        // way, and only under `DYNCHAR_LAYER_META` (the layer names' flag), so `=0` leaves the
        // JSON byte-identical to the export before both fields.
        if std::env::var("DYNCHAR_LAYER_META").as_deref() != Ok("0")
            && let Some((x, y)) = asset.bg_max_size
            && let Some(map) = meta.as_object_mut()
        {
            map.insert("maxSize".into(), serde_json::json!([x as f32, y as f32]));
        }
        meta
    };
    if let Ok(text) = serde_json::to_string(&meta)
        && std::fs::write(spine_dir.join(format!("{}[scene].json", asset.name)), text).is_ok()
    {
        saved += 1;
    }
    saved
}

/// Collapse an enemy asset's numbered variant suffix (`enemy_1234_foo_2`) to its
/// base name, so variant forms share one output directory.
#[must_use]
pub fn enemy_dir_name(asset_name: &str) -> String {
    if let Some((base, suffix)) = asset_name.rsplit_once('_')
        && base.starts_with("enemy_")
        && !suffix.is_empty()
        && suffix.chars().all(|c| c.is_ascii_digit())
    {
        return base.to_string();
    }
    asset_name.to_string()
}

/// Derive the character name from a bundle subdirectory path.
#[must_use]
pub fn char_name_from_bundle(bundle_subdir: &Path) -> String {
    bundle_subdir.file_name().map_or_else(
        || "unknown".to_string(),
        |n| n.to_string_lossy().to_string(),
    )
}
