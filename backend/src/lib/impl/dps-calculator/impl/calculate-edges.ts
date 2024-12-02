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
    const passF = frame - remainF;

    console.write("**【边缘情况估算(测试)】**");
    console.write(`技能持续时间: ${durationF} 帧, 抬手 ${attackBegin} 帧, 攻击间隔 ${frame} 帧`);
    console.write(`技能结束时，前一次攻击经过: **${passF} 帧**`);
    console.write(`技能结束时，下一次攻击还需: **${remainF} 帧**`);

    Object.assign(dur, {
        remain: remainF,
    });
}
