import { OpRefList } from "frontend";

// The featured operators on a banner row: a wrapping list of OpRef chips. The
// fixture is the live CN pool DOUBLE_77_0_5 (Penance and Zuo Le at 6★; Harmonie,
// Santalla and Gracebearer at 5★).
const entry = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, nationId: string) =>
    [id, { id, name, appellation: " ", rarity, profession, subProfessionId, position, tagList: [], nationId, isNotObtainable: false, groupId: null, teamId: null }] as const;

const LOOKUP = new Map<string, never>([
    entry("char_4065_judge", "Penance", 6, "TANK", "unyield", "MELEE", "siracusa"),
    entry("char_4121_zuole", "Zuo Le", 6, "WARRIOR", "musha", "MELEE", "yan"),
    entry("char_297_hamoni", "Harmonie", 5, "CASTER", "mystic", "RANGED", "victoria"),
    entry("char_341_sntlla", "Santalla", 5, "CASTER", "splashcaster", "RANGED", "sami"),
    entry("char_4187_graceb", "Gracebearer", 5, "WARRIOR", "primguard", "MELEE", "bolivar"),
] as never);

export const BannerFeatured = () => (
    <div className="flex w-full max-w-md flex-col gap-2 p-4">
        <OpRefList ids={["char_4065_judge", "char_4121_zuole"]} lookup={LOOKUP} />
        <OpRefList ids={["char_297_hamoni", "char_341_sntlla", "char_4187_graceb"]} lookup={LOOKUP} />
    </div>
);

// A debut banner whose operators are not on EN yet, named by the planner.
export const UpcomingDebuts = () => (
    <div className="w-full max-w-md p-4">
        <OpRefList ids={["char_4219_yukari", "char_4235_thumpy"]} lookup={LOOKUP} names={{ char_4219_yukari: { text: "Yukari Takeba", source: "appellation" }, char_4235_thumpy: { text: "Thumpy", source: "appellation" } }} />
    </div>
);
