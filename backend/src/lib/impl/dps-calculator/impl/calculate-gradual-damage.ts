import type { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { checkSpecs } from "./check-specs";

export function calculateGradDamage(_: {
    charId: string;
    skillId: string;
    isSkill: boolean;
    options: any;
    basicFrame: {
        [key: string]: any;
    };
    buffFrame: {
        [key: string]: any;
    };
    finalFrame: {
        [key: string]: any;
    };
    buffList: any;
    blackboard: any;
    dur: {
        attackCount: number;
        times: any;
        hitCount: number;
        duration: number;
        stunDuration: number;
        prepDuration: number;
        tags: string[];
        startSp: number;
    };
    attackTime: number;
    hitDamage: number;
    damageType: number;
    edef: number;
    ecount: number;
    emrpct: number;
    char: Operator;
}) {
    // _ -> args
    let ret = 0;
    let dmg_table: number[] = [];
    const _seq = [...Array(_.dur.attackCount).keys()]; // [0, 1, ..., attackCount-1]
    const subProf = _.char.subProfessionId;

    if (subProf == "funnel") {
        // 驭蟹术士
        // 基于当前伤害直接乘算atk_scale倍率即可
        const base_scale = _.skillId == "skchr_gdglow_3" && _.isSkill ? 0 : 1;
        let base_table = [0, 1, 2, 3, 4, 5, 6];
        let max_funnel_scale = 1.1;
        if (_.skillId == "skchr_rockr_2" && _.options.overdrive_mode) {
            const start = 1.1;
            max_funnel_scale = _.blackboard.scale * 1.1;
            const stacks = Math.ceil((_.blackboard.scale * 1.1 - start) / 0.15 + 1);
            base_table = [...Array(stacks).keys()].map((x) => x + 6);
        }

        let funnel = 1;
        if (_.isSkill) funnel = Number(checkSpecs(_.skillId, "funnel")) || 1;

        const tb = base_table.map((x) => base_scale + Math.min(max_funnel_scale, 0.2 + 0.15 * x) * funnel);
        let acount = _.dur.attackCount;
        if (
            _.charId == "char_377_gdglow" &&
            (
                _.dur as unknown as {
                    critHitCount: number;
                }
            ).critHitCount > 0 &&
            _.isSkill
        ) {
            acount -= (
                _.dur as unknown as {
                    critHitCount: number;
                }
            ).critHitCount;
        }
        dmg_table = [...Array(acount * _.buffFrame.times).keys()].map((x) => (x >= tb.length ? Math.round(_.hitDamage * tb[tb.length - 1]) : Math.round(_.hitDamage * tb[x])));
    } else if (_.skillId == "skchr_kalts_3") {
        // 凯尔希: 每秒改变一次攻击力, finalFrame.atk为第一次攻击力
        const range = _.basicFrame.atk * _.blackboard["attack@atk"];
        const n = Math.floor(_.dur.duration);
        const atk_by_sec = [...Array(n + 1).keys()].map((x) => _.finalFrame.atk - (range * x) / n);
        // 抬手时间
        const atk_begin = Math.round(Number(checkSpecs(_.skillId, "attack_begin")) || 12) / 30;
        const atk_timing = _seq.map((i) => atk_begin + _.attackTime * i);

        dmg_table = atk_timing.map((x) => atk_by_sec[Math.floor(x)] * _.buffFrame.damage_scale);
    } else if (_.skillId == "skchr_billro_3") {
        // 卡涅利安: 每秒改变一次攻击力（多一跳），蓄力时随攻击次数改变damage_scale倍率, finalFrame.atk为最后一次攻击力
        const range = _.basicFrame.atk * _.blackboard.atk;
        const n = Math.floor(_.dur.duration);
        // rate = (x-1)/(n-1), thus t=0, x=n, rate=1; t=(n-1), x=1, rate=0
        const atk_by_sec = [...Array(n + 1).keys()].reverse().map((x) => _.finalFrame.atk - (range * (x - 1)) / (n - 1));
        // 抬手时间
        const atk_begin = Math.round(Number(checkSpecs(_.skillId, "attack_begin") || 12)) / 30;
        const atk_timing = _seq.map((i) => atk_begin + _.attackTime * i);
        // damage_scale
        const sc = [1.2, 1.4, 1.6, 1.8, 2];
        const scale_table = _seq.map((i) => (i >= sc.length ? 2 : sc[i]));

        dmg_table = atk_timing.map((x) => atk_by_sec[Math.floor(x)] * _.ecount * Math.max(1 - _.emrpct, 0.05) * _.buffFrame.damage_scale);
        const kwargs = { duration: n, atk_by_sec, atk_timing, dmg_table };
        if (_.options.charge) {
            dmg_table = _seq.map((i) => dmg_table[i] * scale_table[i]);
            Object.assign(kwargs, { scale_table: scale_table.map((x) => x * _.buffFrame.damage_scale) });
            kwargs.dmg_table = dmg_table;
        }
    } else {
        let a = _.buffFrame.atk_table.map((x: number) => _.basicFrame.atk * x);
        const pivot = a[a.length - 1];
        a = a.map((x: number) => _.finalFrame.atk - pivot + x);

        // 计算每次伤害
        if (_.damageType == 0) {
            dmg_table = a.map((x: number) => Math.max(x - _.edef, x * 0.05) * _.buffFrame.damage_scale);
        } else if (_.damageType == 3) {
            dmg_table = a.map((x: number) => x * _.buffFrame.damage_scale);
        }
    }
    if (dmg_table.length > 0) ret = dmg_table.reduce((x, y) => x + y);

    // 至简暴击（均摊计算）
    if ("tachr_4054_malist_1" in _.buffList && _.options.crit) {
        const entry = _.buffList["tachr_4054_malist_1"];
        const crit_scale = 1 + entry.prob * (entry.atk_scale - 1);
        ret *= crit_scale;
    }
    return ret;
}
