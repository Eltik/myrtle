// What a tier list placement points at, as the frontend sees it.
//
// The backend stores a placement as `(entity_kind, entity_id)` and resolves it
// into an `EntitySummary` against the reader's server (see
// `backend/src/app/services/tier_entity.rs`). This module turns that summary
// into the view model every tile, the editor and the stats read. Pure: no
// server functions, so the editor's reducer and its tests can import it.
//
// The data side of each kind lives in two maps keyed by kind, `IKindFields`
// (its typed fields) and `KIND_MODELS` (how they are read, its art fit, its
// owner). How a kind presents (names, detail line, pool) is `KIND_DEFINITIONS`
// in `components/tier-lists/kinds.tsx`. TypeScript refuses a kind missing from
// any of the three.
import type { EntityKind } from "#/types/generated/EntityKind";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import type { FacetValue } from "#/types/generated/FacetValue";
import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";
import { DEFAULT_GAMEDATA_SERVER } from "./gamedata";

export type TierEntityKind = EntityKind;

/** What a list offers when the backend sent nothing (an older cache entry). */
export const DEFAULT_ENTITY_KINDS: readonly TierEntityKind[] = ["operator"];

/** The editor's identity for one entity: `${kind}:${id}`. Ids never repeat across kinds under this key. */
export function entityKey(kind: TierEntityKind, id: string): string {
    return `${kind}:${id}`;
}

/** Inverse of {@link entityKey}. Splits at the FIRST colon: kinds never contain one, ids might. */
export function parseEntityKey(key: string): { kind: TierEntityKind; id: string } {
    const at = key.indexOf(":");
    return { kind: key.slice(0, at) as TierEntityKind, id: key.slice(at + 1) };
}

/** How many of `keys` are of each kind. */
export function countKeysByKind(keys: Iterable<string>): Partial<Record<TierEntityKind, number>> {
    const counts: Partial<Record<TierEntityKind, number>> = {};
    for (const key of keys) {
        const { kind } = parseEntityKey(key);
        counts[kind] = (counts[kind] ?? 0) + 1;
    }
    return counts;
}

interface ITierEntityBase {
    /** `${kind}:${id}`, see {@link entityKey}. */
    key: string;
    kind: TierEntityKind;
    id: string;
    name: string;
    /** Image path under the API root, e.g. `/avatar/char_002_amiya`. */
    icon: string | null;
    /** Site route of the entity's own page. */
    href: string | null;
    facets: Partial<Record<string, FacetValue>>;
    subOrder: number;
    /** Editor-authored blurb explaining why this entity sits where it does. */
    description: string | null;
    /** ISO timestamp of when this placement was last updated. */
    updatedAt: string;
}

type EnemyLevel = "NORMAL" | "ELITE" | "BOSS";

type FactionLevel = "nation" | "group" | "team";

/** Each kind's own fields, on top of the ones every entity has. */
interface IKindFields {
    operator: {
        appellation: string | null;
        rarity: OperatorRarity;
        profession: OperatorProfession;
        subProfessionId: string;
        position: OperatorPosition;
        nationId: string | null;
    };
    /** One operator's skill slot, so a shared generic skill is one entry per operator. */
    skill: {
        charId: string;
        operatorName: string | null;
        profession: OperatorProfession | null;
        /** 1-based skill slot. */
        slot: number;
        /** `manual`, `auto` or `passive`. */
        skillType: string | null;
        /** SP recovery: `auto`, `offensive` or `defensive`; `null` for a passive. */
        spType: string | null;
    };
    module: {
        charId: string;
        operatorName: string | null;
        profession: OperatorProfession | null;
        /** The module's letter: `X`, `Y`, `D` (shown as Delta), `A`, `B`. */
        moduleType: string | null;
        /** The subclass code the game prints before the letter, e.g. `SUM`. */
        typeCode: string | null;
    };
    /** An operator outfit. Its art is the outfit's avatar. */
    skin: {
        /** The wearer's char id, and their name as the roster spells it. */
        charId: string;
        operatorName: string | null;
        rarity: OperatorRarity;
        profession: OperatorProfession | null;
        /** The store brand the outfit is filed under (`EPOQUE`), absent for collaborations. */
        brand: string | null;
    };
    /** A class (profession). `id` is the wire profession, `WARRIOR`. */
    class: Record<never, never>;
    /** A subclass (sub-profession), with the class it belongs to. */
    subclass: {
        profession: OperatorProfession;
    };
    /** A nation, a group inside one, or a team. */
    faction: {
        powerLevel: FactionLevel;
    };
    /** An enemy from the enemy handbook. */
    enemy: {
        level: EnemyLevel;
        /** The handbook code, e.g. `B1`. */
        index: string | null;
    };
    /** An event (activity). Its art is a wide banner. */
    event: {
        /** The Archives shelf: `SIDESTORY`, `MINISTORY`, `BRANCHLINE` or `NONE`. */
        displayType: string;
        /** Unix seconds. */
        startTime: number;
        rerun: boolean;
    };
    /** A main story episode. Its art is the Story Collection key visual, a square poster. */
    main_story: {
        /** The EPISODE number, `0` up. */
        episode: number | null;
        /** The act's `chapter_table` index, and its name (`Hour of An Awakening`). */
        act: number | null;
        actName: string | null;
    };
    /** An Integrated Strategies theme or item. */
    integrated_strategies: {
        /** The game's theme id, `rogue_N`. The game's own numbering is N + 1 (`rogue_1` is IS2). */
        theme: string;
        /** `theme`, or the item type in lowercase: `relic`, `band`, `totem`, ... */
        itemType: string;
    };
    /** A Stronghold Protocol bond. */
    stronghold_bond: {
        bondType: "season" | "regular";
    };
    /** A story character sprite. Its icon is the backend's head-and-shoulders thumbnail, already cropped. */
    story_sprite: {
        source: "operator" | "npc";
    };
}

type EntityOfKind<K extends TierEntityKind> = ITierEntityBase & { kind: K; resolved: true } & IKindFields[K];

/** Every resolved kind, one member each. */
export type ITierResolvedEntity = { [K in TierEntityKind]: EntityOfKind<K> }[TierEntityKind];

/** The resolved entity of one kind. */
export type ITierEntityOf<K extends TierEntityKind> = Extract<ITierResolvedEntity, { kind: K }>;

/** A placed operator the served game data knows. */
export type ITierOperator = ITierEntityOf<"operator">;

/** A placement whose id the served game data does not know. Rendered as a placeholder, never dropped. */
export interface ITierUnresolvedEntity extends ITierEntityBase {
    resolved: false;
}

export type ITierEntity = ITierResolvedEntity | ITierUnresolvedEntity;

export function isEntityOfKind<K extends TierEntityKind>(entity: ITierEntity | null | undefined, kind: K): entity is ITierEntityOf<K> {
    return entity?.kind === kind && entity.resolved;
}

export function isOperatorEntity(entity: ITierEntity | null | undefined): entity is ITierOperator {
    return isEntityOfKind(entity, "operator");
}

/** The `rogue_N` theme's player-facing number: IS1 never shipped in the data, so `rogue_1` is IS2. */
export function integratedStrategiesNumber(theme: string): number | null {
    const n = Number(theme.replace(/^rogue_/, ""));
    return Number.isInteger(n) && n > 0 ? n + 1 : null;
}

/**
 * How a kind's art sits in its square tile.
 * - `cover`: a picture that fills the tile edge to edge (an avatar, a banner, a theme's key visual, module art, a story sprite's head).
 * - `object`: a small object on transparency in its own aspect (a relic, a squad badge, a module type badge): drawn whole, slightly inset.
 * - `glyph`: a white glyph on transparency (class, faction, bond icons): drawn further inset and inverted for the light theme.
 */
export type ArtFit = "cover" | "object" | "glyph";

/** One facet of a summary as a single string: the first of several, `null` when absent. */
type FacetReader = (name: string) => string | null;

interface IKindModel<K extends TierEntityKind> {
    /** How the kind's art sits in its tile. Every kind's art is square on EN 2026-10-01 except an event's banner, which has its own wide tile. */
    artFit: ArtFit;
    /** A fit read off the entity itself, for a kind whose art comes in more than one style. */
    entityArtFit?: (entity: ITierEntityOf<K>) => ArtFit;
    /** The operator the entity belongs to, by roster name, so "Stick and Sack" reads as whose it is. */
    owner?: (entity: ITierEntityOf<K>) => string | null;
    /** The kind's own fields, from its summary's facets. */
    fields: (facet: FacetReader) => IKindFields[K];
}

/** A numeric facet, `null` when absent. */
function optionalNumber(value: string | null): number | null {
    if (value === null || value === "") return null;
    const n = Number(value);
    return Number.isFinite(n) ? n : null;
}

/** A module whose own art the extract lacks falls back to its type badge (72 x 52), which is an object, not a picture. */
const MODULE_BADGE_DIR = "ui_equip_type_direction";

/** Each kind's data model, in the order the settings and the pool tabs list kinds. */
const KIND_MODELS: { [K in TierEntityKind]: IKindModel<K> } = {
    operator: {
        artFit: "cover",
        fields: (facet) => ({
            appellation: facet("appellation") || null,
            rarity: (Number(facet("rarity")) || 1) as OperatorRarity,
            profession: (facet("profession") ?? "UNKNOWN") as OperatorProfession,
            subProfessionId: facet("sub_profession_id") ?? "",
            position: (facet("position") ?? "NONE") as OperatorPosition,
            nationId: facet("nation_id") || null,
        }),
    },
    skill: {
        artFit: "cover",
        owner: (entity) => entity.operatorName,
        fields: (facet) => ({
            charId: facet("char_id") ?? "",
            operatorName: facet("operator"),
            profession: facet("profession") as OperatorProfession | null,
            slot: Number(facet("slot")) || 1,
            skillType: facet("skill_type"),
            spType: facet("sp_type"),
        }),
    },
    module: {
        // `ui_equip_big_img_hub`, 511 or 512 px square.
        artFit: "cover",
        entityArtFit: (entity) => (entity.icon?.includes(MODULE_BADGE_DIR) ? "object" : "cover"),
        owner: (entity) => entity.operatorName,
        fields: (facet) => ({
            charId: facet("char_id") ?? "",
            operatorName: facet("operator"),
            profession: facet("profession") as OperatorProfession | null,
            moduleType: facet("module_type"),
            typeCode: facet("type_code"),
        }),
    },
    skin: {
        artFit: "cover",
        owner: (entity) => entity.operatorName,
        fields: (facet) => ({
            charId: facet("char_id") ?? "",
            operatorName: facet("operator"),
            rarity: (Number(facet("rarity")) || 1) as OperatorRarity,
            profession: facet("profession") as OperatorProfession | null,
            brand: facet("brand"),
        }),
    },
    class: {
        artFit: "glyph",
        fields: () => ({}),
    },
    subclass: {
        artFit: "glyph",
        fields: (facet) => ({ profession: (facet("profession") ?? "UNKNOWN") as OperatorProfession }),
    },
    faction: {
        artFit: "glyph",
        fields: (facet) => {
            const level = facet("power_level");
            return { powerLevel: level === "group" || level === "team" ? level : "nation" };
        },
    },
    enemy: {
        artFit: "cover",
        fields: (facet) => {
            const level = facet("enemy_level");
            return { level: level === "ELITE" || level === "BOSS" ? level : "NORMAL", index: facet("enemy_index") || null };
        },
    },
    event: {
        artFit: "cover",
        fields: (facet) => ({
            displayType: facet("display_type") ?? "NONE",
            startTime: Number(facet("start_time")) || 0,
            rerun: facet("rerun") === "true",
        }),
    },
    main_story: {
        // The library's 432 px square key visual, title typeset in.
        artFit: "cover",
        fields: (facet) => ({ episode: optionalNumber(facet("episode")), act: optionalNumber(facet("act")), actName: facet("act_name") }),
    },
    integrated_strategies: {
        // Items: relics, tools, foldartals and Tongbao are 184 px objects with transparent margins; squads, capsules and Wrath badges vary in aspect.
        artFit: "object",
        // A theme's art is not an item but its home key visual, 1000 px square.
        entityArtFit: (entity) => (entity.itemType === "theme" ? "cover" : "object"),
        fields: (facet) => ({ theme: facet("theme") ?? "", itemType: facet("item_type") ?? "" }),
    },
    stronghold_bond: {
        artFit: "glyph",
        fields: (facet) => ({ bondType: facet("bond_type") === "season" ? "season" : "regular" }),
    },
    story_sprite: {
        artFit: "cover",
        fields: (facet) => ({ source: facet("source") === "operator" ? "operator" : "npc" }),
    },
};

/** Every kind a list may offer, in the order the settings and the pool tabs list them. */
export const ALL_ENTITY_KINDS = Object.keys(KIND_MODELS) as readonly TierEntityKind[];

/** A kind's model, typed for any entity of that kind. The cast joins the two: TypeScript cannot correlate `KIND_MODELS[kind]` with the entity's own member of the union. */
function modelOf(kind: TierEntityKind): IKindModel<TierEntityKind> {
    return KIND_MODELS[kind] as unknown as IKindModel<TierEntityKind>;
}

/** The operator a skin, module or skill belongs to, by roster name. `null` for every other kind, or when the data names no one. */
export function entityOwner(entity: ITierEntity): string | null {
    if (!entity.resolved) return null;
    return modelOf(entity.kind).owner?.(entity) ?? null;
}

/** A kind's art fit when only the kind is known. */
export function kindArtFit(kind: TierEntityKind): ArtFit {
    return KIND_MODELS[kind].artFit;
}

/** How an entity's art sits in its tile: the kind's fit, refined per entity where the kind's art comes in more than one style. */
export function entityArtFit(entity: ITierEntity): ArtFit {
    if (!entity.resolved) return "cover";
    const model = modelOf(entity.kind);
    return model.entityArtFit?.(entity) ?? model.artFit;
}

export interface IPlacementFields {
    subOrder: number;
    description: string | null;
    updatedAt: string;
}

/** The placement fields of an entity that is offered but not placed (a catalogue entry), or whose placement nothing records. */
export const UNPLACED: IPlacementFields = { subOrder: 0, description: null, updatedAt: new Date(0).toISOString() };

function firstFacetValue(summary: EntitySummary, name: string): string | null {
    const value = summary.facets[name];
    if (typeof value === "string") return value;
    if (Array.isArray(value)) return value[0] ?? null;
    return null;
}

/** One placement as a view model: from the backend's resolved summary, or a placeholder when it has none. */
export function toTierEntity(kind: TierEntityKind, id: string, summary: EntitySummary | null, placement: IPlacementFields): ITierEntity {
    const key = entityKey(kind, id);
    if (!summary) {
        return { key, kind, id, name: id, icon: null, href: null, facets: {}, resolved: false, ...placement };
    }
    const fields = modelOf(summary.kind).fields((name) => firstFacetValue(summary, name));
    // The cast pairs `summary.kind` with the fields its own model read, which TypeScript cannot follow through the lookup.
    return { key, id, name: summary.name, icon: summary.icon, href: summary.href, facets: summary.facets, ...placement, kind: summary.kind, resolved: true, ...fields } as ITierResolvedEntity;
}

/** An entity's icon as a URL, from its server-neutral API path. `base` is the backend origin (empty for same-origin); the default server is served unprefixed. */
export function entityIconURL(icon: string, base: string, server?: string): string {
    const prefix = server && server !== DEFAULT_GAMEDATA_SERVER ? `/api/${server}` : "/api";
    return `${base}${prefix}${icon}`;
}
