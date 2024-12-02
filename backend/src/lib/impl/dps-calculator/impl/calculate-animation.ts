import { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { checkSpecs } from "./check-specs";
import dpsAnim from "./dps-animation.json";

export function calculateAnimation(charData: Operator, skillId: string, isSkill: boolean, attackTime: number) {
    const _fps = 30;
    const animData = dpsAnim[charData.id as keyof typeof dpsAnim] || {};
    let animKey: string | boolean = "Attack";
    let attackKey: string | boolean = checkSpecs(charData.id ?? "", "anim_key");
    if (!attackKey) {
        attackKey = ["Attack", "Attack_Loop", "Combat"].find((x) => animData[x as keyof typeof animData]) ?? "";
    }
    const tags: (string | number)[] = [];
    let count = 0; // animKeys中出现的最大技能编号

    // 推断animKey
    if (!isSkill) animKey = attackKey;
    else {
        animKey = checkSpecs(skillId, "anim_key");
        if (!animKey) {
            // 首先，取得技能号
            let skIndex = ~~skillId.split("_")[2];
            const skCount = charData.skills.length;
            // 取得可能描述技能动画时间的animKeys
            const candidates = Object.keys(animData).filter((k) => typeof (animData[k as keyof typeof animData] as { OnAttack: string }).OnAttack == "number" && k.includes("Skill") && !k.includes("Begin") && !k.includes("End"));
            if (typeof (animData as { Skill: string }).Skill == "number") candidates.push("Skill");
            // 分析
            if (candidates.length == 0)
                animKey = attackKey; // 没有合适的技能动画，则使用普攻
            else {
                candidates.forEach((k) => {
                    k.split("_").forEach((t) => {
                        const value = parseInt(t, 10) || t;
                        if (!tags.includes(value)) tags.push(value);
                        if (Number(value) > count) count = Number(value);
                    });
                });
                // 例子：如果有3个技能但是animKeys最大为skill2说明skill2对应3技能
                // 否则skill3对应3技能
                if (skCount > count) skIndex -= 1;
                // 选择最终animKey
                if (skIndex == 0 || count == 0) {
                    animKey = candidates.find((k) => k.includes("Skill")) ?? "";
                } else {
                    animKey = candidates.find((k) => k.includes(String(skIndex))) ?? "";
                }
                if (!animKey) animKey = attackKey;
            }
        }
    }

    // 帧数算法. 258yyds
    const attackFrame = attackTime * _fps; // 理论攻击间隔 换算为帧
    let realAttackFrame = Math.round(attackFrame); // 实际攻击间隔，后面会调整
    let realAttackTime = realAttackFrame / _fps;
    let animFrame = 0,
        eventFrame = -1; // 原本动画帧数，判定帧数
    let scale = 1;
    let scaledAnimFrame = 0; // 缩放后的动画帧数
    let preDelay = 0,
        postDelay = 0;

    if (!animKey || !animData[animKey as keyof typeof animData]) {
    } else {
        const specKey = String(animKey).includes("Attack") ? charData.id : skillId;
        const isLoop = String(animKey).includes("Loop");
        let max_scale: number | boolean = 99;

        if (typeof animData[animKey as keyof typeof animData] == "number") {
            // 没有OnAttack，一般是瞬发或者不攻击的技能
            animFrame = animData[animKey as keyof typeof animData];
        } else if (isLoop && !(animData[animKey as keyof typeof animData] as { OnAttack: string }).OnAttack) {
            animFrame = attackFrame;
            eventFrame = attackFrame;
            scale = 1;
        } else {
            animFrame = (animData[animKey as keyof typeof animData] as { duration: number }).duration;
            eventFrame = (animData[animKey as keyof typeof animData] as { OnAttack: number }).OnAttack;
            if (checkSpecs(specKey ?? "", "anim_max_scale")) {
                max_scale = checkSpecs(specKey ?? "", "anim_max_scale");
            }
            scale = Math.max(Math.min(attackFrame / animFrame, Number(max_scale)), 0.1);
        }

        if (eventFrame >= 0) {
            preDelay = Math.max(Math.round(eventFrame * scale), 1);
            postDelay = Math.max(Math.round(animFrame * scale - preDelay), 1);
            scaledAnimFrame = preDelay + postDelay;
        } else scaledAnimFrame = animFrame;
        if (attackFrame - scaledAnimFrame > 0.5) {
            realAttackFrame = Math.ceil(attackFrame) + 1;
        } else {
            realAttackFrame = Math.max(scaledAnimFrame, Math.round(attackFrame));
        }

        realAttackTime = realAttackFrame / _fps;
    }

    return {
        realAttackTime,
        realAttackFrame,
        preDelay,
        postDelay,
        scaledAnimFrame,
    };
}
