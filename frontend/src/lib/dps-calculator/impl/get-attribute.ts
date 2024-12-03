import type { Operator } from "~/types/impl/api/static/operator";

export function getAttribute(frames: Operator["phases"][0]["attributesKeyFrames"], level: number, minLevel: number, attr: string) {
    const ret = ((level - minLevel) / ((frames[1]?.level ?? 0) - (frames[0]?.level ?? 0))) * (Number(frames[1]?.data[attr as keyof Operator["phases"][0]["attributesKeyFrames"][0]["data"]]) - Number(frames[0]?.data[attr as keyof Operator["phases"][0]["attributesKeyFrames"][0]["data"]])) + Number(frames[0]?.data[attr as keyof Operator["phases"][0]["attributesKeyFrames"][0]["data"]]);
    if (attr != "baseAttackTime") return Math.round(ret);
    else return ret;
}
