import type { ReactNode } from "react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { entityOwner, type ITierEntity, type ITierEntityOf, integratedStrategiesNumber, type TierEntityKind } from "#/lib/api/tier-entities";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatProfession, RARITY_HEX_MUTED } from "#/lib/utils";
import type { OperatorProfession } from "#/types/operators";
import type { messages } from "./kinds.messages";

// How each kind presents: its names, its tile accent and shape, its hover-card
// detail line, its page, and its pool in the editor. Every tile, pool, dialog
// and hover card asks here instead of switching on the kind. The data side of
// a kind (its fields, art fit and owner) is `KIND_MODELS` in
// `#/lib/api/tier-entities`; its strings are its block in `kinds.messages.ts`.

/** The `t` every definition renders through. */
export type KindT = TypedT<typeof messages>;

/** A site route that takes the entity's id as `$id`. */
export type EntityPageRoute = "/enemies/$id";

export interface IFacetOption {
    value: string;
    /** Visible text, or the tooltip and accessible name when `icon` is set. */
    label: string;
    /** Accessible name when it should differ from `label` (`6 star` for `6★`). */
    ariaLabel?: string;
    icon?: ReactNode;
}

/** One filter row of a kind's pool: toggles over one facet of its entities. */
export interface IKindFacet<K extends TierEntityKind> {
    id: string;
    /** The filter row's label, rendered uppercase. */
    label: string;
    /** Accessible name of the row of toggles. */
    groupLabel: string;
    /** `mono` for short tabular values (stars), `text` for words, `icon` for glyph buttons with tooltips. */
    variant: "mono" | "text" | "icon";
    options: IFacetOption[];
    /** The entity's value for this facet; an entity with none never matches a selection. */
    valueOf: (entity: ITierEntityOf<K>) => string | null;
}

export interface IKindPool<K extends TierEntityKind> {
    kicker: string;
    dialogTitle: string;
    searchLabel: string;
    gridLabel: string;
    dropArea: string;
    facets: IKindFacet<K>[];
    /** Strings the search box matches, already in display form. */
    searchTexts: (entity: ITierEntityOf<K>) => (string | null)[];
    /** Pool order. Absent: the catalogue's own order, which the backend sets per kind. */
    compare?: (a: ITierEntityOf<K>, b: ITierEntityOf<K>) => number;
}

export interface IKindDefinition<K extends TierEntityKind> {
    /** `Enemies`: the pool tab and the kinds dialog. */
    plural: (t: KindT) => string;
    /** `Enemy`: tile names and the hover card's kicker. */
    singular: (t: KindT) => string;
    /** The line under the plural in the kinds dialog. */
    description: (t: KindT) => string;
    /** The tile's bottom bar and hover border. Absent: the neutral accent. */
    accent?: (entity: ITierEntityOf<K>) => string;
    /** `wide`: the art is a banner, so the tile spans two square slots of the same grid. */
    shape?: "wide";
    /** The entity's own page on the site, linked from its tile and the tier dialog. */
    page?: EntityPageRoute;
    /** The hover card's line of what kind of thing this is, e.g. `Elite · B1` or `Side Story · Rerun`. */
    detail: (entity: ITierEntityOf<K>, t: KindT) => string[];
    /** The editor pool. `catalogue` is the kind's catalogue, read by filters whose options come from the data (skin brands, Integrated Strategies themes). */
    pool: (t: KindT, catalogue: readonly ITierEntityOf<K>[]) => IKindPool<K>;
}

export const NEUTRAL_ACCENT = RARITY_HEX_MUTED[1] as string;
const LEADER_ACCENT = "#dc4d56";

const CLASS_OPTIONS: OperatorProfession[] = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];
const RARITY_OPTIONS = [6, 5, 4, 3, 2, 1] as const;
const SKILL_SLOTS = [1, 2, 3];
const MODULE_LETTERS = ["X", "Y", "D", "A", "B"];
const EVENT_TYPES = ["SIDESTORY", "MINISTORY", "BRANCHLINE", "NONE"];
/** The Integrated Strategies item types in the order the type filter lists them; a type the data adds later follows these. */
const IS_ITEM_ORDER = ["theme", "band", "relic", "active_tool", "explore_tool", "capsule", "totem", "fragment", "wrath", "copper"];

/** The non-empty values among `parts`. */
function present(...parts: (string | null | undefined)[]): string[] {
    return parts.filter((part): part is string => Boolean(part));
}

/** The distinct non-empty values `read` gives over `entities`, in first-seen order. */
function distinct<E>(entities: readonly E[], read: (entity: E) => string | null): string[] {
    return [...new Set(entities.map(read).filter((value): value is string => Boolean(value)))];
}

/** The pool labels of every kind but the operator: the dialog repeats the kicker, and the drop area is worded for any tile. */
function poolLabels(t: KindT, kicker: string, searchLabel: string, gridLabel: string) {
    return { kicker, dialogTitle: kicker, searchLabel, gridLabel, dropArea: t("edit.pool.dropArea.any") };
}

function classFacet<K extends TierEntityKind>(t: KindT, read: (entity: ITierEntityOf<K>) => string | null): IKindFacet<K> {
    return {
        id: "class",
        label: t("edit.pool.class"),
        groupLabel: t("edit.pool.class.group"),
        variant: "icon",
        options: CLASS_OPTIONS.map((c) => ({ value: c, label: formatProfession(c), icon: <ClassIcon profession={c} size={16} /> })),
        valueOf: read,
    };
}

function rarityFacet<K extends TierEntityKind>(t: KindT, read: (entity: ITierEntityOf<K>) => string | null): IKindFacet<K> {
    return {
        id: "rarity",
        label: t("edit.pool.rarity"),
        groupLabel: t("edit.pool.rarity.group"),
        variant: "mono",
        options: RARITY_OPTIONS.map((r) => ({ value: String(r), label: `${r}★`, ariaLabel: t("edit.pool.rarity.option", { rarity: r }) })),
        valueOf: read,
    };
}

function enemyLevelLabel(t: KindT, level: ITierEntityOf<"enemy">["level"]): string {
    if (level === "BOSS") return t("entity.enemyLevel.boss");
    if (level === "ELITE") return t("entity.enemyLevel.elite");
    return t("entity.enemyLevel.normal");
}

function factionLevelLabel(t: KindT, level: ITierEntityOf<"faction">["powerLevel"]): string {
    if (level === "group") return t("entity.factionLevel.group");
    if (level === "team") return t("entity.factionLevel.team");
    return t("entity.factionLevel.nation");
}

function bondTypeLabel(t: KindT, bondType: ITierEntityOf<"stronghold_bond">["bondType"]): string {
    return bondType === "season" ? t("entity.bondType.season") : t("entity.bondType.regular");
}

/** An Archives shelf's name; an unknown shelf reads as Other. */
function eventTypeLabel(t: KindT, displayType: string): string {
    switch (displayType) {
        case "SIDESTORY":
            return t("entity.eventType.SIDESTORY");
        case "MINISTORY":
            return t("entity.eventType.MINISTORY");
        case "BRANCHLINE":
            return t("entity.eventType.BRANCHLINE");
        default:
            return t("entity.eventType.NONE");
    }
}

/** `IS3` for `rogue_2`; an id that is not a numbered theme reads as itself. */
function isThemeLabel(t: KindT, theme: string): string {
    const n = integratedStrategiesNumber(theme);
    return n === null ? theme : t("entity.isTheme", { number: n });
}

/** An Integrated Strategies item type's name; a type the data adds later reads as its raw id. */
function isItemLabel(t: KindT, itemType: string): string {
    switch (itemType) {
        case "theme":
            return t("entity.isItem.theme");
        case "band":
            return t("entity.isItem.band");
        case "relic":
            return t("entity.isItem.relic");
        case "active_tool":
            return t("entity.isItem.active_tool");
        case "explore_tool":
            return t("entity.isItem.explore_tool");
        case "capsule":
            return t("entity.isItem.capsule");
        case "totem":
            return t("entity.isItem.totem");
        case "fragment":
            return t("entity.isItem.fragment");
        case "wrath":
            return t("entity.isItem.wrath");
        case "copper":
            return t("entity.isItem.copper");
        default:
            return itemType;
    }
}

function spriteSourceLabel(t: KindT, source: ITierEntityOf<"story_sprite">["source"]): string {
    return source === "operator" ? t("entity.spriteSource.operator") : t("entity.spriteSource.npc");
}

/** The game prints the `D` module as Delta. */
function moduleLetter(letter: string | null): string | null {
    return letter === "D" ? "Δ" : letter;
}

function skillSlotLabel(t: KindT, slot: number): string {
    return t("entity.skill.slot", { slot });
}

function rarityAccent(rarity: number): string {
    return RARITY_HEX_MUTED[rarity] ?? NEUTRAL_ACCENT;
}

/** Each kind's presentation. Kept in `ALL_ENTITY_KINDS` order, like its strings. */
export const KIND_DEFINITIONS: { [K in TierEntityKind]: IKindDefinition<K> } = {
    operator: {
        plural: (t) => t("entity.kinds.operator"),
        singular: (t) => t("entity.kind.operator"),
        description: (t) => t("edit.kinds.desc.operator"),
        accent: (e) => rarityAccent(e.rarity),
        detail: (e) => [formatProfession(e.profession)],
        pool: (t) => ({
            kicker: t("edit.pool.kicker"),
            dialogTitle: t("edit.pool.dialogTitle"),
            searchLabel: t("edit.pool.search.label"),
            gridLabel: t("edit.pool.gridLabel"),
            dropArea: t("edit.pool.dropArea"),
            facets: [rarityFacet(t, (e) => String(e.rarity)), classFacet(t, (e) => e.profession)],
            searchTexts: (e) => [e.name, e.appellation],
            compare: (a, b) => b.rarity - a.rarity || a.name.localeCompare(b.name),
        }),
    },
    skill: {
        plural: (t) => t("entity.kinds.skill"),
        singular: (t) => t("entity.kind.skill"),
        description: (t) => t("edit.kinds.desc.skill"),
        detail: (e, t) => [...present(e.operatorName), skillSlotLabel(t, e.slot)],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.skill"), t("edit.pool.search.skill"), t("edit.pool.grid.skill")),
            facets: [
                {
                    id: "slot",
                    label: t("edit.pool.slot"),
                    groupLabel: t("edit.pool.slot.group"),
                    variant: "mono",
                    options: SKILL_SLOTS.map((n) => ({ value: String(n), label: `S${n}`, ariaLabel: skillSlotLabel(t, n) })),
                    valueOf: (e) => String(e.slot),
                },
                classFacet(t, (e) => e.profession),
                {
                    id: "sp",
                    label: t("edit.pool.sp"),
                    groupLabel: t("edit.pool.sp.group"),
                    variant: "text",
                    options: [
                        { value: "auto", label: t("edit.pool.sp.auto") },
                        { value: "offensive", label: t("edit.pool.sp.offensive") },
                        { value: "defensive", label: t("edit.pool.sp.defensive") },
                        { value: "passive", label: t("edit.pool.sp.passive") },
                    ],
                    valueOf: (e) => e.spType ?? "passive",
                },
            ],
            searchTexts: (e) => [e.name, e.operatorName],
        }),
    },
    module: {
        plural: (t) => t("entity.kinds.module"),
        singular: (t) => t("entity.kind.module"),
        description: (t) => t("edit.kinds.desc.module"),
        detail: (e) => {
            const letter = moduleLetter(e.moduleType);
            const code = e.typeCode && letter ? `${e.typeCode}-${letter}` : (letter ?? e.typeCode);
            return present(e.operatorName, code);
        },
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.module"), t("edit.pool.search.module"), t("edit.pool.grid.module")),
            facets: [
                {
                    id: "letter",
                    label: t("edit.pool.type"),
                    groupLabel: t("edit.pool.type.group"),
                    variant: "mono",
                    options: MODULE_LETTERS.map((v) => ({ value: v, label: moduleLetter(v) ?? v })),
                    valueOf: (e) => e.moduleType,
                },
                classFacet(t, (e) => e.profession),
            ],
            searchTexts: (e) => [e.name, e.operatorName, e.typeCode],
        }),
    },
    skin: {
        plural: (t) => t("entity.kinds.skin"),
        singular: (t) => t("entity.kind.skin"),
        description: (t) => t("edit.kinds.desc.skin"),
        accent: (e) => rarityAccent(e.rarity),
        detail: (e) => present(e.operatorName, e.brand),
        pool: (t, catalogue) => ({
            ...poolLabels(t, t("edit.pool.kicker.skin"), t("edit.pool.search.skin"), t("edit.pool.grid.skin")),
            facets: [
                rarityFacet(t, (e) => String(e.rarity)),
                classFacet(t, (e) => e.profession),
                {
                    id: "brand",
                    label: t("edit.pool.brand"),
                    groupLabel: t("edit.pool.brand.group"),
                    variant: "text",
                    options: distinct(catalogue, (e) => e.brand)
                        .sort((a, b) => a.localeCompare(b))
                        .map((b) => ({ value: b, label: b })),
                    valueOf: (e) => e.brand,
                },
            ],
            searchTexts: (e) => [e.name, e.operatorName, e.brand],
        }),
    },
    class: {
        plural: (t) => t("entity.kinds.class"),
        singular: (t) => t("entity.kind.class"),
        description: (t) => t("edit.kinds.desc.class"),
        detail: () => [],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.class"), t("edit.pool.search.class"), t("edit.pool.grid.class")),
            facets: [],
            searchTexts: (e) => [e.name],
        }),
    },
    subclass: {
        plural: (t) => t("entity.kinds.subclass"),
        singular: (t) => t("entity.kind.subclass"),
        description: (t) => t("edit.kinds.desc.subclass"),
        detail: (e) => [formatProfession(e.profession)],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.subclass"), t("edit.pool.search.subclass"), t("edit.pool.grid.subclass")),
            facets: [classFacet(t, (e) => e.profession)],
            searchTexts: (e) => [e.name],
        }),
    },
    faction: {
        plural: (t) => t("entity.kinds.faction"),
        singular: (t) => t("entity.kind.faction"),
        description: (t) => t("edit.kinds.desc.faction"),
        detail: (e, t) => [factionLevelLabel(t, e.powerLevel)],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.faction"), t("edit.pool.search.faction"), t("edit.pool.grid.faction")),
            facets: [
                {
                    id: "level",
                    label: t("edit.pool.level"),
                    groupLabel: t("edit.pool.level.group"),
                    variant: "text",
                    options: (["nation", "group", "team"] as const).map((v) => ({ value: v, label: factionLevelLabel(t, v) })),
                    valueOf: (e) => e.powerLevel,
                },
            ],
            searchTexts: (e) => [e.name],
        }),
    },
    enemy: {
        plural: (t) => t("entity.kinds.enemy"),
        singular: (t) => t("entity.kind.enemy"),
        description: (t) => t("edit.kinds.desc.enemy"),
        accent: (e) => {
            if (e.level === "BOSS") return LEADER_ACCENT;
            if (e.level === "ELITE") return rarityAccent(5);
            return NEUTRAL_ACCENT;
        },
        page: "/enemies/$id",
        detail: (e, t) => [enemyLevelLabel(t, e.level), ...present(e.index)],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.enemy"), t("edit.pool.search.enemy"), t("edit.pool.grid.enemy")),
            facets: [
                {
                    id: "rank",
                    label: t("edit.pool.rank"),
                    groupLabel: t("edit.pool.rank.group"),
                    variant: "text",
                    options: (["NORMAL", "ELITE", "BOSS"] as const).map((v) => ({ value: v, label: enemyLevelLabel(t, v) })),
                    valueOf: (e) => e.level,
                },
            ],
            searchTexts: (e) => [e.name, e.index],
        }),
    },
    event: {
        plural: (t) => t("entity.kinds.event"),
        singular: (t) => t("entity.kind.event"),
        description: (t) => t("edit.kinds.desc.event"),
        shape: "wide",
        detail: (e, t) => [eventTypeLabel(t, e.displayType), ...(e.rerun ? [t("entity.event.rerun")] : [])],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.event"), t("edit.pool.search.event"), t("edit.pool.grid.event")),
            facets: [
                {
                    id: "type",
                    label: t("edit.pool.type"),
                    groupLabel: t("edit.pool.type.group"),
                    variant: "text",
                    options: EVENT_TYPES.map((v) => ({ value: v, label: eventTypeLabel(t, v) })),
                    valueOf: (e) => e.displayType,
                },
                {
                    id: "edition",
                    label: t("edit.pool.edition"),
                    groupLabel: t("edit.pool.edition.group"),
                    variant: "text",
                    options: [
                        { value: "original", label: t("edit.pool.edition.original") },
                        { value: "rerun", label: t("entity.event.rerun") },
                    ],
                    valueOf: (e) => (e.rerun ? "rerun" : "original"),
                },
            ],
            searchTexts: (e) => [e.name],
        }),
    },
    integrated_strategies: {
        plural: (t) => t("entity.kinds.integrated_strategies"),
        singular: (t) => t("entity.kind.integrated_strategies"),
        description: (t) => t("edit.kinds.desc.integrated_strategies"),
        detail: (e, t) => (e.itemType === "theme" ? [isThemeLabel(t, e.theme)] : [isThemeLabel(t, e.theme), isItemLabel(t, e.itemType)]),
        pool: (t, catalogue) => {
            const inCatalogue = new Set(distinct(catalogue, (e) => e.itemType));
            const itemTypes = [...IS_ITEM_ORDER.filter((v) => inCatalogue.has(v)), ...[...inCatalogue].filter((v) => !IS_ITEM_ORDER.includes(v))];
            return {
                ...poolLabels(t, t("edit.pool.kicker.integrated_strategies"), t("edit.pool.search.integrated_strategies"), t("edit.pool.grid.integrated_strategies")),
                facets: [
                    {
                        id: "theme",
                        label: t("edit.pool.theme"),
                        groupLabel: t("edit.pool.theme.group"),
                        variant: "mono",
                        options: distinct(catalogue, (e) => e.theme).map((v) => ({ value: v, label: isThemeLabel(t, v) })),
                        valueOf: (e) => e.theme,
                    },
                    {
                        id: "type",
                        label: t("edit.pool.type"),
                        groupLabel: t("edit.pool.type.group"),
                        variant: "text",
                        options: itemTypes.map((v) => ({ value: v, label: isItemLabel(t, v) })),
                        valueOf: (e) => e.itemType,
                    },
                ],
                searchTexts: (e) => [e.name],
            };
        },
    },
    stronghold_bond: {
        plural: (t) => t("entity.kinds.stronghold_bond"),
        singular: (t) => t("entity.kind.stronghold_bond"),
        description: (t) => t("edit.kinds.desc.stronghold_bond"),
        detail: (e, t) => [bondTypeLabel(t, e.bondType)],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.stronghold_bond"), t("edit.pool.search.stronghold_bond"), t("edit.pool.grid.stronghold_bond")),
            facets: [
                {
                    id: "type",
                    label: t("edit.pool.type"),
                    groupLabel: t("edit.pool.type.group"),
                    variant: "text",
                    options: (["season", "regular"] as const).map((v) => ({ value: v, label: bondTypeLabel(t, v) })),
                    valueOf: (e) => e.bondType,
                },
            ],
            searchTexts: (e) => [e.name],
        }),
    },
    story_sprite: {
        plural: (t) => t("entity.kinds.story_sprite"),
        singular: (t) => t("entity.kind.story_sprite"),
        description: (t) => t("edit.kinds.desc.story_sprite"),
        detail: (e, t) => [spriteSourceLabel(t, e.source)],
        pool: (t) => ({
            ...poolLabels(t, t("edit.pool.kicker.story_sprite"), t("edit.pool.search.story_sprite"), t("edit.pool.grid.story_sprite")),
            facets: [
                {
                    id: "source",
                    label: t("edit.pool.source"),
                    groupLabel: t("edit.pool.source.group"),
                    variant: "text",
                    options: (["operator", "npc"] as const).map((v) => ({ value: v, label: spriteSourceLabel(t, v) })),
                    valueOf: (e) => e.source,
                },
            ],
            searchTexts: (e) => [e.name, e.id],
        }),
    },
};

/** A kind's definition, typed for any entity of that kind. The cast joins the two: TypeScript cannot correlate `KIND_DEFINITIONS[kind]` with the entity's own member of the union. */
export function kindDefinition(kind: TierEntityKind): IKindDefinition<TierEntityKind> {
    return KIND_DEFINITIONS[kind] as unknown as IKindDefinition<TierEntityKind>;
}

/** The site page of an entity whose kind has one. */
export function entityPage(entity: ITierEntity): EntityPageRoute | null {
    return entity.resolved ? (KIND_DEFINITIONS[entity.kind].page ?? null) : null;
}

/**
 * The `t` the definitions render through. The i18n extractor binds `t` per
 * file, so this hook's `useT` binding is also what makes every definition's
 * `t("...")` call above count as a use of its key.
 */
export function useKindT(): KindT {
    const t: KindT = useT("tierLists");
    return t;
}

/** The translated labels every kind's tile, hover card, pool and settings read. */
export function useEntityLabels() {
    const t = useKindT();
    return {
        /** `Enemies`, for the pool tabs and the kinds dialog. */
        plural: (kind: TierEntityKind) => KIND_DEFINITIONS[kind].plural(t),
        /** `Enemy`, for tile names and hover cards. */
        singular: (kind: TierEntityKind) => KIND_DEFINITIONS[kind].singular(t),
        description: (kind: TierEntityKind) => KIND_DEFINITIONS[kind].description(t),
        /** One line of what kind of thing this is. Empty for an unresolved placement. */
        detail: (entity: ITierEntity): string[] => (entity.resolved ? kindDefinition(entity.kind).detail(entity, t) : []),
        /** The tile's accessible name: its name and kind, with the owning operator for a skin, module or skill. */
        tileLabel: (entity: ITierEntity): string => {
            if (!entity.resolved) return entity.id;
            const owner = entityOwner(entity);
            const kind = KIND_DEFINITIONS[entity.kind].singular(t);
            return owner ? t("entity.tile.labelOwned", { name: entity.name, owner, kind }) : t("entity.tile.label", { name: entity.name, kind });
        },
        openPage: t("entity.preview.open"),
    };
}
