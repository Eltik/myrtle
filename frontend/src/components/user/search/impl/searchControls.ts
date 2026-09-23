import { CLASSES } from "#/components/operators/list/impl/constants";
import type { messages as sortMessages } from "./searchControls.messages";

/**
 * The `sort` tokens `GET /search` ranks by. `score` is today's order and the
 * only sort whose rows carry no `metric`. The two scoped shapes, `class:X` and
 * `sub:Y`, count owned operators of a class or an archetype and are built with
 * {@link scopeToken}.
 */
export const METRIC_SORTS = ["score", "operators", "joined", "enemies", "potentials", "masteries", "modules", "skins"] as const;
export type MetricSort = (typeof METRIC_SORTS)[number];
export type SortDir = "asc" | "desc";

export const DEFAULT_SORT: MetricSort = "score";

export const METRIC_SORT_LABEL_KEYS: Record<MetricSort, keyof typeof sortMessages> = {
    score: "search.sort.score",
    operators: "search.sort.operators",
    joined: "search.sort.joined",
    enemies: "search.sort.enemies",
    potentials: "search.sort.potentials",
    masteries: "search.sort.masteries",
    modules: "search.sort.modules",
    skins: "search.sort.skins",
};

/** A class or an archetype: what a scoped sort counts and what the `all` filter requires. */
export type Scope = { kind: "class"; profession: string } | { kind: "sub"; subProfessionId: string };

/** Mirrors `MAX_HAS` in `backend/src/app/services/search.rs`: past it the endpoint answers 400. */
export const MAX_HAS = 20;

const SUB_PROFESSION_ID = /^[a-z0-9_]+$/;
const OPERATOR_ID = /^[a-z0-9_]+$/i;

export function isMetricSort(value: string): value is MetricSort {
    return (METRIC_SORTS as readonly string[]).includes(value);
}

export function isProfession(value: string): boolean {
    return (CLASSES as readonly string[]).includes(value);
}

/** `class:WARRIOR` or `sub:centurion` to a scope; anything else is null. */
export function parseScope(token: unknown): Scope | null {
    if (typeof token !== "string") return null;
    const [kind, rest, ...extra] = token.split(":");
    if (extra.length > 0 || !rest) return null;
    if (kind === "class" && isProfession(rest)) return { kind: "class", profession: rest };
    if (kind === "sub" && SUB_PROFESSION_ID.test(rest)) return { kind: "sub", subProfessionId: rest };
    return null;
}

export function scopeToken(scope: Scope): string {
    return scope.kind === "class" ? `class:${scope.profession}` : `sub:${scope.subProfessionId}`;
}

/** A valid `sort` token, or `score` for anything else so a stale link still loads. */
export function parseSort(raw: unknown): string {
    if (typeof raw !== "string") return DEFAULT_SORT;
    if (isMetricSort(raw)) return raw;
    return parseScope(raw) ? raw : DEFAULT_SORT;
}

/** A valid `all` scope token, or the empty string (no filter). */
export function parseAll(raw: unknown): string {
    return parseScope(raw) ? (raw as string) : "";
}

export function parseDir(raw: unknown): SortDir | undefined {
    return raw === "asc" || raw === "desc" ? raw : undefined;
}

/** Oldest accounts first is the interesting `joined` order; every count reads high to low. */
export function defaultDir(sort: string): SortDir {
    return sort === "joined" ? "asc" : "desc";
}

export function parseOperatorId(raw: unknown): string {
    return typeof raw === "string" && OPERATOR_ID.test(raw) ? raw : "";
}

/** Comma-separated operator ids, malformed ones and duplicates dropped, back to a comma-separated string. */
export function parseOperatorIds(raw: unknown): string {
    return splitOperatorIds(raw).join(",");
}

export function splitOperatorIds(raw: unknown): string[] {
    if (typeof raw !== "string" || !raw) return [];
    const seen = new Set<string>();
    for (const part of raw.split(",")) {
        const id = parseOperatorId(part.trim());
        if (id) seen.add(id);
        if (seen.size === MAX_HAS) break;
    }
    return [...seen];
}
