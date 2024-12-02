import type { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import type { Skill } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/skills";
import { checkResetAttack } from "./check-reset-attack";
import { checkSpecs } from "./check-specs";
import dpsAnim from "./dps-animation.json";

export function calcDurations(isSkill: boolean, attackTime: number, attackSpeed: number, levelData: Skill["levels"][0], buffList: any, buffFrame: { [key: string]: any }, enemyCount: number, options: any, char: Operator) {
    const blackboard = buffList.skill;
    const skillId = blackboard.id;
    const spData = levelData.spData;
    let duration = 0;
    let attackCount = 0;
    let stunDuration = 0;
    let prepDuration = 0;
    let startSp = 0;
    const rst = checkResetAttack(skillId, blackboard, options);
    const subProf = char.subProfessionId;

    console.write("\n**【循环计算】**");

    const spTypeTags = {
        1: "time",
        2: "attack",
        4: "hit",
        8: "special",
    };

    const tags = [spTypeTags[spData.spType as unknown as keyof typeof spTypeTags]]; // 技能类型标记

    // 需要模拟的技能（自动回复+自动释放+有充能）
    if (checkSpecs(skillId, "sim")) {
        console.log("模拟120s时间轴");
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
        const TimelineMarks = {
            attack: "-",
            skill: "+",
            ifrit: "",
            archet: "",
            chen: "",
            recover_sp: "\\*",
            recover_overflow: "x",
            reset_animation: "\\*",
            cancel_attack: "!",
        };
        // 技能动画(阻回)时间-帧
        const cast_time = checkSpecs(skillId, "cast_time") || (Number(checkSpecs(skillId, "cast_bat")) * 100) / attackSpeed || attackTime * fps;
        const skill_time = Math.max(Number(cast_time), attackTime * fps);

        function time_since(key: string) {
            return now - (last[key as keyof typeof last] || -999);
        }

        function action(key: string) {
            if (!timeline[now as keyof typeof timeline]) (timeline[now as keyof typeof timeline] as []) = [];
            (timeline[now as keyof typeof timeline] as string[]).push(key);
            (last[key as keyof typeof last] as number) = now;
            (total[key as keyof typeof total] as number) = (total[key as keyof typeof total] || 0) + 1;
        }

        // charge
        const cast_sp = options.charge && checkSpecs(skillId, "charge") ? spData.spCost * 2 : spData.spCost;
        // init sp
        if (skillId == "skchr_amgoat_2" && buffList["tachr_180_amgoat_2"]) sp = ((buffList["tachr_180_amgoat_2"].sp_min + buffList["tachr_180_amgoat_2"].sp_max) / 2) * fps;
        else if (buffList["tachr_222_bpipe_2"]) sp = buffList["tachr_222_bpipe_2"].sp * fps;
        else if (buffList["uniequip_002_archet"] && buffList["uniequip_002_archet"].talent["archet_e_t_2.sp"]) sp = buffList["uniequip_002_archet"].talent["archet_e_t_2.sp"] * fps;

        (last["ifrit" as keyof typeof last] as any) = (last["archet" as keyof typeof last] as any) = (last["chen" as keyof typeof last] as any) = 1; // 落地即开始计算 记为1帧
        startSp = cast_sp - sp / fps;

        // sp barrier
        max_sp = cast_sp * fps;
        if (blackboard.ct) max_sp = spData.spCost * fps * blackboard.ct;
        if (blackboard.cnt) max_sp = spData.spCost * fps * blackboard.cnt;

        console.write(`[模拟] T = 120s, 初始sp = ${(sp / fps).toFixed(1)}, 技能sp = ${cast_sp}, 技能动画时间 = ${Math.round(Number(cast_time))} 帧, sp上限设为 ${max_sp / fps}`);
        console.write(`[模拟] 攻击间隔 ${attackTime.toFixed(3)}s`);
        console.log(`技能动画 ${cast_time} 帧`);
        let attackAnim: number | boolean = checkSpecs(skillId, "attack_animation");
        if (attackAnim) {
            // 缩放至攻击间隔
            attackAnim = Math.min(Math.round(attackTime * fps), Number(attackAnim));
            console.write(`[模拟] 攻击动画 = ${attackAnim} 帧`);
        }

        if (Number(spData.spType) == 1) {
            sp += fps; // 落地时恢复1sp
            console.write("[模拟] +1落地sp");
        }
        while (now <= duration * fps) {
            // normal attack
            if (sp < cast_sp * fps && time_since("attack") >= attackTime * fps && time_since("skill") >= skill_time) {
                action("attack");
                if (Number(spData.spType) === 2) sp += fps;
            }
            // 技能已经cd好
            if (sp >= cast_sp * fps && time_since("skill") >= skill_time) {
                // 正常：普通攻击间隔结束后进入技能
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
            // 乱火
            if (buffList["tachr_134_ifrit_2"] && time_since("ifrit") >= buffList["tachr_134_ifrit_2"].interval * fps) {
                action("ifrit");
                extra_sp = buffList["tachr_134_ifrit_2"].sp;
            }
            // 兰登战术/呵斥
            const intv_archet = buffList["tachr_332_archet_1"] ? buffList["tachr_332_archet_1"].interval : 2.5;
            const intv_chen = buffList["tachr_010_chen_1"] ? buffList["tachr_010_chen_1"].interval : 4;
            if ((buffList["tachr_332_archet_1"] || options.archet) && time_since("archet") >= intv_archet * fps) {
                action("archet");
                extra_sp += 1;
            }
            if ((buffList["tachr_010_chen_1"] || options.chen) && time_since("chen") >= intv_chen * fps) {
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

            // 打印时间轴和特殊动作
            let line_str = "";
            console.log(timeline);
            Object.keys(timeline).forEach((t) => {
                line_str += (timeline[t as keyof typeof timeline] as (keyof typeof TimelineMarks)[]).map((x: keyof typeof TimelineMarks) => TimelineMarks[x]).join("");
            });
            console.write(`[模拟] 时间轴: `);
            console.write(`${line_str}`);
            console.write(`( \-: 普攻, \+: 技能, \\*: 取消后摇, x: sp溢出, !: 取消普攻 )`);

            if ((total as { ifrit: string }).ifrit) console.write(`[模拟] 莱茵回路(\\*): 触发 ${(total as { recover_sp: string }).recover_sp} / ${(total as { ifrit: string }).ifrit} 次, sp + ${buffList["tachr_134_ifrit_2"].sp * (total as { recover_sp: number }).recover_sp}`);
            if ((total as { archet: string }).archet) console.write(`[模拟] 兰登战术: 触发 ${(total as { archet: string }).archet} 次`);
            if ((total as { chen: string }).chen) console.write(`[模拟] 呵斥: 触发 ${(total as { chen: string }).chen} 次`);
            if ((total as { recover_sp: number }).recover_sp) console.write(`[模拟] sp恢复成功 ${(total as { recover_sp: number }).recover_sp} 次, 溢出 ${(total as { recover_overflow: number }).recover_overflow || 0} 次, 其余为技能期间无法恢复sp`);
            if ((total as { reset_animation: number }).reset_animation) console.write(`[模拟] 取消攻击间隔(\\*) ${(total as { reset_animation: number }).reset_animation} 次`);
        }
    } else {
        if (isSkill) {
            // 准备时间
            switch (skillId) {
                case "skchr_mudrok_3":
                    prepDuration = blackboard.sleep;
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
                    console.log("落地1s，不影响技能时间");
                    break;
                case "skchr_texas2_2":
                    console.log("落地1s，不影响技能时间");
                    prepDuration = 0.167;
                    break;
            }

            // 快速估算
            attackCount = Math.ceil((levelData.duration - prepDuration) / attackTime);
            duration = attackCount * attackTime;
            startSp = spData.spCost - spData.initSp;

            if (buffList["tachr_180_amgoat_2"]) {
                // 乱火
                const init_sp = spData.initSp + (buffList["tachr_180_amgoat_2"].sp_min + buffList["tachr_180_amgoat_2"].sp_max) / 2;
                startSp = spData.spCost - init_sp;
            } else if (buffList["tachr_222_bpipe_2"]) {
                // 军事传统
                startSp = spData.spCost - spData.initSp - buffList["tachr_222_bpipe_2"].sp - (buffList["tachr_222_bpipe_2"]["bpipe_e_2[locate].sp"] || 0);
            } else if (buffList["tachr_456_ash_2"]) {
                startSp = spData.spCost - spData.initSp - buffList["tachr_456_ash_2"].sp;
            } else if (buffList["uniequip_002_archet"] && "archet_e_t_2.sp" in buffList["uniequip_002_archet"].talent) {
                startSp = spData.spCost - spData.initSp - buffList["uniequip_002_archet"].talent["archet_e_t_2.sp"];
            }

            console.write(`技能启动需要SP: ${startSp.toFixed(1)}`);
            // 重置普攻
            if (rst) {
                if (duration > levelData.duration - prepDuration && String(rst) !== "ogcd") {
                    if (options.overdrive_mode) console.write("[结束时重置普攻] 截断最后一个攻击间隔");
                    else console.write(`[重置普攻] 截断最后一个攻击间隔`);
                }
                duration = levelData.duration - prepDuration;
                // 抬手时间
                let frameBegin = Math.round(Number(checkSpecs(skillId, "attack_begin")) || 12);
                if (skillId == "skchr_glaze_2" && options.far) {
                    console.log("技能前摇增加至27帧");
                    frameBegin = 27;
                }
                const t = frameBegin / 30;
                attackCount = Math.ceil((duration - t) / attackTime);
                console.write(`技能前摇: ${t.toFixed(3)}s, ${frameBegin} 帧`);
                if (!checkSpecs(skillId, "attack_begin")) console.write("（计算器默认值；请参考动画时间）");
                else console.log(`技能前摇: ${frameBegin} 帧`);
            }
            // 技能类型
            if (levelData.description.includes("持续时间无限") || checkSpecs(skillId, "toggle")) {
                if (skillId == "skchr_thorns_3" && !options.warmup) {
                } else if (skillId == "skchr_tuye_2") {
                    console.log("取技能时间=暖机时间");
                    duration = spData.spCost / (1 + buffFrame.spRecoveryPerSec);
                    attackCount = Math.ceil(duration / attackTime);
                } else if (skillId == "skchr_surtr_3") {
                    const lock_time = buffList["tachr_350_surtr_2"]["surtr_t_2[withdraw].interval"];
                    duration = Math.sqrt(600) + lock_time;
                    attackCount = Math.ceil(duration / attackTime);
                    console.write(`损失100%血量耗时: ${Math.sqrt(600).toFixed(1)}s，锁血时间: ${lock_time}s`);
                    console.log("不治疗最大维持时间");
                } else {
                    const d = options.short_mode ? 180 : 1000;
                    attackCount = Math.ceil(d / attackTime);
                    duration = attackCount * attackTime;
                    if (checkSpecs(skillId, "toggle")) {
                        console.log(`切换类技能 (以${d}s计算)`);
                        tags.push("toggle", "infinity");
                    } else {
                        console.log(`永续技能 (以${d}s计算)`);
                        tags.push("infinity");
                    }
                }
            } else if (Number(spData.spType) === 8) {
                if (levelData.duration <= 0 && blackboard.duration > 0) {
                    // 砾的技能也是落地点火，但是持续时间在blackboard里
                    levelData.duration = blackboard.duration;
                    duration = blackboard.duration;
                    attackCount = Math.ceil(levelData.duration / attackTime);
                }
                if (levelData.duration > 0) {
                    // 自动点火
                    tags.push("auto");
                    console.write("落地点火");
                    if (prepDuration > 0) duration = levelData.duration - prepDuration;
                } else if (checkSpecs(skillId, "passive")) {
                    // 被动
                    attackCount = 1;
                    duration = attackTime;
                    tags.push("passive");
                    console.write("被动");
                } else if (skillId == "skchr_phatom_2") {
                    // 傀影2
                    attackCount = blackboard.times;
                    duration = attackTime * attackCount;
                } else {
                    // 摔炮
                    attackCount = 1;
                    duration = 0;
                    tags.push("auto", "instant");
                    console.write("落地点火, 瞬发");
                }
            } else if (levelData.duration <= 0) {
                if (checkSpecs(skillId, "instant_buff")) {
                    // 瞬发的有持续时间的buff，例如血浆
                    duration = blackboard.duration || checkSpecs(skillId, "duration");
                    attackCount = Math.ceil(duration / attackTime);
                    tags.push("instant", "buff");
                    console.write("瞬发Buff，技能周期为Buff持续时间");
                } else if (checkSpecs(skillId, "magazine")) {
                    // 弹药技能
                    let mag: number | boolean = checkSpecs(skillId, "magazine");
                    if (options.charge && skillId == "skchr_chen2_2") mag = 20;
                    else if (skillId == "skchr_ctable_2") mag = blackboard["attack@trigger_time"];
                    if (buffList["tachr_1013_chen2_1"]) {
                        const prob = buffList["tachr_1013_chen2_1"]["spareshot_chen.prob"];
                        const new_mag = Math.floor(Number(mag) / (1 - prob));
                        console.write(`计入 ${new_mag - Number(mag)} 发额外弹药`);
                        mag = new_mag;
                    }
                    attackCount = Number(mag);
                    duration = attackTime * attackCount;
                    if (rst) duration -= attackTime;
                } else if (skillId == "skchr_blkngt_2" && options.token) {
                    duration = blackboard["blkngt_s_2.duration"];
                    attackCount = Math.ceil(duration / attackTime);
                } else {
                    // 普通瞬发
                    attackCount = 1;
                    // 不占用普攻的瞬发技能，持续时间等于动画时间。否则持续时间为一次普攻间隔
                    if (String(checkSpecs(skillId, "reset_attack")) !== "ogcd") duration = attackTime;
                    tags.push("instant");
                    console.write("瞬发");
                    // 施法时间-基于动画
                    if (checkSpecs(skillId, "anim_key") && checkSpecs(skillId, "anim_cast")) {
                        const animKey = checkSpecs(skillId, "anim_key");
                        const dpsAnimKey = char.id as keyof typeof dpsAnim;
                        const dpsAnimKeyKey = String(animKey) as keyof typeof dpsAnim;
                        const animData = (dpsAnim[dpsAnimKey] as any)[dpsAnimKeyKey];
                        const ct = animData.duration || animData;

                        console.write(`技能动画：${animKey}, 释放时间 ${ct} 帧`);
                        console.log(`技能动画: ${ct} 帧`);
                        if ((duration < ct / 30 && Number(spData.spType) === 1) || String(rst) === "ogcd") duration = ct / 30;
                    }
                    // 施法时间
                    if (checkSpecs(skillId, "cast_time")) {
                        const ct = checkSpecs(skillId, "cast_time");
                        if (duration < Number(ct) / 30 || String(rst) === "ogcd") {
                            console.write(`技能动画: ${ct} 帧(基于动画数据)`);
                            console.log(`技能动画: ${ct} 帧`);
                            if (Number(spData.spType) === 1 || Number(spData.spType) === 2 || String(rst) === "ogcd") duration = Number(ct) / 30;
                        }
                    }
                }
            } else if (skillId == "skchr_glady_3") {
                attackCount = 6;
                attackTime = 1.5;
                console.log("[特殊] 持续9秒，第7次拖拽无伤害");
            } else if (options.annie) {
                duration = 20;
                attackCount = Math.ceil(duration / attackTime);
                console.write("傀儡师替身 - 持续20s");
            }

            // 过载
            if (checkSpecs(skillId, "overdrive")) {
                // 重新估算前半时间
                let attackCountHalf = Math.ceil((levelData.duration - prepDuration) / 2 / attackTime);
                let durationHalf = attackCountHalf * attackTime;
                if (checkSpecs(skillId, "magazine")) {
                    attackCountHalf = Math.ceil(attackCount / 2);
                    durationHalf = attackCountHalf * attackTime;
                    console.write(`一半弹药攻击 ${attackCountHalf} 次`);
                }
                if (options.overdrive_mode) {
                    // 过载: 减去前半部分
                    duration -= durationHalf;
                    attackCount -= attackCountHalf;
                    if (options.od_trigger) {
                        // 立即结束
                        console.log("立即结束过载");
                        duration = attackCount = 0;
                        if (skillId == "skchr_horn_2") {
                            duration = 1.066; // 32f
                            attackCount = attackCountHalf;
                        }
                    }
                } else {
                    // 前半
                    duration = durationHalf;
                    attackCount = attackCountHalf;
                }
            }

            // 特判
            if (skillId == "skchr_huang_3") {
                attackCount -= 2;
            } else if (skillId == "skchr_sunbr_2") {
                // 古米2准备时间延长技能时间
                prepDuration = blackboard.disarm;
            } else if (skillId == "skchr_takila_2" && options.charge) {
                duration = blackboard.enhance_duration;
                attackCount = Math.ceil(duration / attackTime);
            } else if (char.id === "char_4055_bgsnow" && options.token) {
                // 不管精英化等级 统一按25秒计算
                if (duration > 25) {
                    duration = 25;
                    attackCount = Math.ceil(duration / attackTime);
                    console.log("[打字机]按持续25秒计算");
                }
            } else if (skillId == "skchr_ironmn_3" && options.token) {
                attackCount = isSkill ? 10 : 0;
                duration = 15;
                console.log("以攻击10次计算");
            }
        } else {
            // 普攻
            // 眩晕处理
            if (skillId == "skchr_fmout_2") {
                stunDuration = blackboard.time;
            } else if (skillId == "skchr_peacok_2") {
                stunDuration = blackboard["failure.stun"] * (1 - blackboard.prob);
                console.write(`[特殊] 计算平均晕眩时间`);
            } else if (["skchr_amiya_2", "skchr_liskam_2", "skchr_ghost_2", "skchr_broca_2", "skchr_serum_1", "skchr_aurora_1"].includes(skillId)) {
                stunDuration = blackboard.stun;
            } else if (skillId == "skchr_folivo_2" && options.token) {
                stunDuration = blackboard.stun;
            } else if (skillId == "skchr_rockr_2" && !options.od_trigger) {
                stunDuration = 20;
            }
            if (stunDuration > 0) console.write(`晕眩: ${stunDuration}s`);

            // 快速估算
            let spRatio = 1;
            if (buffFrame.spRecoverRatio != 0) {
                spRatio += buffFrame.spRecoverRatio;
                console.write(`技力回复 ${((1 + buffFrame.spRecoveryPerSec) * spRatio).toFixed(2)}/s`);
            }
            let attackDuration = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio) - stunDuration;
            if (spRatio == 0) {
                attackDuration = 180;
                console.log("以180s计算普攻DPS");
            }

            // 施法时间
            if (checkSpecs(skillId, "cast_time")) {
                const ct = checkSpecs(skillId, "cast_time");
                if (attackTime > Number(ct) / 30 && String(rst) !== "ogcd") {
                    attackDuration -= attackTime - Number(ct) / 30;
                    console.write(`[特殊] 技能释放时间: ${ct} 帧, 普攻时间偏移 ${(Number(ct) / 30 - attackTime).toFixed(3)}s (${attackDuration.toFixed(3)}s)`);
                    console.log(`技能动画(阻回): ${ct} 帧`);
                }
            }
            attackCount = Math.ceil(attackDuration / attackTime);
            duration = attackCount * attackTime;
            // 重置普攻（瞬发/ogcd除外）
            if (rst && String(rst) !== "ogcd" && Number(spData.spType) !== 8 && spRatio != 0) {
                const dd = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio) - stunDuration;
                if (duration > dd) console.write(`[重置普攻] 截断最后一个攻击间隔`);
                duration = dd;
                // 抬手时间
                const frameBegin = Math.round(Number(checkSpecs(skillId, "attack_begin")) || 12);
                const t = frameBegin / 30;
                attackCount = Math.ceil((duration - t) / attackTime);
                console.write(`技能前摇: ${t.toFixed(3)}s, ${frameBegin} 帧`);
                if (!checkSpecs(skillId, "attack_begin")) console.write("（计算器默认值；请参考动画时间）");
            }

            // 技能类型
            switch (Number(spData.spType)) {
                case 8: // 被动或落地点火
                    if (levelData.duration <= 0 && blackboard.duration > 0) {
                        console.log("Duration? l/b", skillId, levelData.duration, blackboard.duration);
                        levelData.duration = blackboard.duration;
                    }
                    if (levelData.duration > 0) {
                        tags.push("auto");
                        if (skillId == "skchr_nearl2_2") {
                            attackCount = 0;
                            duration = 1;
                            console.log("不进行普攻");
                        } else {
                            console.write(`[特殊] 落地点火 - 取普攻时间=技能持续时间`);
                            console.log("取普攻时间=技能持续时间");
                            attackDuration = levelData.duration;
                            attackCount = Math.ceil(attackDuration / attackTime);
                            duration = attackCount * attackTime;
                        }
                    } else if (checkSpecs(skillId, "passive")) {
                        // 被动
                        attackCount = 10;
                        duration = attackCount * attackTime;
                        tags.push("passive");
                        console.write(`[特殊] 被动 - 以10次普攻计算`);
                        console.log("以10次普攻计算");
                    } else {
                        attackDuration = 10;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackCount * attackTime;
                        tags.push("auto", "instant");
                        console.write(`[特殊] 落地点火/瞬发 - 以10s普攻计算`);
                        console.log("以10s普攻计算");
                    }
                    break;
                case 4: // 受击回复
                    console.write(`受击回复`);
                    break;
                case 2: // 攻击恢复
                    console.write(`攻击回复`);
                    attackCount = spData.spCost;

                    const intv_chen = buffList["tachr_010_chen_1"] ? buffList["tachr_010_chen_1"].interval : 4;
                    const intv_archet = buffList["tachr_332_archet_1"] ? buffList["tachr_332_archet_1"].interval : 2.5;
                    let extra_sp = 0,
                        next = true;

                    // 枚举所需的最少攻击次数
                    while (attackCount > 0 && next) {
                        duration = attackCount * attackTime;
                        extra_sp = 0;
                        if (buffList["tachr_010_chen_1"] || options.chen) extra_sp += Math.floor(duration / intv_chen);
                        if (buffList["tachr_332_archet_1"] || options.archet) extra_sp += Math.floor(duration / intv_archet);
                        if (buffList["tachr_301_cutter_1"]) {
                            const p = buffList["tachr_301_cutter_1"].prob;
                            extra_sp += skillId == "skchr_cutter_1" ? (attackCount * 2 + 1) * p : attackCount * 2 * p;
                        }
                        next = attackCount + extra_sp >= spData.spCost;
                        if (next) attackCount -= 1;
                    }
                    if (!next) attackCount += 1;
                    duration = attackCount * attackTime;
                    const line = [];
                    if (buffList["tachr_010_chen_1"] || options.chen) line.push(`呵斥触发 ${Math.floor(duration / intv_chen)} 次`);
                    if (buffList["tachr_332_archet_1"] || options.archet) line.push(`兰登战术触发 ${Math.floor(duration / intv_archet)} 次`);
                    if (buffList["tachr_301_cutter_1"]) {
                        const p = buffList["tachr_301_cutter_1"].prob;
                        const _n = skillId == "skchr_cutter_1" ? (attackCount * 2 + 1) * p : attackCount * 2 * p;
                        line.push(`光蚀刻痕触发 ${_n.toFixed(2)} 次`);
                    }
                    if (line.length > 0) console.write(`[特殊] ${line.join(", ")}`);
                    if (rst) {
                        duration -= attackTime;
                    }
                    break;
                case 1: // 普通，前面已经算过一遍了，这里只特判
                    const sp_rate = 1 + buffFrame.spRecoveryPerSec;
                    if (buffList["tachr_002_amiya_1"]) {
                        // 情绪吸收
                        attackCount = Math.ceil((spData.spCost - stunDuration * sp_rate) / (buffList["tachr_002_amiya_1"]["amiya_t_1[atk].sp"] + attackTime * sp_rate));
                        duration = attackCount * attackTime;
                    } else if (buffList["tachr_134_ifrit_2"]) {
                        // [莱茵回路]. 需要解出攻击次数
                        const i = buffList["tachr_134_ifrit_2"].interval;
                        const isp = i * sp_rate + buffList["tachr_134_ifrit_2"].sp;
                        const recoverCount = Math.ceil((spData.spCost - i) / isp); // recoverCount >= (spCost - i) / isp
                        const r = (spData.spCost - recoverCount * isp) / sp_rate;
                        attackDuration = recoverCount * i + r;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        //console.log(i, isp, recoverCount, r, attackDuration, attackCount);
                        duration = attackDuration;
                    } else if (checkSpecs(skillId, "instant_buff")) {
                        // 不稳定血浆: 减去buff持续时间
                        attackDuration -= blackboard.duration || checkSpecs(skillId, "duration");
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackCount * attackTime;
                        console.log("瞬发Buff，技能周期为Buff持续时间");
                    } else if (buffList["tachr_400_weedy_2"] && options.cannon) {
                        // 水炮充能，持续20s/cd35s
                        const m = Math.floor(spData.spCost / 55);
                        const a = m * 6 + m * 55 * sp_rate; // 前m个水炮充能+自然恢复的sp量
                        const b = 6 + 20 * sp_rate; // 最后一个水炮持续期间最多恢复的sp
                        //let c = 6; // 最后一个水炮充的sp
                        let r = 0; // 计算还需要多少时间充满
                        if (a + b > spData.spCost) {
                            // 技能会在b期间蓄好
                            const y = Math.floor((spData.spCost - a) / (3 * sp_rate + 1.0));
                            const z = (spData.spCost - a - y) / sp_rate - y * 3;
                            r = 3 * y + z;
                            //c = Math.floor(r / 3);
                        } else {
                            r = (spData.spCost - a - b) / sp_rate + 20;
                        }
                        attackDuration = m * 55 + r;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackDuration;
                    } else if (options.charge && checkSpecs(skillId, "charge")) {
                        // 蓄力
                        let chargeDuration = spData.spCost;
                        if (buffList["tachr_426_billro_2"]) {
                            chargeDuration /= 1 + buffFrame.spRecoveryPerSec + buffList["tachr_426_billro_2"].sp_recovery_per_sec;
                        }
                        attackDuration += chargeDuration;
                        duration = attackDuration;
                        attackCount = Math.ceil(attackDuration / attackTime);
                    } else if (options.equip && subProf == "longrange") {
                        // 守林模组
                        const entry = buffList["uniequip_002_milu"] || buffList["uniequip_003_fartth"] || buffList["uniequip_002_lunacu"];
                        if (entry) {
                            console.log("每次攻击恢复1sp");
                            attackCount = Math.ceil((spData.spCost - stunDuration * sp_rate) / (entry.trait.sp + attackTime * sp_rate));
                            console.write(`[特殊] 攻击恢复SP = ${attackCount * entry.trait.sp}`);
                            duration = attackCount * attackTime;
                        }
                    } else if ("uniequip_002_leizi" in buffList && options.cond && "sp" in buffList["uniequip_002_leizi"].talent) {
                        // 惊蛰模组
                        console.log("每次命中恢复1sp");
                        attackCount = Math.ceil((spData.spCost - stunDuration * sp_rate) / (enemyCount + attackTime * sp_rate));
                        duration = attackCount * attackTime;
                    } else if (buffList["tachr_489_serum_1"] && skillId == "skchr_serum_1") {
                        const esp = buffList["tachr_489_serum_1"].sp_recovery_per_sec * (stunDuration - buffList["tachr_489_serum_1"].delay);
                        console.write(`眩晕时额外恢复 ${esp.toFixed(1)}sp`);
                        attackDuration = (spData.spCost - esp) / (1 + buffFrame.spRecoveryPerSec) - stunDuration;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = attackDuration;
                    } else if (buffList["tachr_422_aurora_1"]) {
                        attackDuration = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio) / 2;
                        if (attackDuration < stunDuration) attackDuration = 0;
                        attackCount = Math.ceil(attackDuration / attackTime);
                        duration = spData.spCost / ((1 + buffFrame.spRecoveryPerSec) * spRatio);
                        console.write("(晕眩期间不回复技力)");
                    } else if (skillId == "skchr_blkngt_2" && options.token) {
                        duration = attackDuration - blackboard["blkngt_s_2.duration"];
                        attackCount = Math.ceil(duration / attackTime);
                    }
                    break;
                // todo: cast time
            } // switch

            // ogcd穿插收益
            if (String(rst) === "ogcd") {
                const _ct = (Number(checkSpecs(skillId, "cast_time")) || 12) / 30;
                const weavingGain = ((duration - spData.spCost - _ct) / duration) * 100;
                console.write("[提示] 非GCD技能（技能不影响普攻间隔），计算器不计入穿插收益");
                if (weavingGain > 0) {
                    console.log(`OGCD技能/穿插收益: ${weavingGain.toFixed(1)}%`);
                }
            }
        } // else
    } // sim else

    // 计算实际命中次数
    // attackCount = 发动攻击的次数(swings), hitCount = 命中敌人的次数(hits)
    let hitCount = attackCount * buffFrame.times * enemyCount;
    // 蓝毒2
    if (isSkill) {
        if (skillId == "skchr_bluep_2") {
            hitCount += attackCount * (blackboard["attack@times"] - 1);
        } else if (["skcom_assist_cost[2]", "skchr_utage_1", "skchr_tachak_1"].includes(skillId)) {
            // 投降类
            hitCount = 0;
        } else if (skillId == "skchr_kroos2_2") {
            const extra_atk_count = attackCount - blackboard["attack@max_stack_count"] / 2;
            if (extra_atk_count > 0) {
                hitCount += extra_atk_count * 2;
                console.log(`4连击次数: ${extra_atk_count}`);
            }
        }
    }

    console.write(`持续: ${duration.toFixed(3)} s`);
    console.write(`攻击次数: ${attackCount * buffFrame.times} (${buffFrame.times} 连击 x ${attackCount})`);

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
