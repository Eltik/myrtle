import { CalculateDPSParams } from "../../../../types/impl/lib/impl/dps-calculator";
import { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { applyBuff } from "./apply-buff";
import { calculateAnimation } from "./calculate-animation";
import { calcDurations } from "./calculate-durations";
import { calcEdges } from "./calculate-edges";
import { calculateGradDamage } from "./calculate-gradual-damage";
import { checkResetAttack } from "./check-reset-attack";
import { checkSpecs } from "./check-specs";
import { extractDamageType } from "./extract-damage-type";
import { getAttributes } from "./get-attributes";
import { getBuffedAttributes } from "./get-buffed-attributes";

export function calculateAttack(
    charAttr: {
        basic: { [key: string]: any };
        buffs: { [key: string]: any };
        buffList: any;
        char: Operator;
    },
    options: CalculateDPSParams["options"],
    operatorData: {
        equipId: string;
        equipLevel: number;
        potentialRank: number;
        phaseIndex: number;
        level: number;
        favor: number;
        skillId: string;
    },
    enemy: {
        def: number;
        res: number;
        count: number;
    },
    raidBlackboard: {
        atk: number;
        atk_override: number;
        attack_speed: number;
        sp_recovery_per_sec: number;
        base_atk: number;
        damage_scale: number;
    },
    isSkill: boolean,
    charData: Operator,
    levelData: {
        name: string;
        rangeId: string | null;
        description: string;
        skillType: number;
        duration: number;
        spData: {
            spType: string;
            levelUpCost: [];
            maxChargeTime: number;
            spCost: number;
            initSp: number;
            increment: number;
        };
        prefabId: string;
        blackboard: {
            key: string;
            value: number;
            valueStr: string | null;
        }[];
    },
) {
    const charId = charAttr.char.id ?? "";
    const buffList = charAttr.buffList;
    const blackboard = buffList.skill;
    const basicFrame = charAttr.basic;

    // 备注信息
    if (isSkill && checkSpecs(charId, "note")) console.log(checkSpecs(charId, "note"));
    if (options.equip && operatorData.equipId) console.log("满足模组触发条件");

    // 计算面板属性
    console.write("**【Buff计算】**");
    let buffFrame = {
        atk_scale: 1,
        def_scale: 1,
        heal_scale: 1,
        damage_scale: 1,
        maxTarget: 1,
        times: 1,
        edef: 0,
        edef_scale: 1,
        edef_pene: 0,
        edef_pene_scale: 0,
        emr_pene: 0,
        emr: 0,
        emr_scale: 1,
        atk: 0,
        def: 0,
        attackSpeed: 0,
        maxHp: 0,
        baseAttackTime: 0,
        spRecoveryPerSec: 0,
        spRecoverRatio: 0,
        spRecoverIntervals: [],
        applied: {},
    };
    for (const b in buffList) {
        const buffName = b === "skill" ? buffList[b].id : b;
        //console.log(buffName);
        if (!checkSpecs(buffName, "crit")) buffFrame = applyBuff(charAttr, options, operatorData, buffFrame, b, buffList[b], isSkill, false, enemy) as any;
    }
    // 计算团辅
    if (options.buff) buffFrame = applyBuff(charAttr, options, operatorData, buffFrame, "raidBuff", raidBlackboard as any, isSkill, false, enemy) as any;

    // 攻击类型
    let damageType = extractDamageType(charData, charAttr.char, isSkill, levelData.description, blackboard, options);
    if (damageType == 2) buffFrame.atk_scale *= buffFrame.heal_scale;
    // 灰喉-特判
    if (buffList["tachr_367_swllow_1"]) {
        buffFrame.attackSpeed += buffList["tachr_367_swllow_1"].attack_speed;
    }
    // 泡泡
    if (isSkill && blackboard.id == "skchr_bubble_2") {
        buffFrame.atk = basicFrame.def + buffFrame.def - basicFrame.atk;
    }
    // 迷迭香
    if (["char_391_rosmon", "char_1027_greyy2", "char_421_crow", "char_431_ashlok", "char_4066_highmo", "char_4039_horn"].includes(charId)) {
        if (charId == "char_4039_horn" && options.melee) {
        } else {
            buffFrame.maxTarget = 999;
        }
    }
    // 连击特判
    if (!isSkill && checkSpecs(charId, "times")) {
        const t = checkSpecs(charId, "times");
        Object.assign(buffFrame, { times: t });
    }
    if (isSkill && checkSpecs(blackboard.id, "times")) {
        const t = checkSpecs(blackboard.id, "times");
        Object.assign(buffFrame, { times: t });
    }

    let finalFrame = getBuffedAttributes(basicFrame, buffFrame);
    let critBuffFrame = {
        atk_scale: 1,
        def_scale: 1,
        heal_scale: 1,
        damage_scale: 1,
        maxTarget: 1,
        times: 1,
        edef: 0,
        edef_scale: 1,
        edef_pene: 0,
        edef_pene_scale: 0,
        emr_pene: 0,
        emr: 0,
        emr_scale: 1,
        atk: 0,
        def: 0,
        attackSpeed: 0,
        maxHp: 0,
        baseAttackTime: 0,
        spRecoveryPerSec: 0,
        spRecoverRatio: 0,
        spRecoverIntervals: [],
        applied: {},
    };
    let critFrame = {};
    // 暴击面板
    if (options.crit) {
        console.write("**【暴击Buff计算】**");
        for (const b in buffList) {
            critBuffFrame = applyBuff(charAttr, options, operatorData, critBuffFrame, b, buffList[b], isSkill, true, enemy) as any;
        }
        // 计算团辅
        if (options.buff) critBuffFrame = applyBuff(charAttr, options, operatorData, critBuffFrame, "raidBuff", raidBlackboard as any, isSkill, true, enemy) as any;
        critFrame = getBuffedAttributes(basicFrame, critBuffFrame);
    }
    // ---- 计算攻击参数
    // 最大目标数
    if (charData.description.includes("阻挡的<@ba.kw>所有敌人") && buffFrame.maxTarget < basicFrame.blockCnt) {
        buffFrame.maxTarget = basicFrame.blockCnt;
    } else if (["所有敌人", "群体法术伤害", "群体物理伤害"].some((kw) => charData.description.includes(kw))) {
        buffFrame.maxTarget = 999;
    } else if (charData.description.includes("恢复三个") && !(isSkill && charId == "char_275_breeze")) {
        buffFrame.maxTarget = Math.max(buffFrame.maxTarget, 3);
    }
    if (options.token) {
        if (blackboard.id == "skchr_mgllan_3" || (isSkill && blackboard.id == "skchr_mgllan_2")) buffFrame.maxTarget = 999;
        if (blackboard.id == "skchr_ling_3") buffFrame.maxTarget = options.ling_fusion ? 4 : 2;
    }
    // 计算最终攻击间隔，考虑fps修正
    const fps = 30;
    // 攻速上下界
    const _spd = Math.min(Math.max(10, finalFrame.attackSpeed), 600);
    if (finalFrame.attackSpeed != _spd) {
        finalFrame.attackSpeed = _spd;
        console.log("达到攻速极限");
    }

    // sec spec
    if ((checkSpecs(blackboard.id, "sec") && isSkill) || (options.annie && charId == "char_1023_ghost2")) {
        let intv: number | boolean = 1;
        if (checkSpecs(blackboard.id, "interval")) {
            intv = checkSpecs(blackboard.id, "interval");
        }
        finalFrame.baseAttackTime = intv;
        finalFrame.attackSpeed = 100;
        buffFrame.attackSpeed = 0;
        console.log(`每 ${intv} 秒造成一次伤害/治疗`);
    }

    const realAttackTime = (finalFrame.baseAttackTime * 100) / finalFrame.attackSpeed;
    let frame = realAttackTime * fps;
    // 额外帧数补偿 https://bbs.nga.cn/read.php?tid=20555008
    let corr = checkSpecs(charId, "frame_corr") || 0;
    const corr_s = checkSpecs(blackboard.id, "frame_corr");
    if (!(corr_s === false) && isSkill) corr = corr_s;
    if (corr != 0) {
        let real_frame = Math.ceil(frame); // 有误差时，不舍入而取上界，并增加补正值(一般为1)
        real_frame += Number(corr);
        frame = real_frame;
    } else {
        frame = Math.round(frame); // 无误差时，舍入成帧数
    }
    const frameAttackTime = frame / fps;
    const attackTime = frameAttackTime;
    calculateAnimation(charData, blackboard.id, isSkill, realAttackTime);
    if (isSkill && blackboard.id == "skchr_platnm_2") {
        let rate = (attackTime - 1) / (buffList["tachr_204_platnm_1"]["attack@max_delta"] - 1);
        // 熔断
        rate = Math.min(Math.max(rate, 0), 1);
        buffFrame.atk_scale = 1 + rate * (buffList["tachr_204_platnm_1"]["attack@max_atk_scale"] - 1);
        finalFrame = getBuffedAttributes(basicFrame, buffFrame); // 重算
    } else if (buffList["tachr_215_mantic_1"] && attackTime >= buffList["tachr_215_mantic_1"].delay) {
        // 狮蝎
        const atk = basicFrame.atk * buffList["tachr_215_mantic_1"].atk;
        finalFrame.atk += atk;
        buffFrame.atk = finalFrame.atk - basicFrame.atk;
    }

    // 敌人属性
    let enemyBuffFrame = JSON.parse(JSON.stringify(buffFrame));
    // 处理对普攻也生效的debuff
    for (const b in buffList) {
        const buffName = b == "skill" ? buffList[b].id : b;
        if (checkSpecs(buffName, "keep_debuff") && !enemyBuffFrame.applied[buffName]) {
            enemyBuffFrame = applyBuff(charAttr, options, operatorData, enemyBuffFrame, buffName, buffList[b], true, false, enemy);
        }
    }
    let edef = Math.max(0, ((enemy.def + enemyBuffFrame.edef) * enemyBuffFrame.edef_scale - enemyBuffFrame.edef_pene) * (1 - enemyBuffFrame.edef_pene_scale));
    let emr = Math.min((enemy.res + enemyBuffFrame.emr) * enemyBuffFrame.emr_scale, 100);
    emr = Math.max(emr - enemyBuffFrame.emr_pene, 0);
    const emrpct = emr / 100;
    let ecount = Math.min(buffFrame.maxTarget, enemy.count);
    if (blackboard.id == "skchr_pudd_2" && isSkill && ecount > 1) {
        ecount = buffFrame.maxTarget;
    }

    // 平均化链法伤害
    if (["chain", "chainhealer"].includes(charData.subProfessionId)) {
        let scale = 0.85,
            s = 1,
            tot = 1;
        const sks = [1];
        if (isSkill && blackboard.id == "skchr_leizi_2") scale = 1;
        else if (operatorData.equipId && charAttr.char.modules.find((module) => module.id === operatorData.equipId)) scale = basicFrame.equip_blackboard.trait["attack@chain.atk_scale"];
        else if (charData.subProfessionId == "chainhealer") {
            const prefabId = charId.replace("char", "tachr") + "_trait";
            scale = buffList[prefabId]["attack@chain.atk_scale"];
        }

        for (let i = 0; i < ecount - 1; ++i) {
            s *= scale;
            tot += s;
            sks.push(s);
        }

        buffFrame.damage_scale *= tot / ecount;
    }

    // 计算攻击次数和持续时间
    const dur = calcDurations(isSkill, attackTime, finalFrame.attackSpeed, levelData as any, buffList, buffFrame, ecount, options, charData);

    // 计算边缘情况
    const rst = checkResetAttack(blackboard.id, blackboard, options);
    if (rst && String(rst) !== "ogcd" && isSkill) {
        calcEdges(blackboard, frame, dur);
    }
    // 暴击次数
    if (options.crit && critBuffFrame["prob" as keyof typeof critBuffFrame]) {
        if (damageType != 2) {
            if (buffList["tachr_155_tiger_1"]) {
                Object.assign(dur, {
                    critCount:
                        (dur.duration / 3) *
                        (
                            critBuffFrame as unknown as {
                                prob: number;
                            }
                        ).prob,
                });
            } else if (charId == "char_420_flamtl") {
                Object.assign(dur, {
                    critCount: Math.floor(dur.duration / 5),
                });

                switch (blackboard.id) {
                    case "skchr_flamtl_1":
                    case "skchr_flamtl_2":
                        if (!isSkill) (dur as unknown as { critCount: number }).critCount += 1;
                        break;
                    case "skchr_flamtl_3":
                        if (isSkill) (dur as unknown as { critCount: number }).critCount += 2;
                        break;
                }
                console.log(`按闪避 ${(dur as unknown as { critCount: number }).critCount} 次计算`);
            } else if (blackboard.id == "skchr_aurora_2" && isSkill) {
                (dur as unknown as { critCount: number }).critCount = options.freeze ? 9 : 3;
                console.log(`按 ${(dur as unknown as { critCount: number }).critCount} 次暴击计算`);
            } else (dur as unknown as { critCount: number }).critCount = dur.attackCount * (critBuffFrame as unknown as { prob: number }).prob;

            if ((dur as unknown as { critCount: number }).critCount > 1) (dur as unknown as { critCount: number }).critCount = Math.floor((dur as unknown as { critCount: number }).critCount);
            // 折算为命中次数
            if (buffList["tachr_222_bpipe_1"]) {
                (dur as unknown as { critHitCount: number }).critHitCount = (dur as unknown as { critCount: number }).critCount * dur.times * Math.min(enemy.count, 2);
            } else if (charId == "char_420_flamtl") {
                (dur as unknown as { critHitCount: number }).critHitCount = (dur as unknown as { critCount: number }).critCount * 2 * enemy.count;
            } else (dur as unknown as { critHitCount: number }).critHitCount = (dur as unknown as { critCount: number }).critCount * dur.times * ecount;

            if (charId == "char_1021_kroos2") {
                (dur as unknown as { critHitCount: number }).critHitCount = Math.floor(dur.hitCount * (critBuffFrame as unknown as { prob: number }).prob);
                dur.hitCount -= (dur as unknown as { critHitCount: number }).critHitCount;
            } else {
                dur.hitCount = (dur.attackCount - (dur as unknown as { critCount: number }).critCount) * dur.times * ecount;
            }
        } else {
            (dur as unknown as { critCount: number }).critCount = 0;
            (dur as unknown as { critHitCount: number }).critHitCount = 0;
        }
    } else {
        (dur as unknown as { critCount: number }).critCount = 0;
        (dur as unknown as { critHitCount: number }).critHitCount = 0;
    }

    // 输出面板数据
    console.write("\n**【最终面板】**");
    const atk_line = `(${basicFrame.atk.toFixed(1)} + ${buffFrame.atk.toFixed(1)}) * ${buffFrame.atk_scale.toFixed(2)}`;
    // if (buffFrame.damage_scale != 1) { atk_line += ` * ${buffFrame.damage_scale.toFixed(2)}`; }
    console.write(`攻击力 / 倍率:  ${finalFrame.atk.toFixed(2)} = ${atk_line}`);
    console.write(`攻击间隔: ${finalFrame.baseAttackTime.toFixed(3)} s`);
    console.write(`攻速: ${finalFrame.attackSpeed} %`);
    console.write(`最终攻击间隔: ${(realAttackTime * 30).toFixed(2)} 帧, ${realAttackTime.toFixed(3)} s`);
    if (corr != 0) {
        console.write(`**帧数补正后攻击间隔: ${frame} 帧, ${frameAttackTime.toFixed(3)} s**`);
    } else {
        console.write(`**帧对齐攻击间隔: ${frame} 帧, ${frameAttackTime.toFixed(3)} s**`);
    }

    if (edef != enemy.def) console.write(`敌人防御: ${edef.toFixed(1)} (${(edef - enemy.def).toFixed(1)})`);
    if (emr != enemy.res) {
        const rate = (emr - enemy.res) / enemy.res;
        console.write(`敌人魔抗: ${emr.toFixed(1)}% (${(rate * 100).toFixed(1)}%)`);
    }
    if (ecount > 1 || enemy.count > 1) console.write(`目标数: ${ecount} / ${enemy.count}`);

    // 计算伤害
    console.write("\n**【伤害计算】**");
    console.write(`伤害类型: ${["物理", "法术", "治疗", "真伤"][damageType]}`);
    const dmgPrefix = damageType == 2 ? "治疗" : "伤害";
    let hitDamage = finalFrame.atk;
    let critDamage = 0;
    const damagePool = [0, 0, 0, 0, 0]; // 物理，魔法，治疗，真伤，盾
    const extraDamagePool = [0, 0, 0, 0, 0];
    let move = 0;

    function calculateHitDamage(frame: { [key: string]: any }, scale: number) {
        let minRate = 0.05,
            ret = 0;
        if (buffList["tachr_144_red_1"]) minRate = buffList["tachr_144_red_1"].atk_scale;
        if (buffList["tachr_366_acdrop_1"]) {
            minRate = options.cond ? buffList["tachr_366_acdrop_1"].atk_scale_2 : buffList["tachr_366_acdrop_1"].atk_scale;
        }
        if (damageType == 0) ret = Math.max(frame.atk - edef, frame.atk * minRate);
        else if (damageType == 1) ret = Math.max(frame.atk * (1 - emrpct), frame.atk * minRate);
        else ret = frame.atk;
        if (ret <= frame.atk * minRate) console.write("[抛光]");
        if (scale != 1) {
            ret *= scale;
            console.write(`攻击增伤: ${scale.toFixed(2)}x`);
        }
        return ret;
    }

    hitDamage = calculateHitDamage(finalFrame, buffFrame.damage_scale);
    damagePool[damageType] += hitDamage * dur.hitCount;
    console.write(`${dmgPrefix} ${hitDamage.toFixed(2)} * 命中 ${dur.hitCount.toFixed(1)} = ${(hitDamage * dur.hitCount).toFixed(1)} 直接${dmgPrefix}`);

    // 计算额外伤害
    // 暴击
    if (options.crit) {
        // console.log(critBuffFrame);
        if (blackboard.id == "skchr_peacok_2") {
            (dur as unknown as { critHitCount: number }).critHitCount = 0;
            if (isSkill) {
                damageType = 1;
                ecount = enemy.count;
                (dur as unknown as { critHitCount: number }).critHitCount = enemy.count;
            }
        }
        edef = Math.max(0, ((enemy.def + critBuffFrame.edef) * critBuffFrame.edef_scale - critBuffFrame.edef_pene) * (1 - critBuffFrame.edef_pene_scale));
        if (edef != enemy.def) console.write(`[暴击]敌人防御: ${edef.toFixed(1)} (${(edef - enemy.def).toFixed(1)})`);
        critDamage = calculateHitDamage(critFrame, critBuffFrame.damage_scale);
        if (critDamage > 0 && (dur as unknown as { critHitCount: number }).critHitCount > 0) {
            console.write(`暴击${dmgPrefix}: ${critDamage.toFixed(2)}, 命中 ${(dur as unknown as { critHitCount: number }).critHitCount}`);
        }
        damagePool[damageType] += critDamage * (dur as unknown as { critHitCount: number }).critHitCount;
    }
    // 空(被动治疗没有写在天赋中)
    if (["char_1012_skadi2", "char_101_sora", "char_4045_heidi"].includes(charId)) {
        let ratio_sora = 0.1;
        if (isSkill && blackboard.id == "skchr_skadi2_3") ratio_sora = 0;
        else if (isSkill && blackboard["attack@atk_to_hp_recovery_ratio"]) ratio_sora = blackboard["attack@atk_to_hp_recovery_ratio"];
        extraDamagePool[2] = ratio_sora * finalFrame.atk * dur.duration * enemy.count;
        damagePool[2] = 0;
        damagePool[3] = 0;
        console.write("[特殊] 伤害为0 （以上计算无效），可以治疗召唤物");
        console.log("可以治疗召唤物");
    }
    // 反射类-增加说明
    if (checkSpecs(blackboard.id, "reflect") && isSkill) {
        console.log(`技能伤害为反射 ${dur.attackCount} 次的伤害`);
    }
    // 可变攻击力-重新计算
    if (checkSpecs(charId, "grad") || (checkSpecs(blackboard.id, "grad") && isSkill)) {
        if (blackboard.id == "skchr_kalts_3" && !options.token) {
            /* skip */
        } else {
            const kwargs = {
                charId,
                skillId: blackboard.id,
                isSkill,
                options,
                basicFrame,
                buffFrame,
                finalFrame,
                buffList,
                blackboard,
                dur,
                attackTime,
                hitDamage,
                damageType,
                edef,
                ecount,
                emrpct,
                char: charData,
            };

            console.write("[特殊] 可变技能，重新计算伤害 ----");
            damagePool[damageType] = calculateGradDamage(kwargs);
        }
    }

    // 额外伤害
    for (const b in buffList) {
        let buffName = b;
        const bb = buffList[b]; // blackboard
        if (buffName == "skill") {
            buffName = bb.id;
        }
        const pool = [0, 0, 0, 0, 0]; // 物理，魔法，治疗，真伤，盾
        let damage = 0;
        let heal = 0,
            atk = 0;

        if (!isSkill) {
            // 只在非技能期间生效
            switch (buffName) {
                // 伤害
                case "skchr_ethan_1":
                    pool[1] += bb["attack@poison_damage"] * dur.duration * (1 - emrpct) * ecount;
                    break;
                case "skchr_aglina_2":
                case "skchr_aglina_3":
                case "skchr_beewax_1":
                case "skchr_beewax_2":
                case "skchr_billro_1":
                case "skchr_billro_2":
                case "skchr_billro_3":
                case "skchr_mint_1":
                case "skchr_mint_2":
                    damagePool[1] = 0;
                    break;
                case "skchr_takila_1":
                case "skchr_takila_2":
                case "skchr_mlynar_1":
                case "skchr_mlynar_2":
                case "skchr_mlynar_3":
                    damagePool[0] = damagePool[3] = 0;
                    break;
                case "skcom_heal_up[3]":
                    if (options.token) {
                        damagePool[0] = damagePool[2] = 0;
                    }
                    break;
                case "skchr_silent_2":
                    if (options.token) {
                        damagePool[2] = 0;
                    }
                    break;
                case "skchr_ghost2_1":
                case "skchr_ghost2_2":
                case "skchr_ghost2_3":
                    if (options.annie) {
                        damagePool[1] = 0;
                    }
                    break;
                case "skchr_ironmn_1":
                case "skchr_ironmn_2":
                case "skchr_ironmn_3":
                    if (options.token) {
                        damagePool[0] = 0;
                        console.write("不普攻");
                    }
                default:
                    if (b == "skill") continue; // 非技能期间，跳过其他技能的额外伤害判定
            }
        }
        //console.log(buffName);
        switch (buffName) {
            case "tachr_129_bluep_1":
                damage = Math.max(bb.poison_damage * (1 - emrpct), bb.poison_damage * 0.05);
                let total_damage = damage * dur.duration * ecount;
                if (isSkill && blackboard.id == "skchr_bluep_1" && ecount > 1) {
                    const damage2 = damage * blackboard.atk_scale;
                    total_damage = damage * dur.duration + damage2 * 3;
                }
                pool[1] += total_damage;
                console.log("毒伤按循环时间计算");
                break;
            case "tachr_293_thorns_1":
                const poison: number = options.thorns_ranged ? bb["damage[ranged]"] : bb["damage[normal]"];
                damage = Math.max(poison * (1 - emrpct), poison * 0.05) * dur.duration * ecount;
                pool[1] = damage;
                if (isSkill) console.log("毒伤按循环时间计算");
                break;
            case "tachr_346_aosta_1":
                let poison2 = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale;
                if (blackboard.id == "skchr_aosta_2") poison2 *= blackboard.talent_scale;
                console.write(`流血伤害/秒: ${poison2.toFixed(1)}`);
                damage = Math.max(poison2 * (1 - emrpct), poison2 * 0.05) * dur.duration * ecount;
                pool[1] = damage;
                if (isSkill) console.log("毒伤按循环时间计算");
                break;
            case "tachr_181_flower_1":
                pool[2] += bb.atk_to_hp_recovery_ratio * finalFrame.atk * dur.duration * enemy.count;
                if (isSkill) console.log("可以治疗召唤物");
                break;
            case "tachr_436_whispr_1":
                if (options.cond) {
                    const ts = blackboard.id == "skchr_whispr_2" ? blackboard.talent_scale : 1;
                    const extra_hps = bb.atk_to_hp_recovery_ratio * finalFrame.atk * ts;
                    pool[2] += extra_hps * dur.duration * enemy.count;
                    console.write(`天赋hps: ${extra_hps.toFixed(1)}`);
                    if (isSkill) console.log("天赋可以治疗召唤物");
                }
                break;
            case "tachr_188_helage_trait":
            case "tachr_337_utage_trait":
            case "tachr_475_akafyu_trait":
                pool[2] += bb.value * dur.hitCount;
                break;
            case "tachr_485_pallas_2":
                pool[2] += bb.value * dur.hitCount;
                if ("pallas_e_t_2.value" in bb) {
                    pool[2] += bb["pallas_e_t_2.value"] * dur.hitCount;
                }
                break;
            case "tachr_421_crow_trait":
            case "tachr_4066_highmo_trait":
                pool[2] += bb.value * dur.attackCount * Math.min(ecount, 2);
                break;
            case "tachr_2013_cerber_1":
                damage = bb.atk_scale * edef * Math.max(1 - emrpct, 0.05);
                pool[1] += damage * dur.hitCount;
                break;
            case "tachr_391_rosmon_trait":
            case "tachr_1027_greyy2_trait":
                let ntimes = 1;
                if (isSkill && blackboard.id == "skchr_rosmon_2") ntimes = 3;
                const quake_atk = (finalFrame.atk / buffFrame.atk_scale) * bb["attack@append_atk_scale"];
                const quake_damage = Math.max(quake_atk - edef, quake_atk * 0.05);

                damage = quake_damage * dur.hitCount * ntimes;
                pool[0] += damage;
                break;
            // 技能
            // 伤害类
            case "skchr_ifrit_2":
                damage = basicFrame.atk * bb["burn.atk_scale"] * Math.floor(bb.duration) * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * dur.attackCount * ecount;
                break;
            case "skchr_amgoat_2":
                damage = (finalFrame.atk / 2) * (1 - enemy.res / 100) * buffFrame.damage_scale;
                pool[1] += damage * dur.attackCount * (enemy.count - 1);
                break;
            case "skchr_nightm_2":
                move = bb.duration / 4;
                console.log(`以位移${move.toFixed(1)}格计算`);
                pool[3] += bb.value * move * ecount * buffFrame.damage_scale;
                break;
            case "skchr_weedy_3":
                if (options.token) {
                    move = (bb.force * bb.force) / 3 + bb.duration / 5;
                    console.log("召唤物伤害计算无效");
                    console.log("应为本体技能伤害");
                } else move = (bb.force * bb.force) / 4 + bb.duration / 5;
                console.log(`以位移${move.toFixed(1)}格计算`);
                pool[3] += bb.value * move * ecount * buffFrame.damage_scale;
                break;
            case "skchr_huang_3":
                const finishAtk = finalFrame.atk * bb.damage_by_atk_scale;
                damage = Math.max(finishAtk - enemy.def, finishAtk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * ecount;
                break;
            case "skchr_chen_2":
                damage = finalFrame.atk * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * dur.hitCount;
                break;
            case "skchr_bibeak_1":
                if (enemy.count > 1) {
                    damage = finalFrame.atk * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage;
                }
                break;
            case "skchr_ayer_2":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * enemy.count * dur.hitCount;
                break;
            case "skcom_assist_cost[2]":
            case "skcom_assist_cost[3]":
            case "skchr_myrtle_2":
            case "skchr_elysm_2":
            case "skchr_skgoat_2":
            case "skchr_utage_1":
            case "skchr_snakek_2":
            case "skchr_blitz_1":
            case "skchr_robrta_2":
                damagePool[0] = 0;
                damagePool[1] = 0;
                break;
            case "skchr_zebra_1":
                damagePool[2] = 0;
                break;
            case "skchr_sddrag_2":
                damage = finalFrame.atk * bb["attack@skill.atk_scale"] * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * dur.hitCount;
                break;
            case "skchr_haak_2":
            case "skchr_haak_3":
                console.log(`攻击队友15次(不计入自身dps)`);
                break;
            case "skchr_podego_2":
                damage = finalFrame.atk * bb.projectile_delay_time * (1 - emrpct) * enemy.count * buffFrame.damage_scale;
                pool[1] = damage;
                damagePool[1] = 0;
                break;
            case "skchr_beewax_2":
            case "skchr_mint_2":
                if (isSkill) {
                    damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * ecount * buffFrame.damage_scale;
                    pool[1] = damage;
                }
                break;
            case "skchr_tomimi_2":
                if (isSkill && options.crit) {
                    damage = Math.max(finalFrame.atk - enemy.def, finalFrame.atk * 0.05);
                    pool[0] +=
                        damage *
                        (
                            dur as unknown as {
                                critHitCount: number;
                            }
                        ).critHitCount *
                        (enemy.count - 1);
                }
                break;
            case "skchr_archet_1":
                atk = (finalFrame.atk / bb.atk_scale) * bb.atk_scale_2;
                const hit = Math.min(enemy.count - 1, bb.show_max_target) * dur.hitCount;
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * hit;
                break;
            case "skchr_archet_2":
                const n = Math.min(4, enemy.count - 1);
                if (n > 0) {
                    const hit = ((9 - n) * n) / 2;
                    pool[0] += hitDamage * hit;
                }
                break;
            case "tachr_338_iris_trait":
            case "tachr_469_indigo_trait":
            case "tachr_4046_ebnhlz_trait":
            case "tachr_297_hamoni_trait":
                if (isSkill && ["skchr_iris_2", "skchr_ebnhlz_2"].includes(blackboard.id)) {
                } else {
                    const talent_key = charId.replace("char", "tachr") + "_1";
                    // 倍率
                    let scale = buffList[talent_key].atk_scale || 1;
                    if (isSkill && blackboard.id == "skchr_ebnhlz_3") scale *= buffList.skill.talent_scale_multiplier;
                    // 个数
                    let nBalls = bb.times;
                    if (talent_key == "tachr_4046_ebnhlz_1" && options.cond_elite) ++nBalls;
                    // 伤害
                    let extra_scale = 0;
                    if ("tachr_4046_ebnhlz_2" in buffList && enemy.count == 1) {
                        extra_scale = buffList["tachr_4046_ebnhlz_2"].atk_scale;
                    }
                    damage = hitDamage * (scale + extra_scale); // hitDamage已经包含了damage_scale和法抗
                    const md = damage * nBalls + hitDamage * (1 + extra_scale);
                    const delta = md - hitDamage * (1 + extra_scale) * (1 + nBalls);
                    console.log(`满蓄力伤害 ${md.toFixed(1)}`);
                    if (isSkill) console.log("DPS按满蓄力1次计算");
                    pool[1] += delta;
                }
                break;
            case "skchr_ash_3":
                atk = (finalFrame.atk / bb.atk_scale) * (options.cond ? bb.hitwall_scale : bb.not_hitwall_scale);
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * enemy.count;
                break;
            case "skchr_blitz_2":
                atk = finalFrame.atk * bb.atk_scale;
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * enemy.count;
                break;
            case "skchr_rfrost_2":
                atk = (finalFrame.atk / bb.atk_scale) * bb.trap_atk_scale;
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage;
                break;
            case "skchr_tachak_1":
                atk = finalFrame.atk * bb.atk_scale;
                damage = Math.max(atk * (1 - emrpct), atk * 0.05) * buffFrame.damage_scale;
                pool[1] += damage * bb.projectile_delay_time * enemy.count;
                break;
            case "skchr_pasngr_3":
                atk = finalFrame.atk * bb.atk_scale;
                damage = Math.max(atk * (1 - emrpct), atk * 0.05) * buffFrame.damage_scale;
                pool[1] += damage * ecount * 8;
                break;
            case "skchr_toddi_2":
                atk = (finalFrame.atk / bb["attack@atk_scale"]) * bb["attack@splash_atk_scale"];
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * enemy.count * dur.hitCount;
                break;
            case "skchr_indigo_2":
                if (options.cond) {
                    atk = finalFrame.atk * bb["indigo_s_2[damage].atk_scale"];
                    damage = Math.max(atk * (1 - emrpct), atk * 0.05) * buffFrame.damage_scale;
                    pool[1] += damage * enemy.count * dur.duration * 2;
                }
                break;
            case "tachr_426_billro_1":
                if (isSkill) {
                    damage = bb.heal_scale * finalFrame.maxHp;
                    if (options.charge) damage *= 2;
                    pool[2] += damage;
                }
            case "tachr_486_takila_1":
                if (!isSkill) {
                    damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                    console.log(`技能未开启时反弹法伤最高为 ${damage.toFixed(1)}`);
                }
                break;
            case "tachr_437_mizuki_1":
                let scale = bb["attack@mizuki_t_1.atk_scale"];
                if (blackboard.id == "skchr_mizuki_1" && isSkill) scale *= buffList.skill.talent_scale;
                console.write(`法伤倍率: ${scale.toFixed(2)}x`);
                damage = (finalFrame.atk / buffFrame.atk_scale) * scale * (1 - emrpct) * buffFrame.damage_scale;
                let nHit = bb["attack@max_target"];
                if (isSkill) {
                    if (blackboard.id == "skchr_mizuki_2") nHit += 1;
                    else if (blackboard.id == "skchr_mizuki_3") nHit += 2;
                }
                nHit = dur.attackCount * Math.min(ecount, nHit);
                pool[1] += damage * nHit;
                break;
            case "tachr_1014_nearl2_1":
                let _scale = bb.atk_scale;
                const _nHit = options.cond ? 2 : 1;
                damage = finalFrame.atk * _scale * buffFrame.damage_scale;
                switch (blackboard.id) {
                    case "skchr_nearl2_1":
                        if (!isSkill) console.log(`本体落地伤害 ${damage.toFixed(1)}, 不计入总伤害`);
                        break;
                    case "skchr_nearl2_2":
                        if (isSkill) {
                            pool[3] += damage * ecount * _nHit;
                        }
                        break;
                    case "skchr_nearl2_3":
                        if (!isSkill) console.log(`本体落地伤害 ${damage.toFixed(1)}, 不计入总伤害`);
                        else {
                            _scale = buffList.skill.value;
                            damage = finalFrame.atk * _scale * buffFrame.damage_scale;
                            pool[3] += damage * ecount * _nHit;
                        }
                        break;
                }
                break;
            case "skchr_lmlee_2":
                const lmlee_2_scale = bb.default_atk_scale + bb.factor_atk_scale * bb.max_stack_cnt;
                damage = finalFrame.atk * lmlee_2_scale * (1 - emrpct) * buffFrame.damage_scale;
                break;
            case "uniequip_002_rope":
            case "uniequip_002_slchan":
            case "uniequip_002_snsant":
            case "uniequip_002_glady":
                if (isSkill) {
                    const force = buffList.skill.force || buffList.skill["attack@force"];
                    move = force + 1;
                    console.log(`以位移${move}格计算`);
                    pool[1] += bb.trait.value * move * (1 - emrpct) * ecount * buffFrame.damage_scale;
                }
                break;
            // 间接治疗
            case "skchr_tiger_2":
                pool[2] += damagePool[1] * bb.heal_scale;
                break;
            case "skchr_strong_2":
                pool[2] += damagePool[0] * bb.scale;
                break;
            case "skcom_heal_self[1]":
            case "skcom_heal_self[2]":
                damagePool[2] = 0;
                // console.log(finalFrame);
                pool[2] += bb.heal_scale * finalFrame.maxHp;
                break;
            case "skchr_nightm_1":
                pool[2] += damagePool[1] * bb["attack@heal_scale"] * Math.min(enemy.count, bb["attack@max_target"]);
                break;
            case "tachr_1024_hbisc2_trait":
            case "tachr_1020_reed2_trait":
                pool[2] += damagePool[1] * bb.scale;
                break;
            case "skchr_folnic_2":
                pool[2] += ((bb["attack@heal_scale"] * finalFrame.atk) / buffFrame.atk_scale) * dur.hitCount;
                break;
            case "skchr_breeze_2":
                damage = finalFrame.atk / 2;
                pool[2] += damage * dur.attackCount * (enemy.count - 1);
                break;
            case "skchr_ccheal_1":
                heal = (finalFrame.atk * bb.heal_scale * bb.duration * dur.duration) / attackTime; // 乘以技能次数
                pool[2] += heal;
                break;
            case "skchr_ccheal_2":
                heal = finalFrame.atk * bb.heal_scale * bb.duration;
                pool[2] += heal * enemy.count;
                break;
            case "skchr_shining_2":
            case "skchr_tuye_1":
                heal = finalFrame.atk * bb.atk_scale;
                pool[4] += heal;
                break;
            case "skchr_cgbird_2":
                heal = finalFrame.atk * bb.atk_scale;
                pool[4] += heal * ecount;
                break;
            case "skchr_tknogi_2":
            case "skchr_lisa_3":
                heal = finalFrame.atk * bb["attack@atk_to_hp_recovery_ratio"] * enemy.count * (dur.duration - 1);
                pool[2] += heal;
                damagePool[2] = 0;
                break;
            case "skchr_blemsh_1":
                heal = (finalFrame.atk * bb.heal_scale) / buffFrame.atk_scale;
                pool[2] += heal;
                break;
            case "skchr_blemsh_2":
                heal = (finalFrame.atk * bb["attack@atk_to_hp_recovery_ratio"]) / buffFrame.atk_scale;
                console.write(`每秒单体治疗: ${heal.toFixed(1)}`);
                console.log("可以治疗召唤物");
                pool[2] += heal * dur.duration * enemy.count;
                break;
            case "skchr_blemsh_3":
                damage = finalFrame.atk * bb["attack@blemsh_s_3_extra_dmg[magic].atk_scale"];
                damage = Math.max(damage * (1 - emrpct), damage * 0.05);
                heal = (finalFrame.atk / buffFrame.atk_scale) * bb.heal_scale;
                console.write(`每次攻击额外法伤：${damage.toFixed(1)} （计算天赋加成），额外治疗: ${heal.toFixed(1)}`);
                pool[1] += damage * dur.attackCount;
                pool[2] += heal * dur.attackCount;
                break;
            case "skchr_rosmon_1":
                damage = finalFrame.atk * bb.extra_atk_scale;
                damage = Math.max(damage * (1 - emrpct), damage * 0.05) * dur.hitCount;
                pool[1] += damage;
                break;
            case "skchr_kirara_1":
                damage = finalFrame.atk * bb["kirara_s_1.atk_scale"];
                damage = Math.max(damage * (1 - emrpct), damage * 0.05) * dur.hitCount;
                pool[1] += damage;
                break;
            case "skchr_amiya2_2":
                const arts_atk = finalFrame.atk * bb.atk_scale;
                const real_atk = finalFrame.atk * bb.atk_scale_2;
                const arts_dmg = Math.max(arts_atk * (1 - emrpct), arts_atk * 0.05);
                console.write(`[斩击] 法术伤害 ${arts_dmg.toFixed(1)}, 命中 9, 真实伤害 ${real_atk.toFixed(1)}, 命中 1`);
                pool[1] += arts_dmg * 9;
                pool[3] += real_atk;
                break;
            case "skchr_kafka_1":
                damage = finalFrame.atk * (1 - emrpct) * enemy.count;
                pool[1] = damage;
                damagePool[1] = 0;
                break;
            case "skchr_kafka_2":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * enemy.count;
                pool[1] = damage;
                break;
            case "skchr_tuye_2":
                pool[2] = finalFrame.atk * bb.heal_scale;
                console.log(`瞬间治疗量 ${pool[2].toFixed(1)}`);
                pool[2] *= 3;
                break;
            case "skchr_nothin_1":
            case "skchr_nothin_2":
                const a = finalFrame.atk * buffList["tachr_455_nothin_1"].atk_scale;
                damage = Math.max(a - edef, a * 0.05);
                console.log(`首次攻击伤害 ${damage.toFixed(1)}`);
                break;
            case "skchr_heidi_1":
            case "skchr_heidi_2":
            case "skchr_skadi2_2":
            case "skchr_sora_2":
                if (bb.max_hp) {
                    const buff_hp = finalFrame.maxHp * bb.max_hp;
                    console.log(`队友HP增加 ${buff_hp.toFixed(1)}`);
                }
                if (bb.def) {
                    const buff_def = finalFrame.def * bb.def;
                    console.log(`队友防御力增加 ${buff_def.toFixed(1)}`);
                }
                if (bb.atk) {
                    const buff_atk = finalFrame.atk * bb.atk;
                    console.log(`队友攻击力增加 ${buff_atk.toFixed(1)}`);
                }
                break;
            case "skchr_skadi2_3":
                const buff_atk = finalFrame.atk * bb.atk;
                damage = finalFrame.atk * bb.atk_scale * buffFrame.damage_scale;
                pool[3] += damage * enemy.count * dur.duration;
                console.log(`队友攻击力增加 ${buff_atk.toFixed(1)}`);
                console.log(`每秒真实伤害 ${damage.toFixed(1)}, 总伤害 ${pool[3]}`);
                console.log(`叠加海嗣时真伤x2，不另行计算`);
                break;
            case "skchr_mizuki_3":
                if (ecount < 3) {
                    damage = bb["attack@hp_ratio"] * finalFrame.maxHp;
                    console.log(`目标数<3，自身伤害 ${damage.toFixed(1)}`);
                    pool[2] -= damage * dur.attackCount;
                }
                break;
            case "tachr_473_mberry_trait":
            case "tachr_449_glider_trait":
            case "tachr_4041_chnut_trait":
                let ep_ratio = bb.ep_heal_ratio;
                let ep_scale = 1;
                if (isSkill) {
                    switch (blackboard.id) {
                        case "skchr_mberry_1":
                            ep_ratio = buffList.skill.ep_heal_ratio;
                            break;
                        case "skchr_glider_1":
                            ep_ratio = buffList.skill["glider_s_1.ep_heal_ratio"];
                            ep_scale = 3;
                            console.log("计算3秒内总元素回复量");
                            break;
                        case "skchr_chnut_1":
                            ep_scale = buffList.skill.trait_scale;
                            break;
                        case "skchr_chnut_2":
                            ep_scale = buffList.skill["attack@heal_continuously_scale"];
                            break;
                    }
                }
                if (buffList["tachr_4041_chnut_1"] && options.cond) {
                    ep_scale *= buffList["tachr_4041_chnut_1"].ep_heal_scale;
                }
                console.write(`元素治疗系数: ${ep_ratio.toFixed(2)}x`);
                if (ep_scale != 1) console.write(`元素治疗倍率: ${ep_scale.toFixed(2)}x`);

                damage = (finalFrame.atk / buffFrame.heal_scale) * ep_ratio * ep_scale;
                const ep_total = damage * dur.hitCount;
                console.log(`元素治疗 ${damage.toFixed(1)} (${(ep_ratio * ep_scale).toFixed(2)} x)`);
                console.log(`技能元素HPS ${(ep_total / dur.duration).toFixed(1)}`);
                break;
            case "skchr_sleach_2":
                damagePool[0] = 0;
                damagePool[1] = 0;
                damagePool[2] = 0;
                console.write("伤害为0（以上计算无效）");
                pool[2] += finalFrame.atk * bb.atk_to_hp_recovery_ratio * dur.duration;
                break;
            case "skchr_sleach_3":
                damagePool[0] = 0;
                damagePool[1] = 0;
                damagePool[2] = 0;
                console.write("伤害为0（以上计算无效）");
                damage = Math.max(finalFrame.atk - edef, finalFrame.atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * ecount;
                console.write(`摔炮伤害 ${damage.toFixed(1)} (damage_scale=${buffFrame.damage_scale.toFixed(3)}), 命中 ${ecount}`);
                break;
            case "skchr_gnosis_1":
                const scale_mul_g1 = options.freeze ? 1 : buffList["tachr_206_gnosis_1"].damage_scale_freeze / buffList["tachr_206_gnosis_1"].damage_scale_cold;
                damage = finalFrame.atk * (1 - emrpct) * buffFrame.damage_scale * scale_mul_g1;
                pool[1] += damage * dur.hitCount;
                console.write(`冻结伤害 ${damage.toFixed(1)} (damage_scale=${(buffFrame.damage_scale * scale_mul_g1).toFixed(2)}), 命中 ${dur.hitCount}`);
                break;
            case "skchr_gnosis_3":
                const scale_mul_g3 = options.freeze ? 1 : buffList["tachr_206_gnosis_1"].damage_scale_freeze / buffList["tachr_206_gnosis_1"].damage_scale_cold;
                damage = finalFrame.atk * (1 - emrpct) * bb.atk_scale * buffFrame.damage_scale * scale_mul_g3;
                pool[1] += damage * ecount;
                console.write(`终结伤害 ${damage.toFixed(1)} (damage_scale=${(buffFrame.damage_scale * scale_mul_g3).toFixed(2)}), 命中 ${ecount}, 按冻结计算`);
                break;
            case "skchr_ling_3":
                if (options.token) {
                    console.log("不计算范围法伤");
                    console.log("(去掉“计算召唤物数据”才能计算范围伤害)");
                } else {
                    damage = finalFrame.atk * (1 - emrpct) * bb.atk_scale * buffFrame.damage_scale;
                    pool[1] += damage * ecount * dur.duration * 2;
                    console.log(`召唤物范围法术伤害 ${Number(damage.toFixed(1)) * 2}/s`);
                }
                break;
            case "tachr_377_gdglow_1":
                if (
                    (
                        dur as unknown as {
                            critHitCount: number;
                        }
                    ).critHitCount > 0 &&
                    isSkill
                ) {
                    damage = finalFrame.atk * (1 - emrpct) * bb["attack@atk_scale_2"] * buffFrame.damage_scale;
                    const funnel = Number(checkSpecs(blackboard.id, "funnel")) || 1;
                    pool[1] +=
                        damage *
                        enemy.count *
                        funnel *
                        (
                            dur as unknown as {
                                critHitCount: number;
                            }
                        ).critHitCount;
                    console.log(
                        `爆炸 ${
                            (
                                dur as unknown as {
                                    critHitCount: number;
                                }
                            ).critHitCount * funnel
                        } 次, 爆炸伤害 ${damage.toFixed(1)}`,
                    );
                }
                break;
            case "skchr_bena_1":
            case "skchr_bena_2":
                if (options.annie && isSkill) {
                    damagePool[0] = 0;
                    damagePool[1] = 0;
                }
                break;
            case "skchr_kazema_1":
                if (options.annie) {
                    let kazema_scale = buffList["tachr_4016_kazema_1"].damage_scale;
                    if ("uniequip_002_kazema" in buffList && buffList["uniequip_002_kazema"].talent && !options.token) kazema_scale = buffList["uniequip_002_kazema"].talent.damage_scale;
                    damage = (finalFrame.atk / buffFrame.atk_scale) * kazema_scale * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage * ecount;
                    console.log(`替身落地法伤 ${damage.toFixed(1)} (${kazema_scale.toFixed(2)}x)，命中 ${ecount}`);
                    if (isSkill) {
                        damagePool[0] = 0;
                        damagePool[1] = 0;
                    }
                }
                break;
            case "skchr_kazema_2":
                let kazema_scale = buffList["tachr_4016_kazema_1"].damage_scale;
                let kz_name = "[纸偶]";
                let kz_invalid = false;
                if (options.annie) {
                    kz_name = "[替身]";
                    if ("uniequip_002_kazema" in buffList && buffList["uniequip_002_kazema"].talent && !options.token) kazema_scale = buffList["uniequip_002_kazema"].talent.damage_scale;
                } else if (!options.token) {
                    console.log("落地伤害需要勾选\n[替身]或[召唤物]进行计算");
                    kz_invalid = true;
                }
                if (!kz_invalid) {
                    damage = finalFrame.atk * kazema_scale * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage * ecount;

                    console.log(`${kz_name}落地法伤 ${damage.toFixed(1)} (${kazema_scale.toFixed(2)}x)，命中 ${ecount}`);
                }
                if (options.annie && isSkill) {
                    damagePool[0] = 0;
                    damagePool[1] = 0;
                }
                break;
            case "skchr_phenxi_2":
                const ph_2_atk = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale_2;
                damage = Math.max(ph_2_atk - edef, ph_2_atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * 2 * dur.hitCount;
                console.log(`子爆炸伤害 ${damage.toFixed(1)}\n以2段子爆炸计算`);
                break;
            case "skchr_horn_2":
                if (options.overdrive_mode) {
                    damage = (finalFrame.atk / bb["attack@s2.atk_scale"]) * bb["attack@s2.magic_atk_scale"] * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage * dur.hitCount;
                    console.write(`法术伤害 ${damage.toFixed(1)}, 命中 ${dur.hitCount}`);
                }
                break;
            case "skchr_horn_3":
                if (options.overdrive_mode && !options.od_trigger) {
                    const horn_3_pct = (dur.duration * (dur.duration - 0.2)) / 2; // 0.4, 1.4,...,11.4
                    damage = (finalFrame.maxHp * horn_3_pct) / 100;
                    pool[2] -= damage;
                    console.log(`生命流失 ${damage.toFixed(1)}`);
                }
                break;
            case "skcom_heal_up[3]":
                if (options.token) {
                    damagePool[0] = damagePool[2] = 0;
                }
                break;
            case "skchr_irene_3":
                const irene_3_edef = Math.max(0, (enemy.def - enemyBuffFrame.edef_pene) * (1 - buffList["tachr_4009_irene_1"].def_penetrate));
                const irene_3_atk = (finalFrame.atk / buffFrame.atk_scale) * bb.multi_atk_scale;
                damage = Math.max(irene_3_atk - irene_3_edef, irene_3_atk * 0.05) * buffFrame.damage_scale;
                pool[0] += damage * bb.multi_times * ecount;
                break;
            case "skchr_lumen_1":
                heal = finalFrame.atk * bb["aura.heal_scale"];
                const lumen_1_hitcount = bb["aura.projectile_life_time"] * dur.attackCount * enemy.count;
                pool[2] += heal * lumen_1_hitcount;
                break;
            case "skchr_ghost2_3":
                if (isSkill && !options.annie) {
                    if (options.cond) {
                        const ghost2_3_atk = finalFrame.atk * bb["attack@atk_scale_ex"];
                        damage = Math.max(ghost2_3_atk - edef, ghost2_3_atk * 0.05) * buffFrame.damage_scale;
                        pool[0] += damage * dur.hitCount;
                    } else {
                        damage = finalFrame.maxHp * bb["attack@hp_ratio"];
                        pool[2] -= damage * dur.hitCount;
                    }
                }
                break;
            case "skchr_pianst_2":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * enemy.count;
                break;
            case "tachr_4047_pianst_1":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                console.log(`反弹伤害 ${damage.toFixed(1)}, 不计入DPS`);
                break;
            case "tachr_4046_ebnhlz_2":
                if (enemy.count == 1) {
                    damage = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage * dur.hitCount;
                } else if (enemy.count > 1 && "atk_scale_2" in bb) {
                    damage = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale_2 * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage * dur.attackCount * (enemy.count - 1);
                }
                break;
            case "skchr_greyy2_2":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                const greyy2_2_count = bb.projectile_delay_time / bb.interval;
                pool[1] += damage * greyy2_2_count * enemy.count;
                damagePool[1] = 0;
                extraDamagePool[0] = 0;
                break;
            case "skchr_gvial2_1":
                let gvial2_scale = 1;
                if ("tachr_1026_gvial2_2" in buffList) gvial2_scale = options.cond ? buffList["tachr_1026_gvial2_2"].heal_scale_2 : buffList["tachr_1026_gvial2_2"].heal_scale_1;
                pool[2] = damagePool[0] * bb.heal_scale * gvial2_scale;
                console.write(`治疗倍率: ${bb.heal_scale} * ${gvial2_scale.toFixed(2)}`);
                break;
            case "skchr_provs_2":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * enemy.count;
                break;
            case "tachr_4064_mlynar_2":
                let mlynar_t2_scale = bb.atk_scale;
                if (isSkill && blackboard.id == "skchr_mlynar_3") {
                    mlynar_t2_scale += buffList.skill.atk_scale;
                    console.log("额外真伤对反弹也生效");
                }
                damage = (finalFrame.atk / buffFrame.atk_scale) * mlynar_t2_scale * buffFrame.damage_scale;
                console.write(`反弹伤害 ${damage.toFixed(1)}`);
                if (isSkill) console.log(`反弹伤害 ${damage.toFixed(1)}`);
                break;
            case "skchr_mlynar_3":
                if (isSkill) {
                    damage = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale * buffFrame.damage_scale;
                    pool[3] += damage * dur.hitCount;
                }
                break;
            case "skchr_lolxh_2":
                if (isSkill && options.cond) {
                    const lolxh_2_edef = Math.max(0, edef - bb["attack@def_penetrate_fixed"]);
                    damage = Math.max(finalFrame.atk - lolxh_2_edef, finalFrame.atk * 0.05) * buffFrame.damage_scale;
                    pool[0] += damage * dur.hitCount;
                }
                break;
            case "tachr_117_myrrh_1":
                if (!isSkill) {
                    heal = finalFrame.atk * bb.heal_scale;
                    pool[2] += heal * enemy.count;
                }
            case "skchr_qanik_2":
                const qanik_2_damage_scale = options.cond ? buffFrame.damage_scale / buffList["tachr_466_qanik_1"].damage_scale : buffFrame.damage_scale;
                damage = (finalFrame.atk / buffFrame.atk_scale) * bb.critical_damage_scale * (1 - emrpct) * qanik_2_damage_scale;
                pool[1] += damage * ecount * enemy.count;
                console.write(`落地伤害倍率 ${qanik_2_damage_scale.toFixed(2)}x，命中 ${ecount * enemy.count}`);
                break;
            case "tachr_157_dagda_2":
                if (options.cond) {
                    pool[2] += damagePool[0] * bb.heal_scale;
                }
                break;
            case "skchr_dagda_1":
                const dagda_1_atk = finalFrame.atk * bb["attack@defensive_atk_scale"];
                damage = Math.max(dagda_1_atk - enemy.def, dagda_1_atk * 0.05) * buffFrame.damage_scale;
                console.log(`反击伤害 ${damage.toFixed(1)}/不计入dps`);
                break;
            case "skchr_judge_1":
                damage = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale_2 * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * dur.hitCount;
                console.write(`法术伤害 ${damage.toFixed(1)}, 命中 ${dur.hitCount}`);
                break;
            case "tachr_4065_judge_1":
                const judge_shield_1 = finalFrame.maxHp * bb.born_hp_ratio;
                let judge_shield_2 = finalFrame.maxHp * bb.kill_hp_ratio;
                if (isSkill) {
                    if (blackboard.id == "skchr_judge_2") judge_shield_2 *= 1 + buffList.skill.shield_scale;
                    console.log(`初始护盾 ${judge_shield_1.toFixed(1)}`);
                    console.log(`技能击杀护盾 ${judge_shield_2.toFixed(1)}`);
                }
                break;
            case "tachr_4065_judge_2":
                damage = (finalFrame.atk / buffFrame.atk_scale) * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                if (isSkill) console.log(`反弹伤害 ${damage.toFixed(1)}`);
                break;
            case "skchr_texas2_1":
                const texas2_s1_dur = dur.duration + bb["attack@texas2_s_1[dot].duration"] - 1;
                damage = bb["attack@texas2_s_1[dot].dot_damage"] * (1 - emrpct) * buffFrame.damage_scale;
                console.write(`持续法伤 ${damage.toFixed(1)}, 按持续 ${texas2_s1_dur.toFixed(1)}s计算`);
                pool[1] += damage * texas2_s1_dur;
                break;
            case "skchr_texas2_2":
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * enemy.count;
                console.write(`落地法伤 ${damage.toFixed(1)}, 命中 ${enemy.count}`);
                break;
            case "skchr_texas2_3":
                const texas2_s3_aoe = finalFrame.atk * bb["appear.atk_scale"] * (1 - emrpct) * buffFrame.damage_scale;
                const texas2_s3_target = Math.min(enemy.count, bb.max_target);
                damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += texas2_s3_aoe * enemy.count * 2 + damage * texas2_s3_target * dur.duration;
                console.write(`落地法伤 ${texas2_s3_aoe.toFixed(1)}, 命中 ${enemy.count * 2}`);
                console.write(`剑雨法伤 ${damage.toFixed(1)}, 命中 ${texas2_s3_target * dur.duration}`);
                break;
            case "skchr_vigil_2":
                if (options.token) {
                    pool[2] += finalFrame.maxHp * bb["vigil_wolf_s_2.hp_ratio"];
                }
                break;
            case "skchr_vigil_3":
                if (options.cond || options.token) {
                    // 计算本体属性。狼的法伤不享受特性加成
                    let vigil_final_atk = finalFrame.atk;
                    if (options.token) {
                        const token_id = charAttr.char.id;
                        charAttr.char.id = "char_427_vigil";
                        const vigil = getAttributes(
                            charAttr.char,
                            {
                                equipId: operatorData.equipId,
                                equipLevel: operatorData.equipLevel,
                                favor: operatorData.favor,
                                level: operatorData.level,
                                potentialRank: operatorData.potentialRank,
                                skillId: operatorData.skillId,
                            },
                            operatorData.phaseIndex,
                        );
                        const vigil_final = getBuffedAttributes(vigil.basic, buffFrame);
                        charAttr.char.id = token_id;
                        // console.log(vigil_final);
                        vigil_final_atk = vigil_final.atk;
                        if (!options.cond) {
                            console.log("必定满足阻挡条件");
                        }
                    }
                    damage = vigil_final_atk * bb["attack@vigil_s_3.atk_scale"] * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1] += damage * dur.hitCount;
                    console.write(`额外法伤 ${damage.toFixed(1)}, 命中 ${dur.hitCount}`);
                }
                break;
            case "skchr_ironmn_1":
                if (!options.token) {
                    const ironmn_s1_atk = 12 * bb.fake_scale;
                    console.log(`常态加攻+12%, 技能加攻+${ironmn_s1_atk}%`);
                } else {
                    damagePool[0] = 0;
                    console.log("召唤物结果无效");
                }
                break;
            case "skchr_ironmn_2":
                if (!options.token) {
                    const ironmn_s2_skill_hp = 24; // 30s * 0.8%/s
                    const ironmn_s2_normal_time = (100 - ironmn_s2_skill_hp * 2) / 0.4;
                    const ironmn_s2_skill_sp = Math.floor(ironmn_s2_skill_hp / bb.fake_interval / 0.8);
                    const ironmn_s2_normal_sp = Math.floor(ironmn_s2_normal_time / 3.5 / 0.8);
                    console.log(`以开2次技能计算`);
                    console.log(`技能恢复SP: ${ironmn_s2_skill_sp} / 30s`);
                    console.log(`常态恢复SP: ${ironmn_s2_normal_sp} / ${ironmn_s2_normal_time}s`);
                } else {
                    damagePool[0] = 0;
                    console.log("召唤物结果无效");
                }
                break;
            case "skchr_ironmn_3":
                if (!isSkill && options.token) {
                    damagePool[0] = 0;
                    console.log("不普攻");
                }
                break;
            case "sktok_ironmn_pile3":
                if (isSkill) {
                    const pile3_atk = finalFrame.atk / 2;
                    damage = Math.max(pile3_atk - edef, pile3_atk * 0.05) * buffFrame.damage_scale;
                    console.write(`范围伤害 ${damage.toFixed(1)}, 命中 ${enemy.count * dur.hitCount}`);
                    pool[0] += damage * enemy.count * dur.hitCount;
                }
                break;
            case "skchr_reed2_2":
                if (isSkill) {
                    damage = finalFrame.atk * bb.atk_scale * (1 - emrpct) * buffFrame.damage_scale;
                    heal = damage * buffList["tachr_1020_reed2_trait"].scale;
                    const reed2_interval = options.reed2_fast ? 1.567 : 0.8;
                    let reed2_hitCount = Math.ceil((dur.duration - 0.167) / reed2_interval); // 减去抬手时间

                    if (options.reed2_fast) {
                        console.log("理想情况, 火球立即引爆");
                        console.log(`每${reed2_interval}s命中三个火球`);
                        reed2_hitCount = Math.ceil((dur.duration - 0.167) / reed2_interval) * 3;
                    } else {
                        console.log(`每${reed2_interval}s命中一个火球`);
                    }
                    if (options.rosmon_double) {
                        console.log("计算两组火球伤害");
                        reed2_hitCount *= 2;
                    }
                    console.write(`火球伤害 ${damage.toFixed(1)}, 治疗 ${heal.toFixed(1)}, 命中 ${reed2_hitCount}`);
                    pool[1] += damage * reed2_hitCount;
                    pool[2] += heal * reed2_hitCount;
                }
                break;
            case "skchr_reed2_3":
                damage = finalFrame.atk * bb["talent@s3_atk_scale"] * (1 - emrpct) * buffFrame.damage_scale;
                const reed2_boom_damage = finalFrame.atk * bb["talent@aoe_scale"] * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * dur.duration * ecount;
                console.write(`灼痕伤害 ${damage.toFixed(1)}, 命中 ${dur.duration * ecount}`);
                console.log(`爆炸伤害 ${reed2_boom_damage.toFixed(1)}, 半径1.7`);
                break;
            case "skchr_puzzle_2":
                damage = finalFrame.atk * bb["attack@atk_scale_2"] * (1 - emrpct) * buffFrame.damage_scale;
                const puzzle_hitCount = 15 * 10 + 6 * (dur.attackCount - 10);
                const puzzle_hitCount_skill = 55;
                pool[1] += damage * puzzle_hitCount_skill;
                console.log("法伤按8s/60跳估算");
                console.log(`总法伤 ${(damage * puzzle_hitCount).toFixed(1)}/${puzzle_hitCount}跳估算`);
                break;
            case "skchr_hamoni_2":
                damage = bb.damage_value * (1 - emrpct) * buffFrame.damage_scale;
                pool[1] += damage * dur.duration * enemy.count;
                console.write(`范围伤害 ${damage.toFixed(1)}, 命中 ${dur.duration * enemy.count}`);
                break;
            case "tachr_197_poca_1":
                if (options.cond && "extra_atk_scale" in bb) {
                    const poca_t1_atk = finalFrame.atk * bb.extra_atk_scale;
                    damage = Math.max(poca_t1_atk - edef, poca_t1_atk * 0.05) * buffFrame.damage_scale;
                    console.write(`额外伤害 ${damage.toFixed(1)}, 命中 ${dur.hitCount}`);
                    pool[0] += damage * dur.hitCount;
                }
                break;
        } // extraDamage switch ends here

        // 百分比/固定回血
        let hpratiosec = bb["hp_recovery_per_sec_by_max_hp_ratio"];
        const hpsec = bb["hp_recovery_per_sec"];
        if (hpratiosec) {
            if (buffName == "tachr_478_kirara_1") {
                if (options.cond) hpratiosec = bb["kirara_t_2.hp_recovery_per_sec_by_max_hp_ratio"];
                if (isSkill && blackboard.id == "skchr_kirara_2") {
                    hpratiosec *= buffList["skill"].talent_scale;
                }
                console.write(`天赋回血比例: ${(hpratiosec * 100).toFixed(1)}%/s`);
            }

            if (buffName == "tachr_344_beewax_1" && isSkill) {
            } else if (buffName == "tachr_362_saga_2") {
            } else if (buffName == "tachr_293_thorns_2") {
                if (blackboard.id == "skchr_thorns_2" && isSkill) {
                    pool[2] += hpratiosec * finalFrame.maxHp * (dur.duration + dur.stunDuration - 2);
                    console.log("治疗从2秒后开始计算");
                } else {
                }
            } else if (buffName == "tachr_422_aurora_1") {
                if (!isSkill) {
                    const aurora_hp_time = levelData.spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * (1 + buffFrame.spRecoverRatio)) / 2 + dur.stunDuration;
                    const aurora_hps = hpratiosec * finalFrame.maxHp;
                    pool[2] += aurora_hps * aurora_hp_time;
                    console.write(`HP恢复时间: ${aurora_hp_time.toFixed(3)}s, HPS ${aurora_hps.toFixed(1)}`);
                }
            } else if (buffName == "skchr_blkngt_1") {
                if (isSkill && options.token) {
                    const blkngt_hps = hpratiosec * finalFrame.maxHp;
                    console.log(`HPS: ${blkngt_hps.toFixed(1)}`);
                } // else {}
            } else {
                pool[2] += hpratiosec * finalFrame.maxHp * (dur.duration + dur.stunDuration);
            }
        }
        if (hpsec) {
            if ((buffName == "tachr_291_aglina_2" && isSkill) || (buffName == "tachr_188_helage_2" && !options.noblock)) {
                /* skip */
            } else {
                pool[2] += hpsec * (dur.duration + dur.stunDuration);
            }
        }
        // 自身血量百分比相关的治疗/伤害
        if (bb["hp_ratio"]) {
            switch (buffName) {
                case "skchr_huang_3": // 自爆
                case "skchr_utage_2":
                case "skchr_akafyu_2":
                case "skchr_kazema_2":
                    if (!options.annie && !options.token) {
                        damage = bb.hp_ratio * finalFrame.maxHp;
                        pool[2] -= damage;
                        console.log(`对自身伤害 ${damage.toFixed(1)}`);
                    }
                    break;
                case "skchr_ifrit_3": // 自己掉血
                case "skchr_skadi2_3":
                case "skchr_aprot2_2":
                    pool[2] -= bb.hp_ratio * finalFrame.maxHp * dur.duration;
                    break;
                case "skchr_bldsk_2":
                    pool[2] -= bb.hp_ratio * finalFrame.maxHp * bb.duration * 2;
                    break;
                case "tachr_225_haak_trait": // 阿-特性
                    pool[2] -= bb.hp_ratio * finalFrame.maxHp * dur.duration;
                    break;
                case "tachr_225_haak_1":
                    if (options.crit) {
                        heal = bb.hp_ratio * finalFrame.maxHp * buffFrame.heal_scale;
                        pool[2] +=
                            heal *
                            (
                                dur as unknown as {
                                    critHitCount: number;
                                }
                            ).critHitCount;
                    }
                    break;
                case "skchr_kazema_1":
                    if (!isSkill) break;
                case "skchr_bena_2":
                    if (!options.annie) {
                        pool[2] -= bb.hp_ratio * dur.attackCount * finalFrame.maxHp;
                        console.log(`每次技能攻击HP-${(bb.hp_ratio * finalFrame.maxHp).toFixed(1)}`);
                    }
                    break;
                case "sktok_ironmn_pile3":
                    if (options.token && isSkill) {
                        damage = bb.hp_ratio * finalFrame.maxHp;
                        console.write(`每次攻击HP-${damage.toFixed(1)}`);
                        pool[2] -= damage * dur.hitCount;
                    }
                    break;
                case "tachr_017_huang_1":
                case "skchr_ccheal_1":
                case "skchr_ccheal_2":
                case "tachr_174_slbell_1":
                case "tachr_254_vodfox_1":
                case "tachr_343_tknogi_1":
                case "tachr_405_absin_1":
                case "tachr_416_zumama_1":
                case "tachr_362_saga_2":
                case "skchr_dusk_2":
                case "tachr_472_pasngr_1":
                case "skchr_crow_2":
                case "tachr_4019_ncdeer_1":
                case "tachr_492_quercu_1":
                case "skchr_ling_2":
                case "tachr_4039_horn_1":
                case "tachr_4042_lumen_1":
                case "tachr_1026_gvial2_2":
                case "tachr_003_kalts_2":
                case "tachr_1028_texas2_1":
                case "tachr_4017_puzzle_1":
                    break;
                case "skchr_gravel_2":
                    pool[4] += bb.hp_ratio * finalFrame.maxHp * 0.9;
                    break;
                case "skchr_phatom_1":
                    pool[4] += bb.hp_ratio * finalFrame.maxHp;
                    break;
                case "skchr_surtr_3":
                    pool[4] -= finalFrame.maxHp + 5000;
                    const surtr_maxHps = finalFrame.maxHp * bb.hp_ratio;
                    console.log(`最大生命流失量 ${surtr_maxHps.toFixed(1)}/s`);
                    break;
                case "tachr_311_mudrok_1":
                    pool[2] += ((bb.hp_ratio * finalFrame.maxHp) / bb.interval) * (dur.duration + dur.prepDuration);
                    break;
                case "uniequip_002_skadi":
                case "uniequip_002_flameb":
                case "uniequip_002_gyuki":
                    if (options.equip) {
                        console.log(`HP上限减少至 ${(finalFrame.maxHp * bb.max_hp).toFixed(1)}`);
                        finalFrame.maxHp = finalFrame.maxHp * bb.trait.max_hp;
                    }
                    break;
                case "tachr_300_phenxi_1":
                    heal = Math.ceil(bb.hp_ratio * finalFrame.maxHp) * 10;
                    console.log(`最大生命流失率 ${heal.toFixed(1)}/s`);
                    break;
                case "skchr_horn_2":
                    if (options.od_trigger && options.overdrive_mode) {
                        pool[2] -= finalFrame.maxHp * bb.hp_ratio;
                        console.log(`自爆伤害 ${pool[2].toFixed(1)}`);
                    }
                    break;
                case "skchr_highmo_2":
                    heal = bb.hp_ratio * finalFrame.maxHp;
                    console.log(`击杀恢复HP ${heal.toFixed(1)}`);
                    break;
                case "tachr_4071_peper_1":
                    if (options.cond) {
                        pool[2] += bb.value * dur.hitCount;
                    }
                    break;
                case "skchr_judge_3":
                    pool[4] += bb.hp_ratio * finalFrame.maxHp;
                    break;
                case "tachr_437_mizuki_2":
                    if (bb.hp_ratio < 0.5) {
                        heal = bb.hp_ratio * finalFrame.maxHp;
                        console.log(`击杀恢复HP ${heal.toFixed(1)}`);
                    }
                    break;
                default:
                    pool[2] += bb.hp_ratio * finalFrame.maxHp * dur.attackCount;
            }
        }

        for (let i = 0; i < 5; ++i) extraDamagePool[i] += pool[i];
    }

    // 整理返回
    const totalDamage = [0, 1, 3].reduce((x, y) => x + damagePool[y] + extraDamagePool[y], 0);
    const totalHeal = [2, 4].reduce((x, y) => x + damagePool[y] + extraDamagePool[y], 0);
    const extraDamage = [0, 1, 3].reduce((x, y) => x + extraDamagePool[y], 0);
    const extraHeal = [2, 4].reduce((x, y) => x + extraDamagePool[y], 0);

    console.write(`总伤害: ${totalDamage.toFixed(2)}`);
    if (totalHeal != 0) console.write(`总治疗: ${totalHeal.toFixed(2)}`);

    let dps = totalDamage / (dur.duration + dur.stunDuration + dur.prepDuration);
    let hps = totalHeal / (dur.duration + dur.stunDuration + dur.prepDuration);
    // 均匀化重置普攻时的普攻dps
    if (!isSkill && checkResetAttack(blackboard.id, blackboard, options)) {
        const d = dur.attackCount * attackTime;
        console.write(`以 ${d.toFixed(3)}s (${dur.attackCount} 个攻击间隔) 计算普攻dps`);
        dps = totalDamage / d;
        hps = totalHeal / d;
    }
    console.write(`DPS: ${dps.toFixed(1)}, HPS: ${hps.toFixed(1)}`);
    console.write("----");

    return {
        atk: finalFrame.atk,
        dps,
        hps,
        dur,
        damageType,
        hitDamage,
        critDamage,
        extraDamage,
        extraHeal,
        totalDamage,
        totalHeal,
        maxTarget: ecount,
        damagePool,
        extraDamagePool,
        attackTime,
        frame,
        attackCount: dur.attackCount,
        spType: levelData.spData.spType,
    };
}
