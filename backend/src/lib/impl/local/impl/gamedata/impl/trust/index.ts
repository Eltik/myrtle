import type { Favor } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/trust";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Favor => {
    const data = STATIC_DATA?.FAVOR_TABLE as Favor;
    return data;
};

export default (trust: number): number => {
    const favorTable = getAll();
    const keyFrames = favorTable.favorFrames.map((frame) => frame.data.favorPoint);
    return keyFrames.findIndex((frame: number) => frame >= trust);
};
