import type { ReactNode } from "react";
import { backgroundSources, DEFAULT_FOCUS_Y, objectPosition, zoomOf } from "#/components/user/profile/impl/background";
import { DEFAULT_GAMEDATA_SERVER, gamedataPath } from "#/lib/api/gamedata";
import { cachedOperatorsList, deepCamelize } from "#/lib/api/operators";
import { loadRecruitmentData } from "#/lib/api/recruitment";
import { stagePreviewAssetPaths } from "#/lib/api/stages";
import type { IRosterEntry } from "#/lib/api/user";
import { assetStoreURL } from "#/lib/assets";
import { backendFetch } from "#/lib/fetch";
import { metaSourceForLocale } from "#/lib/meta";
import { DEFAULT_AVATAR_ID, formatNumber, groupLabel, nationLabel, rarityToNumber, teamLabel, toAvatarStem } from "#/lib/utils";
import type { ArchiveOperator } from "#/types/generated/ArchiveOperator";
import type { Enemy } from "#/types/generated/Enemy";
import type { Grid } from "#/types/generated/Grid";
import type { PlacementDetail } from "#/types/generated/PlacementDetail";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { SirenAlbumDetail } from "#/types/generated/SirenAlbumDetail";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import type { SupportUnit } from "#/types/generated/SupportUnit";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import type { IStage, IZone } from "#/types/stages";
import type { IUserProfile } from "#/types/user";
import { buildArchiveOgData, clampExcerpt, firstProseRecord, type IArchiveOgData } from "./archive";
import { backendBaseURL, fetchToDataURI, inlineArt, ogEntityIconURL, serverSideURL } from "./art";
import { buildEnemyOgData, type IEnemyOgData } from "./enemy";
import { buildGridImageData, type IGridImageData, parseGridOgId } from "./grid";
import { type IOgHasher, type OgData, type OgKind, ogHashers } from "./hashers";
import { parseServerScopedOgId } from "./ids";
import { buildMusicOgData, type IMusicOgData, isMusicAlbumId, sirenCoverAt } from "./music";
import { DEFAULT_OG_PRESETS, type DefaultOgPresetSlug, defaultOgPreset, literalDefaultOgData, resolveDefaultOgPreset } from "./presets";
import { type IRecruitmentOgData, parseRecruitmentOgId } from "./recruitment";
import { buildRecruitmentOgData } from "./recruitment.build";
import type { IRenderDimensions } from "./render";
import { buildSquadOgData, type ISquadOgData, type ISquadOgSeat, parseSquadOgId } from "./squad";
import { buildStoryOgData, type IStoryOgData, parseStoryOgId } from "./story";
import { ArchiveTemplate } from "./templates/Archive";
import { DefaultTemplate, type IDefaultOgData } from "./templates/Default";
import { EnemyTemplate } from "./templates/Enemy";
import { GridImageTemplate, gridImageDimensions } from "./templates/GridImage";
import { MusicTemplate } from "./templates/Music";
import { type IOperatorOgData, OperatorTemplate } from "./templates/Operator";
import { RecruitmentTemplate } from "./templates/Recruitment";
import { SquadTemplate } from "./templates/Squad";
import { buildStageOgData, type IStageOgData, StageTemplate } from "./templates/Stage";
import { StoryTemplate } from "./templates/Story";
import { type ITierListOgData, type ITierListOperatorPreview, type ITierListTierPreview, TierListTemplate } from "./templates/TierList";
import { type ITierListBoardImageData, type ITierListBoardImageOperator, type ITierListBoardImageTier, TierListBoardImageTemplate, tierListBoardImageDimensions } from "./templates/TierListBoardImage";
import { UserTemplate } from "./templates/User";
import { buildUserOgData, type IUserOgArt, type IUserOgData, type IUserOgTile } from "./user";

export interface IOgHandler<TData> {
    /**
     * Load the template's data. `locale` is the locale the embedding page was
     * rendered in, read off the request by `ogLocale`; a handler whose text
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

/** Servers other than the default serve their assets behind a `/{server}`
 *  prefix. Mirrors `components/operators/detail/impl/assets.ts`. */
type AssetServer = "en" | "cn";

/** A raw-tree asset for the renderer to fetch: the object store when one is configured, else the backend's own origin. */
function assetURL(path: string, server?: AssetServer): string {
    const store = assetStoreURL(path, server);
    if (store) return store;
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

const campLogoURL = (id: string, server?: AssetServer) => assetURL(`/textures/spritepack/ui_camp_logo_0/logo_${id.toLowerCase()}.png`, server);
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

/** The first of `urls` that inlines, tried in order: each is a fallback for the one before. */
async function firstDataURI(urls: readonly string[]): Promise<string | undefined> {
    for (const url of urls) {
        const uri = await fetchToDataURI(serverSideURL(url));
        if (uri) return uri;
    }
    return undefined;
}

/** The hero art's candidates: the profile's chosen background, else the secretary's art read the same way (`#1` outfits are elite 1 art). */
function heroBackground(data: IUserOgData): ProfileBackground | null {
    if (data.background) return data.background;
    if (!data.secretary) return null;
    const skin = data.secretarySkinId;
    if (skin?.includes("@")) return { kind: "skin", id: skin };
    return { kind: "operator", id: data.secretary, ...(skin?.endsWith("#1") ? { elite: 1 } : {}) };
}

function skillIconFor(op: IOperatorListItem, index: number | null | undefined): string | undefined {
    const sk = index != null && index >= 0 ? op.skills?.[index] : undefined;
    if (!sk) return undefined;
    if (sk.static?.image) return assetURL(sk.static.image);
    const iconId = sk.static?.iconId ?? sk.static?.skillId ?? sk.skillId;
    return iconId ? skillIconURL(iconId) : undefined;
}

/** The best three of the roster when the player set no supports: highest elite, then level, then rarity. */
function topRosterTiles(roster: readonly IRosterEntry[], opById: Map<string, IOperatorListItem>): IUserOgTile[] {
    return roster
        .flatMap((entry) => {
            const op = opById.get(entry.operator_id);
            return op ? [{ entry, op, rarity: rarityToNumber(op.rarity) }] : [];
        })
        .sort((a, b) => b.entry.elite - a.entry.elite || b.entry.level - a.entry.level || b.rarity - a.rarity)
        .slice(0, 3)
        .map(({ entry, op, rarity }) => ({
            rarity,
            elite: entry.elite,
            level: entry.level,
            avatarURL: avatarURL(entry.skin_id || entry.operator_id),
            skillIconURL: skillIconFor(op, entry.default_skill),
            mastery: (entry.masteries ?? []).find((m) => m.index === entry.default_skill)?.mastery ?? 0,
        }));
}

/** Everything the template draws beyond the hashed fields, art inlined as data URIs. */
async function resolveUserOgArt(data: IUserOgData, roster: readonly IRosterEntry[], operators: readonly IOperatorListItem[]): Promise<IUserOgArt> {
    const opById = new Map(operators.flatMap((op) => (op.id ? [[op.id, op] as const] : [])));

    const supportTiles: IUserOgTile[] = data.supports.flatMap((s) => {
        const op = opById.get(s.id);
        if (!op) return [];
        return [{ rarity: rarityToNumber(op.rarity), elite: s.elite, level: s.level, avatarURL: avatarURL(s.skinId || s.id), skillIconURL: skillIconFor(op, s.skillIndex), mastery: s.specializeLevel }];
    });
    const tilesKind = supportTiles.length > 0 ? "supports" : "roster";
    const tiles = (supportTiles.length > 0 ? supportTiles : topRosterTiles(roster, opById)).slice(0, 3);

    const rarityCounts: Record<number, number> = { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0 };
    for (const entry of roster) {
        const op = opById.get(entry.operator_id);
        if (op) rarityCounts[rarityToNumber(op.rarity)]++;
    }

    const hero = heroBackground(data);
    const [avatar, heroURL, tileArt] = await Promise.all([fetchToDataURI(avatarURL(data.avatarId || DEFAULT_AVATAR_ID)), hero ? firstDataURI(backgroundSources(hero)) : Promise.resolve(undefined), inlineArt(tiles.flatMap((t) => [t.avatarURL, t.skillIconURL]))]);

    return {
        avatarURL: avatar,
        heroURL,
        // A chosen background keeps its own crop and zoom; the secretary fallback starts
        // where a fresh pick of that art would, near the face.
        heroPosition: data.background ? objectPosition(data.background) : `50% ${DEFAULT_FOCUS_Y}%`,
        heroZoom: data.background ? zoomOf(data.background) : 100,
        tiles: tiles.map((t) => ({ ...t, avatarURL: t.avatarURL ? tileArt.get(t.avatarURL) : undefined, skillIconURL: t.skillIconURL ? tileArt.get(t.skillIconURL) : undefined })),
        tilesKind,
        rarityCounts: Object.values(rarityCounts).some((n) => n > 0) ? rarityCounts : undefined,
    };
}

async function jsonArrayOrNull<T>(res: Response): Promise<T[] | null> {
    if (!res.ok) return null;
    const body = await res.json().catch(() => null);
    return Array.isArray(body) ? (body as T[]) : null;
}

const userHandler = /* @__PURE__ */ defineOgHandler<IUserOgData>(ogHashers.user, {
    // Read anonymously, so the card is a visitor's view: a private profile is
    // 403 here (no card), a private Roster tab 403s `/roster`, and `/get-user`
    // leaves out the score and grade behind a private Score tab.
    fetch: async (uid) => {
        const enc = encodeURIComponent(uid);
        const [userRes, supportsRes, rosterRes, operators] = await Promise.all([backendFetch(`/get-user?uid=${enc}`), backendFetch(`/get-user-supports?uid=${enc}`), backendFetch(`/roster?uid=${enc}`), cachedOperatorsList(undefined).catch(() => [] as IOperatorListItem[])]);
        if (!userRes.ok) return null;
        const profile = (await userRes.json()) as IUserProfile;
        const [supports, roster] = await Promise.all([jsonArrayOrNull<SupportUnit>(supportsRes), jsonArrayOrNull<IRosterEntry>(rosterRes)]);
        const data = buildUserOgData(profile, supports, roster);
        const visibleRoster = data.rosterDigest ? (roster ?? []) : [];
        return { ...data, art: await resolveUserOgArt(data, visibleRoster, operators) };
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
    // here (satori is handed a plain element tree on the server), so the
    // locale arrives as a request param and the catalog is fetched for it.
    // `metaSourceForLocale` returns null for the source locale, which resolves
    // every key to the bundled English without a round trip.
    fetch: async (id, locale) => {
        const source = await metaSourceForLocale(locale);
        const preset = defaultOgPreset(id);
        if (preset) return resolveDefaultOgPreset(preset, source);
        // Not a registered slug: the id IS the title, for one-off pages.
        return literalDefaultOgData(decodeURIComponent(id), source);
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

const recruitmentHandler = /* @__PURE__ */ defineOgHandler<IRecruitmentOgData>(ogHashers.recruitment, {
    // Tag ids the server does not offer are dropped; when none is left the card
    // is the calculator's preset, so a mangled link still unfurls as the tool.
    fetch: async (id, locale) => {
        const { server, tagIds } = parseRecruitmentOgId(id);
        const recruitment = await loadRecruitmentData(server).catch(() => null);
        const data = recruitment && tagIds.length > 0 ? buildRecruitmentOgData(tagIds, recruitment, server) : null;
        if (!data) {
            const preset = defaultOgPreset("tools-recruitment");
            return preset ? { server, tags: [], best: null, preset: resolveDefaultOgPreset(preset, await metaSourceForLocale(locale)) } : null;
        }
        if (!data.best) return data;
        const art = await inlineArt(data.best.operators.map((op) => avatarURL(op.id)));
        return { ...data, best: { ...data.best, operators: data.best.operators.map((op) => ({ ...op, avatarURL: art.get(avatarURL(op.id)) })) } };
    },
    template: (data) => RecruitmentTemplate(data),
});

const enemyHandler = /* @__PURE__ */ defineOgHandler<IEnemyOgData>(ogHashers.enemy, {
    fetch: async (raw) => {
        const { server, id } = parseServerScopedOgId(raw);
        const res = await backendFetch(gamedataPath(server, `/enemies/${encodeURIComponent(id)}`));
        if (!res.ok) return null;
        const detail = (await res.json()) as { enemy?: Enemy };
        if (!detail.enemy) return null;
        const data = buildEnemyOgData(detail.enemy, server);
        // The icon route serves a 158 px PNG; it is the only enemy art the backend has.
        const prefix = server === DEFAULT_GAMEDATA_SERVER ? "" : `/${server}`;
        return { ...data, iconURL: await fetchToDataURI(`${backendBaseURL()}/api${prefix}/enemy-icon/${encodeURIComponent(id)}`) };
    },
    template: (data) => EnemyTemplate(data),
});

async function presetData(slug: DefaultOgPresetSlug, locale?: string): Promise<IDefaultOgData> {
    return resolveDefaultOgPreset(DEFAULT_OG_PRESETS[slug], await metaSourceForLocale(locale));
}

const squadHandler = /* @__PURE__ */ defineOgHandler<ISquadOgData>(ogHashers.squad, {
    // An id that does not decode, is over the cap, or holds no seat draws the
    // tool's preset, so a mangled link still unfurls as the tool.
    fetch: async (raw, locale) => {
        const parsed = parseSquadOgId(raw);
        if (!parsed) return { server: DEFAULT_GAMEDATA_SERVER, share: "", name: "", stageId: null, seats: [], support: null, preset: await presetData("tools-squad-builder", locale) };
        const { server, draft } = parsed;
        const data = buildSquadOgData(draft, server);
        const [index, stageRes] = await Promise.all([
            backendFetch(gamedataPath(server, "/operators/index"))
                .then((r) => (r.ok ? (r.json() as Promise<IOperatorIndexEntry[]>) : []))
                .catch(() => [] as IOperatorIndexEntry[]),
            data.stageId ? backendFetch(gamedataPath(server, `/stages/${encodeURIComponent(data.stageId)}/detail`)).catch(() => null) : Promise.resolve(null),
        ]);
        const rarity = new Map(index.map((op) => [op.id, op.rarity] as const));
        const stage = stageRes?.ok ? ((await stageRes.json()) as { stage?: { code?: string; name?: string | null } }).stage : undefined;
        const seats = [...data.seats, data.support];
        const prefix = server === DEFAULT_GAMEDATA_SERVER ? "" : `/${server}`;
        const avatar = (id: string) => `${backendBaseURL()}/api${prefix}/avatar/${toAvatarStem(id)}`;
        const art = await inlineArt(seats.flatMap((s) => (s ? [avatar(s.id)] : [])));
        const resolve = (s: ISquadOgSeat | null): ISquadOgSeat | null => (s ? { ...s, rarity: rarity.get(s.id), avatarURL: art.get(avatar(s.id)) } : null);
        return {
            ...data,
            seats: data.seats.map(resolve),
            support: resolve(data.support),
            stage: stage?.code ? { code: stage.code, name: stage.name ?? "" } : undefined,
        };
    },
    template: (data) => SquadTemplate(data),
});

interface IHandbookResponse {
    skin?: string | null;
    portrait?: string | null;
    handbook?: { storyTextAudio?: { stories: { storyText: string }[] }[] } | null;
}

const archiveHandler = /* @__PURE__ */ defineOgHandler<IArchiveOgData>(ogHashers.archive, {
    fetch: async (raw) => {
        const { server, id } = parseServerScopedOgId(raw);
        const enc = encodeURIComponent(id);
        const [fileRes, opRes] = await Promise.all([backendFetch(gamedataPath(server, `/archives/operator/${enc}`)), backendFetch(gamedataPath(server, `/operators/${enc}`)).catch(() => null)]);
        if (!fileRes.ok) return null;
        const data = buildArchiveOgData((await fileRes.json()) as ArchiveOperator, server);
        const op = opRes?.ok ? ((await opRes.json()) as IHandbookResponse) : null;
        const prose = firstProseRecord(op?.handbook?.storyTextAudio ?? []);
        // Elite 1 art, the file photo: the operator card draws the elite 2 art, so
        // the two cards of one operator never show the same picture.
        const artURL = await firstDataURI(backgroundSources({ kind: "operator", id, elite: 1 }));
        return { ...data, excerpt: prose ? clampExcerpt(prose, /[\u3040-\u9fff]/.test(prose) ? 66 : 160) : undefined, artURL };
    },
    template: (data) => ArchiveTemplate(data),
});

const musicHandler = /* @__PURE__ */ defineOgHandler<IMusicOgData>(ogHashers.music, {
    fetch: async (cid) => {
        if (!isMusicAlbumId(cid)) return null;
        const res = await backendFetch(`/music/siren/albums/${encodeURIComponent(cid)}`);
        if (!res.ok) return null;
        const data = buildMusicOgData((await res.json()) as SirenAlbumDetail);
        const tiny = sirenCoverAt(data.coverSource, 8);
        const [coverURL, backdropURL] = await Promise.all([fetchToDataURI(data.coverSource), tiny ? fetchToDataURI(tiny) : Promise.resolve(undefined)]);
        return { ...data, coverURL, backdropURL };
    },
    template: (data) => MusicTemplate(data),
});

export const ogRegistry: { [K in OgKind]: IOgHandler<OgData<K>> } = {
    operator: operatorHandler,
    user: userHandler,
    "tier-list": tierListHandler,
    "tier-list-image": tierListBoardImageHandler,
    "grid-image": gridImageHandler,
    stage: stageHandler,
    story: storyHandler,
    recruitment: recruitmentHandler,
    enemy: enemyHandler,
    squad: squadHandler,
    archive: archiveHandler,
    music: musicHandler,
    default: defaultHandler,
};

export type { OgKind };

// biome-ignore lint/suspicious/noExplicitAny: registry erases per-kind data type at lookup
export type IAnyOgHandler = IOgHandler<any>;

export function getHandler(kind: string): IAnyOgHandler {
    return (ogRegistry as Record<string, IAnyOgHandler>)[kind] ?? ogRegistry.default;
}
