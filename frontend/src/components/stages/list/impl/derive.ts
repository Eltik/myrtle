import { type IStageIndexEntry, type StageVariantKind, stageVariantKind } from "#/lib/api/stages";
import { STAGE_GROUP_LABEL, STAGE_GROUP_TONE, type StageGroupKey, stageGroupOrder } from "#/lib/registry/stage-groups";
import type { StageDifficulty } from "#/types/stages";

/** A harder variant folded into a stage card. */
export interface IStageVariantVM {
    stageId: string;
    kind: StageVariantKind;
    canView: boolean;
}

/** A single stage card in an event row (deduped across difficulty variants). */
export interface IStageCardVM {
    stageId: string;
    /** Short code shown in the badge, when the stage has one (null for IS/RA/Paradox nodes). */
    badge: string | null;
    title: string;
    apCost: number;
    boss: boolean;
    /** CM / Adverse variants that share this card's code, CM first. */
    variants: IStageVariantVM[];
    /** Detail / map viewer can render this stage (links only when true). */
    canView: boolean;
    difficulty: StageDifficulty;
    /** Asset-relative path to the map preview, or null. */
    preview: string | null;
}

/** An expandable event row - one per zone/season. */
export interface IEventVM {
    zoneId: string;
    title: string;
    /** Ordinal kicker for mainline episodes ("Episode 1"), null otherwise. */
    kicker: string | null;
    /** Zone/event banner key-art (asset-relative path), or null. */
    banner: string | null;
    codeRange: string;
    statusLabel: string;
    bossCount: number;
    stageCount: number;
    /** Originite Prime from first clears of every stage in the zone, CM and Adverse included. */
    originitePrime: number;
    /** Sort key within a group. */
    order: number;
    tone: string;
    group: StageGroupKey;
    haystack: string;
    stages: IStageCardVM[];
}

/** A category section on the timeline spine. */
export interface IGroupVM {
    key: StageGroupKey;
    label: string;
    tone: string;
    zoneCount: number;
    stageCount: number;
    events: IEventVM[];
}

export interface IStageTree {
    groups: IGroupVM[];
    totalStages: number;
    totalZones: number;
    featured: IEventVM | null;
}

/** Whether a string reads as a short in-game stage code (`1-7`, `LT-5`, `01-02`). */
function codeLike(s: string | null | undefined): s is string {
    return !!s && s.length <= 8 && !/\s/.test(s);
}

/**
 * Split the classifier's `(code, name)` into a badge + title. Story/event/CC
 * stages have a short code and a descriptive name; IS/RA nodes carry the
 * descriptive label in `code` and the short code in `name`, so pick whichever
 * looks like a code for the badge.
 */
function labelParts(code: string, name: string | null | undefined): { badge: string | null; title: string } {
    const nm = name ?? "";
    if (codeLike(code)) return { badge: code, title: nm || code };
    if (codeLike(nm)) return { badge: nm, title: code };
    // Neither is a code: the spaced (more descriptive) string is the title.
    const title = /\s/.test(nm) ? nm : /\s/.test(code) ? code : nm || code;
    return { badge: null, title };
}

function naturalKey(s: string): (string | number)[] {
    const out: (string | number)[] = [];
    const re = /(\d+)|(\D+)/g;
    let m: RegExpExecArray | null;
    // biome-ignore lint/suspicious/noAssignInExpressions: standard tokenizer loop
    while ((m = re.exec(s)) !== null) out.push(m[1] !== undefined ? Number(m[1]) : (m[2] ?? "").toLowerCase());
    return out;
}

/** Compare two stage codes with their digit runs as numbers, so `1-10` sorts after `1-9`. */
export function naturalCompare(a: string, b: string): number {
    const ka = naturalKey(a);
    const kb = naturalKey(b);
    const n = Math.min(ka.length, kb.length);
    for (let i = 0; i < n; i++) {
        const x = ka[i] ?? "";
        const y = kb[i] ?? "";
        if (x === y) continue;
        if (typeof x === "number" && typeof y === "number") return x - y;
        return String(x) < String(y) ? -1 : 1;
    }
    return ka.length - kb.length;
}

/** Rank a stage variant for dedupe: viewable first, then NORMAL difficulty. */
function variantRank(e: IStageIndexEntry): number {
    return (e.canView ? 0 : 2) + (e.difficulty === "NORMAL" ? 0 : 1);
}
/** The better of two index entries for one card: the viewable, then the NORMAL, variant; `a` on a tie. */
export function preferEntry(a: IStageIndexEntry, b: IStageIndexEntry): IStageIndexEntry {
    return variantRank(b) < variantRank(a) ? b : a;
}

const KNOWN_GROUPS = new Set<StageGroupKey>(["story", "events", "annihilation", "is", "ra", "sss", "paradox", "cc", "supplies", "other"]);
function asGroupKey(g: string): StageGroupKey {
    return KNOWN_GROUPS.has(g as StageGroupKey) ? (g as StageGroupKey) : "other";
}

/** Fallback title for zones without a display name: "guide_1" -> "Guide 1". */
function humanizeZoneId(id: string): string {
    return id
        .replace(/[_-]+/g, " ")
        .trim()
        .replace(/\b[a-z]/g, (c) => c.toUpperCase());
}

interface IZoneAccum {
    zoneName: string;
    order: number;
    group: StageGroupKey;
    banner: string | null;
    /** code (badge or title) -> every entry with that code. */
    byCode: Map<string, IStageIndexEntry[]>;
    originitePrime: number;
}

/**
 * Build the category -> event(zone) -> stage tree from the backend's authoritative
 * stage index. Grouping/labelling comes straight from the index; the frontend
 * only dedupes CM variants, orders, and formats.
 */
export function buildStageTree(entries: IStageIndexEntry[]): IStageTree {
    const zones = new Map<string, IZoneAccum>();

    for (const e of entries) {
        const group = asGroupKey(e.group);
        let zone = zones.get(e.zoneId);
        if (!zone) {
            zone = { zoneName: e.zoneName || humanizeZoneId(e.zoneId), order: e.zoneOrder, group, banner: e.banner ?? null, byCode: new Map(), originitePrime: 0 };
            zones.set(e.zoneId, zone);
        } else if (!zone.banner && e.banner) {
            zone.banner = e.banner;
        }
        zone.originitePrime += e.originitePrime;
        const key = e.code || e.stageId;
        const existing = zone.byCode.get(key);
        if (existing) existing.push(e);
        else zone.byCode.set(key, [e]);
    }

    const eventsByGroup = new Map<StageGroupKey, IEventVM[]>();
    for (const [zoneId, zone] of zones) {
        const cards: IStageCardVM[] = [];
        for (const siblings of zone.byCode.values()) {
            // Some chapters carry a retired "Story mode" (easy_*) stage with the
            // Standard one's code but no level file, so prefer the viewable, then
            // NORMAL, variant and the card links to a playable stage.
            const entry = siblings.reduce(preferEntry);
            const variants: IStageVariantVM[] = [];
            for (const v of siblings) {
                const kind = v === entry ? null : stageVariantKind(v);
                if (kind) variants.push({ stageId: v.stageId, kind, canView: v.canView });
            }
            variants.sort((a, b) => (a.kind === b.kind ? 0 : a.kind === "cm" ? -1 : 1));
            const { badge, title } = labelParts(entry.code, entry.name);
            cards.push({
                stageId: entry.stageId,
                badge,
                title,
                apCost: entry.apCost,
                boss: entry.boss,
                variants,
                canView: entry.canView,
                difficulty: entry.difficulty,
                preview: entry.preview ?? null,
            });
        }
        if (cards.length === 0) continue;
        cards.sort((a, b) => naturalCompare(a.badge ?? a.title, b.badge ?? b.title));

        const badges = cards.map((c) => c.badge).filter(codeLike);
        const first = badges[0];
        const last = badges[badges.length - 1];
        const codeRange = first !== undefined && last !== undefined ? (first === last ? first : `${first} ~ ${last}`) : "";
        const bossCount = cards.filter((c) => c.boss).length;
        // Mainline zone names split into "Episode 1" (kicker) + the arc name.
        const kicker = zone.group === "story" && /^(episode|prologue|interlude)/i.test(zone.zoneName) ? zone.zoneName : null;
        const haystack = `${zone.zoneName} ${zoneId} ${cards.map((c) => `${c.badge ?? ""} ${c.title}`).join(" ")}`.toLowerCase();

        const event: IEventVM = {
            zoneId,
            title: zone.zoneName,
            kicker,
            banner: zone.banner,
            codeRange,
            statusLabel: STAGE_GROUP_LABEL[zone.group],
            bossCount,
            stageCount: cards.length,
            originitePrime: zone.originitePrime,
            order: zone.order,
            tone: STAGE_GROUP_TONE[zone.group],
            group: zone.group,
            haystack,
            stages: cards,
        };
        const list = eventsByGroup.get(zone.group) ?? [];
        list.push(event);
        eventsByGroup.set(zone.group, list);
    }

    const groups: IGroupVM[] = [];
    let totalStages = 0;
    let totalZones = 0;
    for (const [key, events] of eventsByGroup) {
        // Story reads oldest->newest by episode; everything else newest first.
        if (key === "story") events.sort((a, b) => a.order - b.order || naturalCompare(a.zoneId, b.zoneId));
        else events.sort((a, b) => b.order - a.order || naturalCompare(b.zoneId, a.zoneId));
        const stageCount = events.reduce((n, e) => n + e.stageCount, 0);
        totalStages += stageCount;
        totalZones += events.length;
        groups.push({ key, label: STAGE_GROUP_LABEL[key], tone: STAGE_GROUP_TONE[key], zoneCount: events.length, stageCount, events });
    }
    groups.sort((a, b) => stageGroupOrder(a.key) - stageGroupOrder(b.key));

    // Feature the newest main-theme zone (highest episode number).
    const storyEvents = groups.find((g) => g.key === "story")?.events ?? [];
    const featured = storyEvents.length ? storyEvents.reduce((best, e) => (e.order > best.order ? e : best)) : null;

    return { groups, totalStages, totalZones, featured };
}

/** Best available cover art for an event row/hero/dialog: banner, else first stage preview. */
export function coverImage(event: IEventVM): string | null {
    return event.banner ?? event.stages.find((s) => s.preview)?.preview ?? null;
}

/** Filter the tree by a search query and/or active group, preserving order. */
export function filterTree(tree: IStageTree, query: string, group: StageGroupKey | "all"): IGroupVM[] {
    const q = query.trim().toLowerCase();
    const out: IGroupVM[] = [];
    for (const g of tree.groups) {
        if (group !== "all" && g.key !== group) continue;
        const events = q ? g.events.filter((e) => e.haystack.includes(q)) : g.events;
        if (events.length === 0) continue;
        out.push({ ...g, events });
    }
    return out;
}
