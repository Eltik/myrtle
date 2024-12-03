import type { Skill } from "~/types/impl/api/static/skills";
import { checkResetAttack } from "./check-reset-attack";
import { checkSpecs } from "./check-specs";
import dpsAnim from "./dps-animation.json";
import type { BuffFrame, BuffList, CalculateDPSParams } from "~/types/impl/frontend/impl/dps-calculator";
import type { Operator } from "~/types/impl/api/static/operator";

export function calcDurations(isSkill: boolean, attackTime: number, attackSpeed: number, levelData: Skill["levels"][0], buffList: BuffList, buffFrame: BuffFrame, enemyCount: number, options: CalculateDPSParams["options"], char: Operator) {
    const blackboard = buffList.skill;
    const skillId = blackboard.id;
    const spData = levelData.spData;
    let duration = 0;
    let attackCount = 0;
    let stunDuration = 0;
    let prepDuration = 0;
    let startSp = 0;
    const rst = checkResetAttack(
        skillId,
        blackboard as unknown as {
            key: string;
            value: number;
            valueStr: string | null;
        }[],
        options,
    );
    const subProf = char.subProfessionId;

    const spTypeTags = {
        1: "time",
        2: "attack",
        4: "hit",
        8: "special",
    };

    const tags = [spTypeTags[spData.spType as unknown as keyof typeof spTypeTags]]; // 技能类型标记

    // 需要模拟的技能（自动回复+自动释放+有充能）
    if (checkSpecs(skillId, "sim")) {
        tags.push("sim");
        duration = 120;
        const fps = 30;
        let now = fps,
            sp = spData.initSp * fps,
            max_sp = 999 * fps;
        const last = {},
            timeline = {},
            total = {};
        let extra_sp = 0;
        const cast_time = checkSpecs(skillId, "cast_time") || (Number(checkSpecs(skillId, "cast_bat")) * 100) / attackSpeed || attackTime * fps;
        const skill_time = Math.max(Number(cast_time), attackTime * fps);

        function time_since(key: string) {
            return now - (last[key as keyof typeof last] || -999);
        }

        function action(key: string) {
            if (!timeline[now as keyof typeof timeline]) (timeline[now as keyof typeof timeline] as []) = [];
            (timeline[now as keyof typeof timeline] as string[]).push(key);
            (last[key as keyof typeof last] as number) = now;
            (total[key as keyof typeof total] as number) = Number(total[key as keyof typeof total] ?? 0) + 1;
        }

        // charge
        const cast_sp = options.charge && checkSpecs(skillId, "charge") ? spData.spCost * 2 : spData.spCost;
        // init sp
        if (skillId == "skchr_amgoat_2" && buffList.tachr_180_amgoat_2)
            sp =
                (((
                    buffList.tachr_180_amgoat_2 as {
                        sp_min: number;
                    }
                ).sp_min +
                    (
                        buffList.tachr_180_amgoat_2 as {
                            sp_max: number;
                        }
                    ).sp_max) /
                    2) *
                fps;
        else if (buffList.tachr_222_bpipe_2)
            sp =
                (
                    buffList.tachr_222_bpipe_2 as {
                        sp: number;
                    }
                ).sp * fps;
        else if (
            (
                buffList.uniequip_002_archet as {
                    talent: {
                        "archet_e_t_2.sp": number;
                    };
                }
            )?.talent["archet_e_t_2.sp"]
        )
            sp =
                (
                    buffList.uniequip_002_archet as {
                        talent: {
                            "archet_e_t_2.sp": number;
                        };
                    }
                ).talent["archet_e_t_2.sp"] * fps;

        (last["ifrit" as keyof typeof last] as unknown) = (last["archet" as keyof typeof last] as unknown) = (last["chen" as keyof typeof last] as unknown) = 1;
        startSp = cast_sp - sp / fps;

        // sp barrier
        max_sp = cast_sp * fps;
        if (
            (
                blackboard as unknown as {
                    ct: number;
                }
            ).ct
        )
            max_sp =
                spData.spCost *
                fps *
                (
                    blackboard as unknown as {
                        ct: number;
                    }
                ).ct;
        if (
            (
                blackboard as unknown as {
                    cnt: number;
                }
            ).cnt
        )
            max_sp =
                spData.spCost *
                fps *
                (
                    blackboard as unknown as {
                        cnt: number;
                    }
                ).cnt;

        let attackAnim: number | boolean = checkSpecs(skillId, "attack_animation");
        if (attackAnim) {
            attackAnim = Math.min(Math.round(attackTime * fps), Number(attackAnim));
        }

        if (Number(spData.spType) == 1) {
            sp += fps;
        }
        while (now <= duration * fps) {
            // normal attack
            if (sp < cast_sp * fps && time_since("attack") >= attackTime * fps && time_since("skill") >= skill_time) {
                action("attack");
                if (Number(spData.spType) === 2) sp += fps;
            }
            if (sp >= cast_sp * fps && time_since("skill") >= skill_time) {
                if (time_since("attack") >= attackTime * fps) action("skill");
                else if (attackAnim && time_since("attack") == attackAnim) {
                    action("reset_animation");
                    action("skill");
                }
            }
            // sp recover
            if (time_since("skill") == 0) sp -= cast_sp * fps;
            if (time_since("skill") >= Number(cast_time) && sp < max_sp) {
                if (Number(spData.spType) === 1) sp += 1 + buffFrame.spRecoveryPerSec;
            }
            if (
                buffList.tachr_134_ifrit_2 &&
                time_since("ifrit") >=
                    (
                        buffList.tachr_134_ifrit_2 as {
                            interval: number;
                        }
                    ).interval *
                        fps
            ) {
                action("ifrit");
                extra_sp = (
                    buffList.tachr_134_ifrit_2 as {
                        sp: number;
                    }
                ).sp;
            }

            const intv_archet = (buffList.tachr_332_archet_1 as { interval: number }) ? (buffList.tachr_332_archet_1 as { interval: number }).interval : 2.5;
            const intv_chen = (buffList.tachr_010_chen_1 as { interval: number }) ? (buffList.tachr_010_chen_1 as { interval: number }).interval : 4;
            if ((buffList.tachr_332_archet_1 || options.archet) && time_since("archet") >= intv_archet * fps) {
                action("archet");
                extra_sp += 1;
            }
            if ((buffList.tachr_010_chen_1 || options.chen) && time_since("chen") >= intv_chen * fps) {
                action("chen");
                extra_sp += 1;
            }
            if (time_since("skill") >= Number(cast_time) && extra_sp > 0) {
                sp += extra_sp * fps;
                if (sp <= max_sp) action("recover_sp");
                else {
                    sp = max_sp;
                    action("recover_overflow");
                }
            }
            extra_sp = 0;
            ++now;
        }

        if (isSkill) {
            attackCount = (total as { skill: number }).skill;
            duration = (attackCount * skill_time) / fps;
        } else {
            attackCount = (total as { attack: number }).attack;
            duration -= ((total as { skill: number }).skill * skill_time) / fps;
        }
    } else {
        if (isSkill) {
            switch (skillId) {
                case "skchr_mudrok_3":
                    prepDuration = (
                        blackboard as unknown as {
                            sleep: number;
                        }
                    ).sleep;
                    break;
                case "skchr_amiya2_2":
                    prepDuration = 3.33;
                    break;
                case "skchr_surtr_3":
                    prepDuration = 0.67;
                    break;
                case "skchr_ash_2":
                case "skchr_nearl2_2":
                case "skchr_blemsh_2":
                case "skchr_ctable_1":
                    prepDuration = 1;
                    break;
                case "skchr_gnosis_3":
                    prepDuration = 1.167;
                    break;
                case "skchr_mint_2":
                    prepDuration = 1.33;
                    break;
                case "skchr_provs_2":
                    prepDuration = 0.767;
                    break;
                case "skchr_red_1":
                    break;
                case "skchr_texas2_2":
                    prepDuration = 0.167;
                    break;
            }

            attackCount = Math.ceil((levelData.duration - prepDuration) / attackTime);
            duration = attackCount * attackTime;
            startSp = spData.spCost - spData.initSp;

            if (buffList.tachr_180_amgoat_2) {
                const init_sp =
                    spData.initSp +
                    ((
                        buffList.tachr_180_amgoat_2 as {
                            sp_min: number;
                        }
                    ).sp_min +
                        (
                            buffList.tachr_180_amgoat_2 as {
                                sp_max: number;
                            }
                        ).sp_max) /
                        2;
                startSp = spData.spCost - init_sp;
            } else if (buffList.tachr_222_bpipe_2) {
                startSp =
                    spData.spCost -
                    spData.initSp -
                    (
                        buffList.tachr_222_bpipe_2 as {
                            sp: number;
                        }
                    ).sp -
                    ((
                        buffList.tachr_222_bpipe_2 as {
                            ["bpipe_e_2[locate].sp"]: number;
                        }
                    )["bpipe_e_2[locate].sp"] || 0);
            } else if (buffList.tachr_456_ash_2) {
                startSp =
                    spData.spCost -
                    spData.initSp -
                    (
                        buffList.tachr_456_ash_2 as {
                            sp: number;
                        }
                    ).sp;
            } else if (
                buffList.uniequip_002_archet &&
                "archet_e_t_2.sp" in
                    (
                        buffList.uniequip_002_archet as {
                            talent: Record<string, number>;
                        }
                    ).talent
            ) {
                startSp =
                    spData.spCost -
                    spData.initSp -
                    (
                        buffList.uniequip_002_archet as {
                            talent: {
                                ["archet_e_t_2.sp"]: number;
                            };
                        }
                    ).talent["archet_e_t_2.sp"];
            }

            if (rst) {
                duration = levelData.duration - prepDuration;
                let frameBegin = Math.round(Number(checkSpecs(skillId, "attack_begin")) || 12);
                if (skillId == "skchr_glaze_2" && options.far) {
                    frameBegin = 27;
                }
                const t = frameBegin / 30;
                attackCount = Math.ceil((duration - t) / attackTime);
            }
            /**
             * @todo: Change to English
             */
            if (levelData.description.includes("持续时间无限") || checkSpecs(skillId, "toggle")) {
                if (skillId == "skchr_thorns_3" && !options.warmup) {
                    // do nothing
                } else if (skillId == "skchr_tuye_2") {
                    duration = spData.spCost / (1 + buffFrame.spRecoveryPerSec);
                    attackCount = Math.ceil(duration / attackTime);
                } else if (skillId == "skchr_surtr_3") {
                    const lock_time = (
                        buffList.tachr_350_surtr_2 as {
                            ["surtr_t_2[withdraw].interval"]: number;
                        }
                    )["surtr_t_2[withdraw].interval"];
                    duration = Math.sqrt(600) + lock_time;
                    attackCount = Math.ceil(duration / attackTime);
                } else {
                    const d = options.short_mode ? 180 : 1000;
                    attackCount = Math.ceil(d / attackTime);
                    duration = attackCount * attackTime;
                    if (checkSpecs(skillId, "toggle")) {
                        tags.push("toggle", "infinity");
                    } else {
                        tags.push("infinity");
                    }
                }
            } else if (Number(spData.spType) === 8) {
                if (
                    levelData.duration <= 0 &&
                    (
                        blackboard as unknown as {
                            duration: number;
                        }
                    ).duration > 0
                ) {
                    levelData.duration = (
                        blackboard as unknown as {
                            duration: number;
                        }
                    ).duration;
                    duration = (
                        blackboard as unknown as {
                            duration: number;
                        }
                    ).duration;
                    attackCount = Math.ceil(levelData.duration / attackTime);
                }
                if (levelData.duration > 0) {
                    tags.push("auto");
                    if (prepDuration > 0) duration = levelData.duration - prepDuration;
                } else if (checkSpecs(skillId, "passive")) {
                    attackCount = 1;
                    duration = attackTime;
                    tags.push("passive");
                } else if (skillId == "skchr_phatom_2") {
                    attackCount = (
                        blackboard as unknown as {
                            times: number;
                        }
                    ).times;
                    duration = attackTime * attackCount;
                } else {
                    attackCount = 1;
                    duration = 0;
                    tags.push("auto", "instant");
                }
            } else if (levelData.duration <= 0) {
                if (checkSpecs(skillId, "instant_buff")) {
                    duration =
                        (
                            blackboard as unknown as {
                                duration: number;
                            }
                        ).duration || Number(checkSpecs(skillId, "duration"));
                    attackCount = Math.ceil(duration / attackTime);
                    tags.push("instant", "buff");
                } else if (checkSpecs(skillId, "magazine")) {
                    let mag: number | boolean = checkSpecs(skillId, "magazine");
                    if (options.charge && skillId == "skchr_chen2_2") mag = 20;
                    else if (skillId == "skchr_ctable_2")
                        mag = (
                            blackboard as unknown as {
                                ["attack@trigger_time"]: number;
                            }
                        )["attack@trigger_time"];
                    if (buffList.tachr_1013_chen2_1) {
                        const prob = (
                            buffList.tachr_1013_chen2_1 as unknown as {
                                ["spareshot_chen.prob"]: number;
                            }
                        )["spareshot_chen.prob"];
                        const new_mag = Math.floor(Number(mag) / (1 - prob));
                        mag = new_mag;
                    }
                    attackCount = Number(mag);
                    duration = attackTime * attackCount;
                    if (rst) duration -= attackTime;
                } else if (skillId == "skchr_blkngt_2" && options.token) {
                    duration = (
                        blackboard as unknown as {
                            ["blkngt_s_2.duration"]: number;
                        }
                    )["blkngt_s_2.duration"];
                    attackCount = Math.ceil(duration / attackTime);
                } else {
                    attackCount = 1;
                    if (String(checkSpecs(skillId, "reset_attack")) !== "ogcd") duration = attackTime;
                    tags.push("instant");
                    if (checkSpecs(skillId, "anim_key") && checkSpecs(skillId, "anim_cast")) {
                        const animKey = checkSpecs(skillId, "anim_key");
                        const dpsAnimKey = char.id as keyof typeof dpsAnim;
                        const dpsAnimKeyKey = String(animKey) as keyof typeof dpsAnim;
                        const animData = (dpsAnim[dpsAnimKey] as Record<string, number>)[dpsAnimKeyKey];
                        const ct =
                            (
                                animData as unknown as {
                                    duration: number;
                                }
                            ).duration || animData;

                        if ((duration < (ct ?? 0) / 30 && Number(spData.spType) === 1) || String(rst) === "ogcd") duration = (ct ?? 0) / 30;
                    }
                    if (checkSpecs(skillId, "cast_time")) {
                        const ct = checkSpecs(skillId, "cast_time");
                        if (duration < Number(ct) / 30 || String(rst) === "ogcd") {
                            if (Number(spData.spType) === 1 || Number(spData.spType) === 2 || String(rst) === "ogcd") duration = Number(ct) / 30;
                        }
                    }
                }
            } else if (skillId == "skchr_glady_3") {
                attackCount = 6;
                attackTime = 1.5;
            } else if (options.annie) {
                duration = 20;
                attackCount = Math.ceil(duration / attackTime);
            }

            if (checkSpecs(skillId, "overdrive")) {
                let attackCountHalf = Math.ceil((levelData.duration - prepDuration) / 2 / attackTime);
                let durationHalf = attackCountHalf * attackTime;
                if (checkSpecs(skillId, "magazine")) {
                    attackCountHalf = Math.ceil(attackCount / 2);
                    durationHalf = attackCountHalf * attackTime;
                }
                if (options.overdrive_mode) {
                    duration -= durationHalf;
                    attackCount -= attackCountHalf;
                    if (options.od_trigger) {
                        duration = attackCount = 0;
                        if (skillId == "skchr_horn_2") {
                            duration = 1.066; // 32f
                            attackCount = attackCountHalf;
                        }
                    }
                } else {
                    duration = durationHalf;
                    attackCount = attackCountHalf;
                }
            }

            if (skillId == "skchr_huang_3") {
                attackCount -= 2;
            } else if (skillId == "skchr_sunbr_2") {
                prepDuration = (
                    blackboard as unknown as {
                        disarm: number;
                    }
                ).disarm;
            } else if (skillId == "skchr_takila_2" && options.charge) {
                duration = (
                    blackboard as unknown as {
                        enhance_duration: number;
                    }
                ).enhance_duration;
                attackCount = Math.ceil(duration / attackTime);
            } else if (char.id === "char_4055_bgsnow" && options.token) {
                if (duration > 25) {
                    duration = 25;
                    attackCount = Math.ceil(duration / attackTime);
                }
            } else if (skillId == "skchr_ironmn_3" && options.token) {
                attackCount = isSkill ? 10 : 0;
                duration = 15;
            }
        } else {
            if (skillId == "skchr_fmout_2") {
                stunDuration = (
                    blackboard as unknown as {
                        time: number;
                    }
                ).time;
            } else if (skillId == "skchr_peacok_2") {
                stunDuration =
                    (
                        blackboard as unknown as {
                            ["failure.stun"]: number;
                        }
                    )["failure.stun"] *
                    (1 -
                        (
                            blackboard as unknown as {
                                prob: number;
                            }
                        ).prob);
            } else if (["skchr_amiya_2", "skchr_liskam_2", "skchr_ghost_2", "skchr_broca_2", "skchr_serum_1", "skchr_aurora_1"].includes(skillId)) {
                stunDuration = (
                    blackboard as unknown as {
                        stun: number;
                    }
                ).stun;
            } else if (skillId == "skchr_folivo_2" && options.token) {
                stunDuration = (
                    blackboard as unknown as {
                        stun: number;
                    }
                ).stun;
            } else if (skillId == "skchr_rockr_2" && !options.od_trigger) {
                stunDuration = 20;
            }

            let spRatio = 1;
            if (buffFrame.spRecoverRatio != 0) {
                spRatio += buffFrame.spRecoverRatio;
            }
            let attackDuration = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio) - stunDuration;
            if (spRatio == 0) {
                attackDuration = 180;
            }

            if (checkSpecs(skillId, "cast_time")) {
                const ct = checkSpecs(skillId, "cast_time");
                if (attackTime > Number(ct) / 30 && String(rst) !== "ogcd") {
                    attackDuration -= attackTime - Number(ct) / 30;
                }
            }
            attackCount = Math.ceil(attackDuration / attackTime);
            duration = attackCount * attackTime;
            if (rst && String(rst) !== "ogcd" && Number(spData.spType) !== 8 && spRatio != 0) {
                const dd = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio) - stunDuration;
                duration = dd;
                const frameBegin = Math.round(Number(checkSpecs(skillId, "attack_begin")) || 12);
                const t = frameBegin / 30;
                attackCount = Math.ceil((duration - t) / attackTime);
            }

            switch (Number(spData.spType)) {
                case 8:
                    if (
                        levelData.duration <= 0 &&
                        (
                            blackboard as unknown as {
                                duration: number;
                            }
                        ).duration > 0
                    ) {
                        levelData.duration = (
                            blackboard as unknown as {
                                duration: number;
                            }
                        ).duration;
                    }
                    if (levelData.duration > 0) {
                        tags.push("auto");
                        if (skillId == "skchr_nearl2_2") {
                            attackCount = 0;
                            duration = 1;
                        } else {
                            attackDuration = levelData.duration;
                            attackCount = Math.ceil(attackDuration / attackTime);
                            duration = attackCount * attackTime;
                        }
                    } else if (checkSpecs(skillId, "passive")) {
                        attackCount = 10;
                        duration = attackCount * attackTime;
                        tags.push("passive");
                    } else {
                        attackDuration = 10;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackCount * attackTime;
                        tags.push("auto", "instant");
                    }
                    break;
                case 4:
                    break;
                case 2:
                    attackCount = spData.spCost;

                    const intv_chen = (buffList.tachr_010_chen_1 as {
                        interval: number;
                    })
                        ? (
                              buffList.tachr_010_chen_1 as {
                                  interval: number;
                              }
                          ).interval
                        : 4;
                    const intv_archet = (buffList.tachr_332_archet_1 as {
                        interval: number;
                    })
                        ? (
                              buffList.tachr_332_archet_1 as {
                                  interval: number;
                              }
                          ).interval
                        : 2.5;
                    let extra_sp = 0,
                        next = true;

                    while (attackCount > 0 && next) {
                        duration = attackCount * attackTime;
                        extra_sp = 0;
                        if (buffList.tachr_010_chen_1 || options.chen) extra_sp += Math.floor(duration / intv_chen);
                        if (buffList.tachr_332_archet_1 || options.archet) extra_sp += Math.floor(duration / intv_archet);
                        if (buffList.tachr_301_cutter_1) {
                            const p = (
                                buffList.tachr_301_cutter_1 as {
                                    prob: number;
                                }
                            ).prob;
                            extra_sp += skillId == "skchr_cutter_1" ? (attackCount * 2 + 1) * p : attackCount * 2 * p;
                        }
                        next = attackCount + extra_sp >= spData.spCost;
                        if (next) attackCount -= 1;
                    }
                    if (!next) attackCount += 1;
                    duration = attackCount * attackTime;
                    const line = [];
                    /**
                     * @todo Change to English
                     */
                    if (buffList.tachr_010_chen_1 || options.chen) line.push(`呵斥触发 ${Math.floor(duration / intv_chen)} 次`);
                    if (buffList.tachr_332_archet_1 || options.archet) line.push(`兰登战术触发 ${Math.floor(duration / intv_archet)} 次`);
                    if (buffList.tachr_301_cutter_1) {
                        const p = (
                            buffList.tachr_301_cutter_1 as {
                                prob: number;
                            }
                        ).prob;
                        const _n = skillId == "skchr_cutter_1" ? (attackCount * 2 + 1) * p : attackCount * 2 * p;
                        line.push(`光蚀刻痕触发 ${_n.toFixed(2)} 次`);
                    }
                    if (rst) {
                        duration -= attackTime;
                    }
                    break;
                case 1:
                    const sp_rate = 1 + buffFrame.spRecoveryPerSec;
                    if (buffList.tachr_002_amiya_1) {
                        attackCount = Math.ceil(
                            (spData.spCost - stunDuration * sp_rate) /
                                ((
                                    buffList.tachr_002_amiya_1 as {
                                        ["amiya_t_1[atk].sp"]: number;
                                    }
                                )["amiya_t_1[atk].sp"] +
                                    attackTime * sp_rate),
                        );
                        duration = attackCount * attackTime;
                    } else if (buffList.tachr_134_ifrit_2) {
                        const i = (
                            buffList.tachr_134_ifrit_2 as {
                                interval: number;
                            }
                        ).interval;
                        const isp =
                            i * sp_rate +
                            (
                                buffList.tachr_134_ifrit_2 as {
                                    sp: number;
                                }
                            ).sp;
                        const recoverCount = Math.ceil((spData.spCost - i) / isp); // recoverCount >= (spCost - i) / isp
                        const r = (spData.spCost - recoverCount * isp) / sp_rate;

                        attackDuration = recoverCount * i + r;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackDuration;
                    } else if (checkSpecs(skillId, "instant_buff")) {
                        attackDuration -=
                            (
                                blackboard as unknown as {
                                    duration: number;
                                }
                            ).duration || Number(checkSpecs(skillId, "duration"));
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackCount * attackTime;
                    } else if (buffList.tachr_400_weedy_2 && options.cannon) {
                        const m = Math.floor(spData.spCost / 55);
                        const a = m * 6 + m * 55 * sp_rate;
                        const b = 6 + 20 * sp_rate;
                        let r = 0;
                        if (a + b > spData.spCost) {
                            const y = Math.floor((spData.spCost - a) / (3 * sp_rate + 1.0));
                            const z = (spData.spCost - a - y) / sp_rate - y * 3;
                            r = 3 * y + z;
                        } else {
                            r = (spData.spCost - a - b) / sp_rate + 20;
                        }
                        attackDuration = m * 55 + r;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackDuration;
                    } else if (options.charge && checkSpecs(skillId, "charge")) {
                        let chargeDuration = spData.spCost;
                        if (buffList.tachr_426_billro_2) {
                            chargeDuration /=
                                1 +
                                buffFrame.spRecoveryPerSec +
                                (
                                    buffList.tachr_426_billro_2 as {
                                        sp_recovery_per_sec: number;
                                    }
                                ).sp_recovery_per_sec;
                        }
                        attackDuration += chargeDuration;
                        duration = attackDuration;
                        attackCount = Math.ceil(attackDuration / attackTime);
                    } else if (options.equip && subProf == "longrange") {
                        const entry = (buffList.uniequip_002_milu || buffList.uniequip_003_fartth || buffList.uniequip_002_lunacu) as {
                            trait: {
                                sp: number;
                            };
                        };
                        if (entry) {
                            attackCount = Math.ceil((spData.spCost - stunDuration * sp_rate) / (entry.trait.sp + attackTime * sp_rate));
                            duration = attackCount * attackTime;
                        }
                    } else if (
                        "uniequip_002_leizi" in buffList &&
                        options.cond &&
                        "sp" in
                            (
                                buffList.uniequip_002_leizi as {
                                    talent: {
                                        ["leizi_e_t_2.sp"]: number;
                                    };
                                }
                            ).talent
                    ) {
                        attackCount = Math.ceil((spData.spCost - stunDuration * sp_rate) / (enemyCount + attackTime * sp_rate));
                        duration = attackCount * attackTime;
                    } else if (buffList.tachr_489_serum_1 && skillId == "skchr_serum_1") {
                        const esp =
                            (
                                buffList.tachr_489_serum_1 as {
                                    sp_recovery_per_sec: number;
                                }
                            ).sp_recovery_per_sec *
                            (stunDuration -
                                (
                                    buffList.tachr_489_serum_1 as {
                                        delay: number;
                                    }
                                ).delay);
                        attackDuration = (spData.spCost - esp) / (1 + buffFrame.spRecoveryPerSec) - stunDuration;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackDuration;
                    } else if (buffList.tachr_422_aurora_1) {
                        attackDuration = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio) / 2;
                        if (attackDuration < stunDuration) attackDuration = 0;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio);
                    } else if (skillId == "skchr_blkngt_2" && options.token) {
                        duration =
                            attackDuration -
                            (
                                blackboard as unknown as {
                                    ["blkngt_s_2.duration"]: number;
                                }
                            )["blkngt_s_2.duration"];
                        attackCount = Math.ceil(duration / attackTime);
                    }
                    break;
                // todo: cast time
            }
        } // else
    } // sim else

    let hitCount = attackCount * buffFrame.times * enemyCount;
    if (isSkill) {
        if (skillId == "skchr_bluep_2") {
            hitCount +=
                attackCount *
                ((
                    blackboard as unknown as {
                        ["attack@times"]: number;
                    }
                )["attack@times"] -
                    1);
        } else if (["skcom_assist_cost[2]", "skchr_utage_1", "skchr_tachak_1"].includes(skillId)) {
            hitCount = 0;
        } else if (skillId == "skchr_kroos2_2") {
            const extra_atk_count =
                attackCount -
                (
                    blackboard as unknown as {
                        ["attack@max_stack_count"]: number;
                    }
                )["attack@max_stack_count"] /
                    2;
            if (extra_atk_count > 0) {
                hitCount += extra_atk_count * 2;
            }
        }
    }

    return {
        attackCount,
        times: buffFrame.times,
        hitCount,
        duration,
        stunDuration,
        prepDuration,
        tags,
        startSp,
    };
}
