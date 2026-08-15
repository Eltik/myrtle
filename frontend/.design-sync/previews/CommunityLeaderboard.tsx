import { CommunityLeaderboard } from "frontend";

const TOTAL_PULLS = 200_670;

const INDEX: Array<[string, string, string]> = [
    ["char_1016_agoat2", "Eyjafjalla the Hvít Aska", "MEDIC"],
    ["char_245_cello", "Virtuosa", "SUPPORT"],
    ["char_4064_mlynar", "Młynar", "WARRIOR"],
    ["char_2012_typhon", "Typhon", "SNIPER"],
    ["char_4087_ines", "Ines", "PIONEER"],
    ["char_249_mlyss", "Muelsyse", "PIONEER"],
    ["char_4117_ray", "Ray", "SNIPER"],
    ["char_4116_blkkgt", "Degenbrecher", "WARRIOR"],
    ["char_143_ghost", "Specter", "WARRIOR"],
    ["char_140_whitew", "Lappland", "WARRIOR"],
    ["char_102_texas", "Texas", "PIONEER"],
    ["char_128_plosis", "Ptilopsis", "MEDIC"],
    ["char_219_meteo", "Meteorite", "SNIPER"],
    ["char_144_red", "Projekt Red", "SPECIAL"],
    ["char_151_myrtle", "Myrtle", "PIONEER"],
    ["char_109_fmout", "Gitano", "CASTER"],
    ["char_183_skgoat", "Earthspirit", "SUPPORT"],
    ["char_110_deepcl", "Deepcolor", "SUPPORT"],
];

const OPERATORS_BY_ID = new Map(INDEX.map(([id, name, profession]) => [id, { id, name, profession }]));

const op = (charId: string, rarity: number, pullCount: number) => ({
    charId,
    charName: "",
    rarity,
    pullCount,
    percentage: pullCount / TOTAL_PULLS,
});

const OPS = [
    op("char_1016_agoat2", 6, 823),
    op("char_245_cello", 6, 796),
    op("char_4064_mlynar", 6, 512),
    op("char_2012_typhon", 6, 431),
    op("char_4087_ines", 6, 402),
    op("char_249_mlyss", 6, 388),
    op("char_4117_ray", 6, 352),
    op("char_4116_blkkgt", 6, 311),
    op("char_143_ghost", 5, 3834),
    op("char_140_whitew", 5, 1301),
    op("char_102_texas", 5, 1188),
    op("char_128_plosis", 5, 1064),
    op("char_219_meteo", 5, 947),
    op("char_144_red", 5, 862),
    op("char_151_myrtle", 4, 2195),
    op("char_109_fmout", 4, 2212),
    op("char_183_skgoat", 4, 2182),
    op("char_110_deepcl", 4, 2175),
];

export const TopSixStars = () => <CommunityLeaderboard ops={OPS} operatorsById={OPERATORS_BY_ID} isLoading={false} />;

export const Loading = () => <CommunityLeaderboard ops={[]} operatorsById={OPERATORS_BY_ID} isLoading={true} />;

/** The 6★ tab is selected by default, so a 5★-only corpus lands on the empty branch. */
export const NoDataForRarity = () => <CommunityLeaderboard ops={OPS.filter((o) => o.rarity === 5)} operatorsById={OPERATORS_BY_ID} isLoading={false} />;
