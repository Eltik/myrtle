import { SkinCard } from "frontend";
import { useState } from "react";

// A selectable outfit card in the Planner tab: portrait over the outfit's own
// colours, price (or how it is obtained), class icon, rarity stars and the
// operator strip. Fixtures are live /release/skins rows.
const entry = (id: string, name: string, profession: string, subProfessionId: string, position: string, nationId: string) =>
    [id, { id, name, appellation: " ", rarity: 6, profession, subProfessionId, position, tagList: [], nationId, isNotObtainable: false, groupId: null, teamId: null }] as const;
const LOOKUP = new Map<string, never>([
    entry("char_249_mlyss", "Muelsyse", "PIONEER", "tactician", "RANGED", "columbia"),
    entry("char_293_thorns", "Thorns", "WARRIOR", "lord", "MELEE", "iberia"),
    entry("char_003_kalts", "Kal'tsit", "MEDIC", "physician", "RANGED", "rhodes"),
] as never);

const MLYSS = { skinId: "char_249_mlyss@boc#8", charId: "char_249_mlyss", skinName: "Young Branch", skinNameEn: "Young Branch", skinNameAuto: null, charName: { text: "Muelsyse", source: "memory" as const }, groupName: "Bloodline of Combat/VIII", groupNameAuto: null, brand: "boc", anchored: false, portraitPath: "/en/skin-portrait/char_249_mlyss_boc#8", rerun: true, price: { price: 24, store: true, obtain: "Store" }, colors: ["#28bd9c", "#28bd9c", "#ff8d3b", "#28bd9c", "#1d1d1d"] };
const THORNS = { skinId: "char_293_thorns@boc#8", charId: "char_293_thorns", skinName: "Blade-cleaved Tides", skinNameEn: "Blade-cleaved Tides", skinNameAuto: null, charName: { text: "Thorns", source: "memory" as const }, groupName: "Bloodline of Combat/VIII", groupNameAuto: null, brand: "boc", anchored: false, portraitPath: "/en/skin-portrait/char_293_thorns_boc#8", rerun: true, price: { price: 18, store: true, obtain: "Store" }, colors: ["#773f2f", "#1d1d1d", "#ffffff", "#1d1d1d", "#773f2f"] };
const KALTS = { skinId: "char_003_kalts@sale#14", charId: "char_003_kalts", skinName: "时遗", skinNameEn: null, skinNameAuto: { text: "The Remains of Time", source: "memory" as const }, charName: { text: "Kal'tsit", source: "memory" as const }, groupName: "忒斯特收藏/XVI", groupNameAuto: { text: "Test Collection/XVI", source: "memory" as const }, brand: "sale", anchored: true, portraitPath: "/en/skin-portrait/char_003_kalts_sale#14", rerun: false, price: { price: 0, store: false, obtain: "Obtain from Special Pack" }, colors: ["#6f7177", "#2e2e2f", "#ffffff", "#14151a", "#688a0b"] };

function Pick({ skin, initial }: { skin: typeof MLYSS | typeof KALTS; initial: boolean }) {
    const [on, setOn] = useState(initial);
    return <SkinCard skin={skin as never} on={on} lookup={LOOKUP} onPick={(_, v) => setOn(v)} />;
}

// Two store outfits from one set, the second picked.
export const StoreSet = () => (
    <div className="flex gap-3 p-4">
        <Pick skin={MLYSS} initial={false} />
        <Pick skin={THORNS} initial={true} />
    </div>
);

// Not sold in the store: the price slot names how it is obtained.
export const PackOutfit = () => (
    <div className="flex gap-3 p-4">
        <Pick skin={KALTS} initial={false} />
    </div>
);

export const Selected = () => (
    <div className="flex gap-3 p-4">
        <Pick skin={MLYSS} initial={true} />
    </div>
);
