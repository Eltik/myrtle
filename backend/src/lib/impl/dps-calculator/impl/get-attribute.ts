import { Operator } from "../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";

export function getAttribute(frames: Operator["phases"][0]["attributesKeyFrames"], level: number, minLevel: number, attr: string) {
    const ret =
        ((level - minLevel) / (frames[1].level - frames[0].level)) * (Number(frames[1].data[attr as keyof Operator["phases"][0]["attributesKeyFrames"][0]["data"]]) - Number(frames[0].data[attr as keyof Operator["phases"][0]["attributesKeyFrames"][0]["data"]])) +
        Number(frames[0].data[attr as keyof Operator["phases"][0]["attributesKeyFrames"][0]["data"]]);
    if (attr != "baseAttackTime") return Math.round(ret);
    else return ret;
}
