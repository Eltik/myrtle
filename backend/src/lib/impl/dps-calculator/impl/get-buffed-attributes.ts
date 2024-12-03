import { AttributeKeys } from "..";
import { BuffFrame } from "../../../../types/impl/lib/impl/dps-calculator";

export function getBuffedAttributes(basic: Record<string, number>, buffs: BuffFrame) {
    const { ...final } = basic;
    AttributeKeys.forEach((key) => {
        if (buffs[key]) final[key] += buffs[key];
    });

    Object.assign(final, {
        atk: (final.atk ?? 1) * buffs.atk,
        def: (final.def ?? 1) * buffs.def_scale,
    });

    return final;
}