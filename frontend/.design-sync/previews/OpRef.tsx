import { OpRef } from "frontend";

// An operator chip in the release planner: portrait in a rarity square, name and
// rarity, linking to the operator page. An operator EN has not released yet has
// no index entry, so the chip falls back to the CN portrait and an
// auto-translated name with its source tag.
const entry = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, nationId: string) =>
    [id, { id, name, appellation: " ", rarity, profession, subProfessionId, position, tagList: [], nationId, isNotObtainable: false, groupId: null, teamId: null }] as const;

const LOOKUP = new Map<string, never>([
    entry("char_4065_judge", "Penance", 6, "TANK", "unyield", "MELEE", "siracusa"),
    entry("char_4121_zuole", "Zuo Le", 6, "WARRIOR", "musha", "MELEE", "yan"),
    entry("char_297_hamoni", "Harmonie", 5, "CASTER", "mystic", "RANGED", "victoria"),
] as never);

// Released on EN: a link chip with the rarity.
export const Released = () => (
    <div className="flex gap-2 p-4">
        <OpRef id="char_4065_judge" lookup={LOOKUP} />
        <OpRef id="char_297_hamoni" lookup={LOOKUP} />
    </div>
);

// CN-only operator, named by the planner (an appellation).
export const CnOnlyAutoName = () => (
    <div className="p-4">
        <OpRef id="char_4219_yukari" lookup={LOOKUP} name={{ text: "Yukari Takeba", source: "appellation" }} />
    </div>
);

// CN-only with no name to use: the raw id in mono.
export const CnOnlyRawId = () => (
    <div className="p-4">
        <OpRef id="char_4228_closur" lookup={LOOKUP} />
    </div>
);
