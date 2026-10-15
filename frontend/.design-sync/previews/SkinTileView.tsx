import { SkinTileView } from "frontend";

// A skin portrait tile from the schedule detail and planner: portrait, operator,
// and the skin's name. Clicking opens the skin popup. Fixtures are the live
// Bloodline of Combat/VIII rerun forecast and a new Kal'tsit outfit.
const API_PORTRAIT = (p: string) => p;
const entry = (id: string, name: string, profession: string, subProfessionId: string, position: string, nationId: string) =>
    [id, { id, name, appellation: " ", rarity: 6, profession, subProfessionId, position, tagList: [], nationId, isNotObtainable: false, groupId: null, teamId: null }] as const;
const LOOKUP = new Map<string, never>([
    entry("char_249_mlyss", "Muelsyse", "PIONEER", "tactician", "RANGED", "columbia"),
    entry("char_293_thorns", "Thorns", "WARRIOR", "lord", "MELEE", "iberia"),
    entry("char_4087_ines", "Ines", "PIONEER", "agent", "MELEE", ""),
    entry("char_003_kalts", "Kal'tsit", "MEDIC", "physician", "RANGED", "rhodes"),
] as never);

const BOC = [
    { skinId: "char_249_mlyss@boc#8", charId: "char_249_mlyss", skinName: "Young Branch", charName: { text: "Muelsyse", source: "memory" as const }, portraitPath: API_PORTRAIT("/en/skin-portrait/char_249_mlyss_boc#8") },
    { skinId: "char_293_thorns@boc#8", charId: "char_293_thorns", skinName: "Blade-cleaved Tides", charName: { text: "Thorns", source: "memory" as const }, portraitPath: API_PORTRAIT("/en/skin-portrait/char_293_thorns_boc#8") },
    { skinId: "char_4087_ines@boc#8", charId: "char_4087_ines", skinName: "Under the Flaming Dome", charName: { text: "Ines", source: "memory" as const }, portraitPath: API_PORTRAIT("/en/skin-portrait/char_4087_ines_boc#8") },
];

export const RerunSet = () => (
    <div className="flex gap-2 p-4">
        {BOC.map((s) => (
            <SkinTileView key={s.skinId} {...s} skinNameEn={s.skinName} lookup={LOOKUP} />
        ))}
    </div>
);

// A CN outfit not yet on EN: auto-translated skin name.
export const NewOutfit = () => (
    <div className="flex gap-2 p-4">
        <SkinTileView skinId="char_003_kalts@sale#14" charId="char_003_kalts" charName={{ text: "Kal'tsit", source: "memory" }} skinName="时遗" skinNameEn={null} skinNameAuto={{ text: "The Remains of Time", source: "memory" }} portraitPath="/en/skin-portrait/char_003_kalts_sale#14" lookup={LOOKUP} />
    </div>
);

// No portrait available: the operator's initial on the dark plate.
export const NoPortrait = () => (
    <div className="flex gap-2 p-4">
        <SkinTileView skinId="char_293_thorns@boc#8" charId="char_293_thorns" charName={{ text: "Thorns", source: "memory" }} skinName="Blade-cleaved Tides" skinNameEn="Blade-cleaved Tides" portraitPath={null} lookup={LOOKUP} />
    </div>
);
