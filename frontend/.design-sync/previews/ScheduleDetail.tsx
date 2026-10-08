import { ScheduleDetail } from "frontend";

// The panel that opens under a calendar pill: kind stripe, art, name and tags,
// CN dates, featured operators or outfit tiles, and the EN-date badge. `today`
// is pinned to 2026-09-01; fixtures are live /release rows.
const TODAY = new Date("2026-09-01T12:00:00+09:00");
const entry = (id: string, name: string, profession: string, subProfessionId: string, position: string, nationId: string) =>
    [id, { id, name, appellation: " ", rarity: 6, profession, subProfessionId, position, tagList: [], nationId, isNotObtainable: false, groupId: null, teamId: null }] as const;
const LOOKUP = new Map<string, never>([
    entry("char_4065_judge", "Penance", "TANK", "unyield", "MELEE", "siracusa"),
    entry("char_4121_zuole", "Zuo Le", "WARRIOR", "musha", "MELEE", "yan"),
    entry("char_249_mlyss", "Muelsyse", "PIONEER", "tactician", "RANGED", "columbia"),
    entry("char_293_thorns", "Thorns", "WARRIOR", "lord", "MELEE", "iberia"),
    entry("char_4087_ines", "Ines", "PIONEER", "agent", "MELEE", ""),
] as never);

// An event (the Act or Die rerun): 5:2 art, side-story tag, CN dates and an
// estimated EN window.
export const Event = () => (
    <div className="w-full p-4">
        <ScheduleDetail
            item={{ key: "event:act43sre", kind: "event", nameCn: "红丝绒·复刻", nameEn: null, nameAuto: { text: "Act or Die", source: "memory" }, tag: "TYPE_ACT9D0", detail: "", hasStage: true, imagePath: "/cn/event-image/act43sre", start: 1798140600, end: null, cnStart: 1784491200, cnEnd: 1785355199, resolution: { status: "estimated", enStart: 1798140600, lo: 1798000200, hi: 1798505100 }, charIds: [], estimated: true }}
            lookup={LOOKUP}
            today={TODAY}
            onClose={() => {}}
        />
    </div>
);

// A banner: its featured 6★ operators as chips.
export const Banner = () => (
    <div className="w-full p-4">
        <ScheduleDetail
            item={{ key: "banner:DOUBLE_77_0_5", kind: "banner", nameCn: "适合多种场合的强力干员", nameEn: null, nameAuto: { text: "Rare Operators useful in all kinds of stages", source: "memory" }, tag: "DOUBLE", detail: "", imagePath: "/cn/banner-image/DOUBLE_77_0_5", start: 1791403200, end: 1792612799, cnStart: 1791403200, cnEnd: 1792612799, resolution: { status: "independent" }, charIds: ["char_4065_judge", "char_4121_zuole"], estimated: false }}
            lookup={LOOKUP}
            today={TODAY}
            onClose={() => {}}
        />
    </div>
);

// A skin rerun: no art, the set's outfits as portrait tiles.
export const SkinRerun = () => (
    <div className="w-full p-4">
        <ScheduleDetail
            item={{
                key: "rerun:2024#boc",
                kind: "rerun",
                nameCn: "Bloodline of Combat/VIII",
                nameEn: "Bloodline of Combat/VIII",
                nameAuto: null,
                tag: "",
                detail: "",
                imagePath: null,
                start: 1776168000,
                end: null,
                cnStart: 1760443200,
                cnEnd: 1762858799,
                resolution: { status: "estimated", enStart: 1776168000, lo: 1762239600, hi: 1777813200 },
                charIds: [],
                estimated: true,
                skins: [
                    { skinId: "char_249_mlyss@boc#8", charId: "char_249_mlyss", skinName: "Young Branch", skinNameEn: "Young Branch", skinNameAuto: null, charName: { text: "Muelsyse", source: "memory" }, portraitPath: "/en/skin-portrait/char_249_mlyss_boc#8" },
                    { skinId: "char_293_thorns@boc#8", charId: "char_293_thorns", skinName: "Blade-cleaved Tides", skinNameEn: "Blade-cleaved Tides", skinNameAuto: null, charName: { text: "Thorns", source: "memory" }, portraitPath: "/en/skin-portrait/char_293_thorns_boc#8" },
                    { skinId: "char_4087_ines@boc#8", charId: "char_4087_ines", skinName: "Under the Flaming Dome", skinNameEn: "Under the Flaming Dome", skinNameAuto: null, charName: { text: "Ines", source: "memory" }, portraitPath: "/en/skin-portrait/char_4087_ines_boc#8" },
                ],
            }}
            lookup={LOOKUP}
            today={new Date("2026-03-01T12:00:00+09:00")}
            onClose={() => {}}
        />
    </div>
);
