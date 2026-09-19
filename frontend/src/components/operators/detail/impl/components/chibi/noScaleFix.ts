/**
 * Fixes a bug in `@pixi-spine/runtime-3.8` (v4.0.6) `Bone.updateWorldTransformWith()` for
 * `TransformMode.NoScale` bones whose parent is scaled to zero on an axis.
 *
 * pixi-spine renders Y-down by folding a Y flip into every world matrix (`settings.yDown`: the
 * skeleton's scaleY is negated at the root and again at the end of every non-Normal branch),
 * so every bone in the tree carries a NEGATIVE determinant. The NoScale branch then decides
 * whether to mirror the bone's y axis with spine-ts's test, yDown-adjusted:
 *
 *     (parentDet < 0) != (skeleton reflected)      // mirror when they differ
 *
 * which keeps a NoScale bone in its parent's handedness. A parent flattened to scale 0 on one
 * axis has determinant EXACTLY 0, `0 < 0` is false, and the test mirrors the bone: its frame
 * comes out with a positive determinant in a tree where everything else is negative, i.e.
 * flipped against the whole skeleton. Angelina "Mellow Wish" (`char_1015_aglna2_2`) is built on
 * that rig: the 2.5D head turn flattens `HM_Head_X` (scaleY 0) and `HM_Head_Y` (scaleX 0), and
 * their NoScale children `HM_Head_X'` / `HM_Head_Y'` carry 40 of the 63 face vertices, which
 * fold across the face as a skin-coloured sheet over the eyes and mouth (`HM_Body_X'` is the
 * same rig on the torso). In the editor and in plain spine-ts (Y-up, nothing reflected) the
 * same zero determinant reads "not reflected", which matches the unreflected skeleton, so the
 * bone is left alone; the yDown fold is what turns the degenerate parent into a flip.
 *
 * The fix: when the parent's determinant is zero, take the parent's handedness from the
 * nearest ancestor whose matrix is not degenerate, the way a non-flattened parent would have
 * handed it down, and re-run pixi-spine's own NoScale arithmetic with that answer. Everything
 * else is untouched: the original runs first, and a parent with a non-zero determinant is left
 * exactly as it computed it. `?noscalefix=0` reverts. Patches the prototype of the ACTUAL Bone
 * class off a live instance (immune to how the bundle wires runtimes), gated to the 3.8
 * signature (`matrix` + applied `arotation`, vs 4.x's `a`/`b`/`c`/`d` fields), once.
 */

interface Matrix38 {
    a: number;
    b: number;
    c: number;
    d: number;
    tx: number;
    ty: number;
}
interface Bone38 {
    parent: Bone38 | null;
    matrix: Matrix38;
    data: { transformMode: number };
    skeleton: { scaleX: number; scaleY: number };
    arotation: number;
    ascaleX: number;
    ascaleY: number;
    ashearX: number;
    ashearY: number;
    updateWorldTransformWith(x: number, y: number, rotation: number, scaleX: number, scaleY: number, shearX: number, shearY: number): void;
}

/** `TransformMode.NoScale` in the 3.8 enum (Normal, OnlyTranslation, NoRotationOrReflection, NoScale, NoScaleOrReflection). */
const NO_SCALE = 3;
/** The runtime's own threshold: its NoScale branch treats a length below this as zero when it normalises the axis. */
const EPS = 1e-5;
const PATCH_FLAG = "__myrtleNoScaleFix38";
const DEG_RAD = Math.PI / 180;

let enabled: boolean | null = null;
function noScaleFixOn(): boolean {
    if (enabled === null) enabled = typeof window === "undefined" ? true : new URLSearchParams(window.location.search).get("noscalefix") !== "0";
    return enabled;
}

const det = (m: Matrix38): number => m.a * m.d - m.b * m.c;

/** Re-run the 3.8 NoScale branch for `bone` with the parent's handedness taken from its nearest non-degenerate ancestor. */
function recomputeNoScale(bone: Bone38, parent: Bone38, yDown: boolean): void {
    let ancestor: Bone38 | null = parent;
    while (ancestor && Math.abs(det(ancestor.matrix)) <= EPS) ancestor = ancestor.parent;
    const sk = bone.skeleton;
    const skeletonReflected = yDown ? sk.scaleX < 0 !== sk.scaleY > 0 : sk.scaleX < 0 !== sk.scaleY < 0;
    // No non-degenerate ancestor at all: nothing to inherit, so match the skeleton (no mirror).
    const parentReflected = ancestor ? det(ancestor.matrix) < 0 : skeletonReflected;
    const o = sk.scaleX;
    const dY = yDown ? -sk.scaleY : sk.scaleY;
    const pm = parent.matrix;
    const cos = Math.cos(bone.arotation * DEG_RAD);
    const sin = Math.sin(bone.arotation * DEG_RAD);
    let w = (pm.a * cos + pm.c * sin) / o;
    let b = (pm.b * cos + pm.d * sin) / dY;
    let p = Math.sqrt(w * w + b * b);
    if (p > EPS) p = 1 / p;
    w *= p;
    b *= p;
    p = Math.sqrt(w * w + b * b);
    if (parentReflected !== skeletonReflected) p = -p;
    const r = Math.PI / 2 + Math.atan2(b, w);
    const zb = Math.cos(r) * p;
    const zd = Math.sin(r) * p;
    const la = Math.cos(bone.ashearX * DEG_RAD) * bone.ascaleX;
    const lb = Math.cos((90 + bone.ashearY) * DEG_RAD) * bone.ascaleY;
    const lc = Math.sin(bone.ashearX * DEG_RAD) * bone.ascaleX;
    const ld = Math.sin((90 + bone.ashearY) * DEG_RAD) * bone.ascaleY;
    const m = bone.matrix;
    m.a = (w * la + zb * lc) * o;
    m.c = (w * lb + zb * ld) * o;
    m.b = (b * la + zd * lc) * dY;
    m.d = (b * lb + zd * ld) * dY;
}

/**
 * If `spine`'s skeleton uses the 3.8 Bone, wrap its `updateWorldTransformWith` (once) so a
 * NoScale bone under a zero-determinant parent keeps the skeleton's handedness. `yDown` is
 * the runtime's `settings.yDown`, the same flag its own branch reads. A no-op on 4.x.
 */
export function patchSpine38NoScaleUnderFlattenedParent(spine: unknown, yDown: boolean): void {
    const bones = (spine as { skeleton?: { bones?: unknown[] } })?.skeleton?.bones;
    if (!bones || bones.length === 0) return;
    const bone = bones[0] as Record<string, unknown>;
    // 3.8 keeps the world transform in `matrix` and the applied pose in `arotation`; 4.x has
    // neither and does not carry the yDown fold in its bones.
    if (!("matrix" in bone) || !("arotation" in bone)) return;
    const proto = (bone as { constructor: { prototype: Record<string, unknown> } }).constructor.prototype;
    if (proto[PATCH_FLAG]) return;
    const original = proto.updateWorldTransformWith as Bone38["updateWorldTransformWith"];
    proto.updateWorldTransformWith = function (this: Bone38, x: number, y: number, rotation: number, scaleX: number, scaleY: number, shearX: number, shearY: number): void {
        original.call(this, x, y, rotation, scaleX, scaleY, shearX, shearY);
        if (this.data.transformMode !== NO_SCALE || !this.parent || !noScaleFixOn()) return;
        if (Math.abs(det(this.parent.matrix)) > EPS) return;
        recomputeNoScale(this, this.parent, yDown);
    };
    proto[PATCH_FLAG] = true;
}
