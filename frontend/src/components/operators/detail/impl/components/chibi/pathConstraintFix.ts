/**
 * Fixes a bug in `@pixi-spine/runtime-3.8` (v4.0.6) `PathConstraint.update()`.
 *
 * The shipped build has a duplicated guard in the rotation branch:
 *
 *     if (tangents)
 *       if (tangents) r = positions[p - 1];   // ← the extra `if (tangents)`
 *       else if (spaces[i + 1] == 0) r = positions[p + 2];
 *       else r = Math.atan2(dy, dx);
 *
 * so when `tangents` is false (RotateMode.Chain — the common case) the whole
 * block is skipped and `r` stays 0: the path-constrained bones never rotate to
 * follow their path. Weighted meshes bound to those bones then collapse into
 * horizontal smears — e.g. Zuo Le "Youthful Journey" loses his legs, which are
 * driven by Chain-mode path constraints (`ZL_Pelvis*` → `ZL_*_Leg_A_Path`).
 * runtime-4.1 ships the correct code, so this only touches the 3.8 class.
 *
 * We patch the prototype of the ACTUAL class the loaded skeleton uses (grabbed
 * off a live instance), so it's immune to how the loader-uni bundles runtimes,
 * and gated to the 3.8 signature (`rotateMix`/`translateMix`, vs 4.x's
 * `mixRotate`/`mixX`/`mixY`) so a 4.x skeleton is never touched. Runs once.
 */

interface Bone38 {
    data: { length: number };
    matrix: { a: number; b: number; c: number; d: number; tx: number; ty: number };
    appliedValid: boolean;
}
interface PathConstraint38 {
    target: { getAttachment(): unknown; bone: { matrix: { a: number; b: number; c: number; d: number } } };
    bones: Bone38[];
    rotateMix: number;
    translateMix: number;
    spacing: number;
    spaces: number[];
    lengths: number[];
    data: { spacingMode: number; rotateMode: number; positionMode: number; offsetRotation: number };
    computeWorldPositions(attachment: unknown, spacesCount: number, tangents: boolean, percentPosition: boolean, percentSpacing: boolean): number[];
}

const setArraySize = (a: number[], n: number): number[] => {
    if (a.length > n) a.length = n;
    else while (a.length < n) a.push(0);
    return a;
};

/** The correct spine-3.8 `PathConstraint.update()` (see file header). */
function fixedUpdate(this: PathConstraint38): void {
    const attachment = this.target.getAttachment() as { closed?: boolean; worldVerticesLength?: number } | null;
    // Duck-type PathAttachment (avoids importing the 3.8-only class): only a path
    // attachment carries both `closed` and `worldVerticesLength`.
    if (!attachment || attachment.closed === undefined || attachment.worldVerticesLength === undefined) return;
    const rotateMix = this.rotateMix;
    const translateMix = this.translateMix;
    const translate = translateMix > 0;
    const rotate = rotateMix > 0;
    if (!translate && !rotate) return;
    const data = this.data;
    const spacingMode = data.spacingMode; // Length=0, Fixed=1, Percent=2
    const lengthSpacing = spacingMode === 0;
    const rotateMode = data.rotateMode; // Tangent=0, Chain=1, ChainScale=2
    const tangents = rotateMode === 0;
    const scale = rotateMode === 2;
    const boneCount = this.bones.length;
    const spacesCount = tangents ? boneCount : boneCount + 1;
    const bones = this.bones;
    const spaces = setArraySize(this.spaces, spacesCount);
    let lengths: number[] | null = null;
    const spacing = this.spacing;
    const EPS = 1e-5;
    if (scale || lengthSpacing) {
        if (scale) lengths = setArraySize(this.lengths, boneCount);
        for (let i = 0, n = spacesCount - 1; i < n; ) {
            const bone = bones[i];
            const setupLength = bone.data.length;
            if (setupLength < EPS) {
                if (scale && lengths) lengths[i] = 0;
                spaces[++i] = 0;
            } else {
                const x = setupLength * bone.matrix.a;
                const y = setupLength * bone.matrix.b;
                const length = Math.sqrt(x * x + y * y);
                if (scale && lengths) lengths[i] = length;
                spaces[++i] = ((lengthSpacing ? setupLength + spacing : spacing) * length) / setupLength;
            }
        }
    } else {
        for (let i = 1; i < spacesCount; i++) spaces[i] = spacing;
    }
    const positions = this.computeWorldPositions(attachment, spacesCount, tangents, data.positionMode === 1, spacingMode === 2);
    let boneX = positions[0];
    let boneY = positions[1];
    let offsetRotation = data.offsetRotation;
    let tip = false;
    if (offsetRotation === 0) {
        tip = rotateMode === 1;
    } else {
        tip = false;
        const p = this.target.bone.matrix;
        offsetRotation *= p.a * p.d - p.b * p.c > 0 ? Math.PI / 180 : -Math.PI / 180;
    }
    const PI = Math.PI;
    const PI2 = Math.PI * 2;
    for (let i = 0, p = 3; i < boneCount; i++, p += 3) {
        const bone = bones[i];
        const mat = bone.matrix;
        mat.tx += (boneX - mat.tx) * translateMix;
        mat.ty += (boneY - mat.ty) * translateMix;
        const x = positions[p];
        const y = positions[p + 1];
        const dx = x - boneX;
        const dy = y - boneY;
        if (scale && lengths) {
            const length = lengths[i];
            if (length !== 0) {
                const s = (Math.sqrt(dx * dx + dy * dy) / length - 1) * rotateMix + 1;
                mat.a *= s;
                mat.b *= s;
            }
        }
        boneX = x;
        boneY = y;
        if (rotate) {
            const a = mat.a;
            const b = mat.c;
            const c = mat.b;
            const d = mat.d;
            let r: number;
            let cos: number;
            let sin: number;
            // THE FIX: the shipped build wraps this in a second `if (tangents)`.
            if (tangents) r = positions[p - 1];
            else if (spaces[i + 1] === 0) r = positions[p + 2];
            else r = Math.atan2(dy, dx);
            r -= Math.atan2(c, a);
            if (tip) {
                cos = Math.cos(r);
                sin = Math.sin(r);
                const length = bone.data.length;
                boneX += (length * (cos * a - sin * c) - dx) * rotateMix;
                boneY += (length * (sin * a + cos * c) - dy) * rotateMix;
            } else {
                r += offsetRotation;
            }
            if (r > PI) r -= PI2;
            else if (r < -PI) r += PI2;
            r *= rotateMix;
            cos = Math.cos(r);
            sin = Math.sin(r);
            mat.a = cos * a - sin * c;
            mat.c = cos * b - sin * d;
            mat.b = sin * a + cos * c;
            mat.d = sin * b + cos * d;
        }
        bone.appliedValid = false;
    }
}

const PATCH_FLAG = "__myrtlePathFix38";

/**
 * If `spine`'s skeleton uses the buggy 3.8 PathConstraint, patch its prototype
 * (once). Safe to call on every loaded spine; a no-op for skeletons without path
 * constraints or on the 4.x runtime.
 */
export function patchSpine38PathConstraint(spine: unknown): void {
    const pcs = (spine as { skeleton?: { pathConstraints?: unknown[] } })?.skeleton?.pathConstraints;
    if (!pcs || pcs.length === 0) return;
    const pc = pcs[0] as Record<string, unknown>;
    // 3.8 uses rotateMix/translateMix; 4.x uses mixRotate/mixX/mixY (and ships the
    // correct code) — only touch the 3.8 class.
    if (!("rotateMix" in pc) || !("translateMix" in pc)) return;
    const proto = (pc as { constructor: { prototype: Record<string, unknown> } }).constructor.prototype;
    if (proto[PATCH_FLAG]) return;
    proto.update = fixedUpdate;
    proto[PATCH_FLAG] = true;
}
