import { ClassIcon, FacetFilter, TooltipProvider } from "frontend";
import { useState } from "react";

// One facet's row of toggles, shared by the tier-list pool dialog (KindPool)
// and the grid picker: an uppercase mono label, then a multi-select outline
// ToggleGroup. `mono` suits short tabular values (stars), `text` words, `icon`
// glyph buttons whose label lives in a tooltip (so the caller wraps it in a
// TooltipProvider). Unselected options dim; a row too long for its box scrolls
// with a faded edge. Facets mirror `KIND_DEFINITIONS` (operator class, rarity,
// enemy rank, skin brand).

const valueOf = () => null;

const CLASSES: [string, string][] = [
    ["PIONEER", "Vanguard"],
    ["WARRIOR", "Guard"],
    ["TANK", "Defender"],
    ["SNIPER", "Sniper"],
    ["CASTER", "Caster"],
    ["MEDIC", "Medic"],
    ["SUPPORT", "Supporter"],
    ["SPECIAL", "Specialist"],
];

const CLASS_FACET = {
    id: "class",
    label: "Class",
    groupLabel: "Filter by class",
    variant: "icon" as const,
    options: CLASSES.map(([value, label]) => ({ value, label, icon: <ClassIcon profession={value} size={16} /> })),
    valueOf,
};

const RARITY_FACET = {
    id: "rarity",
    label: "Rarity",
    groupLabel: "Filter by rarity",
    variant: "mono" as const,
    options: [6, 5, 4, 3, 2, 1].map((r) => ({ value: String(r), label: `${r}★`, ariaLabel: `${r} star` })),
    valueOf,
};

const RANK_FACET = {
    id: "rank",
    label: "Rank",
    groupLabel: "Filter by rank",
    variant: "text" as const,
    options: [
        { value: "NORMAL", label: "Normal" },
        { value: "ELITE", label: "Elite" },
        { value: "BOSS", label: "Leader" },
    ],
    valueOf,
};

/** Real skin brands from the live skin catalogue; more than fit, so the row scrolls. */
const BRAND_FACET = {
    id: "brand",
    label: "Brand",
    groupLabel: "Filter by brand",
    variant: "text" as const,
    options: ["EPOQUE", "Coral Coast", "Bloodline of Combat", "Collab Series", "Test Collection", "Made by 0011", "Icefield Messenger", "Ambience Synesthesia", "0011/Tempest", "Achievement Star", "MARTHE", "Witch Feast"].map((b) => ({ value: b, label: b })),
    valueOf,
};

function Row({ facet, initial }: { facet: typeof CLASS_FACET | typeof RARITY_FACET | typeof RANK_FACET | typeof BRAND_FACET; initial: string[] }) {
    const [value, setValue] = useState<string[]>(initial);
    return (
        <TooltipProvider delay={300}>
            <div className="w-full max-w-xl p-5">
                <FacetFilter facet={facet} value={value} onChange={setValue} />
            </div>
        </TooltipProvider>
    );
}

/** Operator class as icon toggles, Guard and Caster selected. */
export const ClassIcons = () => <Row facet={CLASS_FACET} initial={["WARRIOR", "CASTER"]} />;

/** Rarity as mono star toggles, 6★ and 5★ selected. */
export const Rarity = () => <Row facet={RARITY_FACET} initial={["6", "5"]} />;

/** Enemy rank as word toggles, nothing selected: every option dims equally. */
export const EnemyRank = () => <Row facet={RANK_FACET} initial={[]} />;

/** Skin brands: a long row that scrolls, its cut-off edge faded. */
export const LongRowScrolls = () => <Row facet={BRAND_FACET} initial={["EPOQUE"]} />;
