// What a tier list placement points at, as the frontend sees it.
//
// The backend stores a placement as `(entity_kind, entity_id)` and resolves it
// into an `EntitySummary` against the reader's server (see
// `backend/src/app/services/tier_entity.rs`). This module turns that summary
// into the view model every tile, the editor and the stats read. Pure: no
// server functions, so the editor's reducer and its tests can import it.
import type { EntityKind } from "#/types/generated/EntityKind";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import type { FacetValue } from "#/types/generated/FacetValue";
import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";

export type TierEntityKind = EntityKind;

/** What a list offers when the backend sent nothing (an older cache entry). */
export const DEFAULT_ENTITY_KINDS: readonly TierEntityKind[] = ["operator"];

/** Every kind a list may offer, in the order the settings and the pool tabs list them. */
export const ALL_ENTITY_KINDS: readonly TierEntityKind[] = ["operator", "class", "subclass", "faction", "enemy", "event", "stronghold_bond"];

/** The editor's identity for one entity: `${kind}:${id}`. Ids never repeat across kinds under this key. */
export function entityKey(kind: TierEntityKind, id: string): string {
    return `${kind}:${id}`;
}

/** Inverse of {@link entityKey}. Splits at the FIRST colon: kinds never contain one, ids might. */
export function parseEntityKey(key: string): { kind: TierEntityKind; id: string } {
    const at = key.indexOf(":");
    return { kind: key.slice(0, at) as TierEntityKind, id: key.slice(at + 1) };
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

/** A placed operator the served game data knows. */
export interface ITierOperator extends ITierEntityBase {
    kind: "operator";
    resolved: true;
    appellation: string | null;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
    nationId: string | null;
}

/** A placed class (profession). `id` is the wire profession, `WARRIOR`. */
export interface ITierClass extends ITierEntityBase {
    kind: "class";
    resolved: true;
}

/** A placed subclass (sub-profession), with the class it belongs to. */
export interface ITierSubclass extends ITierEntityBase {
    kind: "subclass";
    resolved: true;
    profession: OperatorProfession;
}

export type EnemyLevel = "NORMAL" | "ELITE" | "BOSS";

/** A placed enemy, from the enemy handbook. */
export interface ITierEnemy extends ITierEntityBase {
    kind: "enemy";
    resolved: true;
    level: EnemyLevel;
    /** The handbook code, e.g. `B1`. */
    index: string | null;
}

/** A placed event (activity). Its art is a wide banner. */
export interface ITierEvent extends ITierEntityBase {
    kind: "event";
    resolved: true;
    /** The Archives shelf: `SIDESTORY`, `MINISTORY`, `BRANCHLINE` or `NONE`. */
    displayType: string;
    /** Unix seconds. */
    startTime: number;
    rerun: boolean;
}

export type FactionLevel = "nation" | "group" | "team";

/** A placed faction: a nation, a group inside one, or a team. */
export interface ITierFaction extends ITierEntityBase {
    kind: "faction";
    resolved: true;
    powerLevel: FactionLevel;
}

/** A placed Stronghold Protocol bond. */
export interface ITierStrongholdBond extends ITierEntityBase {
    kind: "stronghold_bond";
    resolved: true;
    bondType: "season" | "regular";
}

/** A placement whose id the served game data does not know. Rendered as a placeholder, never dropped. */
export interface ITierUnresolvedEntity extends ITierEntityBase {
    resolved: false;
}

/** Every resolved kind, one member each. */
export type ITierResolvedEntity = ITierOperator | ITierClass | ITierSubclass | ITierEnemy | ITierEvent | ITierFaction | ITierStrongholdBond;

export type ITierEntity = ITierResolvedEntity | ITierUnresolvedEntity;

export function isOperatorEntity(entity: ITierEntity | null | undefined): entity is ITierOperator {
    return entity?.kind === "operator" && entity.resolved;
}

export interface IPlacementFields {
    subOrder: number;
    description: string | null;
    updatedAt: string;
}

function facet(summary: EntitySummary, name: string): string | null {
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
    const base = { key, id, name: summary.name, icon: summary.icon, href: summary.href, facets: summary.facets, ...placement };
    switch (summary.kind) {
        case "operator":
            return {
                ...base,
                kind: "operator",
                resolved: true,
                appellation: facet(summary, "appellation") || null,
                rarity: (Number(facet(summary, "rarity")) || 1) as OperatorRarity,
                profession: (facet(summary, "profession") ?? "UNKNOWN") as OperatorProfession,
                subProfessionId: facet(summary, "sub_profession_id") ?? "",
                position: (facet(summary, "position") ?? "NONE") as OperatorPosition,
                nationId: facet(summary, "nation_id") || null,
            };
        case "class":
            return { ...base, kind: "class", resolved: true };
        case "subclass":
            return { ...base, kind: "subclass", resolved: true, profession: (facet(summary, "profession") ?? "UNKNOWN") as OperatorProfession };
        case "enemy": {
            const level = facet(summary, "enemy_level");
            return { ...base, kind: "enemy", resolved: true, level: level === "ELITE" || level === "BOSS" ? level : "NORMAL", index: facet(summary, "enemy_index") || null };
        }
        case "event":
            return {
                ...base,
                kind: "event",
                resolved: true,
                displayType: facet(summary, "display_type") ?? "NONE",
                startTime: Number(facet(summary, "start_time")) || 0,
                rerun: facet(summary, "rerun") === "true",
            };
        case "faction": {
            const level = facet(summary, "power_level");
            return { ...base, kind: "faction", resolved: true, powerLevel: level === "group" || level === "team" ? level : "nation" };
        }
        case "stronghold_bond":
            return { ...base, kind: "stronghold_bond", resolved: true, bondType: facet(summary, "bond_type") === "season" ? "season" : "regular" };
    }
}

/** An entity's icon as a URL, from its server-neutral API path. `base` is the backend origin (empty for same-origin). */
export function entityIconURL(icon: string, base: string, server?: string): string {
    const prefix = server ? `/api/${server}` : "/api";
    return `${base}${prefix}${icon}`;
}
