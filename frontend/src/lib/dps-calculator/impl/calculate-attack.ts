import type { Operator } from "~/types/impl/api/static/operator";
import type { CalculateDPSParams, CharAttr, Enemy } from "~/types/impl/frontend/impl/dps-calculator";
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
import type { Skill } from "~/types/impl/api/static/skills";

export function calculateAttack(
    charAttr: CharAttr,
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
    enemy: Enemy,
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
    levelData: Skill["levels"][0],
) {
    const charId = charAttr.char.id ?? "";
    const buffList = charAttr.buffList;
    const blackboard = buffList.skill;
    const basicFrame = charAttr.basic;

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
        spRecoverIntervals: [] as unknown[],
        applied: {},
    };
    for (const b in buffList) {
        const buffName = b === "skill" ? buffList[b].id : b;
        if (!checkSpecs(buffName, "crit"))
            buffFrame = applyBuff(
                charAttr,
                options,
                operatorData,
                buffFrame,
                b,
                buffList[b] as {
                    key: string;
                    value: number;
                    valueStr: string | null;
                }[],
                isSkill,
                false,
                enemy,
            );
    }

    if (options.buff)
        buffFrame = applyBuff(
            charAttr,
            options,
            operatorData,
            buffFrame,
            "raidBuff",
            raidBlackboard as unknown as {
                key: string;
                value: number;
                valueStr: string | null;
            }[],
            isSkill,
            false,
            enemy,
        );

    let damageType = extractDamageType(charData, charAttr.char, isSkill, levelData.description, blackboard, options);
    if (damageType == 2) buffFrame.atk_scale *= buffFrame.heal_scale;

    if (buffList.tachr_367_swllow_1) {
        buffFrame.attackSpeed += (
            buffList.tachr_367_swllow_1 as {
                attack_speed: number;
            }
        ).attack_speed;
    }

    if (isSkill && blackboard.id == "skchr_bubble_2") {
        buffFrame.atk = (basicFrame.def ?? 0) + buffFrame.def - (basicFrame.atk ?? 0);
    }

    if (["char_391_rosmon", "char_1027_greyy2", "char_421_crow", "char_431_ashlok", "char_4066_highmo", "char_4039_horn"].includes(charId)) {
        if (charId == "char_4039_horn" && options.melee) {
        } else {
            buffFrame.maxTarget = 999;
        }
    }

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
        spRecoverIntervals: [] as unknown[],
        applied: {},
    };
    let critFrame = {};
    if (options.crit) {
        for (const b in buffList) {
            critBuffFrame = applyBuff(
                charAttr,
                options,
                operatorData,
                critBuffFrame,
                b,
                buffList[b] as {
                    key: string;
                    value: number;
                    valueStr: string | null;
                }[],
                isSkill,
                true,
                enemy,
            );
        }
        if (options.buff)
            critBuffFrame = applyBuff(
                charAttr,
                options,
                operatorData,
                critBuffFrame,
                "raidBuff",
                raidBlackboard as unknown as {
                    key: string;
                    value: number;
                    valueStr: string | null;
                }[],
                isSkill,
                true,
                enemy,
            );
        critFrame = getBuffedAttributes(basicFrame, critBuffFrame);
    }

    /**
     * @todo Need to change this to English
     */
    if (charData.description.includes("阻挡的<@ba.kw>所有敌人") && buffFrame.maxTarget < (basicFrame.blockCnt ?? 0)) {
        buffFrame.maxTarget = basicFrame.blockCnt ?? 0;
    } else if (["所有敌人", "群体法术伤害", "群体物理伤害"].some((kw) => charData.description.includes(kw))) {
        buffFrame.maxTarget = 999;
    } else if (charData.description.includes("恢复三个") && !(isSkill && charId == "char_275_breeze")) {
        buffFrame.maxTarget = Math.max(buffFrame.maxTarget, 3);
    }
    if (options.token) {
        if (blackboard.id == "skchr_mgllan_3" || (isSkill && blackboard.id == "skchr_mgllan_2")) buffFrame.maxTarget = 999;
        if (blackboard.id == "skchr_ling_3") buffFrame.maxTarget = options.ling_fusion ? 4 : 2;
    }

    const fps = 30;
    const _spd = Math.min(Math.max(10, Number(finalFrame.attackSpeed)), 600);
    if (finalFrame.attackSpeed !== _spd) {
        finalFrame.attackSpeed = _spd;
    }

    // sec spec
    if ((checkSpecs(blackboard.id, "sec") && isSkill) || (options.annie && charId == "char_1023_ghost2")) {
        let intv: number | boolean = 1;
        if (checkSpecs(blackboard.id, "interval")) {
            intv = checkSpecs(blackboard.id, "interval");
        }
        finalFrame.baseAttackTime = Number(intv);
        finalFrame.attackSpeed = 100;
        buffFrame.attackSpeed = 0;
    }

    const realAttackTime = ((finalFrame.baseAttackTime ?? 1) * 100) / finalFrame.attackSpeed;
    let frame = realAttackTime * fps;
    // 额外帧数补偿 https://bbs.nga.cn/read.php?tid=20555008
    let corr = checkSpecs(charId, "frame_corr") || 0;
    const corr_s = checkSpecs(blackboard.id, "frame_corr");
    if (!(corr_s === false) && isSkill) corr = corr_s;
    if (corr != 0) {
        let real_frame = Math.ceil(frame);
        real_frame += Number(corr);
        frame = real_frame;
    } else {
        frame = Math.round(frame);
    }
    const frameAttackTime = frame / fps;
    const attackTime = frameAttackTime;
    calculateAnimation(charData, blackboard.id, isSkill, realAttackTime);
    if (isSkill && blackboard.id == "skchr_platnm_2") {
        let rate =
            (attackTime - 1) /
            ((
                buffList.tachr_204_platnm_1 as {
                    ["attack@max_delta"]: number;
                }
            )["attack@max_delta"] -
                1);
        rate = Math.min(Math.max(rate, 0), 1);
        buffFrame.atk_scale =
            1 +
            rate *
                ((
                    buffList.tachr_204_platnm_1 as {
                        ["attack@max_atk_scale"]: number;
                    }
                )["attack@max_atk_scale"] -
                    1);
        finalFrame = getBuffedAttributes(basicFrame, buffFrame);
    } else if (
        buffList.tachr_215_mantic_1 &&
        attackTime >=
            (
                buffList.tachr_215_mantic_1 as {
                    delay: number;
                }
            ).delay
    ) {
        const atk =
            (basicFrame.atk ?? 0) *
            (
                buffList.tachr_215_mantic_1 as {
                    atk: number;
                }
            ).atk;

        Object.assign(finalFrame, {
            atk: (finalFrame.atk ?? 0) + atk,
        });

        buffFrame.atk = (finalFrame.atk ?? 0) - (basicFrame.atk ?? 0);
    }

    let enemyBuffFrame = JSON.parse(JSON.stringify(buffFrame)) as typeof buffFrame;
    for (const b in buffList) {
        const buffName = b == "skill" ? buffList[b].id : b;
        if (checkSpecs(buffName, "keep_debuff") && !enemyBuffFrame.applied[buffName as keyof typeof enemyBuffFrame.applied]) {
            enemyBuffFrame = applyBuff(
                charAttr,
                options,
                operatorData,
                enemyBuffFrame,
                buffName,
                buffList[b] as {
                    key: string;
                    value: number;
                    valueStr: string | null;
                }[],
                true,
                false,
                enemy,
            );
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

    if (["chain", "chainhealer"].includes(charData.subProfessionId)) {
        let scale = 0.85,
            s = 1,
            tot = 1;
        const sks = [1];
        if (isSkill && blackboard.id == "skchr_leizi_2") scale = 1;
        else if (operatorData.equipId && charAttr.char.modules.find((module) => module.id === operatorData.equipId))
            scale = (
                basicFrame.equip_blackboard as unknown as {
                    trait: {
                        ["attack@chain.atk_scale"]: number;
                    };
                }
            )?.trait["attack@chain.atk_scale"];
        else if (charData.subProfessionId == "chainhealer") {
            const prefabId = charId.replace("char", "tachr") + "_trait";
            scale = (
                buffList[prefabId] as {
                    ["attack@chain.atk_scale"]: number;
                }
            )["attack@chain.atk_scale"];
        }

        for (let i = 0; i < ecount - 1; ++i) {
            s *= scale;
            tot += s;
            sks.push(s);
        }

        buffFrame.damage_scale *= tot / ecount;
    }

    const dur = calcDurations(isSkill, attackTime, Number(finalFrame.attackSpeed), levelData as unknown as Skill["levels"][0], buffList, buffFrame, ecount, options, charData);

    const rst = checkResetAttack(
        blackboard.id,
        blackboard as unknown as {
            key: string;
            value: number;
            valueStr?: string | null;
        }[],
        options,
    );
    if (rst && String(rst) !== "ogcd" && isSkill) {
        calcEdges(
            blackboard as unknown as {
                key: string;
                value: number;
                valueStr?: string | null;
            }[],
            frame,
            dur,
        );
    }

    if (options.crit && critBuffFrame["prob" as keyof typeof critBuffFrame]) {
        if (damageType != 2) {
            if (buffList.tachr_155_tiger_1) {
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
            } else if (blackboard.id == "skchr_aurora_2" && isSkill) {
                (dur as unknown as { critCount: number }).critCount = options.freeze ? 9 : 3;
            } else (dur as unknown as { critCount: number }).critCount = dur.attackCount * (critBuffFrame as unknown as { prob: number }).prob;

            if ((dur as unknown as { critCount: number }).critCount > 1) (dur as unknown as { critCount: number }).critCount = Math.floor((dur as unknown as { critCount: number }).critCount);
            if (buffList.tachr_222_bpipe_1) {
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

    let hitDamage = Number(finalFrame.atk);
    let critDamage = 0;
    const damagePool = [0, 0, 0, 0, 0];
    const extraDamagePool = [0, 0, 0, 0, 0];
    let move = 0;

    function calculateHitDamage(frame: Record<string, number>, scale: number) {
        let minRate = 0.05,
            ret = 0;
        if (buffList.tachr_144_red_1)
            minRate = (
                buffList.tachr_144_red_1 as {
                    atk_scale: number;
                }
            ).atk_scale;
        if (buffList.tachr_366_acdrop_1) {
            minRate = options.cond
                ? (
                      buffList.tachr_366_acdrop_1 as {
                          atk_scale_2: number;
                      }
                  ).atk_scale_2
                : (
                      buffList.tachr_366_acdrop_1 as {
                          atk_scale: number;
                      }
                  ).atk_scale;
        }
        if (damageType == 0) ret = Math.max((frame.atk ?? 0) - edef, (frame.atk ?? 1) * minRate);
        else if (damageType == 1) ret = Math.max((frame.atk ?? 1) * (1 - emrpct), (frame.atk ?? 1) * minRate);
        else ret = frame.atk ?? 0;
        if (scale != 1) {
            ret *= scale;
        }
        return ret;
    }

    hitDamage = calculateHitDamage(finalFrame, buffFrame.damage_scale);
    Object.assign(damagePool, {
        [damageType]: (damagePool[damageType] ?? 0) + hitDamage * dur.hitCount,
    });

    if (options.crit) {
        if (blackboard.id == "skchr_peacok_2") {
            (dur as unknown as { critHitCount: number }).critHitCount = 0;
            if (isSkill) {
                damageType = 1;
                ecount = enemy.count;
                (dur as unknown as { critHitCount: number }).critHitCount = enemy.count;
            }
        }
        edef = Math.max(0, ((enemy.def + critBuffFrame.edef) * critBuffFrame.edef_scale - critBuffFrame.edef_pene) * (1 - critBuffFrame.edef_pene_scale));
        critDamage = calculateHitDamage(critFrame, critBuffFrame.damage_scale);

        Object.assign(damagePool, {
            [damageType]: (damagePool[damageType] ?? 0) + critDamage * (dur as unknown as { critCount: number }).critCount,
        });
    }

    if (["char_1012_skadi2", "char_101_sora", "char_4045_heidi"].includes(charId)) {
        let ratio_sora = 0.1;
        if (isSkill && blackboard.id == "skchr_skadi2_3") ratio_sora = 0;
        else if (
            isSkill &&
            (
                blackboard as unknown as {
                    ["attack@atk_to_hp_recovery_ratio"]: number;
                }
            )["attack@atk_to_hp_recovery_ratio"]
        )
            ratio_sora = (
                blackboard as unknown as {
                    ["attack@atk_to_hp_recovery_ratio"]: number;
                }
            )["attack@atk_to_hp_recovery_ratio"];
        extraDamagePool[2] = ratio_sora * (finalFrame.atk ?? 1) * dur.duration * enemy.count;
        damagePool[2] = 0;
        damagePool[3] = 0;
    }
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
                blackboard: blackboard as unknown as {
                    key: string;
                    value: number;
                    valueStr: string | null;
                }[],
                dur,
                attackTime,
                hitDamage,
                damageType,
                edef,
                ecount,
                emrpct,
                char: charData,
            };

            damagePool[damageType] = calculateGradDamage(kwargs);
        }
    }

    // 额外伤害
    for (const b in buffList) {
        let buffName = b;
        const bb = buffList[b] as {
            key: string;
            value: number;
            valueStr: string | null;
        }; // blackboard
        if (buffName == "skill") {
            buffName = (
                bb as unknown as {
                    id: string;
                }
            ).id;
        }
        const pool = [0, 0, 0, 0, 0];
        let damage = 0;
        let heal = 0,
            atk = 0;

        if (!isSkill) {
            switch (buffName) {
                case "skchr_ethan_1":
                    pool[1]! +=
                        (
                            bb as unknown as {
                                ["attack@poison_damage"]: number;
                            }
                        )["attack@poison_damage"] *
                        dur.duration *
                        (1 - emrpct) *
                        ecount;
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
                    }
                default:
                    if (b == "skill") continue;
            }
        }
        switch (buffName) {
            case "tachr_129_bluep_1":
                damage = Math.max(
                    (
                        bb as unknown as {
                            poison_damage: number;
                        }
                    ).poison_damage *
                        (1 - emrpct),
                    (
                        bb as unknown as {
                            poison_damage: number;
                        }
                    ).poison_damage * 0.05,
                );
                let total_damage = damage * dur.duration * ecount;
                if (isSkill && blackboard.id == "skchr_bluep_1" && ecount > 1) {
                    const damage2 =
                        damage *
                        (
                            blackboard as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale;
                    total_damage = damage * dur.duration + damage2 * 3;
                }
                pool[1]! += total_damage;
                break;
            case "tachr_293_thorns_1":
                const poison: number = options.thorns_ranged
                    ? (
                          bb as unknown as {
                              ["damage[ranged]"]: number;
                          }
                      )["damage[ranged]"]
                    : (
                          bb as unknown as {
                              ["damage[normal]"]: number;
                          }
                      )["damage[normal]"];
                damage = Math.max(poison * (1 - emrpct), poison * 0.05) * dur.duration * ecount;
                pool[1] = damage;
                break;
            case "tachr_346_aosta_1":
                let poison2 =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                if (blackboard.id == "skchr_aosta_2")
                    poison2 *= (
                        blackboard as unknown as {
                            talent_scale: number;
                        }
                    ).talent_scale;
                damage = Math.max(poison2 * (1 - emrpct), poison2 * 0.05) * dur.duration * ecount;
                pool[1] = damage;
                break;
            case "tachr_181_flower_1":
                pool[2]! +=
                    (
                        bb as unknown as {
                            atk_to_hp_recovery_ratio: number;
                        }
                    ).atk_to_hp_recovery_ratio *
                    (finalFrame.atk ?? 1) *
                    dur.duration *
                    enemy.count;
                break;
            case "tachr_436_whispr_1":
                if (options.cond) {
                    const ts = blackboard.id == "skchr_whispr_2" ? blackboard.talent_scale : 1;
                    const extra_hps =
                        (
                            bb as unknown as {
                                atk_to_hp_recovery_ratio: number;
                            }
                        ).atk_to_hp_recovery_ratio *
                        (finalFrame.atk ?? 1) *
                        (ts ?? 1);
                    pool[2]! += extra_hps * dur.duration * enemy.count;
                }
                break;
            case "tachr_188_helage_trait":
            case "tachr_337_utage_trait":
            case "tachr_475_akafyu_trait":
                pool[2]! += bb.value * dur.hitCount;
                break;
            case "tachr_485_pallas_2":
                pool[2]! += bb.value * dur.hitCount;
                if ("pallas_e_t_2.value" in bb) {
                    pool[2]! +=
                        (
                            bb as {
                                ["pallas_e_t_2.value"]: number;
                            }
                        )["pallas_e_t_2.value"] * dur.hitCount;
                }
                break;
            case "tachr_421_crow_trait":
            case "tachr_4066_highmo_trait":
                pool[2]! += bb.value * dur.attackCount * Math.min(ecount, 2);
                break;
            case "tachr_2013_cerber_1":
                damage =
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    edef *
                    Math.max(1 - emrpct, 0.05);
                pool[1]! += damage * dur.hitCount;
                break;
            case "tachr_391_rosmon_trait":
            case "tachr_1027_greyy2_trait":
                let ntimes = 1;
                if (isSkill && blackboard.id == "skchr_rosmon_2") ntimes = 3;
                const quake_atk =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            ["attack@append_atk_scale"]: number;
                        }
                    )["attack@append_atk_scale"];
                const quake_damage = Math.max(quake_atk - edef, quake_atk * 0.05);

                damage = quake_damage * dur.hitCount * ntimes;
                pool[0]! += damage;
                break;
            case "skchr_ifrit_2":
                damage =
                    (basicFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["burn.atk_scale"]: number;
                        }
                    )["burn.atk_scale"] *
                    Math.floor(
                        (
                            bb as unknown as {
                                duration: number;
                            }
                        ).duration,
                    ) *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * dur.attackCount * ecount;
                break;
            case "skchr_amgoat_2":
                damage = ((finalFrame.atk ?? 0) / 2) * (1 - enemy.res / 100) * buffFrame.damage_scale;
                pool[1]! += damage * dur.attackCount * (enemy.count - 1);
                break;
            case "skchr_nightm_2":
                move =
                    (
                        bb as unknown as {
                            duration: number;
                        }
                    ).duration / 4;
                pool[3]! += bb.value * move * ecount * buffFrame.damage_scale;
                break;
            case "skchr_weedy_3":
                if (options.token) {
                    move =
                        ((
                            bb as unknown as {
                                force: number;
                            }
                        ).force *
                            (
                                bb as unknown as {
                                    force: number;
                                }
                            ).force) /
                            3 +
                        (
                            bb as unknown as {
                                duration: number;
                            }
                        ).duration /
                            5;
                } else
                    move =
                        ((
                            bb as unknown as {
                                force: number;
                            }
                        ).force *
                            (
                                bb as unknown as {
                                    force: number;
                                }
                            ).force) /
                            4 +
                        (
                            bb as unknown as {
                                duration: number;
                            }
                        ).duration /
                            5;
                pool[3]! += bb.value * move * ecount * buffFrame.damage_scale;
                break;
            case "skchr_huang_3":
                const finishAtk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            damage_by_atk_scale: number;
                        }
                    ).damage_by_atk_scale;
                damage = Math.max(finishAtk - enemy.def, finishAtk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * ecount;
                break;
            case "skchr_chen_2":
                damage = (finalFrame.atk ?? 1) * (1 - emrpct) * buffFrame.damage_scale;
                pool[1]! += damage * dur.hitCount;
                break;
            case "skchr_bibeak_1":
                if (enemy.count > 1) {
                    damage = (finalFrame.atk ?? 1) * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1]! += damage;
                }
                break;
            case "skchr_ayer_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * enemy.count * dur.hitCount;
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
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["attack@skill.atk_scale"]: number;
                        }
                    )["attack@skill.atk_scale"] *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * dur.hitCount;
                break;
            case "skchr_haak_2":
            case "skchr_haak_3":
                break;
            case "skchr_podego_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            projectile_delay_time: number;
                        }
                    ).projectile_delay_time *
                    (1 - emrpct) *
                    enemy.count *
                    buffFrame.damage_scale;
                pool[1] = damage;
                damagePool[1] = 0;
                break;
            case "skchr_beewax_2":
            case "skchr_mint_2":
                if (isSkill) {
                    damage =
                        (finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        (1 - emrpct) *
                        ecount *
                        buffFrame.damage_scale;
                    pool[1] = damage;
                }
                break;
            case "skchr_tomimi_2":
                if (isSkill && options.crit) {
                    damage = Math.max((finalFrame.atk ?? 0) - enemy.def, (finalFrame.atk ?? 1) * 0.05);
                    pool[0]! +=
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
                atk =
                    ((finalFrame.atk ?? 0) /
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale) *
                    (
                        bb as unknown as {
                            atk_scale_2: number;
                        }
                    ).atk_scale_2;
                const hit =
                    Math.min(
                        enemy.count - 1,
                        (
                            bb as unknown as {
                                show_max_target: number;
                            }
                        ).show_max_target,
                    ) * dur.hitCount;
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * hit;
                break;
            case "skchr_archet_2":
                const n = Math.min(4, enemy.count - 1);
                if (n > 0) {
                    const hit = ((9 - n) * n) / 2;
                    pool[0]! += hitDamage * hit;
                }
                break;
            case "tachr_338_iris_trait":
            case "tachr_469_indigo_trait":
            case "tachr_4046_ebnhlz_trait":
            case "tachr_297_hamoni_trait":
                if (isSkill && ["skchr_iris_2", "skchr_ebnhlz_2"].includes(blackboard.id)) {
                } else {
                    const talent_key = charId.replace("char", "tachr") + "_1";
                    let scale =
                        (
                            buffList[talent_key] as {
                                atk_scale: number;
                            }
                        ).atk_scale ?? 1;
                    if (isSkill && blackboard.id == "skchr_ebnhlz_3")
                        scale *= (
                            buffList.skill as unknown as {
                                talent_scale_multiplier: number;
                            }
                        ).talent_scale_multiplier;

                    let nBalls = (
                        bb as unknown as {
                            times: number;
                        }
                    ).times;
                    if (talent_key == "tachr_4046_ebnhlz_1" && options.cond_elite) ++nBalls;
                    let extra_scale = 0;
                    if ("tachr_4046_ebnhlz_2" in buffList && enemy.count == 1) {
                        extra_scale = (
                            buffList.tachr_4046_ebnhlz_2 as {
                                atk_scale: number;
                            }
                        ).atk_scale;
                    }
                    damage = hitDamage * (scale + extra_scale);
                    const md = damage * nBalls + hitDamage * (1 + extra_scale);
                    const delta = md - hitDamage * (1 + extra_scale) * (1 + nBalls);
                    pool[1]! += delta;
                }
                break;
            case "skchr_ash_3":
                atk =
                    ((finalFrame.atk ?? 0) /
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale) *
                    (options.cond
                        ? (
                              bb as unknown as {
                                  hitwall_scale: number;
                              }
                          ).hitwall_scale
                        : (
                              bb as unknown as {
                                  not_hitwall_scale: number;
                              }
                          ).not_hitwall_scale);
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * enemy.count;
                break;
            case "skchr_blitz_2":
                atk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * enemy.count;
                break;
            case "skchr_rfrost_2":
                atk =
                    ((finalFrame.atk ?? 0) /
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale) *
                    (
                        bb as unknown as {
                            trap_atk_scale: number;
                        }
                    ).trap_atk_scale;
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage;
                break;
            case "skchr_tachak_1":
                atk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                damage = Math.max(atk * (1 - emrpct), atk * 0.05) * buffFrame.damage_scale;
                pool[1]! +=
                    damage *
                    (
                        bb as unknown as {
                            projectile_delay_time: number;
                        }
                    ).projectile_delay_time *
                    enemy.count;
                break;
            case "skchr_pasngr_3":
                atk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                damage = Math.max(atk * (1 - emrpct), atk * 0.05) * buffFrame.damage_scale;
                pool[1]! += damage * ecount * 8;
                break;
            case "skchr_toddi_2":
                atk =
                    ((finalFrame.atk ?? 0) /
                        (
                            bb as unknown as {
                                ["attack@atk_scale"]: number;
                            }
                        )["attack@atk_scale"]) *
                    (
                        bb as unknown as {
                            ["attack@splash_atk_scale"]: number;
                        }
                    )["attack@splash_atk_scale"];
                damage = Math.max(atk - enemy.def, atk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * enemy.count * dur.hitCount;
                break;
            case "skchr_indigo_2":
                if (options.cond) {
                    atk =
                        (finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                ["indigo_s_2[damage].atk_scale"]: number;
                            }
                        )["indigo_s_2[damage].atk_scale"];
                    damage = Math.max(atk * (1 - emrpct), atk * 0.05) * buffFrame.damage_scale;
                    pool[1]! += damage * enemy.count * dur.duration * 2;
                }
                break;
            case "tachr_426_billro_1":
                if (isSkill) {
                    damage =
                        (
                            bb as unknown as {
                                heal_scale: number;
                            }
                        ).heal_scale * (finalFrame.maxHp ?? 1);
                    if (options.charge) damage *= 2;
                    pool[2]! += damage;
                }
            case "tachr_486_takila_1":
                if (!isSkill) {
                    damage =
                        (finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        (1 - emrpct) *
                        buffFrame.damage_scale;
                }
                break;
            case "tachr_437_mizuki_1":
                let scale = (
                    bb as unknown as {
                        ["attack@mizuki_t_1.atk_scale"]: number;
                    }
                )["attack@mizuki_t_1.atk_scale"];
                if (blackboard.id == "skchr_mizuki_1" && isSkill) scale *= buffList.skill.talent_scale ?? 1;
                damage = ((finalFrame.atk ?? 0) / buffFrame.atk_scale) * scale * (1 - emrpct) * buffFrame.damage_scale;
                let nHit = (
                    bb as unknown as {
                        ["attack@max_target"]: number;
                    }
                )["attack@max_target"];
                if (isSkill) {
                    if (blackboard.id == "skchr_mizuki_2") nHit += 1;
                    else if (blackboard.id == "skchr_mizuki_3") nHit += 2;
                }
                nHit = dur.attackCount * Math.min(ecount, nHit);
                pool[1]! += damage * nHit;
                break;
            case "tachr_1014_nearl2_1":
                let _scale = (
                    bb as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale;
                const _nHit = options.cond ? 2 : 1;
                damage = (finalFrame.atk ?? 1) * _scale * buffFrame.damage_scale;
                switch (blackboard.id) {
                    case "skchr_nearl2_1":
                        break;
                    case "skchr_nearl2_2":
                        if (isSkill) {
                            pool[3]! += damage * ecount * _nHit;
                        }
                        break;
                    case "skchr_nearl2_3":
                        if (!isSkill) {
                            // Do nothing
                        } else {
                            _scale = (
                                buffList.skill as unknown as {
                                    value: number;
                                }
                            ).value;
                            damage = (finalFrame.atk ?? 1) * _scale * buffFrame.damage_scale;
                            pool[3]! += damage * ecount * _nHit;
                        }
                        break;
                }
                break;
            case "skchr_lmlee_2":
                const lmlee_2_scale =
                    (
                        bb as unknown as {
                            default_atk_scale: number;
                        }
                    ).default_atk_scale +
                    (
                        bb as unknown as {
                            factor_atk_scale: number;
                        }
                    ).factor_atk_scale *
                        (
                            bb as unknown as {
                                max_stack_cnt: number;
                            }
                        ).max_stack_cnt;
                damage = (finalFrame.atk ?? 1) * lmlee_2_scale * (1 - emrpct) * buffFrame.damage_scale;
                break;
            case "uniequip_002_rope":
            case "uniequip_002_slchan":
            case "uniequip_002_snsant":
            case "uniequip_002_glady":
                if (isSkill) {
                    const force =
                        (
                            buffList.skill as unknown as {
                                force: number;
                            }
                        ).force ??
                        (
                            buffList.skill as unknown as {
                                ["attack@force"]: number;
                            }
                        )["attack@force"];
                    move = force + 1;
                    pool[1]! +=
                        (
                            bb as unknown as {
                                trait: {
                                    value: number;
                                };
                            }
                        ).trait.value *
                        move *
                        (1 - emrpct) *
                        ecount *
                        buffFrame.damage_scale;
                }
                break;
            case "skchr_tiger_2":
                pool[2]! +=
                    (damagePool[1] ?? 1) *
                    (
                        bb as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale;
                break;
            case "skchr_strong_2":
                pool[2]! +=
                    (damagePool[0] ?? 1) *
                    (
                        bb as unknown as {
                            scale: number;
                        }
                    ).scale;
                break;
            case "skcom_heal_self[1]":
            case "skcom_heal_self[2]":
                damagePool[2] = 0;
                pool[2]! +=
                    (
                        bb as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale * (finalFrame.maxHp ?? 1);
                break;
            case "skchr_nightm_1":
                pool[2]! +=
                    (damagePool[1] ?? 1) *
                    (
                        bb as unknown as {
                            ["attack@heal_scale"]: number;
                        }
                    )["attack@heal_scale"] *
                    Math.min(
                        enemy.count,
                        (
                            bb as unknown as {
                                ["attack@max_target"]: number;
                            }
                        )["attack@max_target"],
                    );
                break;
            case "tachr_1024_hbisc2_trait":
            case "tachr_1020_reed2_trait":
                pool[2]! +=
                    (damagePool[1] ?? 1) *
                    (
                        bb as unknown as {
                            scale: number;
                        }
                    ).scale;
                break;
            case "skchr_folnic_2":
                pool[2]! +=
                    (((
                        bb as unknown as {
                            ["attack@heal_scale"]: number;
                        }
                    )["attack@heal_scale"] *
                        (finalFrame.atk ?? 1)) /
                        buffFrame.atk_scale) *
                    dur.hitCount;
                break;
            case "skchr_breeze_2":
                damage = (finalFrame.atk ?? 0) / 2;
                pool[2]! += damage * dur.attackCount * (enemy.count - 1);
                break;
            case "skchr_ccheal_1":
                heal =
                    ((finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                heal_scale: number;
                            }
                        ).heal_scale *
                        (
                            bb as unknown as {
                                duration: number;
                            }
                        ).duration *
                        dur.duration) /
                    attackTime;
                pool[2]! += heal;
                break;
            case "skchr_ccheal_2":
                heal =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale *
                    (
                        bb as unknown as {
                            duration: number;
                        }
                    ).duration;
                pool[2]! += heal * enemy.count;
                break;
            case "skchr_shining_2":
            case "skchr_tuye_1":
                heal =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                pool[4]! += heal;
                break;
            case "skchr_cgbird_2":
                heal =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                pool[4]! += heal * ecount;
                break;
            case "skchr_tknogi_2":
            case "skchr_lisa_3":
                heal =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["attack@atk_to_hp_recovery_ratio"]: number;
                        }
                    )["attack@atk_to_hp_recovery_ratio"] *
                    enemy.count *
                    (dur.duration - 1);
                pool[2]! += heal;
                damagePool[2] = 0;
                break;
            case "skchr_blemsh_1":
                heal =
                    ((finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                heal_scale: number;
                            }
                        ).heal_scale) /
                    buffFrame.atk_scale;
                pool[2]! += heal;
                break;
            case "skchr_blemsh_2":
                heal =
                    ((finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                ["attack@atk_to_hp_recovery_ratio"]: number;
                            }
                        )["attack@atk_to_hp_recovery_ratio"]) /
                    buffFrame.atk_scale;
                pool[2]! += heal * dur.duration * enemy.count;
                break;
            case "skchr_blemsh_3":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["attack@blemsh_s_3_extra_dmg[magic].atk_scale"]: number;
                        }
                    )["attack@blemsh_s_3_extra_dmg[magic].atk_scale"];
                damage = Math.max(damage * (1 - emrpct), damage * 0.05);
                heal =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale;
                pool[1]! += damage * dur.attackCount;
                pool[2]! += heal * dur.attackCount;
                break;
            case "skchr_rosmon_1":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            extra_atk_scale: number;
                        }
                    ).extra_atk_scale;
                damage = Math.max(damage * (1 - emrpct), damage * 0.05) * dur.hitCount;
                pool[1]! += damage;
                break;
            case "skchr_kirara_1":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["kirara_s_1.atk_scale"]: number;
                        }
                    )["kirara_s_1.atk_scale"];
                damage = Math.max(damage * (1 - emrpct), damage * 0.05) * dur.hitCount;
                pool[1]! += damage;
                break;
            case "skchr_amiya2_2":
                const arts_atk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                const real_atk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale_2: number;
                        }
                    ).atk_scale_2;
                const arts_dmg = Math.max(arts_atk * (1 - emrpct), arts_atk * 0.05);
                pool[1]! += arts_dmg * 9;
                pool[3]! += real_atk;
                break;
            case "skchr_kafka_1":
                damage = (finalFrame.atk ?? 1) * (1 - emrpct) * enemy.count;
                pool[1] = damage;
                damagePool[1] = 0;
                break;
            case "skchr_kafka_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    enemy.count;
                pool[1] = damage;
                break;
            case "skchr_tuye_2":
                pool[2] =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale;
                pool[2] *= 3;
                break;
            case "skchr_nothin_1":
            case "skchr_nothin_2":
                const a =
                    (finalFrame.atk ?? 1) *
                    (
                        buffList.tachr_455_nothin_1 as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                damage = Math.max(a - edef, a * 0.05);
                break;
            case "skchr_heidi_1":
            case "skchr_heidi_2":
            case "skchr_skadi2_2":
            case "skchr_sora_2":
                break;
            case "skchr_skadi2_3":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    buffFrame.damage_scale;
                pool[3]! += damage * enemy.count * dur.duration;
                break;
            case "skchr_mizuki_3":
                if (ecount < 3) {
                    damage =
                        (
                            bb as unknown as {
                                ["attack@hp_ratio"]: number;
                            }
                        )["attack@hp_ratio"] * (finalFrame.maxHp ?? 1);
                    pool[2]! -= damage * dur.attackCount;
                }
                break;
            case "tachr_473_mberry_trait":
            case "tachr_449_glider_trait":
            case "tachr_4041_chnut_trait":
                let ep_ratio = (
                    bb as unknown as {
                        ep_heal_ratio: number;
                    }
                ).ep_heal_ratio;
                let ep_scale = 1;
                if (isSkill) {
                    switch (blackboard.id) {
                        case "skchr_mberry_1":
                            ep_ratio = (
                                buffList.skill as unknown as {
                                    ep_heal_ratio: number;
                                }
                            ).ep_heal_ratio;
                            break;
                        case "skchr_glider_1":
                            ep_ratio = (
                                buffList.skill as unknown as {
                                    ["glider_s_1.ep_heal_ratio"]: number;
                                }
                            )["glider_s_1.ep_heal_ratio"];
                            ep_scale = 3;
                            break;
                        case "skchr_chnut_1":
                            ep_scale = (
                                buffList.skill as unknown as {
                                    trait_scale: number;
                                }
                            ).trait_scale;
                            break;
                        case "skchr_chnut_2":
                            ep_scale = (
                                buffList.skill as unknown as {
                                    ["attack@heal_continuously_scale"]: number;
                                }
                            )["attack@heal_continuously_scale"];
                            break;
                    }
                }
                if (buffList.tachr_4041_chnut_1 && options.cond) {
                    ep_scale *= (
                        buffList.tachr_4041_chnut_1 as unknown as {
                            ep_heal_scale: number;
                        }
                    ).ep_heal_scale;
                }

                damage = ((finalFrame.atk ?? 1) / buffFrame.heal_scale) * ep_ratio * ep_scale;
                break;
            case "skchr_sleach_2":
                damagePool[0] = 0;
                damagePool[1] = 0;
                damagePool[2] = 0;
                pool[2]! +=
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_to_hp_recovery_ratio: number;
                        }
                    ).atk_to_hp_recovery_ratio *
                    dur.duration;
                break;
            case "skchr_sleach_3":
                damagePool[0] = 0;
                damagePool[1] = 0;
                damagePool[2] = 0;
                damage = Math.max((finalFrame.atk ?? 0) - edef, (finalFrame.atk ?? 1) * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * ecount;
                break;
            case "skchr_gnosis_1":
                const scale_mul_g1 = options.freeze
                    ? 1
                    : (
                          buffList.tachr_206_gnosis_1 as unknown as {
                              damage_scale_freeze: number;
                          }
                      ).damage_scale_freeze /
                      (
                          buffList.tachr_206_gnosis_1 as unknown as {
                              damage_scale_cold: number;
                          }
                      ).damage_scale_cold;
                damage = (finalFrame.atk ?? 1) * (1 - emrpct) * buffFrame.damage_scale * scale_mul_g1;
                pool[1]! += damage * dur.hitCount;
                break;
            case "skchr_gnosis_3":
                const scale_mul_g3 = options.freeze
                    ? 1
                    : (
                          buffList.tachr_206_gnosis_1 as unknown as {
                              damage_scale_freeze: number;
                          }
                      ).damage_scale_freeze /
                      (
                          buffList.tachr_206_gnosis_1 as unknown as {
                              damage_scale_cold: number;
                          }
                      ).damage_scale_cold;
                damage =
                    (finalFrame.atk ?? 1) *
                    (1 - emrpct) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    buffFrame.damage_scale *
                    scale_mul_g3;
                pool[1]! += damage * ecount;
                break;
            case "skchr_ling_3":
                if (!options.token) {
                    damage =
                        (finalFrame.atk ?? 1) *
                        (1 - emrpct) *
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        buffFrame.damage_scale;
                    pool[1]! += damage * ecount * dur.duration * 2;
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
                    damage =
                        (finalFrame.atk ?? 1) *
                        (1 - emrpct) *
                        (
                            bb as unknown as {
                                ["attack@atk_scale_2"]: number;
                            }
                        )["attack@atk_scale_2"] *
                        buffFrame.damage_scale;
                    const funnel = Number(checkSpecs(blackboard.id, "funnel")) || 1;
                    pool[1]! +=
                        damage *
                        enemy.count *
                        funnel *
                        (
                            dur as unknown as {
                                critHitCount: number;
                            }
                        ).critHitCount;
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
                    let kazema_scale = (
                        buffList.tachr_4016_kazema_1 as unknown as {
                            damage_scale: number;
                        }
                    ).damage_scale;
                    if (
                        "uniequip_002_kazema" in buffList &&
                        (
                            buffList.uniequip_002_kazema as unknown as {
                                talent: unknown;
                            }
                        ).talent &&
                        !options.token
                    )
                        kazema_scale = (
                            buffList.uniequip_002_kazema as unknown as {
                                talent: {
                                    damage_scale: number;
                                };
                            }
                        ).talent.damage_scale;
                    damage = ((finalFrame.atk ?? 1) / buffFrame.atk_scale) * kazema_scale * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1]! += damage * ecount;
                    if (isSkill) {
                        damagePool[0] = 0;
                        damagePool[1] = 0;
                    }
                }
                break;
            case "skchr_kazema_2":
                let kazema_scale = (
                    buffList.tachr_4016_kazema_1 as unknown as {
                        damage_scale: number;
                    }
                ).damage_scale;
                let kz_invalid = false;
                if (options.annie) {
                    if (
                        "uniequip_002_kazema" in buffList &&
                        (
                            buffList.uniequip_002_kazema as unknown as {
                                talent: unknown;
                            }
                        ).talent &&
                        !options.token
                    )
                        kazema_scale = (
                            buffList.uniequip_002_kazema as unknown as {
                                talent: {
                                    damage_scale: number;
                                };
                            }
                        ).talent.damage_scale;
                } else if (!options.token) {
                    kz_invalid = true;
                }
                if (!kz_invalid) {
                    damage = (finalFrame.atk ?? 1) * kazema_scale * (1 - emrpct) * buffFrame.damage_scale;
                    pool[1]! += damage * ecount;
                }
                if (options.annie && isSkill) {
                    damagePool[0] = 0;
                    damagePool[1] = 0;
                }
                break;
            case "skchr_phenxi_2":
                const ph_2_atk =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            atk_scale_2: number;
                        }
                    ).atk_scale_2;
                damage = Math.max(ph_2_atk - edef, ph_2_atk * 0.05) * buffFrame.damage_scale;
                pool[0]! += damage * 2 * dur.hitCount;
                break;
            case "skchr_horn_2":
                if (options.overdrive_mode) {
                    damage =
                        ((finalFrame.atk ?? 0) /
                            (
                                bb as unknown as {
                                    ["attack@s2.atk_scale"]: number;
                                }
                            )["attack@s2.atk_scale"]) *
                        (
                            bb as unknown as {
                                ["attack@s2.magic_atk_scale"]: number;
                            }
                        )["attack@s2.magic_atk_scale"] *
                        (1 - emrpct) *
                        buffFrame.damage_scale;
                    pool[1]! += damage * dur.hitCount;
                }
                break;
            case "skchr_horn_3":
                if (options.overdrive_mode && !options.od_trigger) {
                    const horn_3_pct = (dur.duration * (dur.duration - 0.2)) / 2; // 0.4, 1.4,...,11.4
                    damage = ((finalFrame.maxHp ?? 1) * horn_3_pct) / 100;
                    pool[2]! -= damage;
                }
                break;
            case "skcom_heal_up[3]":
                if (options.token) {
                    damagePool[0] = damagePool[2] = 0;
                }
                break;
            case "skchr_irene_3":
                const irene_3_edef = Math.max(
                    0,
                    (enemy.def - enemyBuffFrame.edef_pene) *
                        (1 -
                            (
                                buffList.tachr_4009_irene_1 as unknown as {
                                    def_penetrate: number;
                                }
                            ).def_penetrate),
                );
                const irene_3_atk =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            multi_atk_scale: number;
                        }
                    ).multi_atk_scale;
                damage = Math.max(irene_3_atk - irene_3_edef, irene_3_atk * 0.05) * buffFrame.damage_scale;
                pool[0]! +=
                    damage *
                    (
                        bb as unknown as {
                            multi_times: number;
                        }
                    ).multi_times *
                    ecount;
                break;
            case "skchr_lumen_1":
                heal =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["aura.heal_scale"]: number;
                        }
                    )["aura.heal_scale"];
                const lumen_1_hitcount =
                    (
                        bb as unknown as {
                            ["aura.projectile_life_time"]: number;
                        }
                    )["aura.projectile_life_time"] *
                    dur.attackCount *
                    enemy.count;
                pool[2]! += heal * lumen_1_hitcount;
                break;
            case "skchr_ghost2_3":
                if (isSkill && !options.annie) {
                    if (options.cond) {
                        const ghost2_3_atk =
                            (finalFrame.atk ?? 1) *
                            (
                                bb as unknown as {
                                    ["attack@atk_scale_ex"]: number;
                                }
                            )["attack@atk_scale_ex"];
                        damage = Math.max(ghost2_3_atk - edef, ghost2_3_atk * 0.05) * buffFrame.damage_scale;
                        pool[0]! += damage * dur.hitCount;
                    } else {
                        damage =
                            (finalFrame.maxHp ?? 1) *
                            (
                                bb as unknown as {
                                    ["attack@hp_ratio"]: number;
                                }
                            )["attack@hp_ratio"];
                        pool[2]! -= damage * dur.hitCount;
                    }
                }
                break;
            case "skchr_pianst_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * enemy.count;
                break;
            case "tachr_4047_pianst_1":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                break;
            case "tachr_4046_ebnhlz_2":
                if (enemy.count == 1) {
                    damage =
                        ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        (1 - emrpct) *
                        buffFrame.damage_scale;
                    pool[1]! += damage * dur.hitCount;
                } else if (enemy.count > 1 && "atk_scale_2" in bb) {
                    damage =
                        ((finalFrame.atk ?? 1) / buffFrame.atk_scale) *
                        (
                            bb as unknown as {
                                atk_scale_2: number;
                            }
                        ).atk_scale_2 *
                        (1 - emrpct) *
                        buffFrame.damage_scale;
                    pool[1]! += damage * dur.attackCount * (enemy.count - 1);
                }
                break;
            case "skchr_greyy2_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                const greyy2_2_count =
                    (
                        bb as unknown as {
                            projectile_delay_time: number;
                        }
                    ).projectile_delay_time /
                    (
                        bb as unknown as {
                            interval: number;
                        }
                    ).interval;
                pool[1]! += damage * greyy2_2_count * enemy.count;
                damagePool[1] = 0;
                extraDamagePool[0] = 0;
                break;
            case "skchr_gvial2_1":
                let gvial2_scale = 1;
                if ("tachr_1026_gvial2_2" in buffList)
                    gvial2_scale = options.cond
                        ? (
                              buffList.tachr_1026_gvial2_2 as unknown as {
                                  heal_scale_2: number;
                              }
                          ).heal_scale_2
                        : (
                              buffList.tachr_1026_gvial2_2 as unknown as {
                                  heal_scale_1: number;
                              }
                          ).heal_scale_1;
                pool[2] =
                    damagePool[0]! *
                    (
                        bb as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale *
                    gvial2_scale;
                break;
            case "skchr_provs_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * enemy.count;
                break;
            case "tachr_4064_mlynar_2":
                let mlynar_t2_scale = (
                    bb as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale;
                if (isSkill && blackboard.id == "skchr_mlynar_3") {
                    mlynar_t2_scale += (
                        buffList.skill as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale;
                }
                damage = ((finalFrame.atk ?? 0) / buffFrame.atk_scale) * mlynar_t2_scale * buffFrame.damage_scale;
                break;
            case "skchr_mlynar_3":
                if (isSkill) {
                    damage =
                        ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        buffFrame.damage_scale;
                    pool[3]! += damage * dur.hitCount;
                }
                break;
            case "skchr_lolxh_2":
                if (isSkill && options.cond) {
                    const lolxh_2_edef = Math.max(
                        0,
                        edef -
                            (
                                bb as unknown as {
                                    ["attack@def_penetrate_fixed"]: number;
                                }
                            )["attack@def_penetrate_fixed"],
                    );
                    damage = Math.max((finalFrame.atk ?? 0) - lolxh_2_edef, (finalFrame.atk ?? 1) * 0.05) * buffFrame.damage_scale;
                    pool[0]! += damage * dur.hitCount;
                }
                break;
            case "tachr_117_myrrh_1":
                if (!isSkill) {
                    heal =
                        (finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                heal_scale: number;
                            }
                        ).heal_scale;
                    pool[2]! += heal * enemy.count;
                }
            case "skchr_qanik_2":
                const qanik_2_damage_scale = options.cond
                    ? buffFrame.damage_scale /
                      (
                          buffList.tachr_466_qanik_1 as unknown as {
                              damage_scale: number;
                          }
                      ).damage_scale
                    : buffFrame.damage_scale;
                damage =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            critical_damage_scale: number;
                        }
                    ).critical_damage_scale *
                    (1 - emrpct) *
                    qanik_2_damage_scale;
                pool[1]! += damage * ecount * enemy.count;
                break;
            case "tachr_157_dagda_2":
                if (options.cond) {
                    pool[2]! +=
                        (damagePool[0] ?? 1) *
                        (
                            bb as unknown as {
                                heal_scale: number;
                            }
                        ).heal_scale;
                }
                break;
            case "skchr_dagda_1":
                const dagda_1_atk =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["attack@defensive_atk_scale"]: number;
                        }
                    )["attack@defensive_atk_scale"];
                damage = Math.max(dagda_1_atk - enemy.def, dagda_1_atk * 0.05) * buffFrame.damage_scale;
                break;
            case "skchr_judge_1":
                damage =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            atk_scale_2: number;
                        }
                    ).atk_scale_2 *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * dur.hitCount;
                break;
            case "tachr_4065_judge_1":
                break;
            case "tachr_4065_judge_2":
                damage =
                    ((finalFrame.atk ?? 0) / buffFrame.atk_scale) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                break;
            case "skchr_texas2_1":
                const texas2_s1_dur =
                    dur.duration +
                    (
                        bb as unknown as {
                            ["attack@texas2_s_1[dot].duration"]: number;
                        }
                    )["attack@texas2_s_1[dot].duration"] -
                    1;
                damage =
                    (
                        bb as unknown as {
                            ["attack@texas2_s_1[dot].dot_damage"]: number;
                        }
                    )["attack@texas2_s_1[dot].dot_damage"] *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * texas2_s1_dur;
                break;
            case "skchr_texas2_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * enemy.count;
                break;
            case "skchr_texas2_3":
                const texas2_s3_aoe =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["appear.atk_scale"]: number;
                        }
                    )["appear.atk_scale"] *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                const texas2_s3_target = Math.min(
                    enemy.count,
                    (
                        bb as unknown as {
                            max_target: number;
                        }
                    ).max_target,
                );
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += texas2_s3_aoe * enemy.count * 2 + damage * texas2_s3_target * dur.duration;
                break;
            case "skchr_vigil_2":
                if (options.token) {
                    pool[2]! +=
                        (finalFrame.maxHp ?? 1) *
                        (
                            bb as unknown as {
                                ["vigil_wolf_s_2.hp_ratio"]: number;
                            }
                        )["vigil_wolf_s_2.hp_ratio"];
                }
                break;
            case "skchr_vigil_3":
                if (options.cond || options.token) {
                    let vigil_final_atk = (
                        finalFrame as {
                            atk: number;
                        }
                    ).atk;
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
                        vigil_final_atk = (
                            vigil_final as {
                                atk: number;
                            }
                        ).atk;
                    }
                    damage =
                        vigil_final_atk *
                        (
                            bb as unknown as {
                                ["attack@vigil_s_3.atk_scale"]: number;
                            }
                        )["attack@vigil_s_3.atk_scale"] *
                        (1 - emrpct) *
                        buffFrame.damage_scale;
                    pool[1]! += damage * dur.hitCount;
                }
                break;
            case "skchr_ironmn_1":
                if (options.token) {
                    damagePool[0] = 0;
                }
                break;
            case "skchr_ironmn_2":
                if (options.token) {
                    damagePool[0] = 0;
                }
                break;
            case "skchr_ironmn_3":
                if (!isSkill && options.token) {
                    damagePool[0] = 0;
                }
                break;
            case "sktok_ironmn_pile3":
                if (isSkill) {
                    const pile3_atk = (finalFrame.atk ?? 0) / 2;
                    damage = Math.max(pile3_atk - edef, pile3_atk * 0.05) * buffFrame.damage_scale;
                    pool[0]! += damage * enemy.count * dur.hitCount;
                }
                break;
            case "skchr_reed2_2":
                if (isSkill) {
                    damage =
                        (finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        (1 - emrpct) *
                        buffFrame.damage_scale;
                    heal =
                        damage *
                        (
                            buffList.tachr_1020_reed2_trait as unknown as {
                                scale: number;
                            }
                        ).scale;
                    const reed2_interval = options.reed2_fast ? 1.567 : 0.8;
                    let reed2_hitCount = Math.ceil((dur.duration - 0.167) / reed2_interval); // 减去抬手时间

                    if (options.reed2_fast) {
                        reed2_hitCount = Math.ceil((dur.duration - 0.167) / reed2_interval) * 3;
                    }
                    if (options.rosmon_double) {
                        reed2_hitCount *= 2;
                    }
                    pool[1]! += damage * reed2_hitCount;
                    pool[2]! += heal * reed2_hitCount;
                }
                break;
            case "skchr_reed2_3":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["talent@s3_atk_scale"]: number;
                        }
                    )["talent@s3_atk_scale"] *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * dur.duration * ecount;
                break;
            case "skchr_puzzle_2":
                damage =
                    (finalFrame.atk ?? 1) *
                    (
                        bb as unknown as {
                            ["attack@atk_scale_2"]: number;
                        }
                    )["attack@atk_scale_2"] *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                const puzzle_hitCount_skill = 55;
                pool[1]! += damage * puzzle_hitCount_skill;
                break;
            case "skchr_hamoni_2":
                damage =
                    (
                        bb as unknown as {
                            damage_value: number;
                        }
                    ).damage_value *
                    (1 - emrpct) *
                    buffFrame.damage_scale;
                pool[1]! += damage * dur.duration * enemy.count;
                break;
            case "tachr_197_poca_1":
                if (options.cond && "extra_atk_scale" in bb) {
                    const poca_t1_atk =
                        (finalFrame.atk ?? 1) *
                        (
                            bb as unknown as {
                                extra_atk_scale: number;
                            }
                        ).extra_atk_scale;
                    damage = Math.max(poca_t1_atk - edef, poca_t1_atk * 0.05) * buffFrame.damage_scale;
                    pool[0]! += damage * dur.hitCount;
                }
                break;
        } // extraDamage switch ends here

        let hpratiosec = (
            bb as unknown as {
                hp_recovery_per_sec_by_max_hp_ratio: number;
            }
        ).hp_recovery_per_sec_by_max_hp_ratio;
        const hpsec = (
            bb as unknown as {
                hp_recovery_per_sec: number;
            }
        ).hp_recovery_per_sec;
        if (hpratiosec) {
            if (buffName == "tachr_478_kirara_1") {
                if (options.cond)
                    hpratiosec = (
                        bb as unknown as {
                            ["kirara_t_2.hp_recovery_per_sec_by_max_hp_ratio"]: number;
                        }
                    )["kirara_t_2.hp_recovery_per_sec_by_max_hp_ratio"];
                if (isSkill && blackboard.id == "skchr_kirara_2") {
                    hpratiosec *= (
                        buffList.skill as unknown as {
                            talent_scale: number;
                        }
                    ).talent_scale;
                }
            }

            if (buffName == "tachr_344_beewax_1" && isSkill) {
            } else if (buffName == "tachr_362_saga_2") {
            } else if (buffName == "tachr_293_thorns_2") {
                if (blackboard.id == "skchr_thorns_2" && isSkill) {
                    pool[2]! += hpratiosec * (finalFrame.maxHp ?? 1) * (dur.duration + dur.stunDuration - 2);
                } else {
                }
            } else if (buffName == "tachr_422_aurora_1") {
                if (!isSkill) {
                    const aurora_hp_time = levelData.spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * (1 + buffFrame.spRecoverRatio)) / 2 + dur.stunDuration;
                    const aurora_hps = hpratiosec * (finalFrame.maxHp ?? 1);
                    pool[2]! += aurora_hps * aurora_hp_time;
                }
            } else {
                pool[2]! += hpratiosec * (finalFrame.maxHp ?? 1) * (dur.duration + dur.stunDuration);
            }
        }
        if (hpsec) {
            if ((buffName == "tachr_291_aglina_2" && isSkill) || (buffName == "tachr_188_helage_2" && !options.noblock)) {
                /* skip */
            } else {
                pool[2]! += hpsec * (dur.duration + dur.stunDuration);
            }
        }
        if (
            (
                bb as unknown as {
                    hp_ratio: number;
                }
            ).hp_ratio
        ) {
            switch (buffName) {
                case "skchr_huang_3":
                case "skchr_utage_2":
                case "skchr_akafyu_2":
                case "skchr_kazema_2":
                    if (!options.annie && !options.token) {
                        damage =
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio * (finalFrame.maxHp ?? 1);
                        pool[2]! -= damage;
                    }
                    break;
                case "skchr_ifrit_3":
                case "skchr_skadi2_3":
                case "skchr_aprot2_2":
                    pool[2]! -=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio *
                        (finalFrame.maxHp ?? 1) *
                        dur.duration;
                    break;
                case "skchr_bldsk_2":
                    pool[2]! -=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio *
                        (finalFrame.maxHp ?? 1) *
                        (
                            bb as unknown as {
                                duration: number;
                            }
                        ).duration *
                        2;
                    break;
                case "tachr_225_haak_trait":
                    pool[2]! -=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio *
                        (finalFrame.maxHp ?? 1) *
                        dur.duration;
                    break;
                case "tachr_225_haak_1":
                    if (options.crit) {
                        heal =
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio *
                            (finalFrame.maxHp ?? 1) *
                            buffFrame.heal_scale;
                        pool[2]! +=
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
                        pool[2]! -=
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio *
                            dur.attackCount *
                            (finalFrame.maxHp ?? 1);
                    }
                    break;
                case "sktok_ironmn_pile3":
                    if (options.token && isSkill) {
                        damage =
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio * (finalFrame.maxHp ?? 1);
                        pool[2]! -= damage * dur.hitCount;
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
                    pool[4]! +=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio *
                        (finalFrame.maxHp ?? 1) *
                        0.9;
                    break;
                case "skchr_phatom_1":
                    pool[4]! +=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio * (finalFrame.maxHp ?? 1);
                    break;
                case "skchr_surtr_3":
                    pool[4]! -= (finalFrame.maxHp ?? 1) + 5000;
                    break;
                case "tachr_311_mudrok_1":
                    pool[2]! +=
                        (((
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio *
                            (finalFrame.maxHp ?? 1)) /
                            (
                                bb as unknown as {
                                    interval: number;
                                }
                            ).interval) *
                        (dur.duration + dur.prepDuration);
                    break;
                case "uniequip_002_skadi":
                case "uniequip_002_flameb":
                case "uniequip_002_gyuki":
                    if (options.equip) {
                        finalFrame.maxHp =
                            (finalFrame.maxHp ?? 1) *
                            (
                                bb as unknown as {
                                    trait: {
                                        max_hp: number;
                                    };
                                }
                            ).trait.max_hp;
                    }
                    break;
                case "tachr_300_phenxi_1":
                    heal =
                        Math.ceil(
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio * (finalFrame.maxHp ?? 1),
                        ) * 10;
                    break;
                case "skchr_horn_2":
                    if (options.od_trigger && options.overdrive_mode) {
                        pool[2]! -=
                            (finalFrame.maxHp ?? 1) *
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio;
                    }
                    break;
                case "skchr_highmo_2":
                    heal =
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio * (finalFrame.maxHp ?? 1);
                    break;
                case "tachr_4071_peper_1":
                    if (options.cond) {
                        pool[2]! += bb.value * dur.hitCount;
                    }
                    break;
                case "skchr_judge_3":
                    pool[4]! +=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio * (finalFrame.maxHp ?? 1);
                    break;
                case "tachr_437_mizuki_2":
                    if (
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio < 0.5
                    ) {
                        heal =
                            (
                                bb as unknown as {
                                    hp_ratio: number;
                                }
                            ).hp_ratio * (finalFrame.maxHp ?? 1);
                    }
                    break;
                default:
                    pool[2]! +=
                        (
                            bb as unknown as {
                                hp_ratio: number;
                            }
                        ).hp_ratio *
                        (finalFrame.maxHp ?? 1) *
                        dur.attackCount;
            }
        }

        for (let i = 0; i < 5; ++i) extraDamagePool[i]! += pool[i]!;
    }

    const totalDamage = [0, 1, 3].reduce((x, y) => x + (damagePool[y] ?? 0) + (extraDamagePool[y] ?? 0), 0);
    const totalHeal = [2, 4].reduce((x, y) => x + (damagePool[y] ?? 0) + (extraDamagePool[y] ?? 0), 0);
    const extraDamage = [0, 1, 3].reduce((x, y) => x + (extraDamagePool[y] ?? 0), 0);
    const extraHeal = [2, 4].reduce((x, y) => x + (extraDamagePool[y] ?? 0), 0);

    let dps = totalDamage / (dur.duration + dur.stunDuration + dur.prepDuration);
    let hps = totalHeal / (dur.duration + dur.stunDuration + dur.prepDuration);
    if (
        !isSkill &&
        checkResetAttack(
            blackboard.id,
            blackboard as unknown as {
                key: string;
                value: number;
                valueStr?: string | null | undefined;
            }[],
            options,
        )
    ) {
        const d = dur.attackCount * attackTime;
        dps = totalDamage / d;
        hps = totalHeal / d;
    }

    return {
        atk: Number(finalFrame.atk),
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
