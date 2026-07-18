import type { Spine } from "pixi-spine";

/**
 * Drop REDUNDANT IK constraints back to the baked FK pose (dynamic-illustration
 * L2D only).
 *
 * Arknights dyn_illust skeletons ship FULLY-BAKED animations: every bone is keyed
 * every frame (rotate/translate/scale/shear timelines for all 800+ bones), so the
 * FK pose alone fully defines the illustration. On top of that they ALSO carry IK
 * constraints (authoring rigs the animator used). When both are present, IK wins —
 * it overrides the constrained bones' FK rotation toward the IK target. That's fine
 * when the runtime solves the IK the same way the authoring tool baked it, but
 * pixi-spine's 3.8 IK mis-solves some complex layered arm chains (Archetto "Glory
 * of the Devout": her clasp-controller bone lands at the wrong world position, so
 * the arm IK poses her arms DOWN instead of the baked praying clasp). The baked FK
 * is the source of truth, so we detect IK that is REDUNDANT — every bone it poses
 * is already FK-rotation-keyed by the playing animation — and zero its mix, falling
 * back to the authoritative FK.
 *
 * Safe by construction:
 *  - Gated to DynIllust skeletons (see caller) — battle/dorm chibis untouched.
 *  - An IK is only dropped when ALL its posed bones are rotation-keyed by the
 *    CURRENT animation, i.e. FK already defines them → dropping is LOSSLESS for
 *    correctly-solved rigs and only ever removes a wrong override.
 *  - IK that genuinely drives unkeyed bones (target-driven limbs) is kept.
 *
 * Implemented by wrapping `skeleton.updateWorldTransform` (which runs AFTER
 * `state.apply` sets each constraint's animated mix), zeroing redundant IK mixes
 * just before the constraint cache evaluates. Re-evaluated every frame, so it
 * tracks animation changes.
 */

type IkLike = { mix: number; bones: Array<{ data: { index: number } }>; data: { name: string } };
type AnimLike = { timelines: Array<{ boneIndex?: number; constructor: { name: string } }> };
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

export function patchBakedIkRedundancy(spine: Spine): void {
    const skeleton = spine.skeleton as unknown as SkelLike;
    const state = spine.state as unknown as StateLike;
    if (skeleton.__myrtleBakedIkFix) return;
    skeleton.__myrtleBakedIkFix = true;

    const cache = new WeakMap<AnimLike, Set<number>>();
    const orig = skeleton.updateWorldTransform;
    skeleton.updateWorldTransform = function patched(this: SkelLike) {
        const anim = state.tracks?.[0]?.animation;
        if (anim && skeleton.ikConstraints.length) {
            let keyed = cache.get(anim);
            if (!keyed) {
                keyed = keyedRotationBones(anim);
                cache.set(anim, keyed);
            }
            for (const ik of skeleton.ikConstraints) {
                if (ik.mix === 0) continue;
                // Redundant ⇔ every bone it poses is FK-rotation-keyed this animation.
                if (ik.bones.length > 0 && ik.bones.every((b) => keyed.has(b.data.index))) ik.mix = 0;
            }
        }
        orig.call(this as unknown as SkelLike);
    };
}
