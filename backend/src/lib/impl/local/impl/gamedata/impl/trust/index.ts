import type { Favor } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/trust";
import { ExcelTables } from "../../../../../../../types/impl/lib/impl/local/impl/handler";
import { get as getFavor } from "../../../handler/impl/get";

export const getAll = async (): Promise<Favor> => {
    const data = (await getFavor(ExcelTables.FAVOR_TABLE)) as Favor;
    return data;
};

export default async (trust: number): Promise<number> => {
    const favorTable = await getAll();
    const keyFrames = favorTable.favorFrames.map((frame) => frame.data.favorPoint);
    return keyFrames.findIndex((frame: number) => frame >= trust);
};
