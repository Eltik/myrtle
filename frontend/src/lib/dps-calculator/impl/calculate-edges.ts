import { checkSpecs } from "./check-specs";

export function calcEdges(
    blackboard: {
        key: string;
        value: number;
        valueStr?: string | null;
    }[],
    frame: number,
    dur: {
        duration: number;
        attackCount: number;
    },
) {
    const skillId = (blackboard as unknown as { id: string }).id;
    const attackBegin = Number(checkSpecs(skillId, "attack_begin")) || 12; // 抬手时间
    const durationF = Math.round(30 * dur.duration);
    const remainF = attackBegin + frame * dur.attackCount - durationF;

    Object.assign(dur, {
        remain: remainF,
    });
}
