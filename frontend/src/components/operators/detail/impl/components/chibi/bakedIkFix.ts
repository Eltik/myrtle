import type { Spine } from "pixi-spine";

/**
 * Drop REDUNDANT IK constraints back to the baked FK pose (dynamic-illustration
 * L2D only).
 *
 * ⚠️ **INERT BY DEFAULT - see `bakedIkEnabled` below.** The "safe by construction"
 * reasoning in this header was measured and is FALSE: Spine applies constraints at
 * runtime, so an animation's keys are the PRE-constraint local transforms and the IK
 * is load-bearing even when every bone it poses is keyed. Disabling this is worth
 * 1.825 MADC on cello alone. The text below is preserved as the original rationale.
 *
 * Arknights dyn_illust skeletons ship FULLY-BAKED animations: every bone is keyed
 * every frame (rotate/translate/scale/shear timelines for all 800+ bones), so the
 * FK pose alone fully defines the illustration. On top of that they ALSO carry IK
 * constraints (authoring rigs the animator used). When both are present, IK wins -
 * it overrides the constrained bones' FK rotation toward the IK target. That's fine
 * when the runtime solves the IK the same way the authoring tool baked it, but
 * pixi-spine's 3.8 IK mis-solves some complex layered arm chains (Archetto "Glory
 * of the Devout": her clasp-controller bone lands at the wrong world position, so
 * the arm IK poses her arms DOWN instead of the baked praying clasp). The baked FK
 * is the source of truth, so we detect IK that is REDUNDANT - every bone it poses
 * is already FK-rotation-keyed by the playing animation - and zero its mix, falling
 * back to the authoritative FK.
 *
 * Safe by construction:
 *  - Gated to DynIllust skeletons (see caller) - battle/dorm chibis untouched.
 *  - An IK is only dropped when ALL its posed bones are rotation-keyed by the
 *    CURRENT animation, i.e. FK already defines them → dropping is LOSSLESS for
 *    correctly-solved rigs and only ever removes a wrong override.
 *  - IK that genuinely drives unkeyed bones (target-driven limbs), or has a
 *    genuinely varying authored mix, is kept.
 *
 * Implemented by wrapping `skeleton.updateWorldTransform` (which runs AFTER
 * `state.apply` sets each constraint's animated mix), zeroing redundant IK mixes
 * just before the constraint cache evaluates. Re-evaluated every frame, so it
 * tracks animation changes.
 */

type IkLike = { mix: number; bones: Array<{ data: { index: number } }>; data: { name: string } };
type AnimLike = {
    timelines: Array<{
        boneIndex?: number;
        ikConstraintIndex?: number;
        frames?: ArrayLike<number>;
        constructor: { name: string };
    }>;
};
type SkelLike = {
    ikConstraints: IkLike[];
    updateWorldTransform: () => void;
    __myrtleBakedIkFix?: boolean;
};
type StateLike = { tracks: Array<{ animation: AnimLike } | null | undefined> };

/** Bone indices whose ROTATION is keyed by the animation (IK overrides rotation,
 *  so rotation-keyed ⇒ FK already defines the bone's orientation). */
function keyedRotationBones(anim: AnimLike): Set<number> {
    const s = new Set<number>();
    for (const tl of anim.timelines) {
        if (typeof tl.boneIndex === "number" && /rotate/i.test(tl.constructor.name)) s.add(tl.boneIndex);
    }
    return s;
}

const IK_TIMELINE_ENTRIES = 6;
const IK_TIMELINE_MIX_OFFSET = 1;

/** IK-constraint indices whose mix genuinely varies across a bound timeline.
 *
 * Spine exports a single (or constant multi-frame) mix declaration for every
 * constraint by default. Only a real mix transition proves the animator authored
 * a toggle, so only those constraints are exempt from the redundant-IK drop. */
function animatedIkMixIndices(anim: AnimLike): Set<number> {
    const s = new Set<number>();
    for (const tl of anim.timelines) {
        if (typeof tl.ikConstraintIndex !== "number" || !/ikconstraint/i.test(tl.constructor.name) || !tl.frames) continue;
        const frames = tl.frames;
        const count = frames.length / IK_TIMELINE_ENTRIES;
        if (count <= 1) continue;
        let min = Infinity;
        let max = -Infinity;
        for (let i = 0; i < count; i++) {
            const mix = frames[i * IK_TIMELINE_ENTRIES + IK_TIMELINE_MIX_OFFSET];
            if (mix < min) min = mix;
            if (mix > max) max = mix;
        }
        if (max - min > 1e-4) s.add(tl.ikConstraintIndex);
    }
    return s;
}

/** **DEFAULT OFF since 2026-08-05 - the premise below was MEASURED FALSE.** Opt in with
 *  `?bakedik=1`.
 *
 *  The argument for the drop was that it is LOSSLESS: dynchar animations are fully baked, every
 *  bone the IK poses is already FK-keyed, so re-applying the constraint should be a no-op and
 *  zeroing its mix can only remove a wrong override. If that held, toggling this would change
 *  NOTHING.
 *
 *  It changes cello by 1.8 MADC. Spine applies constraints at RUNTIME and an animation's keys
 *  are the authored LOCAL transforms - so when an animator poses through IK, the keys are the
 *  PRE-constraint values and the constraint is load-bearing. "Every bone it poses is keyed" is
 *  not evidence of redundancy; on these rigs it is the normal case.
 *
 *      cel  17.167 -> 15.342  (-1.825; t14 -3.91, t17 -3.11, t12 -2.14, no beat regresses)
 *      mly  17.405 -> 17.360      ska  10.404 -> 10.353
 *
 *  Found by symmetric block matching, which localised cello's late-beat error to her INSTRUMENT
 *  swinging 6-8px sideways (sign flipping per beat) while her body stayed pixel-aligned. With
 *  the patch off the leftover displacement collapses from 2.06-2.72 MAE to 0.47-0.77 - the
 *  mechanism, not just the score.
 *
 *  Archetto, the skin this was written for, does NOT regress: rendered both ways at t=6..30 she
 *  is praying in every frame either way, differing only across a small patch at her hands. An
 *  18-skin render sweep (both variants, 4 beats each) found no blow-out - the largest mover is
 *  1.5 mean / 3.3% of pixels, localised to a hand or held prop, with two skins bit-identical.
 *
 *  Kept rather than deleted: if a skin ever does hit a genuine pixi-spine mis-solve, this is the
 *  workaround, and the right gate would be the PATHOLOGY of the solve (Archetto's controller
 *  landed at worldY ~= -805), never "are the bones keyed". */
function bakedIkEnabled(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("bakedik") === "1";
}

export function patchBakedIkRedundancy(spine: Spine): void {
    const skeleton = spine.skeleton as unknown as SkelLike;
    const state = spine.state as unknown as StateLike;
    if (!bakedIkEnabled()) return;
    if (skeleton.__myrtleBakedIkFix) return;
    skeleton.__myrtleBakedIkFix = true;

    const keyedCache = new WeakMap<AnimLike, Set<number>>();
    const animatedIkCache = new WeakMap<AnimLike, Set<number>>();
    const orig = skeleton.updateWorldTransform;
    skeleton.updateWorldTransform = function patched(this: SkelLike) {
        const anim = state.tracks?.[0]?.animation;
        if (anim && skeleton.ikConstraints.length) {
            let keyed = keyedCache.get(anim);
            if (!keyed) {
                keyed = keyedRotationBones(anim);
                keyedCache.set(anim, keyed);
            }
            let animatedIk = animatedIkCache.get(anim);
            if (!animatedIk) {
                animatedIk = animatedIkMixIndices(anim);
                animatedIkCache.set(anim, animatedIk);
            }
            for (const [index, ik] of skeleton.ikConstraints.entries()) {
                if (ik.mix === 0 || animatedIk.has(index)) continue;
                // Redundant ⇔ every bone it poses is FK-rotation-keyed this animation.
                if (ik.bones.length > 0 && ik.bones.every((b) => keyed.has(b.data.index))) ik.mix = 0;
            }
        }
        orig.call(this as unknown as SkelLike);
    };
}
