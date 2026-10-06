import type { ReactNode } from "react";
import { env } from "#/env";
import { gamedataPath } from "#/lib/api/gamedata";
import { deepCamelize, getOperatorsListFn } from "#/lib/api/operators";
import { stagePreviewAssetPaths } from "#/lib/api/stages";
import type { IRosterEntry } from "#/lib/api/user";
import { backendFetch } from "#/lib/fetch";
import { metaSourceForLocale } from "#/lib/meta";
import { formatNumber, groupLabel, nationLabel, rarityToNumber, teamLabel, toAvatarStem } from "#/lib/utils";
import type { Grid } from "#/types/generated/Grid";
import type { PlacementDetail } from "#/types/generated/PlacementDetail";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import type { IOperatorListItem } from "#/types/operators";
import type { IStage, IZone } from "#/types/stages";
import type { IUserProfile } from "#/types/user";
import { fetchToDataURI, inlineArt, ogEntityIconURL } from "./art";
import { buildGridImageData, type IGridImageData, parseGridOgId } from "./grid";
import { type IOgHasher, type OgData, type OgKind, ogHashers } from "./hashers";
import { defaultOgPreset, defaultOgTagLabels, resolveDefaultOgPreset } from "./presets";
import type { IRenderDimensions } from "./render";
import { buildStoryOgData, type IStoryOgData, parseStoryOgId } from "./story";
import { DefaultTemplate, type IDefaultOgData } from "./templates/Default";
import { GridImageTemplate, gridImageDimensions } from "./templates/GridImage";
import { type IOperatorOgData, OperatorTemplate } from "./templates/Operator";
import { buildStageOgData, type IStageOgData, StageTemplate } from "./templates/Stage";
import { StoryTemplate } from "./templates/Story";
import { type ITierListOgData, type ITierListOperatorPreview, type ITierListTierPreview, TierListTemplate } from "./templates/TierList";
import { type ITierListBoardImageData, type ITierListBoardImageOperator, type ITierListBoardImageTier, TierListBoardImageTemplate, tierListBoardImageDimensions } from "./templates/TierListBoardImage";
import { type IUserOgData, type IUserSupportModule, type IUserSupportSkill, type IUserSupportUnit, UserTemplate } from "./templates/User";

export interface IOgHandler<TData> {
    /**
     * Load the template's data. `locale` is the locale the embedding page was
     * rendered in, read off the request by `ogLocale` - a handler whose text
     * is all game or user data ignores it.
     */
    fetch: (id: string, locale?: string) => Promise<TData | null>;
    hash: (data: TData) => string;
    cacheVersion: (id: string, version?: string) => string;
    template: (data: TData) => ReactNode;
    dimensions?: (data: TData) => IRenderDimensions;
}

// The hash half of every handler comes from `ogHashers`, which the client imports
// alone to build og:image URLs. Each handler below is marked pure because the bundler
// reads `def.fetch` and friends as possible getter side effects and would otherwise keep
// every template in any bundle that so much as imports this module.
interface IOgHandlerDef<TData> {
    fetch: (id: string, locale?: string) => Promise<TData | null>;
    template: (data: TData) => ReactNode;
    dimensions?: (data: TData) => IRenderDimensions;
}

function defineOgHandler<TData>(hasher: IOgHasher<TData>, def: IOgHandlerDef<TData>): IOgHandler<TData> {
    return {
        fetch: def.fetch,
        hash: hasher.hash,
        cacheVersion: hasher.cacheVersion,
        template: def.template,
        dimensions: def.dimensions,
    };
}

function backendBaseURL(): string {
    return (env.BACKEND_URL ?? env.VITE_BACKEND_URL ?? "").replace(/\/$/, "");
}

/** Servers other than the default serve their assets behind a `/{server}`
 *  prefix. Mirrors `components/operators/detail/impl/assets.ts`. */
type AssetServer = "en" | "cn";

function assetURL(path: string, server?: AssetServer): string {
    const base = backendBaseURL();
    if (!base) return "";
    const prefix = server && server !== "en" ? `${server}/` : "";
    return `${base}/api/${prefix}assets${path.startsWith("/") ? path : `/${path}`}`;
}

function avatarURL(charId: string): string {
    const base = backendBaseURL();
    if (!base) return "";
    return `${base}/api/avatar/${toAvatarStem(charId)}`;
}

function skillIconURL(iconId: string): string {
    const base = backendBaseURL();
    if (!base) return "";
    return `${base}/api/skill-icon/${encodeURIComponent(iconId)}`;
}

const charartURL = (operatorId: string, server?: AssetServer) => assetURL(`/textures/chararts/${operatorId}/${operatorId}_2.png`, server);
const skinpackURL = (operatorId: string, skinId: string) => {
    const owner = skinId.split("@")[0] || operatorId;
    return assetURL(`/textures/skinpack/${owner}/${skinId.replaceAll("@", "_").replaceAll("#", "%23")}.png`);
};
const campLogoURL = (id: string, server?: AssetServer) => assetURL(`/textures/spritepack/ui_camp_logo_0/logo_${id.toLowerCase()}.png`, server);
const moduleIconURL = (uniEquipIcon: string) => assetURL(`/textures/spritepack/ui_equip_big_img_hub_0/${uniEquipIcon}.png`);
const masteryIconURL = (mastery: number) => assetURL(`/textures/arts/specialized_hub/specialized_${mastery}.png`);
const secretaryArtURL = (operatorId: string, skinId: string | null, op?: IOperatorListItem): string => {
    if (skinId?.includes("@")) return skinpackURL(operatorId, skinId);
    if (op?.skin) return assetURL(op.skin);
    if (op?.portrait) return assetURL(op.portrait);
    return charartURL(operatorId);
};
const professionIconURL = (profession: string, server?: AssetServer) => assetURL(`/textures/arts/ui/%5Buc%5Dcharcommon/icon_profession_${profession.toLowerCase()}.png`, server);

const operatorHandler = /* @__PURE__ */ defineOgHandler<IOperatorOgData>(ogHashers.operator, {
    fetch: async (id) => {
        // `/operators/{id}` resolves across every loaded server, default first,
        // then CN, and tags the response with the server it was found on. The
        // operators list is Global-only, so going through it would leave every
        // CN-exclusive operator without an embed.
        const res = await backendFetch(`/operators/${encodeURIComponent(id)}`);
        if (!res.ok) return null;
        const op = deepCamelize(await res.json()) as IOperatorListItem;
        if (!op?.name) return null;
        const server = op.server;
        const rarity = rarityToNumber(op.rarity);
        // Faction display chooses the most specific source the data has;
        // matches how OperatorCardCompact picks its logo id.
        const factionId = op.nationId || op.teamId || op.groupId || "";
        const factionLabel = (op.teamId ? teamLabel(op) : op.groupId ? groupLabel(op) : nationLabel(op)) ?? undefined;
        const lastPhase = op.phases?.[op.phases.length - 1];
        const lastFrame = lastPhase?.attributesKeyFrames?.[lastPhase.attributesKeyFrames.length - 1]?.data;
        const stats = lastFrame
            ? [
                  { label: "HP", value: formatNumber(lastFrame.maxHp) },
                  { label: "ATK", value: formatNumber(lastFrame.atk) },
                  { label: "DEF", value: formatNumber(lastFrame.def) },
                  { label: "RES", value: formatNumber(lastFrame.magicResistance) },
              ]
            : undefined;
        return {
            name: op.name,
            appellation: op.appellation ?? "",
            profession: op.profession,
            professionName: op.professionName ?? undefined,
            subProfession: op.subProfessionId ?? "",
            position: op.position ?? "",
            nationId: op.nationId ?? "",
            rarity,
            charArtURL: assetURL(op.skin ?? op.portrait ?? `/textures/chararts/${id}/${id}_2.png`, server),
            factionLogoURL: factionId ? campLogoURL(factionId, server) : undefined,
            factionLabel,
            professionIconURL: op.profession ? professionIconURL(op.profession, server) : undefined,
            stats,
            server,
        };
    },
    template: (data) => OperatorTemplate(data),
});

interface ISupportUnitResponse {
    slot: number;
    operator_id: string;
    skin_id: string | null;
    skill_index: number;
    current_equip: string | null;
    elite: number | null;
    level: number | null;
    potential: number | null;
    skill_level: number | null;
    favor_point: number | null;
    specialize_level: number;
}

function buildSupportUnit(args: { id: string; elite: number; level: number; skinId?: string | null; op: IOperatorListItem; rosterEntry?: IRosterEntry }): IUserSupportUnit {
    const { id, elite, level, skinId, op, rosterEntry } = args;
    const masteryByIndex = new Map<number, number>((rosterEntry?.masteries ?? []).map((m) => [m.index, m.mastery]));
    const moduleStateById = new Map<string, { level: number; locked: boolean }>((rosterEntry?.modules ?? []).map((m) => [m.id, { level: m.level, locked: m.locked }]));

    const skills: IUserSupportSkill[] = (op.skills ?? []).map((sk, idx) => {
        const iconId = sk.static?.iconId ?? sk.static?.skillId ?? sk.skillId;
        const iconURL = sk.static?.image ? assetURL(sk.static.image) : iconId ? skillIconURL(iconId) : undefined;
        const mastery = masteryByIndex.get(idx) ?? 0;
        return {
            iconURL,
            mastery,
            skillLevel: rosterEntry?.skill_level ?? 7,
            masteryIconURL: mastery >= 1 ? masteryIconURL(mastery) : undefined,
        };
    });

    const modules: IUserSupportModule[] = (op.modules ?? [])
        .filter((m) => m.typeName1 !== "ORIGINAL")
        .slice(0, 3)
        .map((m) => ({
            iconURL: m.image ? assetURL(m.image) : moduleIconURL(m.uniEquipIcon),
            level: moduleStateById.get(m.uniEquipId)?.level ?? 0,
        }));

    return {
        id,
        name: op.name,
        rarity: rarityToNumber(op.rarity),
        elite,
        level,
        avatarURL: avatarURL(skinId || id),
        skills,
        modules,
    };
}

interface IRosterIndex {
    byId: Map<string, IRosterEntry>;
    rarityCounts: Record<number, number>;
}

function indexRoster(roster: IRosterEntry[], opByIdMap: Map<string, IOperatorListItem>): IRosterIndex {
    const byId = new Map<string, IRosterEntry>();
    const rarityCounts: Record<number, number> = { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0 };
    for (const entry of roster) {
        byId.set(entry.operator_id, entry);
        const op = opByIdMap.get(entry.operator_id);
        if (!op) continue;
        const rarity = rarityToNumber(op.rarity);
        rarityCounts[rarity]++;
    }
    return { byId, rarityCounts };
}

function topRosterPicks(roster: IRosterEntry[], opByIdMap: Map<string, IOperatorListItem>, limit = 3): IUserSupportUnit[] {
    return roster
        .map((entry) => {
            const op = opByIdMap.get(entry.operator_id);
            return op ? { entry, op, rarity: rarityToNumber(op.rarity) } : null;
        })
        .filter((x) => x !== null)
        .sort((a, b) => {
            if (b.entry.elite !== a.entry.elite) return b.entry.elite - a.entry.elite;
            if (b.entry.level !== a.entry.level) return b.entry.level - a.entry.level;
            return b.rarity - a.rarity;
        })
        .slice(0, limit)
        .map(({ entry, op }) =>
            buildSupportUnit({
                id: entry.operator_id,
                elite: entry.elite,
                level: entry.level,
                skinId: entry.skin_id,
                op,
                rosterEntry: entry,
            }),
        );
}

const userHandler = /* @__PURE__ */ defineOgHandler<IUserOgData>(ogHashers.user, {
    fetch: async (uid) => {
        const enc = encodeURIComponent(uid);
        const [userRes, supportsRes, rosterRes, operators] = await Promise.all([backendFetch(`/get-user?uid=${enc}`), backendFetch(`/get-user-supports?uid=${enc}`), backendFetch(`/roster?uid=${enc}`), getOperatorsListFn().catch(() => [] as IOperatorListItem[])]);
        if (!userRes.ok) return null;
        const u = (await userRes.json()) as IUserProfile;

        const opByIdMap = new Map<string, IOperatorListItem>();
        for (const op of operators) {
            if (op.id) opByIdMap.set(op.id, op);
        }

        const roster = rosterRes.ok ? ((await rosterRes.json()) as IRosterEntry[]) : [];
        const { byId: rosterByIdMap, rarityCounts } = indexRoster(roster, opByIdMap);

        let supportUnits: IUserSupportUnit[] | undefined;
        let supportUnitsKind: "supports" | "roster" = "roster";

        if (supportsRes.ok) {
            try {
                const list = (await supportsRes.json()) as ISupportUnitResponse[];
                if (Array.isArray(list) && list.length > 0) {
                    const built = list
                        .map((s) => {
                            const op = opByIdMap.get(s.operator_id);
                            if (!op) return null;
                            return buildSupportUnit({
                                id: s.operator_id,
                                elite: s.elite ?? 0,
                                level: s.level ?? 1,
                                skinId: s.skin_id,
                                op,
                                rosterEntry: rosterByIdMap.get(s.operator_id),
                            });
                        })
                        .filter((x): x is IUserSupportUnit => x !== null);
                    if (built.length > 0) {
                        supportUnits = built;
                        supportUnitsKind = "supports";
                    }
                }
            } catch (err) {
                console.warn(`[og] /get-user-supports parse failed for ${uid}, falling back to roster:`, err);
            }
        }

        if (!supportUnits && roster.length > 0 && opByIdMap.size > 0) {
            supportUnits = topRosterPicks(roster, opByIdMap);
        }

        const hasRarityCounts = Object.values(rarityCounts).some((n) => n > 0);

        return {
            nickname: u.nickname ?? "Player",
            nickNumber: u.nick_number,
            uid: u.uid ?? uid,
            level: u.level,
            grade: u.grade,
            totalScore: u.total_score,
            operatorCount: u.operator_count ?? 0,
            skinCount: u.skin_count ?? 0,
            itemCount: u.item_count ?? 0,
            lmd: u.lmd ?? 0,
            secretaryArtURL: u.secretary ? secretaryArtURL(u.secretary, u.secretary_skin_id, opByIdMap.get(u.secretary)) : undefined,
            supportUnits,
            supportUnitsKind,
            rarityCounts: hasRarityCounts ? rarityCounts : undefined,
        };
    },
    template: (data) => UserTemplate(data),
});

const HEX_COLOR_RE = /^#([0-9a-fA-F]{6})$/;
const FALLBACK_TIER_HEX = ["#dc4d56", "#e0834a", "#d8b54a", "#5dbf86", "#5aa9d9", "#9b73d4", "#8a8a8a"];

function toHex(color: string | null | undefined, fallback: string): string {
    if (!color) return fallback;
    const trimmed = color.trim();
    if (HEX_COLOR_RE.test(trimmed)) return trimmed;
    return fallback;
}

interface IBackendTierListAuthor {
    id: string;
    uid: string;
    nickname: string | null;
    avatar_id: string | null;
}

interface IBackendTierListFlair {
    id: number;
    code: string;
    label: string;
    color: string | null;
    display_order: number;
    is_active: boolean;
}

interface IBackendTierListStats {
    view_count: number;
    favorite_count: number;
    is_trending: boolean;
}

interface IBackendTierListTier {
    id: string;
    name: string;
    display_order: number;
    color: string | null;
    placements: PlacementDetail[];
}

/** A resolved placement as an OG tile: its own icon, and the operator's rarity when it has one. Unresolved placements are left off the image. */
function entityPreview(p: PlacementDetail): ITierListOperatorPreview | null {
    const entity = p.entity;
    if (!entity) return null;
    const base = backendBaseURL();
    const rarity = entity.facets.rarity;
    return {
        id: entity.id,
        name: entity.name,
        rarity: (typeof rarity === "string" && Number(rarity)) || 1,
        avatarURL: base && entity.icon ? ogEntityIconURL(entity.icon, base) : "",
    };
}

interface IBackendTierListResponse {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    list_type: string;
    updated_at: string;
    flair: IBackendTierListFlair | null;
    author: IBackendTierListAuthor | null;
    stats: IBackendTierListStats | null;
    tiers: IBackendTierListTier[];
}

const tierListHandler = /* @__PURE__ */ defineOgHandler<ITierListOgData>(ogHashers["tier-list"], {
    fetch: async (slug) => {
        const enc = encodeURIComponent(slug);
        const detailRes = await backendFetch(`/tier-lists/${enc}`);
        if (!detailRes.ok) return null;
        const detail = (await detailRes.json()) as IBackendTierListResponse;

        const sortedTiers = [...detail.tiers].sort((a, b) => a.display_order - b.display_order);

        const tiers: ITierListTierPreview[] = sortedTiers.slice(0, 4).map((t, i) => {
            const fallback = FALLBACK_TIER_HEX[i % FALLBACK_TIER_HEX.length] as string;
            const placements = [...t.placements].sort((a, b) => a.sub_order - b.sub_order);
            const operators: ITierListOperatorPreview[] = placements
                .slice(0, 5)
                .map(entityPreview)
                .filter((x): x is ITierListOperatorPreview => x !== null);
            return {
                name: t.name,
                color: toHex(t.color, fallback),
                operators,
                operatorCount: t.placements.length,
            };
        });

        const totalOperators = detail.tiers.reduce((sum, t) => sum + t.placements.length, 0);
        const listType = detail.list_type === "official" ? "official" : "community";

        const updatedDate = new Date(detail.updated_at);
        const updatedRelative = Number.isNaN(updatedDate.getTime()) ? undefined : updatedDate.toLocaleDateString("en-US", { month: "short", year: "numeric" });

        const flairColor = toHex(detail.flair?.color, "");

        return {
            title: detail.name,
            slug: detail.slug,
            description: detail.description ?? undefined,
            listType,
            flairLabel: detail.flair?.label,
            flairColor: flairColor || undefined,
            authorName: detail.author?.nickname?.trim() || (listType === "official" ? "Myrtle" : "Community"),
            authorAvatarURL: detail.author?.avatar_id ? avatarURL(detail.author.avatar_id) : undefined,
            views: detail.stats?.view_count ?? 0,
            favorites: detail.stats?.favorite_count ?? 0,
            isTrending: detail.stats?.is_trending ?? false,
            updatedRelative,
            totalOperators,
            tierCount: detail.tiers.length,
            tiers: await inlineTierAvatars(tiers),
        };
    },
    template: (data) => TierListTemplate(data),
});

// Canonical id for the site-wide fallback OG image. Slugs registered in
// DEFAULT_OG_PRESETS resolve to their preset; anything else is treated as a
// literal (URL-encoded) title for one-off pages.
export const DEFAULT_OG_ID = "_root";

const defaultHandler = /* @__PURE__ */ defineOgHandler<IDefaultOgData>(ogHashers.default, {
    // The only handler whose text is authored rather than fetched, so the only
    // one that needs a catalog. There is no React tree and no router match
    // here - satori is handed a plain element tree on the server - so the
    // locale arrives as a request param and the catalog is fetched for it.
    // `metaSourceForLocale` returns null for the source locale, which resolves
    // every key to the bundled English without a round trip.
    fetch: async (id, locale) => {
        const source = await metaSourceForLocale(locale);
        const preset = defaultOgPreset(id);
        if (preset) return resolveDefaultOgPreset(preset, source);
        // Not a registered slug: the id IS the title, for one-off pages.
        return { title: decodeURIComponent(id), tagLabels: defaultOgTagLabels(source) };
    },
    template: (data) => DefaultTemplate(data),
});

/** Inline every tile's art, so satori is never handed a URL it would fetch and fail to decode (see `./art`). */
async function inlineTierAvatars<T extends { operators: ITierListOperatorPreview[] }>(tiers: T[]): Promise<T[]> {
    const art = await inlineArt(tiers.flatMap((t) => t.operators.map((o) => o.avatarURL)));
    return tiers.map((t) => ({ ...t, operators: t.operators.map((o) => ({ ...o, avatarURL: o.avatarURL ? art.get(o.avatarURL) : undefined })) }));
}

const tierListBoardImageHandler = /* @__PURE__ */ defineOgHandler<ITierListBoardImageData>(ogHashers["tier-list-image"], {
    fetch: async (slug) => {
        const enc = encodeURIComponent(slug);
        const detailRes = await backendFetch(`/tier-lists/${enc}`);
        if (!detailRes.ok) return null;
        const detail = (await detailRes.json()) as IBackendTierListResponse;

        const sortedTiers = [...detail.tiers].sort((a, b) => a.display_order - b.display_order);

        const tiers: ITierListBoardImageTier[] = sortedTiers.map((t, i) => {
            const fallback = FALLBACK_TIER_HEX[i % FALLBACK_TIER_HEX.length] as string;
            const placements = [...t.placements].sort((a, b) => a.sub_order - b.sub_order);
            const operators: ITierListBoardImageOperator[] = placements.map(entityPreview).filter((x): x is ITierListOperatorPreview => x !== null);
            return {
                name: t.name,
                color: toHex(t.color, fallback),
                operators,
            };
        });

        return {
            title: detail.name,
            slug: detail.slug,
            tiers: await inlineTierAvatars(tiers),
        };
    },
    template: (data) => TierListBoardImageTemplate(data),
    dimensions: (data) => tierListBoardImageDimensions(data),
});

const gridImageHandler = /* @__PURE__ */ defineOgHandler<IGridImageData>(ogHashers["grid-image"], {
    fetch: async (id) => {
        const { server, slug } = parseGridOgId(id);
        const res = await backendFetch(`/grids/${encodeURIComponent(slug)}?server=${encodeURIComponent(server)}`);
        if (!res.ok) return null;
        const data = buildGridImageData((await res.json()) as Grid, backendBaseURL(), server);
        const art = await inlineArt(data.cells.map((c) => c.artURL));
        return { ...data, cells: data.cells.map((c) => ({ ...c, artURL: c.artURL ? (art.get(c.artURL) ?? null) : null })) };
    },
    template: (data) => GridImageTemplate(data),
    dimensions: (data) => gridImageDimensions(data),
});

const STATIC_STAGE_TTL_MS = 30 * 60 * 1000;
let stagesCache: { promise: Promise<IStage[]>; expiresAt: number } | null = null;
let zonesCache: { promise: Promise<IZone[]>; expiresAt: number } | null = null;

function cachedStaticList<T>(getCache: () => { promise: Promise<T[]>; expiresAt: number } | null, setCache: (next: { promise: Promise<T[]>; expiresAt: number } | null) => void, path: string): Promise<T[]> {
    const now = Date.now();
    const cache = getCache();
    if (!cache || now >= cache.expiresAt) {
        const promise = backendFetch(path).then(async (res) => {
            if (!res.ok) throw new Error(`${path} failed: ${res.status}`);
            return Object.values((await res.json()) as Record<string, T>);
        });
        setCache({ promise, expiresAt: now + STATIC_STAGE_TTL_MS });
        promise.catch(() => {
            // Only clear if this promise is still the cached one; a late rejection
            // must not null a newer valid entry (mirrors getOperatorsListFn).
            if (getCache()?.promise === promise) setCache(null);
        });
        return promise;
    }
    return cache.promise;
}

function getCachedStages(): Promise<IStage[]> {
    return cachedStaticList(
        () => stagesCache,
        (next) => {
            stagesCache = next;
        },
        "/static/stages",
    );
}

function getCachedZones(): Promise<IZone[]> {
    return cachedStaticList(
        () => zonesCache,
        (next) => {
            zonesCache = next;
        },
        "/static/zones",
    );
}

/** Try preview candidates in priority order; return the first that loads as a data URI. */
async function resolveStagePreview(paths: string[]): Promise<string | undefined> {
    const CHUNK = 6;
    for (let i = 0; i < paths.length; i += CHUNK) {
        const chunk = paths.slice(i, i + CHUNK);
        const resolved = await Promise.all(chunk.map((p) => fetchToDataURI(assetURL(p))));
        const hit = resolved.find((r) => r);
        if (hit) return hit;
    }
    return undefined;
}

const stageHandler = /* @__PURE__ */ defineOgHandler<IStageOgData>(ogHashers.stage, {
    fetch: async (stageId) => {
        const [stages, zones] = await Promise.all([getCachedStages().catch(() => [] as IStage[]), getCachedZones().catch(() => [] as IZone[])]);
        const stage = stages.find((s) => s.stageId === stageId);
        if (!stage) return null;

        const zone = zones.find((z) => z.zoneId === stage.zoneId);
        const previewImageURL = await resolveStagePreview(stagePreviewAssetPaths(stage));

        return buildStageOgData(stage, zone, previewImageURL);
    },
    template: (data) => StageTemplate(data),
});

/**
 * The library index is 706,075 bytes on EN (451 groups, 315 records, measured
 * 2026-09-25), and a card needs one entry of it. It is cached per server for
 * the same 30 minutes the stage lists are, and a cache miss on the card costs
 * one index read, not one per render.
 */
const storyIndexCache = new Map<string, { promise: Promise<StoryIndex>; expiresAt: number }>();

function getCachedStoryIndex(server: string): Promise<StoryIndex> {
    const now = Date.now();
    const cached = storyIndexCache.get(server);
    if (cached && now < cached.expiresAt) return cached.promise;
    const path = gamedataPath(server, "/story/index");
    const promise = backendFetch(path).then(async (res) => {
        if (!res.ok) throw new Error(`${path} failed: ${res.status}`);
        return (await res.json()) as StoryIndex;
    });
    storyIndexCache.set(server, { promise, expiresAt: now + STATIC_STAGE_TTL_MS });
    promise.catch(() => {
        if (storyIndexCache.get(server)?.promise === promise) storyIndexCache.delete(server);
    });
    return promise;
}

const storyHandler = /* @__PURE__ */ defineOgHandler<IStoryOgData>(ogHashers.story, {
    fetch: async (id, locale) => {
        const { server, storyId, explicit } = parseStoryOgId(id);
        const source = await metaSourceForLocale(locale);
        // A bare id is the default server's; when that index does not list it,
        // CN is tried next, the order the operator card resolves in, so a
        // CN-only story still has a card at its bare URL. The page itself
        // always names the server it read (`storyOgId`).
        const servers: string[] = explicit || server === "cn" ? [server] : [server, "cn"];
        for (const s of servers) {
            const index = await getCachedStoryIndex(s).catch(() => null);
            if (!index) continue;
            const data = buildStoryOgData(index, storyId, { server: s, source });
            if (!data) continue;
            const assetServer = s === "cn" ? "cn" : undefined;
            const artURL = data.artPath ? await fetchToDataURI(assetURL(data.artPath, assetServer)) : undefined;
            return { ...data, artURL };
        }
        return null;
    },
    template: (data) => StoryTemplate(data),
});

export const ogRegistry: { [K in OgKind]: IOgHandler<OgData<K>> } = {
    operator: operatorHandler,
    user: userHandler,
    "tier-list": tierListHandler,
    "tier-list-image": tierListBoardImageHandler,
    "grid-image": gridImageHandler,
    stage: stageHandler,
    story: storyHandler,
    default: defaultHandler,
};

export type { OgKind };

// biome-ignore lint/suspicious/noExplicitAny: registry erases per-kind data type at lookup
export type IAnyOgHandler = IOgHandler<any>;

export function getHandler(kind: string): IAnyOgHandler {
    return (ogRegistry as Record<string, IAnyOgHandler>)[kind] ?? ogRegistry.default;
}
