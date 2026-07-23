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
    Building,
    DynIllust,
    Enemy,
}

impl fmt::Display for SpineCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BattleFront => write!(f, "BattleFront"),
            Self::BattleBack => write!(f, "BattleBack"),
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
    /// Orthographic camera half-height, if a display controller exposes it.
    pub bg_camera_size: Option<f64>,
    /// Render-target aspect (`maxSize.x / maxSize.y`) for the camera frame.
    pub bg_max_aspect: Option<f64>,
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
    /// ENTRANCE (`_Start`) cinematic total length in seconds, from the entrance
    /// director MonoBehaviour's `_params.duration` (the client plays the whole
    /// `_Start` set over this span, then hands off to the settled idle). `None`
    /// for the main set or a skin without an entrance director.
    pub bg_entrance_duration: Option<f64>,
    /// ENTRANCE transform beat in seconds — the time of the dominant late cluster
    /// of per-object `_delayTime`s (the reform/gala burst; Virtuosa: 12.0s). The
    /// camera has dollied to the wide stop and the character has reformed by here,
    /// so it's the natural entrance→idle hand-off point. `None` when no entrance.
    pub bg_entrance_transform: Option<f64>,
    /// ENTRANCE camera full view in authored px (`2 · orthographic_size / skeletonScale`)
    /// — the TIGHT close-up the `_Start` opens on (Virtuosa 598px, vs the 1929px display
    /// stop). The client dollies OUT from this to the display frame. `None` when no entrance.
    pub bg_entrance_view: Option<f64>,
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
    /// ENTRANCE voice-line offset (s) from `_params.charVoiceOffset` — when the reformed
    /// cellist starts talking. The seated→standing hand-off beat (the standing form lives at
    /// a different rig position than the seated form, so the entrance hands off to the idle
    /// here rather than trying to reframe onto it). `None` when no entrance director.
    pub bg_entrance_voice: Option<f64>,
    /// Parsed `ParticleSystem`s for dynillusts (emitted as `[particles].json`).
    /// Empty for every other category.
    pub particles: Vec<super::particles::ParticleData>,
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
    /// `_MainTex` `Texture2D` path_id (for claiming against later phases).
    pub tex_pid: i64,
    /// Material `_SrcBlend`/`_DstBlend` factors, for blend-class classification
    /// (distortion-drop / glass-additive) in `export_scene`.
    pub src_blend: f64,
    pub dst_blend: f64,
    /// ENTRANCE reveal time (seconds) — when this layer's GameObject (or a nearest
    /// ancestor group) is switched ON by an `m_IsActive` curve in the `_Start` clips.
    /// `None` = always active (visible from t=0). Only `_Start` scenes carry non-None.
    pub active_from: Option<f32>,
    /// ENTRANCE hide time (seconds) — when this layer's GameObject (or nearest ancestor)
    /// is switched OFF by an `m_IsActive` curve in the `_Start` clips (the cinematic's
    /// environment SWAP). `None` = never hidden (visible to the end). Only `_Start` scenes.
    pub active_until: Option<f32>,
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
    /// material floats `_MainUSpeed`/`_MainVSpeed` (UV/sec, no AnimationClip). Baking only
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
    camera_offset: Option<(f64, f64)>,
    camera_view: Option<f64>,
    camera_offset2: Option<(f64, f64)>,
    camera_view2: Option<f64>,
    claimed_tex: Vec<i64>,
    char_sort: Option<i64>,
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

/// Whether a GameObject is shown in the IDLE display — i.e. no Transform ancestor
/// is `m_IsActive=0` NOR a state-only effect group. Dynchar prefabs bucket effects
/// into groups ("Start Only Effects" / "Interact Only Effects" / "Special Only
/// Effects" / "General Effects") that the game shows only during that state; the
/// groups stay m_IsActive=1 in the prefab (gated at runtime by the animator), so
/// we exclude by NAME too. Descendants of a non-idle group must not appear in the
/// idle scene/particles, else every state's effects render at once → noise.
pub(crate) fn go_effectively_active(
    all_objects: &HashMap<i64, (i32, Value)>,
    go_pid: i64,
    go_to_transform: &HashMap<i64, i64>,
    idle_active: &HashMap<i64, bool>,
) -> bool {
    const STATE_ONLY: &[&str] = &[
        "start", "interact", "special", "skill", "attack", "die", "assist",
    ];
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
            match idle_active.get(&go) {
                Some(false) => return false,
                Some(true) => {}
                None => {
                    if gv.get("m_IsActive").and_then(Value::as_i64).unwrap_or(1) == 0 {
                        return false;
                    }
                }
            }
            let name = gv
                .get("m_Name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_ascii_lowercase();
            if name.contains("only") && STATE_ONLY.iter().any(|s| name.contains(s)) {
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
) -> (Vec<SpineAsset>, HashSet<i64>) {
    let mut claimed = HashSet::new();
    let mut assets = Vec::new();

    // Find SkeletonMecanim MonoBehaviours (class_id=114 with skeletonDataAsset field)
    let skeleton_mecanims: Vec<(i64, &Value)> = all_objects
        .iter()
        .filter(|(_, (class_id, val))| *class_id == 114 && val.get("skeletonDataAsset").is_some())
        .map(|(pid, (_, val))| (*pid, val))
        .collect();

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
        let (scene, skel_scale, particles) = if category == SpineCategory::DynIllust
            && base_name.to_lowercase().starts_with("dyn_illust_")
        {
            let skel_scale = all_objects
                .get(&skel_data_pid)
                .and_then(|(_, v)| v.get("scale"))
                .and_then(serde_json::Value::as_f64);
            let spine_tex_pids: HashSet<i64> = chain.claimed.iter().copied().collect();
            // The per-layer ENTRANCE reveal timeline (`m_IsActive`) applies ONLY to the
            // `_Start` cinematic scene; the MAIN scene's idle/interact/special clips also
            // toggle effects, but those layers are idle-visible, not entrance-sequenced.
            let is_entrance = base_name.to_lowercase().contains("_start");
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
            let cross_root_reveal = if is_entrance {
                find_entrance_timing(all_objects).4
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
                &host,
                &super::particles::EntranceCtx {
                    windows: &particle_windows,
                    rate_curves: &particle_rate_curves,
                    event_rate_gos: &event_rate_gos,
                    transform_curves: &entrance_transform_curves,
                    is_entrance,
                },
                &super::particles::RootScope {
                    own: own_root,
                    skeleton_roots: &skeleton_roots,
                },
            );
            if !particles.is_empty() || skipped > 0 {
                eprintln!(
                    "  particles: {} exported, {} skipped ({base_name})",
                    particles.len(),
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
            bg_entrance_transform,
            bg_entrance_view,
            bg_entrance_cam_offset,
            bg_entrance_ortho_curve,
            bg_entrance_voice,
            bg_entrance_pan_curve,
            bg_entrance_cam_center,
        ) = if category == SpineCategory::DynIllust && base_name.to_lowercase().contains("_start") {
            let (dur, tr, ortho, voice, _) = find_entrance_timing(all_objects);
            // Entrance camera ortho size (world units) → authored-px full view (2·ortho·invScale),
            // the tight close-up the cinematic opens on before dollying out to the display frame.
            let inv = 1.0 / skel_scale.unwrap_or(0.01);
            let cam_off = find_entrance_camera_offset(all_objects).map(|(x, y)| (x * inv, y * inv));
            // The DATA-DRIVEN camera dolly: the animated orthographic-size curve (see
            // `entrance_ortho_curve`). Replaces client-side guesswork about the zoom timing.
            let ortho_curve = super::anim::entrance_ortho_curve(all_objects);
            let pan_curve = super::anim::entrance_pan_curve(all_objects);
            let cam_center = super::anim::entrance_camera_track(all_objects, inv);
            (
                dur,
                tr,
                ortho.map(|o| 2.0 * o * inv),
                cam_off,
                ortho_curve,
                voice,
                pan_curve,
                cam_center,
            )
        } else {
            (None, None, None, None, None, None, None, None)
        };

        assets.push(SpineAsset {
            name: base_name.to_string(),
            skel_data: chain.skel_bytes,
            atlas_text: chain.atlas_text,
            textures: chain.textures,
            category,
            bg_quads: scene.quads,
            bg_skel_scale: skel_scale,
            bg_camera_size: scene.camera_size,
            bg_max_aspect: scene.max_aspect,
            bg_camera_offset: scene.camera_offset,
            bg_camera_view: scene.camera_view,
            bg_camera_offset2: scene.camera_offset2,
            bg_camera_view2: scene.camera_view2,
            bg_character_sort: scene.char_sort,
            bg_entrance_duration,
            bg_entrance_transform,
            bg_entrance_view,
            bg_entrance_cam_offset,
            bg_entrance_ortho_curve,
            bg_entrance_pan_curve,
            bg_entrance_cam_center,
            bg_entrance_voice,
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

/// ENTRANCE visibility window for a GameObject: the window of the nearest ancestor
/// (self first) present in `reveal` — walks `m_Father` up the transform chain.
/// `(None, None)` when neither the object nor any ancestor is toggled (→ always visible).
fn reveal_of_go(
    go_pid: i64,
    reveal: &HashMap<i64, super::anim::ActiveWindow>,
    go_to_transform: &HashMap<i64, i64>,
    all_objects: &HashMap<i64, (i32, Value)>,
) -> super::anim::ActiveWindow {
    let mut cur_go = Some(go_pid);
    for _ in 0..256 {
        let Some(g) = cur_go else { break };
        if let Some(&w) = reveal.get(&g) {
            return w;
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
    (None, None)
}

/// Collect every background-scene mesh quad of a dynillust prefab.
///
/// Approach (graph-driven, not name-based): walk every `MeshRenderer` (class
/// 23) that does not belong to the character spine, resolve its `MeshFilter`
/// mesh, material `_MainTex`/`_AlphaTex`/tint/blend, and full world transform
/// (relative to the spine root). ALL non-character quads are returned (foreground
/// effects included) — the live scene renderer inserts the animated character
/// among them at `char_sort`.
fn collect_dynchar_bg_quads(
    all_objects: &HashMap<i64, (i32, Value)>,
    spine_tex_pids: &HashSet<i64>,
    is_entrance: bool,
    cross_root_reveal: Option<f64>,
    own_root: Option<i64>,
    skeleton_roots: &HashSet<i64>,
    host: &BgParticleHost,
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
    let reveal_map = if is_entrance {
        super::anim::active_windows(all_objects)
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
    // Authored full-illustration display frame from the display controller's
    // `_adjustes[0]` (offset = frame CENTRE, size.x = square full EXTENT, both in
    // spine-authored px). The camera is not centred on the skeleton root, so this
    // is how the game frames the scene.
    // Parse ONE `_adjustes` entry → (offset, size). Used for both the wide adjust[0]
    // (idle framing) and the tight adjust[1] (the zoomed-in open the viewer dollies out from).
    let parse_adjust = |a: &Value| -> (Option<(f64, f64)>, Option<f64>) {
        let off = a.get("offset");
        let ox = off
            .and_then(|o| o.get("x"))
            .and_then(serde_json::Value::as_f64);
        let oy = off
            .and_then(|o| o.get("y"))
            .and_then(serde_json::Value::as_f64);
        let size = a
            .get("size")
            .and_then(|s| s.get("x"))
            .and_then(serde_json::Value::as_f64);
        (ox.zip(oy), size)
    };
    let (camera_offset, camera_view, camera_offset2, camera_view2) = controller_mbs
        .iter()
        .find_map(|(_, v)| {
            let adjustes = v.get("_adjustes")?.as_array()?;
            let (o0, s0) = parse_adjust(adjustes.first()?);
            // The tight endpoint is optional (some controllers ship a single adjust).
            let (o1, s1) = adjustes.get(1).map(parse_adjust).unwrap_or((None, None));
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

    let mut skipped_inactive = 0usize;
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
        // Skip renderers under a state-gated (inactive) group. The prefab keeps
        // "Start Only Effects" / "Interact Only Effects" / "Special Only Effects"
        // groups m_IsActive=0 by default (the game activates them only during
        // those states), so their descendants must not appear in the idle scene.
        // EXCEPT: in the `_Start` scene, a GO whose `m_IsActive` the cinematic
        // clip drives (a reveal window exists) is kept — the clip overrides the
        // prefab flag at runtime and the window gates visibility instead. Covers
        // both delayed reveals (Mlynar's white transition, activeFrom 13.0) and
        // shown-from-0-then-hidden overlays (activeUntil only).
        if !go_effectively_active(all_objects, go_pid, &go_to_transform, &idle_pose.active)
            && window == (None, None)
        {
            skipped_inactive += 1;
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
            Option<[f32; 2]>,
        );
        let mut resolved: Option<ResolvedQuadMaterial> = None;
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
            if mat.get("_meshExtResolved").is_some() && window == (None, None) {
                continue;
            }
            // _MainTex slot: texture + its Scale/Offset (ST). Unresolvable
            // (cross-bundle) or non-Texture2D refs yield None → skip material.
            let (main_pid, tex_val, st) =
                super::particles::mat_texenv(all_objects, mat, "_MainTex");
            let (Some(main_pid), Some(tex_val)) = (main_pid, tex_val) else {
                continue;
            };
            if spine_tex_pids.contains(&main_pid) {
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
            let legacy_scale = legacy_tint_scale(mat);
            let ram_scale = ram_tint_scale(mat);
            let mut cprops = material_color_props(mat);
            let (tint, tint_scale) = if legacy_scale > 1.0 {
                cprops.1 = Some("_TintColor".to_string());
                let tc = cprops
                    .0
                    .iter()
                    .find(|(n, _)| n == "_TintColor")
                    .map_or([1.0; 4], |(_, c)| *c);
                (
                    [
                        (tc[0] * legacy_scale).clamp(0.0, 1.0),
                        (tc[1] * legacy_scale).clamp(0.0, 1.0),
                        (tc[2] * legacy_scale).clamp(0.0, 1.0),
                        (tc[3] * legacy_scale).clamp(0.0, 1.0),
                    ],
                    legacy_scale,
                )
            } else if ram_scale > 1.0 {
                cprops.1 = Some("_MainColor".to_string());
                let mc = cprops
                    .0
                    .iter()
                    .find(|(n, _)| n == "_MainColor")
                    .map_or([1.0; 4], |(_, c)| *c);
                (
                    [
                        (mc[0] * ram_scale).clamp(0.0, 1.0),
                        (mc[1] * ram_scale).clamp(0.0, 1.0),
                        (mc[2] * ram_scale).clamp(0.0, 1.0),
                        (mc[3] * ram_scale).clamp(0.0, 1.0),
                    ],
                    ram_scale,
                )
            } else {
                (material_tint(mat), 1.0)
            };
            // Capability A — Ram-family shader UV-scroll (static material floats, no clip).
            // Gate STRICTLY: only a Ram-family shader with a non-zero `_Main*Speed` carries
            // scroll; every other layer stays byte-identical.
            let uv_scroll = {
                let shader = mat
                    .get("_shaderName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if shader.contains("Ram/") {
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
                uv_scroll,
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
            uv_scroll,
        )) = resolved
        else {
            continue;
        };
        // The layer's animated material colour (entrance scenes only), matched against
        // the material's own colour properties and resolved onto the static tint.
        let color_curve = color_channels.get(&go_pid).and_then(|chs| {
            super::anim::layer_color_curve(
                chs,
                &color_props,
                tint_prop.as_deref(),
                tint,
                tint_scale,
            )
        });
        // The layer's animated `_MainTex_ST` curve (entrance scenes only). When present it
        // supersedes the static ST bake below (the curve carries the full ST).
        let st_curve = st_channels.get(&go_pid).and_then(|chs| {
            super::anim::layer_st_curve(
                chs,
                [st[0] as f32, st[1] as f32, st[2] as f32, st[3] as f32],
            )
        });

        // Geometry from the GameObject's MeshFilter. An in-bundle Mesh (class
        // 43) is parsed; a null (`m_Mesh == 0`) or external/built-in reference
        // (e.g. Unity's Quad primitive, path_id 10210, in default resources)
        // falls back to the unit quad — dynchar scene layers are flat quads.
        let mut mesh = match go_to_mesh.get(&go_pid).copied() {
            Some(mp) if mp != 0 => match all_objects.get(&mp) {
                Some((43, mesh_val)) => match super::mesh::parse_mesh(mesh_val, &HashMap::new()) {
                    Some(m) => m,
                    None => continue,
                },
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
            .map(|&tf| accumulate_matrix(all_objects, tf, &spine_gos, &idle_pose))
            .unwrap_or_else(super::mesh::Mat4::identity);
        let z = world.point([0.0, 0.0, 0.0])[2];

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
            src_blend,
            dst_blend,
            active_from: window.0,
            active_until: window.1,
            root_reveal_from: cross_from,
            color_curve,
            uv_scroll,
            st_curve,
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
        camera_offset,
        camera_view,
        camera_offset2,
        camera_view2,
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
fn legacy_tint_scale(mat: &Value) -> f32 {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let legacy = shader.contains("/Particles/") || shader.starts_with("Particles/");
    let has_tint_color = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .is_some_and(|c| c.contains_key("_TintColor"));
    if legacy && has_tint_color { 2.0 } else { 1.0 }
}

/// Ram-family (`Torappu/Particles-L2D/Ram/*`) scene-layer tint multiplier. These
/// compositors sample `2 × _MainColor × tex` (the frontend Ram GLSL mirrors it as
/// `col += col`), so authors key `_MainColor` α≈0.5 as neutral and dim/colour-shift it
/// below that for a soft wash — while `material_tint` would read the inert Unity
/// `_Color` default and render full white. Returns 2.0 when the shader is Ram-family
/// AND the material carries a `_MainColor` colour property, else 1.0.
fn ram_tint_scale(mat: &Value) -> f32 {
    let shader = mat
        .get("_shaderName")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let has_main_color = mat
        .get("m_SavedProperties")
        .and_then(|sp| sp.get("m_Colors"))
        .and_then(|c| c.as_object())
        .is_some_and(|c| c.contains_key("_MainColor"));
    if shader.contains("Ram/") && has_main_color {
        2.0
    } else {
        1.0
    }
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
fn material_color_props(mat: &Value) -> (Vec<(String, [f32; 4])>, Option<String>) {
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
/// MonoBehaviour. The director is the class-114 that owns `_mainCamera` + `_params`
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
fn find_entrance_timing(all_objects: &HashMap<i64, (i32, Value)>) -> EntranceTiming {
    let params = all_objects.values().find_map(|(cid, v)| {
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
    let cam_ortho = all_objects.values().find_map(|(cid, v)| {
        (*cid == 20)
            .then(|| v.get("orthographic size").and_then(Value::as_f64))
            .flatten()
    });
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

/// Accumulate a GameObject's world TRANSLATION by summing `m_LocalPosition` up the
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
    let cam_go = all_objects.values().find_map(|(cid, v)| {
        (*cid == 20)
            .then(|| v.get("m_GameObject").and_then(get_path_id))
            .flatten()
    })?;
    let skel_go = all_objects.values().find_map(|(cid, v)| {
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
/// map, the set of spine-root GameObjects (the walk stops before them), and the
/// evaluated idle pose. Built once per dynchar prefab.
pub(crate) struct BgParticleHost {
    go_to_transform: HashMap<i64, i64>,
    spine_gos: HashSet<i64>,
    /// GameObject `path_id` → the `_delayTime` (seconds) of a delay/activator
    /// MonoBehaviour on it. The `_Start` cinematic sequences its effect GROUPS by
    /// activating them after a delay; a particle system's own start delay is its
    /// nearest such ancestor's value (see {@link delay_of_go}).
    go_delay: HashMap<i64, f64>,
    /// GameObject `path_id` → spine-unity `BoneFollower` on it: `(boneName,
    /// followBoneRotation)`. Effect-prefab CLONE roots (the director `_effects`,
    /// e.g. Virtuosa's `start_apple_01(Clone)`) ride a SPINE BONE at runtime via
    /// this MonoBehaviour — their serialized transform is only an editor pose the
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

    /// Nearest `BoneFollower` in the GameObject's transform ancestry (self first,
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

    /// Topmost ancestor GameObject of `go_pid` — the prefab-INSTANCE root its
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

    /// Whether the GameObject and every ancestor are active (see `go_effectively_active`).
    pub(crate) fn effectively_active(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
    ) -> bool {
        go_effectively_active(
            all_objects,
            go_pid,
            &self.go_to_transform,
            &self.idle.active,
        )
    }

    /// World matrix of a GameObject's Transform, in the spine root's local frame.
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

    /// The world matrix of the GameObject's Transform PARENT (its `m_Father` chain,
    /// up to but excluding the spine root). Used to project a child's animated LOCAL
    /// position (in the parent's frame) into the spine-root world frame — the pivot
    /// offset of an entrance-driven effect host (see `entrance_transform_curves`).
    /// `None` when the GameObject has no transform or its transform has no parent.
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

    /// The GameObject's serialized local `(scale, position)` from its Transform. Used
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

    /// The nearest ancestor GameObject of `go_pid` (self first, up to the spine root)
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

    /// The GameObject `m_Name` of every ancestor of `go_pid`, nearest-first,
    /// walking `m_Father` up to (but excluding) the spine root. In-game, a
    /// particle emitter that drifts with the character is parented under a
    /// spine-driven bone GameObject (spine-unity's `SkeletonUtilityBone`, named
    /// after the bone it mirrors); its name therefore appears in this chain. The
    /// frontend matches these names against the loaded skeleton's bone names to
    /// decide which emitters follow a bone (and which stay world-fixed). The
    /// emitter's own GameObject name is included first so a directly-on-bone
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

    /// Start delay (seconds) of the GameObject's effect, from `_delayTime` activators
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
    /// (`windows`, from {@link super::anim::active_windows}). The apple/glow/wing spark
    /// emitters start OFF and are toggled on mid-cinematic (Virtuosa ~4.8–9.8s); without
    /// this their `_delayTime` is 0 so they'd wrongly emit from t=0 (during the seated
    /// intro) and be spent before the apple falls. `None` when no ancestor is toggled.
    pub(crate) fn entrance_reveal_of_go(
        &self,
        all_objects: &HashMap<i64, (i32, Value)>,
        go_pid: i64,
        windows: &HashMap<i64, super::anim::ActiveWindow>,
    ) -> Option<f32> {
        let mut cur_go = Some(go_pid);
        for _ in 0..256 {
            let g = cur_go?;
            if let Some(&(Some(from), _)) = windows.get(&g) {
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
        let pos = idle
            .pos
            .get(&tf_pid)
            .copied()
            .unwrap_or_else(|| vec3("m_LocalPosition", 0.0));
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
            bg_camera_size: None,
            bg_max_aspect: None,
            bg_camera_offset: None,
            bg_camera_view: None,
            bg_camera_offset2: None,
            bg_camera_view2: None,
            bg_character_sort: None,
            bg_entrance_duration: None,
            bg_entrance_transform: None,
            bg_entrance_view: None,
            bg_entrance_cam_offset: None,
            bg_entrance_ortho_curve: None,
            bg_entrance_pan_curve: None,
            bg_entrance_cam_center: None,
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
///   3. owning `GameObject` named "Front"/"Back" → `BattleFront`/`BattleBack`
///   4. fallback: atlas front count (f_, c_) >= back count (b_) → `BattleFront`, else `BattleBack`
///
/// Step 3 is required for correctness: front and back battle skeletons often
/// share a single atlas (e.g. `char_1048_orchd2`), so the atlas heuristic
/// classifies both the same way and one overwrites the other on export.
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

        // Write skel
        let skel_path = spine_dir.join(format!("{}.skel", asset.name));
        if std::fs::write(&skel_path, &asset.skel_data).is_ok() {
            count += 1;
        }

        // Write atlas
        let atlas_path = spine_dir.join(format!("{}.atlas", asset.name));
        if std::fs::write(&atlas_path, &asset.atlas_text).is_ok() {
            count += 1;
        }

        // Decode textures and apply alpha merging
        let mut decoded = HashMap::new();
        for (_, tex_val) in &asset.textures {
            match decode_texture_object(tex_val, resources) {
                Ok(Some(tex)) => {
                    decoded.insert(tex.name.clone(), tex);
                }
                Ok(None) => {}
                Err(e) => {
                    let name = tex_val["m_Name"].as_str().unwrap_or("?");
                    eprintln!("  error decoding spine texture {name}: {e}");
                }
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
    for tri in indices.chunks_exact(3) {
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

/// Export the full multi-layer scene for the live renderer. Every non-character
/// mesh quad becomes a textured 2D mesh in spine-authored pixels (Y-up, origin at
/// the skeleton root), geometry inlined in `{name}[scene].json`, textures written
/// to a `{name}[scene]/` folder (deduped by source path_id). The frontend draws
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
    // Signatures of already-emitted layers, to drop exact duplicates (some scenes
    // stack identical quad GameObjects, e.g. Skadi "Red Countess" — frozen they
    // just overdraw, and double-brighten when additive).
    let mut seen_sigs: std::collections::HashSet<u64> = std::collections::HashSet::new();

    // Camera-frame extent in authored px (2 * cameraSize half-height), for the
    // oversize test below.
    let frame_extent = asset.bg_camera_size.map(|c| 2.0 * c as f32 * inv);

    // Textures actually emitted by this skin's PARTICLE systems (main + trail).
    // These are the dynamic effects (snow, sparks, glow flecks) the live
    // `[particles]` renderer draws; the scene mesh may ALSO carry a frozen
    // keyframe copy of them (baked particle quads) that must be dropped so they
    // don't stamp static garbage over the animated version.
    let particle_tex_pids: std::collections::HashSet<i64> = asset
        .particles
        .iter()
        .flat_map(|p| [p.tex_pid, p.trail_tex_pid])
        .flatten()
        .collect();

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
                particle_tex_pids.contains(pid)
                    && *count >= 4
                    && frame_extent.is_some_and(|fe| fe > 0.0 && *max_ext < fe * 0.5)
            })
            .map(|(pid, _)| *pid)
            .collect()
    };

    for quad in order {
        if burst_tex.contains(&quad.tex_pid) {
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
        // UNLESS the entrance cinematic sequences the layer (it carries an
        // active window): a clip-revealed layer is a deliberate visual beat,
        // not a helper (Mlynar's white-transition flash samples `mask_09`).
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
                    eprintln!(
                        "  DROP[oversize] '{name_l}' sort={} extent={extent:.0} frame={fe:.0} ({:.1}x)",
                        quad.sort,
                        extent / fe
                    );
                }
                continue;
            }
        }
        // Drop exact-duplicate layers: same texture, depth, blend, and projected
        // geometry. Hash a rounded signature so tiny float noise still collapses.
        {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            quad.tex_pid.hash(&mut h);
            quad.sort.hash(&mut h);
            quad.additive.hash(&mut h);
            for &v in &pos {
                (v.round() as i64).hash(&mut h);
            }
            quad.mesh.indices.hash(&mut h);
            if !seen_sigs.insert(h.finish()) {
                if dbg {
                    eprintln!("  DROP[duplicate] '{name_l}' sort={}", quad.sort);
                }
                continue;
            }
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
            if image::save_buffer(
                tex_dir.join(format!("{idx}.png")),
                &tex.rgba,
                tex.width,
                tex.height,
                image::ColorType::Rgba8,
            )
            .is_ok()
            {
                saved += 1;
            }
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
        if is_opaque && lum_mean < 0.06 && lum_p90 < 0.12 {
            if dbg {
                eprintln!(
                    "  DROP[grabpass] '{name_l}' sort={} lum_mean={lum_mean:.3} p90={lum_p90:.3}",
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
        let is_black_field_glow = (is_premul_alpha && lum_mean < 0.25 && lum_p90 > 0.75)
            || ((is_premul_alpha || is_opaque)
                && lum_mean < 0.25
                && lum_p98 > 0.9
                && dark_frac > 0.6);
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
        // ENTRANCE reveal time (s) — the layer is hidden until its `m_IsActive` switches
        // ON in the `_Start` cinematic (cathedral first, mirror-world + throne later).
        // Omitted (always visible) for main scenes and always-active `_Start` layers.
        if let Some(t) = quad.active_from {
            layer["activeFrom"] = serde_json::json!(t);
        }
        if let Some(t) = quad.active_until {
            layer["activeUntil"] = serde_json::json!(t);
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
        if let Some(uv) = quad.uv_scroll {
            layer["uvScroll"] = serde_json::json!([uv[0], uv[1]]);
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
        if dbg {
            let ext = (xmx - xmn).max(ymx - ymn);
            eprintln!(
                "  KEEP '{name_l}' sort={} size={:.0}x{:.0} ext={ext:.0} add={additive} src/dst={:.0}/{:.0} lum_mean={lum_mean:.3} p90={lum_p90:.3} p98={lum_p98:.3} darkf={dark_frac:.2}",
                quad.sort,
                xmx - xmn,
                ymx - ymn,
                quad.src_blend,
                quad.dst_blend
            );
        }
        layers.push(layer);
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
        "cameraViewPx": asset.bg_camera_view.map(|v| v as f32),
        // The TIGHT/zoomed-in endpoint from `_adjustes[1]` (present only on special-entry
        // skins whose viewer dollies out from a close-up at open). Null when absent.
        "cameraOffsetPx2": asset.bg_camera_offset2.map(|(x, y)| [x as f32, y as f32]),
        "cameraViewPx2": asset.bg_camera_view2.map(|v| v as f32),
        "aspect": asset.bg_max_aspect,
        "characterSort": asset.bg_character_sort,
        // ENTRANCE (`_Start`) cinematic timing (seconds), from the entrance director's
        // `_params.duration` + the reform `_delayTime` cluster. Present only on `_Start`
        // scenes; drives the client entrance camera dolly (tight→wide across `_adjustes`)
        // and the hand-off to the settled idle — so those are gamedata, not guessed.
        "entranceDuration": asset.bg_entrance_duration.map(|v| v as f32),
        "entranceTransform": asset.bg_entrance_transform.map(|v| v as f32),
        // Tight entrance close-up view (authored px) from the entrance camera's ortho size.
        "entranceViewPx": asset.bg_entrance_view.map(|v| v as f32),
        "entranceCamOffsetPx": asset.bg_entrance_cam_offset.map(|(x, y)| [x as f32, y as f32]),
        // Data-driven camera dolly zoom: [[t_seconds, orthographic_size], …] keyframes.
        "entranceOrthoCurve": asset.bg_entrance_ortho_curve.as_ref().map(|c| c.iter().map(|(t, s)| [*t, *s]).collect::<Vec<_>>()),
        "entrancePanCurve": asset.bg_entrance_pan_curve.as_ref().map(|c| c.iter().map(|(t, s)| [*t, *s]).collect::<Vec<_>>()),
        "entranceCamCenterCurve": asset.bg_entrance_cam_center.as_ref().map(|c| c.iter().map(|(t, x, y)| [*t, *x, *y]).collect::<Vec<_>>()),
        "entranceVoiceOffset": asset.bg_entrance_voice.map(|v| v as f32),
        "textureCount": next_idx,
        "layers": layers,
    });
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
