import { AttributeKeys } from "..";

export function getBuffedAttributes(
    basic: {
        [key: string]: any;
    },
    buffs: {
        [key: string]: any;
    },
) {
    const { ...final } = basic;
    AttributeKeys.forEach((key) => {
        if (buffs[key]) final[key] += buffs[key];
    });

    final.atk *= buffs.atk_scale;
    final.def *= buffs.def_scale;
    return final;
}
