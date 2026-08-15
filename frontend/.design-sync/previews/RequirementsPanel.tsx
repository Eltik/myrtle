import { RequirementsPanel } from "frontend";

// The right-hand half of the planner: everything the selected plans still need,
// bucketed by the backend's `sortGroup` axis (0/1 EXP & LMD, 2 Skill Summaries,
// 3 Chips, 4 Module Data, 5 Materials). Item ids and iconIds are real rows from
// /api/static/materials, so the icons resolve against the live asset API.
//
// Status is derived: `missingCount > 0` is red, a shortfall you can still craft
// is amber ("craft N"), and full coverage is a green tick.

type Req = {
    id: string;
    name: string;
    iconId: string;
    image: null;
    itemType: string;
    rarity: number;
    sortGroup: number;
    sortSubrank: number;
    requiredCount: number;
    inventoryCount: number;
    craftableCount: number;
    missingCount: number;
    canCraft: boolean;
    craftReason: string;
    recipe: { count: number; costs: { count: number; item: Req }[] } | null;
};

const item = (id: string, name: string, iconId: string, rarity: number, sortGroup: number, requiredCount: number, inventoryCount: number, over: Partial<Req> = {}): Req => ({
    id,
    name,
    iconId,
    image: null,
    itemType: "MATERIAL",
    rarity,
    sortGroup,
    sortSubrank: 0,
    requiredCount,
    inventoryCount,
    craftableCount: 0,
    missingCount: 0,
    canCraft: false,
    craftReason: "No recipe",
    recipe: null,
    ...over,
});

const orirockCube = item("30012", "Orirock Cube", "MTL_SL_G2", 2, 5, 24, 61);
const manganeseOre = item("30083", "Manganese Ore", "MTL_SL_MANGANESE1", 3, 5, 9, 12);
const incandescentAlloy = item("31023", "Incandescent Alloy", "MTL_SL_IAM3", 3, 5, 8, 3, { missingCount: 5, craftReason: "Requirements not met: needs 5 more Incandescent Alloy" });
const orirockCluster = item("30013", "Orirock Cluster", "MTL_SL_G3", 3, 5, 12, 4, {
    craftableCount: 8,
    canCraft: true,
    craftReason: "Craftable from Orirock Cube",
    recipe: { count: 1, costs: [{ count: 4, item: orirockCube }] },
});

const REQUIREMENTS: Req[] = [
    item("2004", "Strategic Battle Record", "sprite_exp_card_t4", 5, 0, 148, 148),
    item("4001", "LMD", "GOLD", 4, 1, 1_181_000, 843_620, { missingCount: 337_380, craftReason: "No recipe" }),
    item("3303", "Skill Summary - 3", "MTL_SKILL3", 4, 2, 26, 9, {
        craftableCount: 17,
        canCraft: true,
        craftReason: "Craftable from Skill Summary - 2",
        recipe: { count: 1, costs: [{ count: 3, item: item("3302", "Skill Summary - 2", "MTL_SKILL2", 3, 2, 51, 88) }] },
    }),
    item("3302", "Skill Summary - 2", "MTL_SKILL2", 3, 2, 18, 88),
    item("3223", "Guard Dualchip", "MTL_ASC_GRD3", 5, 3, 2, 0, {
        craftableCount: 2,
        canCraft: true,
        craftReason: "Craftable from Guard Chip Pack + Chip Catalyst",
        recipe: {
            count: 1,
            costs: [
                { count: 2, item: item("3222", "Guard Chip Pack", "MTL_ASC_GRD2", 4, 3, 4, 5) },
                { count: 1, item: item("32001", "Chip Catalyst", "MTL_ASC_DI", 4, 3, 2, 3) },
            ],
        },
    }),
    item("32001", "Chip Catalyst", "MTL_ASC_DI", 4, 3, 2, 3),
    item("mod_unlock_token", "Module Data Block", "mod_unlock_token", 5, 4, 2, 1, { missingCount: 1, craftReason: "Obtained from Module missions only" }),
    item("mod_update_token_2", "Data Supplement Instrument", "mod_update_token_2", 5, 4, 5, 5),
    item("30135", "D32 Steel", "MTL_SL_DS", 5, 5, 4, 1, {
        craftableCount: 1,
        canCraft: true,
        craftReason: "Craftable from Manganese Trihydrate + Orirock Cluster",
        recipe: {
            count: 1,
            costs: [
                { count: 1, item: item("30084", "Manganese Trihydrate", "MTL_SL_MANGANESE2", 4, 5, 4, 2) },
                { count: 1, item: orirockCluster },
            ],
        },
    }),
    item("30125", "Bipolar Nanoflake", "MTL_SL_BN", 5, 5, 3, 0, { missingCount: 3, craftReason: "Requirements not met: needs 2 more Grindstone Pentahydrate and 1 more Incandescent Alloy Block" }),
    item("30094", "Grindstone Pentahydrate", "MTL_SL_PG2", 4, 5, 6, 4, {
        craftableCount: 2,
        canCraft: true,
        craftReason: "Craftable from Grindstone",
        recipe: { count: 1, costs: [{ count: 3, item: item("30093", "Grindstone", "MTL_SL_PG1", 3, 5, 9, 14) }] },
    }),
    item("31024", "Incandescent Alloy Block", "MTL_SL_IAM4", 4, 5, 5, 2, { missingCount: 3, craftReason: "Requirements not met: needs 5 more Incandescent Alloy" }),
    orirockCluster,
    incandescentAlloy,
    manganeseOre,
    item("30073", "Loxic Kohl", "MTL_SL_ALCOHOL1", 3, 5, 15, 22),
];

const plan = (operatorId: string, name: string, targetElite: number, targetLevel: number, skills: { skill_index: number; mastery_level: number }[], modules: { module_id: string; module_stage: number }[]) => ({
    id: `plan_${operatorId}`,
    user_id: "u_1042",
    operator_id: operatorId,
    target_elite: targetElite,
    target_level: targetLevel,
    target_skill_level: 7,
    target_skills: skills,
    target_modules: modules,
    display_on_profile: true,
    created_at: "2024-04-28T09:12:00Z",
    updated_at: "2024-05-12T21:04:00Z",
    groups: ["Mastery queue"],
    operator: { id: operatorId, name, rarity: "TIER_6", profession: "WARRIOR", subProfessionId: "librator", skills: [], modules: [], phases: [] },
});

const ACTIVE_PLANS = [
    plan("char_4064_mlynar", "Młynar", 2, 90, [{ skill_index: 2, mastery_level: 3 }], [{ module_id: "uniequip_002_mlynar", module_stage: 3 }]),
    plan("char_249_mlyss", "Muelsyse", 2, 80, [{ skill_index: 1, mastery_level: 3 }], []),
    plan("char_263_skadi", "Skadi", 2, 80, [], []),
];

// Two 6★ masteries and a promotion queued: EXP is covered, LMD is short, and
// three materials still need crafting.
export const GroupedRequirements = () => <RequirementsPanel activePlans={ACTIVE_PLANS} aggregatedRequirements={REQUIREMENTS} isLoading={false} />;

// Everything already in the depot: every row resolves to the green tick and the
// group subtitles drop their "missing"/"to craft" tails.
const SATISFIED: Req[] = REQUIREMENTS.map((r) => ({ ...r, inventoryCount: Math.max(r.inventoryCount, r.requiredCount), missingCount: 0, craftableCount: 0, canCraft: false }));

export const AllRequirementsMet = () => <RequirementsPanel activePlans={ACTIVE_PLANS} aggregatedRequirements={SATISFIED} isLoading={false} />;

export const Loading = () => <RequirementsPanel activePlans={ACTIVE_PLANS} aggregatedRequirements={[]} isLoading />;

// Every selected plan is already satisfied — or nothing is selected at all.
export const NothingRequired = () => <RequirementsPanel activePlans={[]} aggregatedRequirements={[]} isLoading={false} />;
