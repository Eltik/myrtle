/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-explicit-any */
import { checkSpecs } from "./check-specs";
import type { BuffFrame, CalculateDPSParams, CharAttr, Enemy } from "~/types/impl/frontend/impl/dps-calculator";

export function applyBuff(
    charAttr: CharAttr,
    options: CalculateDPSParams["options"],
    operatorData: {
        equipLevel: number;
        potentialRank: number;
    },
    buffFrm: BuffFrame,
    tag: string,
    blackbd: {
        key: string;
        value: number;
        valueStr: string | null;
    }[],
    isSkill: boolean,
    isCrit: boolean,
    enemy: Enemy,
) {
    const { ...buffFrame } = buffFrm || {
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
    let { ...blackboard } = blackbd;
    const basic = charAttr.basic;
    const charId = charAttr.char.id ?? "";
    const skillId = charAttr.buffList.skill.id;

    // 如果是技能期间，则取得技能ID, 否则不计算技能
    // specialtags里标示的（spType!=8的）被动技能：一直生效
    if (tag == "skill") {
        if (isSkill || checkSpecs(skillId, "passive")) tag = skillId;
        else return buffFrm;
    }

    buffFrm.applied[tag] = true;
    let done = false; // if !done, will call applyBuffDefault() in the end

    function applyBuffDefault() {
        for (const key in blackboard) {
            switch (key) {
                case "atk":
                case "def":
                    buffFrame[key] += (basic[key] ?? 0) * Number(blackboard[key]);
                    break;
                case "max_hp":
                    if (Math.abs(Number(blackboard[key])) > 2) {
                        Object.assign(buffFrame, {
                            maxHp: buffFrame.maxHp + (blackboard[key] as unknown as number),
                        });
                    } else if (Number(blackboard[key]) !== 0) {
                        buffFrame.maxHp += (basic.maxHp ?? 0) * Number(blackboard[key]);
                    }
                    break;
                case "base_attack_time":
                    if (Number((blackboard as unknown as { base_attack_time: string | number }).base_attack_time) < 0) {
                        buffFrame.baseAttackTime += Number((blackboard as unknown as { base_attack_time: string | number }).base_attack_time);
                    } else {
                        buffFrame.baseAttackTime += (basic.baseAttackTime ?? 0) * Number((blackboard as unknown as { base_attack_time: string | number }).base_attack_time);
                    }
                    break;
                case "attack_speed":
                    if (Number(blackboard[key]) === 0) break;
                    buffFrame.attackSpeed += Number((blackboard as unknown as { attack_speed: string | number }).attack_speed);
                    break;
                case "sp_recovery_per_sec":
                    buffFrame.spRecoveryPerSec += Number((blackboard as unknown as { sp_recovery_per_sec: string | number }).sp_recovery_per_sec);
                    break;
                case "atk_scale":
                case "def_scale":
                case "heal_scale":
                case "damage_scale":
                    buffFrame[key] *= Number(blackboard[key]);
                    break;
                case "attack@atk_scale":
                    buffFrame.atk_scale *= Number((blackboard as unknown as { ["attack@atk_scale"]: string | number })["attack@atk_scale"]);
                    break;
                case "attack@heal_scale":
                    buffFrame.heal_scale *= Number((blackboard as unknown as { ["attack@heal_scale"]: string | number })["attack@heal_scale"]);
                    break;
                case "max_target":
                case "attack@max_target":
                    Object.assign(buffFrame, {
                        maxTarget: blackboard[key],
                    });
                    break;
                case "times":
                case "attack@times":
                    Object.assign(buffFrame, {
                        times: blackboard[key],
                    });
                    break;
                case "magic_resistance":
                    if (Number(blackboard[key]) < -1) {
                        Object.assign(buffFrame, {
                            emr: buffFrame.emr + (blackboard[key] as unknown as number),
                        });
                    } else if (Number(blackboard[key]) < 0) {
                        buffFrame.emr_scale *= 1 + Number(blackboard[key]);
                    }
                    break;
                case "prob":
                    if (!(blackboard as unknown as { prob_override: number }).prob_override) {
                        buffFrame.prob = blackboard[key];
                    }
                    break;
                case "edef":
                    Object.assign(buffFrame, {
                        edef: buffFrame.edef + (blackboard[key] as unknown as number),
                    });
                    break;
                case "edef_scale":
                    buffFrame.edef_scale *= 1 + Number(blackboard[key]);
                    break;
                case "edef_pene":
                    Object.assign(buffFrame, {
                        edef_pene: buffFrame.edef_pene + (blackboard[key] as unknown as number),
                    });
                    break;
                case "edef_pene_scale":
                    Object.assign(buffFrame, {
                        edef_pene_scale: blackboard[key],
                    });
                    break;
                case "emr_pene":
                    Object.assign(buffFrame, {
                        emr_pene: buffFrame.emr_pene + (blackboard[key] as unknown as number),
                    });
                    break;
                case "prob_override":
                    buffFrame.prob = blackboard[key];
                    break;
                case "atk_override":
                    Object.assign(buffFrame, {
                        atk: buffFrame.atk + (blackboard[key] as unknown as number),
                    });
                    break;
                case "sp_interval":
                    Object.assign(blackboard[key] ?? {}, {
                        tag: tag,
                    });
                    buffFrame.spRecoverIntervals.push(blackboard[key]);
                    break;
            }
        }
    }

    if (checkSpecs(tag, "cond")) {
        if (!options.cond) {
            switch (tag) {
                case "tachr_348_ceylon_1":
                    Object.assign(blackboard, {
                        atk: (
                            blackboard as unknown as {
                                "ceylon_t_1[common].atk": string | number;
                            }
                        )["ceylon_t_1[common].atk"],
                    });
                    applyBuffDefault();
                    break;
                case "skchr_glacus_2":
                    Object.assign(buffFrame, {
                        atk_scale: (
                            blackboard as unknown as {
                                ["atk_scale[drone]"]: string | number;
                            }
                        )["atk_scale[drone]"],
                    });
                case "tachr_326_glacus_1":
                    if ("sp_recovery_per_sec" in blackboard) delete blackboard.sp_recovery_per_sec;
                    break;
                case "skchr_cutter_2":
                    applyBuffDefault();
                    break;
                case "tachr_145_prove_1":
                    applyBuffDefault();
                    break;
                case "tachr_226_hmau_1":
                    delete (
                        blackboard as unknown as {
                            heal_scale: undefined;
                        }
                    ).heal_scale;
                    applyBuffDefault();
                    break;
                case "tachr_279_excu_trait":
                case "tachr_1013_chen2_trait":
                case "tachr_440_pinecn_trait":
                    if (isSkill && ["skchr_excu_1", "skchr_chen2_1", "skchr_chen2_3", "skchr_pinecn_2"].includes(skillId)) {
                        applyBuffDefault();
                    }
                    break;
                case "tachr_113_cqbw_2":
                    break;
                case "tachr_1012_skadi2_2":
                    Object.assign(blackboard, {
                        atk: (
                            blackboard as unknown as {
                                "skadi2_t_2[atk][1].atk": string | number;
                            }
                        )["skadi2_t_2[atk][1].atk"],
                    });
                    applyBuffDefault();
                    break;
                case "skchr_crow_2":
                    (
                        blackboard as unknown as {
                            base_attack_time: number;
                        }
                    ).base_attack_time *= basic.baseAttackTime ?? 1;
                    applyBuffDefault();
                    break;
                case "tachr_431_ashlok_1":
                    applyBuffDefault();
                    break;
                case "tachr_4013_kjera_1":
                    if (options.freeze) {
                        (
                            blackboard as unknown as {
                                magic_resistance: number;
                            }
                        ).magic_resistance = -15;
                    }
                    applyBuffDefault();
                    break;
                case "tachr_322_lmlee_1":
                    if (options.block) {
                        (
                            blackboard as unknown as {
                                attack_speed: number;
                            }
                        ).attack_speed = (
                            blackboard as unknown as {
                                ["lmlee_t_1[self].attack_speed"]: number;
                            }
                        )["lmlee_t_1[self].attack_speed"];
                        applyBuffDefault();
                    }
                    break;
                case "skchr_phenxi_3":
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["attack@atk_scale_2"]: number;
                        }
                    )["attack@atk_scale_2"];
                    delete (
                        blackboard as unknown as {
                            "attack@atk_scale": undefined;
                        }
                    )["attack@atk_scale"];
                    applyBuffDefault();
                    break;
                case "tachr_4009_irene_2":
                    applyBuffDefault();
                    break;
                case "tachr_4064_mlynar_1":
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            atk_scale_base: number;
                        }
                    ).atk_scale_base;
                    applyBuffDefault();
                    break;
            }
            done = true;
        } else {
            switch (tag) {
                case "tachr_348_ceylon_1":
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk =
                        (
                            blackboard as unknown as {
                                ["ceylon_t_1[common].atk"]: number;
                            }
                        )["ceylon_t_1[common].atk"] +
                        (
                            blackboard as unknown as {
                                ["celyon_t_1[map].atk"]: number;
                            }
                        )["celyon_t_1[map].atk"];
                    break;
                case "skchr_glacus_2":
                    buffFrame.atk_scale = (
                        blackboard as unknown as {
                            ["atk_scale[drone]"]: number;
                        }
                    )["atk_scale[drone]"];
                    done = true;
                    break;
                case "tachr_326_glacus_1":
                    if ("sp_recovery_per_sec" in blackboard) delete blackboard.sp_recovery_per_sec;
                    break;
                case "skchr_cutter_2":
                    buffFrame.maxTarget = (
                        blackboard as unknown as {
                            max_target: number;
                        }
                    ).max_target;
                    buffFrame.atk_scale =
                        (
                            blackboard as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *
                        (
                            blackboard as unknown as {
                                ["cutter_s_2[drone].atk_scale"]: number;
                            }
                        )["cutter_s_2[drone].atk_scale"];
                    done = true;
                    break;
                case "tachr_187_ccheal_1":
                    buffFrame.def += (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def;
                    Object.assign(blackboard, {
                        def: 0,
                    });
                    break;
                case "tachr_145_prove_1":
                    (
                        blackboard as unknown as {
                            prob_override: number;
                        }
                    ).prob_override = (
                        blackboard as unknown as {
                            prob2: number;
                        }
                    ).prob2;
                    break;
                case "tachr_333_sidero_1":
                    delete (
                        blackboard as unknown as {
                            times: undefined;
                        }
                    ).times;
                    break;
                case "tachr_197_poca_1":
                case "skchr_apionr_1":
                    (
                        blackboard as unknown as {
                            edef_pene_scale: number;
                        }
                    ).edef_pene_scale = (
                        blackboard as unknown as {
                            def_penetrate: number;
                        }
                    ).def_penetrate;
                    break;
                case "tachr_358_lisa_2":
                    if (isSkill && skillId == "skchr_lisa_3")
                        delete (
                            blackboard as unknown as {
                                damage_scale: undefined;
                            }
                        ).damage_scale;
                    break;
                case "tachr_366_acdrop_1":
                    done = true;
                    break;
                case "tachr_416_zumama_1":
                    delete (
                        blackboard as unknown as {
                            hp_ratio: undefined;
                        }
                    ).hp_ratio;
                    break;
                case "tachr_347_jaksel_1":
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = (
                        blackboard as unknown as {
                            ["charge_atk_speed_on_evade.attack_speed"]: number;
                        }
                    )["charge_atk_speed_on_evade.attack_speed"];
                    break;
                case "tachr_452_bstalk_trait":
                case "tachr_476_blkngt_trait":
                    if (options.token) {
                        done = true;
                    }
                    break;
                case "tachr_427_vigil_trait":
                    if (options.token) {
                        done = true;
                    }
                    break;
                case "tachr_402_tuye_1":
                    (
                        blackboard as unknown as {
                            heal_scale: number;
                        }
                    ).heal_scale = (
                        blackboard as unknown as {
                            heal_scale_2: number;
                        }
                    ).heal_scale_2;
                    break;
                case "tachr_457_blitz_1":
                    if (isSkill && skillId == "skchr_blitz_2")
                        (
                            blackboard as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale *= charAttr.buffList.skill.talent_scale ?? 1;
                    break;
                case "tachr_472_pasngr_1":
                    (
                        blackboard as unknown as {
                            damage_scale: number;
                        }
                    ).damage_scale = (
                        blackboard as unknown as {
                            ["pasngr_t_1[enhance].damage_scale"]: number;
                        }
                    )["pasngr_t_1[enhance].damage_scale"];
                    break;
                case "tachr_1012_skadi2_2":
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["skadi2_t_2[atk][2].atk"]: number;
                        }
                    )["skadi2_t_2[atk][2].atk"];
                    break;
                case "tachr_485_pallas_1":
                    if (isSkill && skillId == "skchr_pallas_3" && options.pallas) {
                        done = true;
                    } else {
                        (
                            blackboard as unknown as {
                                atk: number;
                            }
                        ).atk = (
                            blackboard as unknown as {
                                ["peak_performance.atk"]: number;
                            }
                        )["peak_performance.atk"];
                    }
                    break;
                case "skchr_crow_2":
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk += (
                        blackboard as unknown as {
                            ["crow_s_2[atk].atk"]: number;
                        }
                    )["crow_s_2[atk].atk"];
                    (
                        blackboard as unknown as {
                            base_attack_time: number;
                        }
                    ).base_attack_time *= basic.baseAttackTime ?? 0;
                    break;
                case "tachr_431_ashlok_1":
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["ashlok_t_1.atk"]: number;
                        }
                    )["ashlok_t_1.atk"];
                    break;
                case "tachr_4013_kjera_1":
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["kjera_t_1[high].atk"]: number;
                        }
                    )["kjera_t_1[high].atk"];
                    if (options.freeze) {
                        (
                            blackboard as unknown as {
                                magic_resistance: number;
                            }
                        ).magic_resistance = -15;
                    }
                    break;
                case "tachr_322_lmlee_1":
                    if (options.block) {
                        (
                            blackboard as unknown as {
                                attack_speed: number;
                            }
                        ).attack_speed =
                            (
                                blackboard as unknown as {
                                    ["lmlee_t_1[self].attack_speed"]: number;
                                }
                            )["lmlee_t_1[self].attack_speed"] * 2;
                    }
                    break;
                case "skchr_phenxi_3":
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["attack@atk_scale"]: number;
                        }
                    )["attack@atk_scale"];
                    delete (
                        blackboard as unknown as {
                            ["attack@atk_scale"]: undefined;
                        }
                    )["attack@atk_scale"];
                    break;
                case "tachr_4039_horn_1":
                    (
                        blackboard as unknown as {
                            max_hp: number;
                        }
                    ).max_hp *= -1;
                    delete (
                        blackboard as unknown as {
                            hp_ratio: undefined;
                        }
                    ).hp_ratio;
                    break;
                case "tachr_4009_irene_2":
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed *= 2;
                    if ("atk" in blackboard)
                        (
                            blackboard as unknown as {
                                atk: number;
                            }
                        ).atk *= 2;
                    break;
                case "tachr_4064_mlynar_1":
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            atk_scale_up: number;
                        }
                    ).atk_scale_up;
                    break;
                case "tachr_363_toddi_1":
                    if (charAttr.buffList.uniequip_002_toddi && operatorData.equipLevel >= 2) {
                        (
                            blackboard as unknown as {
                                atk_scale: number;
                            }
                        ).atk_scale = (
                            charAttr.buffList.uniequip_002_toddi as {
                                talent: {
                                    atk_scale: number;
                                };
                            }
                        ).talent.atk_scale;
                    }
                    break;
                case "tachr_4062_totter_1":
                    delete (
                        blackboard as unknown as {
                            atk_scale: undefined;
                        }
                    ).atk_scale;
                    break;
            }
        }
    } else if (checkSpecs(tag, "ranged_penalty")) {
        if (!options.ranged_penalty) done = true;
    } else if (checkSpecs(tag, "stack")) {
        if (options.stack) {
            switch (tag) {
                case "tachr_300_phenxi_1":
                    delete (
                        blackboard as unknown as {
                            hp_ratio: undefined;
                        }
                    ).hp_ratio;
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["phenxi_t_1[peak_2].peak_performance.atk"]: number;
                        }
                    )["phenxi_t_1[peak_2].peak_performance.atk"];
                    break;
                case "tachr_2015_dusk_1":
                case "tachr_2023_ling_2":
                    if (options.token) done = true;
                    break;
                case "tachr_188_helage_1":
                case "tachr_337_utage_1":
                case "tachr_475_akafyu_1":
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = (
                        blackboard as unknown as {
                            min_attack_speed: number;
                        }
                    ).min_attack_speed;
                    break;
            }
            if (
                !done &&
                (
                    blackboard as unknown as {
                        max_stack_cnt: number;
                    }
                ).max_stack_cnt
            ) {
                ["atk", "def", "attack_speed", "max_hp"].forEach((key) => {
                    if (blackboard[key as keyof typeof blackboard])
                        (blackboard[key as keyof typeof blackboard] as unknown as number) *= (
                            blackboard as unknown as {
                                max_stack_cnt: number;
                            }
                        ).max_stack_cnt;
                });
            }
        } else done = true;
    } else {
        switch (tag) {
            case "tachr_185_frncat_1":
                buffFrame.times =
                    1 +
                    (
                        blackboard as unknown as {
                            prob: number;
                        }
                    ).prob;
                done = true;
                break;
            case "tachr_118_yuki_1":
                buffFrame.atk =
                    (basic.atk ?? 1) *
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk;
                buffFrame.baseAttackTime = (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time;
                done = true;
                break;
            case "tachr_144_red_1":
                done = true;
                break;
            case "tachr_117_myrrh_1":
            case "tachr_2014_nian_2":
            case "tachr_215_mantic_1":
                done = true;
                break;
            case "tachr_164_nightm_1":
                if (skillId == "skchr_nightm_1") done = true;
                break;
            case "tachr_130_doberm_1":
            case "tachr_308_swire_1":
                done = true;
                break;
            case "tachr_109_fmout_1":
                if (skillId == "skcom_magic_rage[2]") {
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = 0;
                } else if (skillId == "skchr_fmout_2") {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = 0;
                }
                break;
            case "tachr_147_shining_1":
                buffFrame.def += (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = 0;
                break;
            case "tachr_367_swllow_1":
                (
                    blackboard as unknown as {
                        attack_speed: number;
                    }
                ).attack_speed = 0;
                break;
            case "tachr_279_excu_1":
            case "tachr_391_rosmon_1":
            case "skchr_pinecn_1":
                (
                    blackboard as unknown as {
                        edef_pene: number;
                    }
                ).edef_pene = (
                    blackboard as unknown as {
                        def_penetrate_fixed: number;
                    }
                ).def_penetrate_fixed;
                break;
            case "tachr_373_lionhd_1":
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk *= Math.min(
                    enemy.count,
                    (
                        blackboard as unknown as {
                            max_valid_stack_cnt: number;
                        }
                    ).max_valid_stack_cnt,
                );
                break;
            case "tachr_290_vigna_1":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = isSkill
                    ? (
                          blackboard as unknown as {
                              prob2: number;
                          }
                      ).prob2
                    : (
                          blackboard as unknown as {
                              prob1: number;
                          }
                      ).prob1;
                if (
                    buffFrame.prob &&
                    buffFrame.prob >
                        (
                            blackboard as unknown as {
                                prob_override: number;
                            }
                        ).prob_override
                )
                    delete (
                        blackboard as unknown as {
                            prob_override: undefined;
                        }
                    ).prob_override;
                break;
            case "tachr_106_franka_1":
                (
                    blackboard as unknown as {
                        edef_pene_scale: number;
                    }
                ).edef_pene_scale = 1;
                if (isSkill && skillId == "skchr_franka_2")
                    (
                        blackboard as unknown as {
                            prob_override: number;
                        }
                    ).prob_override =
                        (
                            blackboard as unknown as {
                                prob: number;
                            }
                        ).prob *
                        (
                            charAttr.buffList.skill as {
                                talent_scale: number;
                            }
                        ).talent_scale;
                break;
            case "tachr_4009_irene_1":
                (
                    blackboard as unknown as {
                        edef_pene_scale: number;
                    }
                ).edef_pene_scale = (
                    blackboard as unknown as {
                        def_penetrate: number;
                    }
                ).def_penetrate;
                break;
            case "tachr_155_tiger_1":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = (
                    blackboard as unknown as {
                        ["tiger_t_1[evade].prob"]: number;
                    }
                )["tiger_t_1[evade].prob"];
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = (
                    blackboard as unknown as {
                        ["charge_on_evade.atk"]: number;
                    }
                )["charge_on_evade.atk"];
                break;
            case "tachr_340_shwaz_1":
                if (isSkill)
                    (
                        blackboard as unknown as {
                            prob_override: number;
                        }
                    ).prob_override = charAttr.buffList.skill["talent@prob"] ?? 0;
                (
                    blackboard as unknown as {
                        edef_scale: number;
                    }
                ).edef_scale = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                delete (
                    blackboard as unknown as {
                        def: undefined;
                    }
                ).def;
                break;
            case "tachr_225_haak_1":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = 0.25;
                break;
            case "tachr_2013_cerber_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_401_elysm_1":
                delete (
                    blackboard as unknown as {
                        attack_speed: undefined;
                    }
                ).attack_speed;
                break;
            case "tachr_345_folnic_1":
                delete (
                    blackboard as unknown as {
                        damage_scale: undefined;
                    }
                ).damage_scale;
                break;
            case "tachr_344_beewax_trait":
            case "tachr_388_mint_trait":
            case "tachr_388_mint_1":
                if (isSkill) done = true;
                break;
            case "tachr_426_billro_2":
                done = true;
                break;
            case "tachr_426_billro_trait":
                if (isSkill && !(skillId == "skchr_billro_1" && options.charge)) {
                    done = true;
                }
                break;
            case "tachr_411_tomimi_1":
                if (!isSkill) done = true;
                break;
            case "tachr_509_acast_1":
            case "tachr_350_surtr_1":
            case "tachr_377_gdglow_2":
                (
                    blackboard as unknown as {
                        emr_pene: number;
                    }
                ).emr_pene = (
                    blackboard as unknown as {
                        magic_resist_penetrate_fixed: number;
                    }
                ).magic_resist_penetrate_fixed;
                break;
            case "skchr_swllow_1":
            case "skchr_helage_1":
            case "skchr_helage_2":
            case "skchr_akafyu_1":
            case "skchr_excu_2":
            case "skchr_bpipe_2":
            case "skchr_acdrop_2":
            case "skchr_spikes_1":
                buffFrame.times = 2;
                break;
            case "skchr_excu_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_texas_2":
            case "skchr_flamtl_2":
                buffFrame.times = 2;
                buffFrame.maxTarget = 999;
                break;
            case "skchr_swllow_2":
            case "skchr_bpipe_3":
                buffFrame.times = 3;
                break;
            case "skchr_milu_2":
                buffFrame.times = Math.min(
                    enemy.count,
                    (
                        blackboard as unknown as {
                            max_cnt: number;
                        }
                    ).max_cnt,
                );
                buffFrame.maxTarget = 999;
                break;
            case "skchr_cqbw_3":
                buffFrame.times = Math.min(
                    enemy.count,
                    (
                        blackboard as unknown as {
                            max_target: number;
                        }
                    ).max_target,
                );
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = 999;
                break;
            case "skchr_iris_2":
                buffFrame.times = Math.min(
                    enemy.count,
                    (
                        blackboard as unknown as {
                            max_target: number;
                        }
                    ).max_target,
                );
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = 999;
                break;
            case "skchr_lava2_1": // sp炎熔1
                delete (
                    blackboard as unknown as {
                        ["attack@max_target"]: undefined;
                    }
                )["attack@max_target"];
                buffFrame.times = Math.min(2, enemy.count);
                break;
            case "skchr_lava2_2":
                buffFrame.times = 2;
                break;
            case "skchr_slbell_1": // 不结算的技能
            case "skchr_shining_2":
            case "skchr_cgbird_2":
                done = true;
                break;
            // 多段暖机
            case "skchr_amgoat_1":
                if (options.warmup) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["amgoat_s_1[b].atk"]: number;
                        }
                    )["amgoat_s_1[b].atk"];
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = (
                        blackboard as unknown as {
                            ["amgoat_s_1[b].attack_speed"]: number;
                        }
                    )["amgoat_s_1[b].attack_speed"];
                } else {
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = (
                        blackboard as unknown as {
                            ["amgoat_s_1[a].attack_speed"]: number;
                        }
                    )["amgoat_s_1[a].attack_speed"];
                }
                break;
            case "skchr_thorns_3":
                if (options.warmup) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["thorns_s_3[b].atk"]: number;
                        }
                    )["thorns_s_3[b].atk"];
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = (
                        blackboard as unknown as {
                            ["thorns_s_3[b].attack_speed"]: number;
                        }
                    )["thorns_s_3[b].attack_speed"];
                }
                if (options.ranged_penalty) {
                    buffFrame.atk_scale = 1;
                }
                break;
            case "skchr_pinecn_2":
                if (options.warmup) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["pinecn_s_2[d].atk"]: number;
                        }
                    )["pinecn_s_2[d].atk"];
                } else {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["pinecn_s_2[a].atk"]: number;
                        }
                    )["pinecn_s_2[a].atk"];
                }
                break;
            case "skchr_amgoat_2":
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        fk: number;
                    }
                ).fk;
                break;
            case "skchr_breeze_2":
                buffFrame.maxTarget = 1;
                break;
            case "skchr_snsant_2":
            case "skchr_demkni_2":
            case "skchr_demkni_3":
            case "skchr_hsguma_3":
            case "skchr_waaifu_2":
            case "skchr_sqrrel_2":
            case "skchr_panda_2":
            case "skchr_red_2":
            case "skchr_phatom_3":
            case "skchr_weedy_3":
            case "skchr_asbest_2":
            case "skchr_folnic_2":
            case "skchr_chiave_2":
            case "skchr_mudrok_2":
            case "skchr_siege_2":
            case "skchr_glady_3":
            case "skchr_gnosis_2":
            case "skchr_ebnhlz_2":
            case "skchr_doroth_2":
            case "skchr_doroth_3":
                buffFrame.maxTarget = 999;
                break;
            case "skchr_durnar_2":
                buffFrame.maxTarget = 3;
                break;
            case "skchr_aprot_1":
            case "skchr_aprot2_1":
                buffFrame.maxTarget = 3;
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time *= basic.baseAttackTime ?? 1;
                break;
            case "skchr_saga_2":
                buffFrame.maxTarget = 6;
                break;
            case "skchr_huang_3":
                buffFrame.maxTarget = 999;
                buffFrame.atk_table = [...Array(8).keys()].map(
                    (x) =>
                        ((
                            blackboard as unknown as {
                                atk: number;
                            }
                        ).atk /
                            8) *
                        (x + 1),
                );
                break;
            case "skchr_phatom_2":
                buffFrame.atk_table = [
                    ...Array(
                        (
                            blackboard as unknown as {
                                times: number;
                            }
                        ).times,
                    ).keys(),
                ]
                    .reverse()
                    .map(
                        (x) =>
                            (
                                blackboard as unknown as {
                                    atk: number;
                                }
                            ).atk *
                            (x + 1),
                    );
                delete (
                    blackboard as unknown as {
                        times: undefined;
                    }
                ).times;
                break;
            case "skchr_bluep_2":
                buffFrame.maxTarget = 3;
                delete (
                    blackboard as unknown as {
                        ["attack@times"]: undefined;
                    }
                )["attack@times"];
                break;
            case "skchr_bluep_1":
            case "skchr_breeze_1":
            case "skchr_grani_2":
            case "skchr_astesi_2":
            case "skchr_hpsts_2":
            case "skchr_myrrh_1":
            case "skchr_myrrh_2":
            case "skchr_whispr_1":
            case "skchr_ling_2":
                buffFrame.maxTarget = 2;
                break;
            case "skchr_folivo_1":
            case "skchr_folivo_2":
            case "skchr_deepcl_1":
                if (!options.token) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = 0;
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def = 0;
                }
                break;
            case "skchr_otter_2":
                if (options.token) {
                    done = true;
                }
                break;
            case "skchr_kalts_2":
                if (options.token) {
                    delete (
                        blackboard as unknown as {
                            attack_speed: undefined;
                        }
                    ).attack_speed;
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["attack@atk"]: number;
                        }
                    )["attack@atk"];
                    buffFrame.maxTarget = 3;
                }
                break;
            case "skchr_kalts_3":
                if (options.token) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["attack@atk"]: number;
                        }
                    )["attack@atk"];
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def = (
                        blackboard as unknown as {
                            ["attack@def"]: number;
                        }
                    )["attack@def"];
                }
                break;
            case "skchr_skadi2_3":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
            case "skchr_sora_2":
            case "skchr_skadi2_2":
            case "skchr_heidi_1":
            case "skchr_heidi_2":
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = 0; // 不增加本体攻击
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = 0;
                (
                    blackboard as unknown as {
                        max_hp: number;
                    }
                ).max_hp = 0;
                break;
            case "skchr_swire_1":
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = 0; // 1技能不加攻击
                break;
            case "skchr_ccheal_2": // hot记为额外治疗，不在这里计算
            case "skchr_ccheal_1":
                delete (
                    blackboard as unknown as {
                        heal_scale: undefined;
                    }
                ).heal_scale;
                break;
            case "skchr_hmau_2":
            case "skchr_spot_1":
            case "tachr_193_frostl_1":
            case "skchr_mantic_2":
            case "skchr_glaze_2":
            case "skchr_zumama_2":
            case "skchr_shwaz_3":
            case "fusion_buff":
            case "skchr_windft_2":
            case "skchr_mlynar_2":
            case "skchr_judge_3":
                buffFrame.baseAttackTime += (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time;
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time = 0;
                break;
            case "skchr_brownb_2":
            case "skchr_whispr_2":
            case "skchr_indigo_1":
            case "skchr_pasngr_2":
            case "skchr_ashlok_2":
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time *= basic.baseAttackTime ?? 1;
                break;
            case "skchr_mudrok_3":
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time *= basic.baseAttackTime ?? 1;
                Object.assign(buffFrame, {
                    maxTarget: basic.blockCnt,
                });
                break;
            case "skchr_rosmon_3":
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time *= basic.baseAttackTime ?? 1;
                if (options.cond) {
                    (
                        blackboard as unknown as {
                            edef: number;
                        }
                    ).edef = -160;
                }
                if (options.rosmon_double) {
                    (
                        blackboard as unknown as {
                            times: number;
                        }
                    ).times = 2;
                }
                break;
            case "skchr_aglina_2": // 攻击间隔缩短，但是是乘算正数
            case "skchr_cerber_2":
            case "skchr_finlpp_2":
            case "skchr_jaksel_2":
            case "skchr_iris_1":
            case "skchr_indigo_2":
            case "skchr_ebnhlz_1":
            case "skchr_hamoni_1":
            case "skchr_hamoni_2":
            case "skchr_mberry_2":
            case "skchr_flamtl_3":
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time =
                    ((
                        blackboard as unknown as {
                            base_attack_time: number;
                        }
                    ).base_attack_time -
                        1) *
                    (basic.baseAttackTime ?? 1);
                break;
            case "skchr_angel_3":
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time *= 2;
                break;
            case "skchr_whitew_2":
            case "skchr_spikes_2":
                buffFrame.maxTarget = 2;
                if (options.ranged_penalty) {
                    buffFrame.atk_scale /= 0.8;
                }
                break;
            case "skchr_ayer_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
            case "skchr_ayer_1":
            case "skchr_svrash_2":
            case "skchr_svrash_1":
            case "skchr_frostl_1":
                if (options.ranged_penalty) {
                    buffFrame.atk_scale = 1;
                }
                break;
            case "skchr_svrash_3":
                if (options.ranged_penalty) {
                    buffFrame.atk_scale = 1;
                }
                (
                    blackboard as unknown as {
                        def_scale: number;
                    }
                ).def_scale =
                    1 +
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def;
                delete (
                    blackboard as unknown as {
                        def: undefined;
                    }
                ).def;
                break;
            case "skchr_ceylon_1":
                if (options.ranged_penalty) {
                    buffFrame.atk_scale /= 0.7;
                }
                break;
            case "skchr_nightm_1":
                delete (
                    blackboard as unknown as {
                        ["attack@max_target"]: undefined;
                    }
                )["attack@max_target"];
                break;
            case "skchr_shotst_1":
            case "skchr_shotst_2":
                (
                    blackboard as unknown as {
                        edef_scale: number;
                    }
                ).edef_scale = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = 0;
                break;
            case "skchr_meteo_2":
                (
                    blackboard as unknown as {
                        edef: number;
                    }
                ).edef = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = 0;
                break;
            case "skchr_slbell_2":
                (
                    blackboard as unknown as {
                        edef_scale: number;
                    }
                ).edef_scale = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = 0;
                break;
            case "skchr_ifrit_2":
                (
                    blackboard as unknown as {
                        edef: number;
                    }
                ).edef = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = 0;
                break;
            case "skchr_nian_3":
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = (
                    blackboard as unknown as {
                        ["nian_s_3[self].atk"]: number;
                    }
                )["nian_s_3[self].atk"];
                break;
            case "skchr_nian_2":
            case "skchr_hsguma_2":
                break;
            case "skchr_yuki_2":
                (
                    blackboard as unknown as {
                        ["attack@atk_scale"]: number;
                    }
                )["attack@atk_scale"] *= 3;
                break;
            case "skchr_waaifu_1":
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = (
                    blackboard as unknown as {
                        ["waaifu_s_1[self].atk"]: number;
                    }
                )["waaifu_s_1[self].atk"];
                break;
            case "skchr_peacok_1":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = (
                    blackboard as unknown as {
                        ["peacok_s_1[crit].prob"]: number;
                    }
                )["peacok_s_1[crit].prob"];
                if (isCrit)
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            atk_scale_fake: number;
                        }
                    ).atk_scale_fake;
                break;
            case "skchr_peacok_2":
                if (isCrit) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["success.atk_scale"]: number;
                        }
                    )["success.atk_scale"];
                    buffFrame.maxTarget = 999;
                }
                break;
            case "skchr_vodfox_1":
                buffFrame.damage_scale =
                    1 +
                    (buffFrame.damage_scale - 1) *
                        (
                            blackboard as unknown as {
                                scale_delta_to_one: number;
                            }
                        ).scale_delta_to_one;
                break;
            case "skchr_elysm_2":
                delete (
                    blackboard as unknown as {
                        def: undefined;
                    }
                ).def;
                delete (
                    blackboard as unknown as {
                        max_target: undefined;
                    }
                ).max_target;
                break;
            case "skchr_asbest_1":
                delete (
                    blackboard as unknown as {
                        damage_scale: undefined;
                    }
                ).damage_scale;
                break;
            case "skchr_beewax_2":
            case "skchr_mint_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_tomimi_2":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override =
                    (
                        blackboard as unknown as {
                            ["attack@tomimi_s_2.prob"]: number;
                        }
                    )["attack@tomimi_s_2.prob"] / 3;
                delete (
                    blackboard as unknown as {
                        base_attack_time: undefined;
                    }
                ).base_attack_time;
                if (isCrit) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["attack@tomimi_s_2.atk_scale"]: number;
                        }
                    )["attack@tomimi_s_2.atk_scale"];
                }
                break;
            case "skchr_surtr_2":
                if (enemy.count == 1) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["attack@surtr_s_2[critical].atk_scale"]: number;
                        }
                    )["attack@surtr_s_2[critical].atk_scale"];
                }
                break;
            case "skchr_surtr_3":
                delete (
                    blackboard as unknown as {
                        hp_ratio: undefined;
                    }
                ).hp_ratio;
                break;
            case "tachr_381_bubble_1":
                delete (
                    blackboard as unknown as {
                        atk: undefined;
                    }
                ).atk;
                break;
            case "tachr_265_sophia_1":
                if (isSkill) {
                    const ts = charAttr.buffList.skill.talent_scale;
                    if (skillId == "skchr_sophia_1") {
                        (
                            blackboard as unknown as {
                                def: number;
                            }
                        ).def =
                            (
                                blackboard as unknown as {
                                    ["sophia_t_1_less.def"]: number;
                                }
                            )["sophia_t_1_less.def"] * (ts ?? 1);
                        (
                            blackboard as unknown as {
                                attack_speed: number;
                            }
                        ).attack_speed =
                            (
                                blackboard as unknown as {
                                    ["sophia_t_1_less.attack_speed"]: number;
                                }
                            )["sophia_t_1_less.attack_speed"] * (ts ?? 1);
                    } else if (skillId == "skchr_sophia_2") {
                        (
                            blackboard as unknown as {
                                def: number;
                            }
                        ).def *= ts ?? 1;
                        (
                            blackboard as unknown as {
                                attack_speed: number;
                            }
                        ).attack_speed *= ts ?? 1;
                        (
                            blackboard as unknown as {
                                max_target: number;
                            }
                        ).max_target = basic.blockCnt ?? 1;
                    }
                } else {
                    delete (
                        blackboard as unknown as {
                            def: undefined;
                        }
                    ).def;
                    delete (
                        blackboard as unknown as {
                            attack_speed: undefined;
                        }
                    ).attack_speed;
                }
                break;
            case "tachr_346_aosta_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_blemsh_1":
                delete (
                    blackboard as unknown as {
                        heal_scale: undefined;
                    }
                ).heal_scale;
                break;
            case "skchr_rosmon_2":
                delete (
                    blackboard as unknown as {
                        ["attack@times"]: undefined;
                    }
                )["attack@times"];
                break;
            case "tachr_1001_amiya2_1":
                if (isSkill) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk *= (
                        charAttr.buffList.skill as {
                            talent_scale: number;
                        }
                    ).talent_scale;
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def *= (
                        charAttr.buffList.skill as {
                            talent_scale: number;
                        }
                    ).talent_scale;
                }
                break;
            case "skchr_amiya2_2":
                delete (
                    blackboard as unknown as {
                        times: undefined;
                    }
                ).times;
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                if (options.stack) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk =
                        (
                            blackboard as unknown as {
                                ["amiya2_s_2[kill].atk"]: number;
                            }
                        )["amiya2_s_2[kill].atk"] *
                        (
                            blackboard as unknown as {
                                ["amiya2_s_2[kill].max_stack_cnt"]: number;
                            }
                        )["amiya2_s_2[kill].max_stack_cnt"];
                }
                break;
            case "tachr_214_kafka_1":
                if (isSkill) applyBuffDefault();
                done = true;
                break;
            case "skchr_kafka_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_f12yin_2":
                (
                    blackboard as unknown as {
                        def_scale: number;
                    }
                ).def_scale =
                    1 +
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def;
                buffFrame.maxTarget = 2;
                delete (
                    blackboard as unknown as {
                        def: undefined;
                    }
                ).def;
                break;
            case "skchr_f12yin_3":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = (
                    blackboard as unknown as {
                        ["talent@prob"]: number;
                    }
                )["talent@prob"];
                break;
            case "tachr_264_f12yin_1":
                delete (
                    blackboard as unknown as {
                        atk: undefined;
                    }
                ).atk;
                break;
            case "tachr_264_f12yin_2":
                delete (
                    blackboard as unknown as {
                        prob: undefined;
                    }
                ).prob;
                break;
            case "skchr_archet_1":
                delete (
                    blackboard as unknown as {
                        max_target: undefined;
                    }
                ).max_target;
                break;
            case "tachr_338_iris_trait":
            case "tachr_469_indigo_trait":
            case "tachr_338_iris_1":
            case "tachr_362_saga_2":
            case "tachr_4046_ebnhlz_trait":
            case "tachr_4046_ebnhlz_1":
            case "tachr_297_hamoni_trait":
                done = true;
                break;
            case "tachr_4046_ebnhlz_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                if ("attack_speed" in blackboard) {
                    if (options.equip && !(skillId == "skchr_ebnhlz_3" && isSkill)) {
                        // Do nothing
                    } else {
                        done = true;
                    }
                } else done = true;
                break;
            case "skchr_tuye_1":
            case "skchr_tuye_2":
                delete (
                    blackboard as unknown as {
                        heal_scale: undefined;
                    }
                ).heal_scale;
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_saga_3":
                buffFrame.maxTarget = 2;
                if (options.cond) {
                    buffFrame.times = 2;
                }
                break;
            case "skchr_dusk_1":
            case "skchr_dusk_3":
                if (options.token) done = true;
                break;
            case "skchr_dusk_2":
                if (options.token) done = true;
                else {
                    if (!options.cond) {
                        delete (
                            blackboard as unknown as {
                                damage_scale: undefined;
                            }
                        ).damage_scale;
                    }
                }
                break;
            case "skchr_weedy_2":
                if (options.token)
                    delete (
                        blackboard as unknown as {
                            base_attack_time: undefined;
                        }
                    ).base_attack_time;
                else buffFrame.maxTarget = 999;
                break;
            case "tachr_455_nothin_1":
                done = true;
                break;
            case "skchr_nothin_2":
                delete (
                    blackboard as unknown as {
                        prob: undefined;
                    }
                ).prob;
                if (!options.cond) {
                    delete (
                        blackboard as unknown as {
                            attack_speed: undefined;
                        }
                    ).attack_speed;
                }
                break;
            case "skchr_ash_2":
                if (options.cond)
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["ash_s_2[atk_scale].atk_scale"]: number;
                        }
                    )["ash_s_2[atk_scale].atk_scale"];
                break;
            case "skchr_ash_3":
                buffFrame.maxTarget = 999;
                break;
            case "skchr_blitz_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_tachak_1":
                (
                    blackboard as unknown as {
                        edef_pene: number;
                    }
                ).edef_pene = (
                    blackboard as unknown as {
                        def_penetrate_fixed: number;
                    }
                ).def_penetrate_fixed;
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_tachak_2":
                (
                    blackboard as unknown as {
                        base_attack_time: number;
                    }
                ).base_attack_time *= basic.baseAttackTime ?? 1;
                if (!isCrit)
                    delete (
                        blackboard as unknown as {
                            atk_scale: undefined;
                        }
                    ).atk_scale;
                break;
            case "skchr_pasngr_1":
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = (
                    blackboard as unknown as {
                        ["pasngr_s_1.max_target"]: number;
                    }
                )["pasngr_s_1.max_target"];
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        ["pasngr_s_1.atk_scale"]: number;
                    }
                )["pasngr_s_1.atk_scale"];
                break;
            case "skchr_pasngr_3":
                done = true;
                break;
            case "skchr_toddi_1":
                (
                    blackboard as unknown as {
                        edef_scale: number;
                    }
                ).edef_scale = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                delete (
                    blackboard as unknown as {
                        def: undefined;
                    }
                ).def;
                break;
            case "skchr_tiger_1":
            case "skchr_bena_1":
                (
                    blackboard as unknown as {
                        edef_pene_scale: number;
                    }
                ).edef_pene_scale = (
                    blackboard as unknown as {
                        def_penetrate: number;
                    }
                ).def_penetrate;
                if (options.annie) {
                    done = true;
                }
                break;
            case "skchr_bena_2":
            case "skchr_kazema_1":
                if (options.annie) {
                    done = true;
                }
                break;
            case "skchr_ghost2_1":
            case "skchr_ghost2_2":
                if (options.annie) {
                    buffFrame.maxTarget = 999;
                    done = true;
                }
                break;
            case "skchr_ghost2_3":
                if (options.annie) {
                    buffFrame.maxTarget = 999;
                    done = true;
                } else {
                    buffFrame.baseAttackTime += (
                        blackboard as unknown as {
                            base_attack_time: number;
                        }
                    ).base_attack_time;
                    (
                        blackboard as unknown as {
                            base_attack_time: number;
                        }
                    ).base_attack_time = 0;
                    buffFrame.maxTarget = 2;
                }
                break;
            case "skchr_kazema_2":
                if (options.annie) {
                    done = true;
                }
                break;
            case "skchr_billro_2":
                if (!options.charge) {
                    delete (
                        blackboard as unknown as {
                            atk: undefined;
                        }
                    ).atk;
                }
                break;
            case "tachr_485_pallas_trait":
            case "tachr_308_swire_trait":
            case "tachr_265_sophia_trait":
                if (!options.noblock) done = true;
                break;
            case "uniequip_002_pallas":
            case "uniequip_002_sophia":
            case "uniequip_002_swire":
                if (!options.noblock) done = true;
                break;
            case "tachr_130_doberm_trait":
                if (!options.noblock) done = true;
                break;
            case "skchr_pallas_3":
                if (options.pallas) {
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def = (
                        blackboard as unknown as {
                            ["attack@def"]: number;
                        }
                    )["attack@def"];
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk += (
                        blackboard as unknown as {
                            ["attack@peak_performance.atk"]: number;
                        }
                    )["attack@peak_performance.atk"];
                }
                break;
            case "tachr_486_takila_1":
                done = true;
                break;
            case "tachr_486_takila_trait":
                if (!options.charge) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = 1;
                }
                break;
            case "skchr_takila_2":
                if (options.charge)
                    buffFrame.maxTarget = (
                        blackboard as unknown as {
                            ["attack@plus_max_target"]: number;
                        }
                    )["attack@plus_max_target"];
                else buffFrame.maxTarget = 2;
                break;
            case "skchr_chen2_2":
            case "skchr_chen2_3":
                (
                    blackboard as unknown as {
                        edef: number;
                    }
                ).edef = (
                    blackboard as unknown as {
                        ["attack@def"]: number;
                    }
                )["attack@def"];
            case "skchr_chen2_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_1013_chen2_1":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = (
                    blackboard as unknown as {
                        ["spareshot_chen.prob"]: number;
                    }
                )["spareshot_chen.prob"];
                break;
            case "tachr_1013_chen2_2":
                (
                    blackboard as unknown as {
                        attack_speed: number;
                    }
                ).attack_speed = (
                    blackboard as unknown as {
                        ["chen2_t_2[common].attack_speed"]: number;
                    }
                )["chen2_t_2[common].attack_speed"];
                if (options.water)
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed += (
                        blackboard as unknown as {
                            ["chen2_t_2[map].attack_speed"]: number;
                        }
                    )["chen2_t_2[map].attack_speed"];
                break;
            case "tachr_479_sleach_1":
                (
                    blackboard as unknown as {
                        attack_speed: number;
                    }
                ).attack_speed = (
                    blackboard as unknown as {
                        ["sleach_t_1[ally].attack_speed"]: number;
                    }
                )["sleach_t_1[ally].attack_speed"];
                break;
            case "skchr_fartth_3":
                if (!options.far)
                    delete (
                        blackboard as unknown as {
                            damage_scale: undefined;
                        }
                    ).damage_scale;
                break;
            case "tachr_1014_nearl2_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_1014_nearl2_2":
                (
                    blackboard as unknown as {
                        edef_pene_scale: number;
                    }
                ).edef_pene_scale = (
                    blackboard as unknown as {
                        def_penetrate: number;
                    }
                ).def_penetrate;
                break;
            case "skchr_nearl2_2":
                delete (
                    blackboard as unknown as {
                        times: undefined;
                    }
                ).times;
                break;
            case "tachr_489_serum_1":
                done = true;
                break;
            case "skchr_glider_1":
                buffFrame.maxTarget = 2;
                break;
            case "skchr_aurora_2":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = 0.1;
                if (!isCrit)
                    delete (
                        blackboard as unknown as {
                            atk_scale: undefined;
                        }
                    ).atk_scale;
                break;
            case "tachr_206_gnosis_1":
                if (options.freeze || (skillId == "skchr_gnosis_2" && isSkill && options.charge)) {
                    (
                        blackboard as unknown as {
                            damage_scale: number;
                        }
                    ).damage_scale = (
                        blackboard as unknown as {
                            damage_scale_freeze: number;
                        }
                    ).damage_scale_freeze;
                    (
                        blackboard as unknown as {
                            magic_resistance: number;
                        }
                    ).magic_resistance = -15;
                } else
                    (
                        blackboard as unknown as {
                            damage_scale: number;
                        }
                    ).damage_scale = (
                        blackboard as unknown as {
                            damage_scale_cold: number;
                        }
                    ).damage_scale_cold;
                break;
            case "skchr_gnosis_3":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "skchr_blkngt_1":
                if (options.token) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["blkngt_hypnos_s_1[rage].atk"]: number;
                        }
                    )["blkngt_hypnos_s_1[rage].atk"];
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = Math.round(
                        (
                            blackboard as unknown as {
                                ["blkngt_hypnos_s_1[rage].attack_speed"]: number;
                            }
                        )["blkngt_hypnos_s_1[rage].attack_speed"] * 100,
                    );
                }
                break;
            case "skchr_blkngt_2":
                if (options.token) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["blkngt_s_2.atk_scale"]: number;
                        }
                    )["blkngt_s_2.atk_scale"];
                    buffFrame.maxTarget = 999;
                }
                break;
            case "skchr_ling_3":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_377_gdglow_1":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = 0.1;
                break;
            case "tachr_4016_kazema_1":
                done = true;
                break;
            case "tachr_300_phenxi_2":
                if (isSkill)
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed =
                        (
                            blackboard as unknown as {
                                ["phenxi_e_t_2[in_skill].attack_speed"]: number;
                            }
                        )["phenxi_e_t_2[in_skill].attack_speed"] || 0;
                break;
            case "skchr_chnut_2":
                (
                    blackboard as unknown as {
                        heal_scale: number;
                    }
                ).heal_scale = (
                    blackboard as unknown as {
                        ["attack@heal_continuously_scale"]: number;
                    }
                )["attack@heal_continuously_scale"];
                break;
            case "tachr_4045_heidi_1":
                if (skillId == "skchr_heidi_1")
                    delete (
                        blackboard as unknown as {
                            def: undefined;
                        }
                    ).def;
                if (skillId == "skchr_heidi_2")
                    delete (
                        blackboard as unknown as {
                            atk: undefined;
                        }
                    ).atk;
                break;
            case "skchr_horn_1":
                if (!options.melee) buffFrame.maxTarget = 999;
                break;
            case "skchr_horn_2":
                buffFrame.maxTarget = 999;
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        ["attack@s2.atk_scale"]: number;
                    }
                )["attack@s2.atk_scale"];
                break;
            case "skchr_horn_3":
                if (!options.melee) buffFrame.maxTarget = 999;
                if (options.overdrive_mode)
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk = (
                        blackboard as unknown as {
                            ["horn_s_3[overload_start].atk"]: number;
                        }
                    )["horn_s_3[overload_start].atk"];
                break;
            case "skchr_rockr_2":
                if (!options.overdrive_mode)
                    delete (
                        blackboard as unknown as {
                            atk: undefined;
                        }
                    ).atk;
                break;
            case "tachr_108_silent_1":
                if (options.token) done = true;
                break;
            case "skchr_silent_2":
                if (options.token) buffFrame.maxTarget = 999;
                break;
            case "skchr_windft_1":
                buffFrame.maxTarget = 999;
                break;
            case "tachr_433_windft_1":
                if (options.stack) {
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk *= 2;
                    if (skillId == "skchr_windft_2" && isSkill)
                        (
                            blackboard as unknown as {
                                atk: number;
                            }
                        ).atk *= (
                            charAttr.buffList.skill as {
                                talent_scale: number;
                            }
                        ).talent_scale;
                } else {
                    done = true;
                }
                break;
            case "tachr_4042_lumen_1":
            case "tachr_4042_lumen_2":
                done = true;
                break;
            case "skchr_lumen_3":
                delete (
                    blackboard as unknown as {
                        heal_scale: undefined;
                    }
                ).heal_scale;
                break;
            case "tachr_1023_ghost2_1":
                if (!options.annie) done = true;
                break;
            case "skchr_irene_1":
            case "skchr_irene_2":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = 1;
                break;
            case "skchr_irene_3":
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = 1;
                buffFrame.maxTarget = 999;
                break;
            case "tachr_4043_erato_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                if (options.cond)
                    (
                        blackboard as unknown as {
                            edef_pene_scale: number;
                        }
                    ).edef_pene_scale = (
                        blackboard as unknown as {
                            def_penetrate: number;
                        }
                    ).def_penetrate;
                else done = true;
                break;
            case "skchr_pianst_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk *= (
                    blackboard as unknown as {
                        max_stack_cnt: number;
                    }
                ).max_stack_cnt;
                break;
            case "tachr_4047_pianst_1":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_258_podego_1":
                if ("sp_recovery_per_sec" in blackboard) delete blackboard.sp_recovery_per_sec;
                break;
            case "tachr_195_glassb_1":
                if (isSkill && "glassb_e_t_1[skill].attack_speed" in blackboard)
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed += (
                        blackboard as unknown as {
                            ["glassb_e_t_1[skill].attack_speed"]: number;
                        }
                    )["glassb_e_t_1[skill].attack_speed"];
                break;
            case "tachr_135_halo_trait":
            case "tachr_4071_peper_trait":
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = (
                    blackboard as unknown as {
                        ["attack@chain.max_target"]: number;
                    }
                )["attack@chain.max_target"];
                break;
            case "skchr_halo_1":
                (
                    blackboard as unknown as {
                        times: number;
                    }
                ).times = Math.min(
                    (
                        blackboard as unknown as {
                            max_target: number;
                        }
                    ).max_target,
                    enemy.count,
                );
                delete (
                    blackboard as unknown as {
                        max_target: undefined;
                    }
                ).max_target;
                break;
            case "skchr_halo_2":
                (
                    blackboard as unknown as {
                        times: number;
                    }
                ).times = Math.min(
                    (
                        blackboard as unknown as {
                            ["attack@max_target"]: number;
                        }
                    )["attack@max_target"],
                    enemy.count,
                );
                delete (
                    blackboard as unknown as {
                        ["attack@max_target"]: undefined;
                    }
                )["attack@max_target"];
                break;
            case "skchr_greyy2_2":
                done = true;
                break;
            case "skchr_doroth_1":
                (
                    blackboard as unknown as {
                        edef_scale: number;
                    }
                ).edef_scale = (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def;
                delete (
                    blackboard as unknown as {
                        def: undefined;
                    }
                ).def;
                break;
            case "tachr_129_bluep_1":
                done = true;
                break;
            case "tachr_1026_gvial2_1":
                if (options.block) {
                    // 确认阻挡数
                    const gvial2_blk = skillId == "skchr_gvial2_3" && isSkill ? 5 : 3;
                    const ecount = Math.min(enemy.count, gvial2_blk);
                    const atk_add =
                        (
                            blackboard as unknown as {
                                atk_add: number;
                            }
                        ).atk_add * ecount;
                    (
                        blackboard as unknown as {
                            atk: number;
                        }
                    ).atk += atk_add;
                    (
                        blackboard as unknown as {
                            def: number;
                        }
                    ).def += atk_add;
                }
                break;
            case "skchr_gvial2_3":
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = 5;
                break;
            case "tachr_4055_bgsnow_2":
                let bgsnow_t2_value = -0.18;
                if (operatorData.potentialRank >= 4) bgsnow_t2_value = -0.2;
                if (options.cond_near) bgsnow_t2_value -= 0.05;
                if (options.token || options.cond_def)
                    (
                        blackboard as unknown as {
                            edef_scale: number;
                        }
                    ).edef_scale = bgsnow_t2_value;
                break;
            case "skchr_bgsnow_1":
                if (!isCrit)
                    delete (
                        blackboard as unknown as {
                            atk_scale: undefined;
                        }
                    ).atk_scale;
                break;
            case "skchr_bgsnow_3":
                if (options.cond_front || options.token) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["bgsnow_s_3[atk_up].atk_scale"]: number;
                        }
                    )["bgsnow_s_3[atk_up].atk_scale"];
                }
                break;
            case "tachr_497_ctable_1":
                if (options.noblock) {
                    delete (
                        blackboard as unknown as {
                            atk: undefined;
                        }
                    ).atk;
                } else {
                    delete (
                        blackboard as unknown as {
                            attack_speed: undefined;
                        }
                    ).attack_speed;
                }
                break;
            case "tachr_472_pasngr_2":
                if (!options.cond_2) done = true;
                break;
            case "skchr_provs_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_4032_provs_1":
                if (!options.equip)
                    delete (
                        blackboard as unknown as {
                            sp_recovery_per_sec: undefined;
                        }
                    ).sp_recovery_per_sec;
                break;
            case "tachr_4064_mlynar_2":
                done = true;
                break;
            case "tachr_4064_mlynar_trait":
                let atk_rate = options.stack ? 1 : 0.5;
                if (isSkill && skillId == "skchr_mlynar_3") atk_rate *= charAttr.buffList.skill.trait_up ?? 1;
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk *= atk_rate;
                break;
            case "skchr_mlynar_3":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_136_hsguma_1":
                if ("atk" in blackboard) {
                    if (!options.equip) {
                        delete blackboard.atk;
                    }
                }
                break;
            case "tachr_325_bison_1":
                Object.assign(charAttr.basic.def ?? {}, {
                    def:
                        (charAttr.basic.def ?? 0) +
                        (
                            blackboard as unknown as {
                                def: number;
                            }
                        ).def,
                });
                done = true;
                break;
            case "skchr_lolxh_1":
                buffFrame.maxTarget = 2;
                if (options.ranged_penalty) {
                    buffFrame.atk_scale = 1;
                }
                break;
            case "skchr_lolxh_2":
                buffFrame.maxTarget = 2;
                if (options.ranged_penalty) {
                    buffFrame.atk_scale = 1;
                }
                break;
            case "skchr_qanik_2":
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trigger_atk_scale: number;
                    }
                ).trigger_atk_scale;
                break;
            case "skchr_totter_2":
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = (
                    blackboard as unknown as {
                        ["attack@s2n.max_target"]: number;
                    }
                )["attack@s2n.max_target"];
                if (enemy.count == 1) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["attack@s2c.atk_scale"]: number;
                        }
                    )["attack@s2c.atk_scale"];
                }
                break;
            case "tachr_157_dagda_1":
                if (options.stack) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            atk_up_max_value: number;
                        }
                    ).atk_up_max_value;
                }
                if (isSkill && skillId == "skchr_dagda_2")
                    (
                        blackboard as unknown as {
                            prob_override: number;
                        }
                    ).prob_override = charAttr.buffList.skill["talent@prob"] ?? 0;
                break;
            case "skchr_quartz_2":
                delete (
                    blackboard as unknown as {
                        damage_scale: undefined;
                    }
                ).damage_scale;
                if (options.crit) {
                    (
                        blackboard as unknown as {
                            prob_override: number;
                        }
                    ).prob_override = (
                        blackboard as unknown as {
                            ["attack@s2_buff_prob"]: number;
                        }
                    )["attack@s2_buff_prob"];
                }
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        ["attack@s2_atk_scale"]: number;
                    }
                )["attack@s2_atk_scale"];
                break;
            case "skchr_peper_2":
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = 4;
                break;
            case "tachr_4014_lunacu_1":
                if (isSkill) {
                    (
                        blackboard as unknown as {
                            base_attack_time: number;
                        }
                    ).base_attack_time *= basic.baseAttackTime ?? 1;
                }
                break;
            case "tachr_4065_judge_2":
                done = true;
                break;
            case "skchr_judge_1":
                if (options.charge) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["judge_s_1_enhance_checker.atk_scale"]: number;
                        }
                    )["judge_s_1_enhance_checker.atk_scale"];
                }
                break;
            case "skchr_judge_2":
                (
                    blackboard as unknown as {
                        max_target: number;
                    }
                ).max_target = 999;
                break;
            case "tachr_1028_texas2_1":
                if (!isSkill) done = true;
                break;
            case "skchr_texas2_2":
                delete (
                    blackboard as unknown as {
                        atk_scale: undefined;
                    }
                ).atk_scale;
                break;
            case "tachr_427_vigil_1":
                if (options.stack && options.token) {
                    (
                        blackboard as unknown as {
                            times: number;
                        }
                    ).times = 3;
                } else
                    (
                        blackboard as unknown as {
                            times: number;
                        }
                    ).times = 1;
                break;
            case "tachr_427_vigil_2":
                if (options.cond)
                    (
                        blackboard as unknown as {
                            edef_pene: number;
                        }
                    ).edef_pene = (
                        blackboard as unknown as {
                            def_penetrate_fixed: number;
                        }
                    ).def_penetrate_fixed;
                break;
            case "skchr_vigil_2":
                if (options.token) {
                    (
                        blackboard as unknown as {
                            atk_scale: number;
                        }
                    ).atk_scale = (
                        blackboard as unknown as {
                            ["vigil_wolf_s_2.atk_scale"]: number;
                        }
                    )["vigil_wolf_s_2.atk_scale"];
                    (
                        blackboard as unknown as {
                            hp_ratio: number;
                        }
                    ).hp_ratio = (
                        blackboard as unknown as {
                            ["vigil_wolf_s_2.hp_ratio"]: number;
                        }
                    )["vigil_wolf_s_2.hp_ratio"];
                }
                break;
            case "skchr_vigil_3":
                if (!options.token)
                    (
                        blackboard as unknown as {
                            times: number;
                        }
                    ).times = 3;
                break;
            case "skchr_ironmn_1":
                if (options.token) done = true;
                break;
            case "skchr_ironmn_2":
                if (options.token) done = true;
                else {
                    buffFrame.maxTarget = 2;
                }
                break;
            case "skchr_ironmn_3":
                if (options.token) done = true;
                break;
            case "sktok_ironmn_pile3":
                delete (
                    blackboard as unknown as {
                        hp_ratio: undefined;
                    }
                ).hp_ratio;
                break;
            case "tachr_420_flamtl_1+":
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale =
                    (
                        blackboard as unknown as {
                            ["attack@atkscale_t1+.atk_scale"]: number;
                        }
                    )["attack@atkscale_t1+.atk_scale"] || 1;
                break;
            case "skchr_texas2_3":
                done = true;
                break;
            case "tachr_1020_reed2_1":
                delete (
                    blackboard as unknown as {
                        atk: undefined;
                    }
                ).atk;
                break;
            case "skchr_reed2_2":
                done = true;
                break;
            case "skchr_reed2_3":
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = (
                    blackboard as unknown as {
                        ["reed2_skil_3[switch_mode].atk"]: number;
                    }
                )["reed2_skil_3[switch_mode].atk"];
                break;
            case "tachr_4017_puzzle_1":
                done = true;
                break;
        }
    }

    if (tag == "skchr_thorns_2") {
        (
            blackboard as unknown as {
                base_attack_time: number;
            }
        ).base_attack_time =
            (
                blackboard as unknown as {
                    cooldown: number;
                }
            ).cooldown -
            ((basic.baseAttackTime ?? 0) + buffFrame.baseAttackTime);
        buffFrame.attackSpeed = 0;
        (
            blackboard as unknown as {
                attack_speed: number;
            }
        ).attack_speed = 0;
    }
    if (["char_416_zumama", "char_422_aurora"].includes(charId) && !options.block && buffFrame.spRecoverRatio == 0) {
        buffFrame.spRecoverRatio = -1;
    }
    switch (tag) {
        case "uniequip_002_cuttle":
        case "uniequip_002_glaze":
        case "uniequip_002_fartth":
            if (options.equip) {
                blackboard = (
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait;
                if (
                    (
                        blackboard as unknown as {
                            damage_scale: number;
                        }
                    ).damage_scale < 1
                )
                    (
                        blackboard as unknown as {
                            damage_scale: number;
                        }
                    ).damage_scale += 1;
            } else
                (
                    blackboard as unknown as {
                        damage_scale: number;
                    }
                ).damage_scale = 1;
            break;
        case "uniequip_002_sddrag":
            if (options.equip) {
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trait: {
                            atk_scale: number;
                        };
                    }
                ).trait.atk_scale;
                if (options.cond_spd) {
                    (
                        blackboard as unknown as {
                            attack_speed: number;
                        }
                    ).attack_speed = (
                        blackboard as unknown as {
                            talent: {
                                attack_speed: number;
                            };
                        }
                    ).talent.attack_speed;
                }
            }
            break;
        case "uniequip_002_vigna":
            if (options.equip)
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trait: {
                            atk_scale: number;
                        };
                    }
                ).trait.atk_scale;
            if (
                "prob1" in
                (
                    blackboard as unknown as {
                        talent: [unknown, unknown];
                    }
                ).talent
            )
                (
                    blackboard as unknown as {
                        prob_override: number;
                    }
                ).prob_override = isSkill
                    ? (
                          blackboard as unknown as {
                              talent: {
                                  prob2: number;
                              };
                          }
                      ).talent.prob2
                    : (
                          blackboard as unknown as {
                              talent: {
                                  prob1: number;
                              };
                          }
                      ).talent.prob1;
            break;
        case "uniequip_002_chen":
        case "uniequip_002_tachak":
        case "uniequip_002_bibeak":
            blackboard = (
                blackboard as unknown as {
                    trait: any;
                }
            ).trait;
            if (!isSkill)
                delete (
                    blackboard as unknown as {
                        damage_scale: undefined;
                    }
                ).damage_scale;
            break;
        case "uniequip_002_cutter":
        case "uniequip_002_phenxi":
        case "uniequip_002_meteo":
        case "uniequip_002_irene":
            blackboard = (
                blackboard as unknown as {
                    trait: any;
                }
            ).trait;
            (
                blackboard as unknown as {
                    edef_pene: number;
                }
            ).edef_pene = (
                blackboard as unknown as {
                    def_penetrate_fixed: number;
                }
            ).def_penetrate_fixed;
            break;
        case "uniequip_002_yuki":
            const bb_yuki = {
                ...(
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait,
            };
            Object.assign(bb_yuki, {
                edef_pene: (
                    bb_yuki as {
                        def_penetrate_fixed: number;
                    }
                ).def_penetrate_fixed,
            });

            if (
                (
                    blackboard as unknown as {
                        talent: {
                            sp_recovery_per_sec: number;
                        };
                    }
                ).talent.sp_recovery_per_sec
            )
                (
                    bb_yuki as {
                        sp_recovery_per_sec: number;
                    }
                ).sp_recovery_per_sec = (
                    blackboard as unknown as {
                        talent: {
                            sp_recovery_per_sec: number;
                        };
                    }
                ).talent.sp_recovery_per_sec;
            blackboard = bb_yuki;
            break;
        case "uniequip_002_nearl2":
        case "uniequip_002_franka":
        case "uniequip_002_peacok":
        case "uniequip_002_cqbw":
        case "uniequip_002_sesa":
        case "uniequip_003_skadi":
            if (options.equip)
                blackboard = (
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait;
            break;
        case "uniequip_002_skadi":
        case "uniequip_002_flameb":
        case "uniequip_002_gyuki":
            if (options.equip)
                (
                    blackboard as unknown as {
                        attack_speed: number;
                    }
                ).attack_speed = (
                    blackboard as unknown as {
                        trait: {
                            attack_speed: number;
                        };
                    }
                ).trait.attack_speed;
            break;
        case "uniequip_002_lisa":
            if (
                "atk" in
                (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent
            )
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = (
                    blackboard as unknown as {
                        talent: {
                            atk: number;
                        };
                    }
                ).talent.atk;
        case "uniequip_002_podego":
        case "uniequip_002_glacus":
            if (options.equip)
                (
                    blackboard as unknown as {
                        sp_recovery_per_sec: number;
                    }
                ).sp_recovery_per_sec = 0.2;
            break;
        case "uniequip_003_aglina":
            if (options.equip)
                blackboard = (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent;
            break;
        case "uniequip_002_lumen":
        case "uniequip_002_ceylon":
        case "uniequip_002_whispr":
            done = true;
            break;
        case "uniequip_002_finlpp":
            if (isSkill)
                blackboard = (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent;
            break;
        case "uniequip_002_ghost2":
        case "uniequip_002_kazema":
        case "uniequip_002_bena":
            blackboard = (
                blackboard as unknown as {
                    trait: any;
                }
            ).trait;
            if (!options.annie || options.token) done = true;
            break;
        case "uniequip_002_zumama":
        case "uniequip_002_aurora":
            if (!options.block) {
                buffFrame.spRecoverRatio = (
                    blackboard as unknown as {
                        trait: {
                            sp_recover_ratio: number;
                        };
                    }
                ).trait.sp_recover_ratio;
            }
            break;
        case "uniequip_002_doberm":
            if (options.equip) {
                blackboard = (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent;
            }
            break;
        case "uniequip_002_plosis":
            if (
                options.equip &&
                "sp_recovery_per_sec" in
                    (
                        blackboard as unknown as {
                            talent: any;
                        }
                    ).talent
            )
                (
                    blackboard as unknown as {
                        sp_recovery_per_sec: number;
                    }
                ).sp_recovery_per_sec =
                    (
                        blackboard as unknown as {
                            talent: {
                                sp_recovery_per_sec: number;
                            };
                        }
                    ).talent.sp_recovery_per_sec - 0.3;
            break;
        case "uniequip_002_red":
        case "uniequip_002_kafka":
        case "uniequip_002_texas2":
            if (options.equip) {
                blackboard = (
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait;
            }
            break;
        case "uniequip_002_waaifu":
            if (options.equip) {
                blackboard = (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent;
            }
            break;
        case "uniequip_002_pasngr":
            if (options.cond_2)
                blackboard = (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent;
            break;
        case "uniequip_002_nearl":
        case "uniequip_002_sunbr":
        case "uniequip_002_demkni":
            if (options.equip || skillId == "skchr_demkni_1")
                (
                    blackboard as unknown as {
                        heal_scale: number;
                    }
                ).heal_scale = (
                    blackboard as unknown as {
                        trait: {
                            heal_scale: number;
                        };
                    }
                ).trait.heal_scale;
            break;
        case "uniequip_002_ash":
        case "uniequip_002_archet":
        case "uniequip_002_aprl":
        case "uniequip_002_swllow":
        case "uniequip_002_bluep":
        case "uniequip_002_jesica":
            if (options.equip)
                (
                    blackboard as unknown as {
                        attack_speed: number;
                    }
                ).attack_speed = 8; // 写死。避免同名词条问题
            break;
        case "uniequip_002_angel":
        case "uniequip_002_kroos2":
        case "uniequip_002_platnm":
        case "uniequip_002_mm":
        case "uniequip_002_clour":
        case "uniequip_003_archet":
            if (options.equip)
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trait: {
                            atk_scale: number;
                        };
                    }
                ).trait.atk_scale;
            break;
        case "uniequip_002_shotst":
            if (options.cond)
                // 流星直接用cond判断
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trait: {
                            atk_scale: number;
                        };
                    }
                ).trait.atk_scale;
            break;
        case "uniequip_002_bgsnow":
            if (options.cond_front && !options.token) {
                // 模组效果对token不生效
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trait: {
                            atk_scale: number;
                        };
                    }
                ).trait.atk_scale;
            }
            break;
        case "uniequip_003_shwaz":
            if (options.equip || (skillId == "skchr_shwaz_3" && isSkill)) {
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = (
                    blackboard as unknown as {
                        trait: {
                            atk_scale: number;
                        };
                    }
                ).trait.atk_scale;
            }
            break;
        case "uniequip_003_zumama":
            if (options.block) {
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk = (
                    blackboard as unknown as {
                        trait: {
                            atk: number;
                        };
                    }
                ).trait.atk;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = (
                    blackboard as unknown as {
                        trait: {
                            def: number;
                        };
                    }
                ).trait.def;
            }
            break;
        case "uniequip_002_nian":
            (
                blackboard as unknown as {
                    def: number;
                }
            ).def = options.block
                ? (
                      blackboard as unknown as {
                          trait: {
                              def: number;
                          };
                      }
                  ).trait.def
                : 0;
            if (
                (
                    blackboard as unknown as {
                        talent: {
                            atk: number;
                        };
                    }
                ).talent.atk
            ) {
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk =
                    (
                        blackboard as unknown as {
                            talent: {
                                atk: number;
                            };
                        }
                    ).talent.atk *
                    (
                        blackboard as unknown as {
                            talent: {
                                max_stack_cnt: number;
                            };
                        }
                    ).talent.max_stack_cnt;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def +=
                    (
                        blackboard as unknown as {
                            talent: {
                                def: number;
                            };
                        }
                    ).talent.def *
                    (
                        blackboard as unknown as {
                            talent: {
                                max_stack_cnt: number;
                            };
                        }
                    ).talent.max_stack_cnt;
            }
            break;
        case "uniequip_002_bison":
        case "uniequip_002_bubble":
        case "uniequip_002_snakek":
            if (options.block)
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = (
                    blackboard as unknown as {
                        trait: {
                            def: number;
                        };
                    }
                ).trait.def;
            break;
        case "uniequip_003_hsguma":
            if (options.equip)
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def = (
                    blackboard as unknown as {
                        trait: {
                            def: number;
                        };
                    }
                ).trait.def;
            break;
        case "uniequip_002_shining":
        case "uniequip_002_folnic":
            if (options.equip) {
                blackboard = (
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait;
            }
            break;
        case "uniequip_002_silent":
            if (options.equip && !options.token) {
                blackboard = (
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait;
            }
            break;
        case "uniequip_002_kalts":
        case "uniequip_002_tuye":
        case "uniequip_002_bldsk":
        case "uniequip_002_susuro":
        case "uniequip_002_myrrh":
            if (options.equip) {
                (
                    blackboard as unknown as {
                        heal_scale: number;
                    }
                ).heal_scale = (
                    blackboard as unknown as {
                        trait: {
                            heal_scale: number;
                        };
                    }
                ).trait.heal_scale;
            }
            break;
        case "uniequip_002_siege":
            const equip_atk = options.equip
                ? (
                      blackboard as unknown as {
                          trait: {
                              atk: number;
                          };
                      }
                  ).trait.atk
                : 0;
            const talent_atk =
                (
                    blackboard as unknown as {
                        talent: {
                            atk: number;
                        };
                    }
                ).talent.atk || 0;
            (
                blackboard as unknown as {
                    atk: number;
                }
            ).atk = (
                blackboard as unknown as {
                    def: number;
                }
            ).def = equip_atk + talent_atk;
            break;
        case "uniequip_002_blackd":
        case "uniequip_002_scave":
        case "uniequip_002_headbr":
            if (options.equip)
                blackboard = (
                    blackboard as unknown as {
                        trait: any;
                    }
                ).trait;
            break;
        case "uniequip_002_texas":
            if (isSkill)
                blackboard = (
                    blackboard as unknown as {
                        talent: any;
                    }
                ).talent;
            break;
        case "uniequip_002_toddi":
        case "uniequip_002_erato":
        case "uniequip_002_totter":
            if (options.equip) {
                (
                    blackboard as unknown as {
                        atk_scale: number;
                    }
                ).atk_scale = 1.15;
            }
            break;
        case "uniequip_002_utage":
            if (options.equip) {
                (
                    blackboard as unknown as {
                        atk: number;
                    }
                ).atk =
                    (
                        blackboard as unknown as {
                            talent: {
                                atk: number;
                            };
                        }
                    ).talent.atk || 0;
                (
                    blackboard as unknown as {
                        def: number;
                    }
                ).def =
                    (
                        blackboard as unknown as {
                            talent: {
                                def: number;
                            };
                        }
                    ).talent.def || 0;
            }
            break;
    }

    if (!done) applyBuffDefault();
    return buffFrame;
}
