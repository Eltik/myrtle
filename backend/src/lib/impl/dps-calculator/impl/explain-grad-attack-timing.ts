export function explainGradAttackTiming(
    _: {
        duration: number;
        atk_by_sec: number[];
        atk_timing: number[];
        dmg_table: any[];
        scale_table?: any[];
    },
    n: number = 7,
) {
    const lines = [];
    let i = 0;
    const row_time = [...Array(_.duration).keys()];
    const l1 = row_time.map(() => ":--:");
    const row_atk = [..._.atk_by_sec.map((x) => Math.round(x))];
    const row_timing = [..._.atk_timing.map((x) => x.toFixed(2))];
    let row_scale: number[] = [];
    const l2 = row_timing.map(() => ":--:");
    const row_dmg = [..._.dmg_table.map((x) => Math.round(x))];
    if (_.scale_table) row_scale = [..._.scale_table.map((x) => x.toFixed(2))];

    while (i < row_time.length) {
        const r1 = ["时间(s)", ...row_time.slice(i, i + n)];
        const ls1 = [":--:", ...l1.slice(i, i + n)];
        const a1 = ["攻击力", ...row_atk.slice(i, i + n)];

        lines.push(`| ${r1.join(" | ")} |`);
        lines.push(`| ${ls1.join(" | ")} |`);
        lines.push(`| ${a1.join(" | ")} |`);
        lines.push("\n");
        i += n;
    }

    i = 0;

    while (i < row_timing.length) {
        const r2 = ["时点(s)", ...row_timing.slice(i, i + n)];
        const ls2 = [":--:", ...l2.slice(i, i + n)];
        let s2 = [];
        const d2 = ["伤害", ...row_dmg.slice(i, i + n)];

        lines.push(`| ${r2.join(" | ")} |`);
        lines.push(`| ${ls2.join(" | ")} |`);
        if (_.scale_table) {
            s2 = ["倍率", ...row_scale.slice(i, i + n)];
            lines.push(`| ${s2.join(" | ")} |`);
        }
        lines.push(`| ${d2.join(" | ")} |`);
        lines.push("\n");
        i += n;
    }
    return lines.join("\n");
}
