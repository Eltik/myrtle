import type { ReactNode } from "react";
import { env } from "#/env";
import { getOperatorsListFn } from "#/lib/api/operators";
import { stagePreviewAssetPaths } from "#/lib/api/stages";
import type { IRosterEntry } from "#/lib/api/user";
import { backendFetch } from "#/lib/fetch";
import { formatGroupId, formatNationId, formatNumber, formatTeamId, rarityToNumber, toAvatarStem } from "#/lib/utils";
import type { IOperatorIndexEntry, IOperatorListItem } from "#/types/operators";
import type { IStage, IZone } from "#/types/stages";
import type { IUserProfile } from "#/types/user";
import { ogHash } from "./hash";
import { DEFAULT_OG_PRESETS } from "./presets";
import type { IRenderDimensions } from "./render";
import { DefaultTemplate, type IDefaultOgData } from "./templates/Default";
import { type IOperatorOgData, OperatorTemplate } from "./templates/Operator";
import { buildStageOgData, type IStageOgData, StageTemplate } from "./templates/Stage";
import { type ITierListOgData, type ITierListOperatorPreview, type ITierListTierPreview, TierListTemplate } from "./templates/TierList";
import { type ITierListBoardImageData, type ITierListBoardImageOperator, type ITierListBoardImageTier, TIER_LIST_BOARD_IMAGE_LAYOUT, TierListBoardImageTemplate, tierListBoardImageDimensions } from "./templates/TierListBoardImage";
import { type IUserOgData, type IUserSupportModule, type IUserSupportSkill, type IUserSupportUnit, UserTemplate } from "./templates/User";

export interface IOgHandler<TData> {
    fetch: (id: string) => Promise<TData | null>;
    hash: (data: TData) => string;
    cacheVersion: (id: string, version?: string) => string;
    template: (data: TData) => ReactNode;
    dimensions?: (data: TData) => IRenderDimensions;
}

// A handler declares its `kind` and `hashVersion` once; `hash` and `cacheVersion` are
// derived from them so neither the kind string nor the version constant is restated.
// Both feed ogHash the same leading `[kind, hashVersion, ...]`, so the resulting cache
// keys are byte-identical to hand-written signatures.
interface IOgHandlerDef<TData> {
    kind: string;
    hashVersion: string;
    fetch: (id: string) => Promise<TData | null>;
    hashParts: (data: TData) => unknown[];
    template: (data: TData) => ReactNode;
    dimensions?: (data: TData) => IRenderDimensions;
}

function defineOgHandler<TData>(def: IOgHandlerDef<TData>): IOgHandler<TData> {
    const { kind, hashVersion } = def;
    return {
        fetch: def.fetch,
        hash: (data) => ogHash([kind, hashVersion, ...def.hashParts(data)]),
        cacheVersion: (id, version) => version || ogHash([kind, hashVersion, id]),
        template: def.template,
        dimensions: def.dimensions,
    };
}

function backendBaseURL(): string {
    return (env.BACKEND_URL ?? env.VITE_BACKEND_URL ?? "").replace(/\/$/, "");
}

function assetURL(path: string): string {
    const base = backendBaseURL();
    if (!base) return "";
    return `${base}/api/assets${path.startsWith("/") ? path : `/${path}`}`;
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

const charartURL = (operatorId: string) => assetURL(`/textures/chararts/${operatorId}/${operatorId}_2.png`);
const skinpackURL = (operatorId: string, skinId: string) => {
    const owner = skinId.split("@")[0] || operatorId;
    return assetURL(`/textures/skinpack/${owner}/${skinId.replaceAll("@", "_").replaceAll("#", "%23")}.png`);
};
const campLogoURL = (id: string) => assetURL(`/textures/spritepack/ui_camp_logo_0/logo_${id.toLowerCase()}.png`);
const moduleIconURL = (uniEquipIcon: string) => assetURL(`/textures/spritepack/ui_equip_big_img_hub_0/${uniEquipIcon}.png`);
const masteryIconURL = (mastery: number) => assetURL(`/textures/arts/specialized_hub/specialized_${mastery}.png`);
const secretaryArtURL = (operatorId: string, skinId: string | null, op?: IOperatorListItem): string => {
    if (skinId?.includes("@")) return skinpackURL(operatorId, skinId);
    if (op?.skin) return assetURL(op.skin);
    if (op?.portrait) return assetURL(op.portrait);
    return charartURL(operatorId);
};
const professionIconURL = (profession: string) => assetURL(`/textures/arts/ui/%5Buc%5Dcharcommon/icon_profession_${profession.toLowerCase()}.png`);

const OPERATOR_HASH_VERSION = "v8";

const operatorHandler = defineOgHandler<IOperatorOgData>({
    kind: "operator",
    hashVersion: OPERATOR_HASH_VERSION,
    fetch: async (id) => {
        const operators = await getOperatorsListFn();
        const op = operators.find((o) => o.id === id);
        if (!op) return null;
        const rarity = rarityToNumber(op.rarity);
        // Faction display chooses the most specific source the data has;
        // matches how OperatorCardCompact picks its logo id.
        const factionId = op.nationId || op.teamId || op.groupId || "";
        const factionLabel = op.teamId ? formatTeamId(op.teamId) : op.groupId ? formatGroupId(op.groupId) : op.nationId ? formatNationId(op.nationId) : undefined;
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
            subProfession: op.subProfessionId ?? "",
            position: op.position ?? "",
            nationId: op.nationId ?? "",
            rarity,
            charArtURL: assetURL(op.skin ?? op.portrait ?? `/textures/chararts/${id}/${id}_2.png`),
            factionLogoURL: factionId ? campLogoURL(factionId) : undefined,
            factionLabel,
            professionIconURL: op.profession ? professionIconURL(op.profession) : undefined,
            stats,
        };
    },
    hashParts: (data) => [data.name, data.appellation, data.profession, data.rarity, data.subProfession, data.position, data.nationId, data.factionLabel ?? "", data.professionIconURL ?? "", (data.stats ?? []).map((s) => `${s.label}=${s.value}`).join("|")],
    template: (data) => OperatorTemplate(data),
});

const USER_HASH_VERSION = "v15";

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

const userHandler = defineOgHandler<IUserOgData>({
    kind: "user",
    hashVersion: USER_HASH_VERSION,
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
            nickname: u.nickname ?? "Doctor",
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
    hashParts: (data) => [
        data.nickname,
        data.nickNumber ?? "",
        data.uid,
        data.level,
        data.grade,
        data.totalScore,
        data.operatorCount,
        data.skinCount,
        data.itemCount,
        data.lmd,
        data.secretaryArtURL ?? "",
        data.supportUnitsKind ?? "",
        (data.supportUnits ?? [])
            .map((u) => {
                const sk = (u.skills ?? []).map((s) => `${s.mastery}.${s.skillLevel}`).join(",");
                const md = (u.modules ?? []).map((m) => m.level).join(",");
                return `${u.id}:${u.elite}:${u.level}:s${sk}:m${md}`;
            })
            .join("|"),
        data.rarityCounts ? [6, 5, 4, 3, 2, 1].map((r) => `${r}=${data.rarityCounts?.[r] ?? 0}`).join(",") : "",
    ],
    template: (data) => UserTemplate(data),
});

const TIER_LIST_HASH_VERSION = "v3";

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

interface IBackendTierListPlacement {
    operator_id: string;
    sub_order: number;
}

interface IBackendTierListTier {
    id: string;
    name: string;
    display_order: number;
    color: string | null;
    placements: IBackendTierListPlacement[];
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

const tierListHandler = defineOgHandler<ITierListOgData>({
    kind: "tier-list",
    hashVersion: TIER_LIST_HASH_VERSION,
    fetch: async (slug) => {
        const enc = encodeURIComponent(slug);
        const [detailRes, opsRes] = await Promise.all([backendFetch(`/tier-lists/${enc}`), backendFetch("/operators/index")]);
        if (!detailRes.ok) return null;
        const detail = (await detailRes.json()) as IBackendTierListResponse;
        const opIndex = opsRes.ok ? ((await opsRes.json()) as IOperatorIndexEntry[]) : [];
        const opById = new Map<string, IOperatorIndexEntry>(opIndex.map((op) => [op.id, op]));

        const sortedTiers = [...detail.tiers].sort((a, b) => a.display_order - b.display_order);

        const tiers: ITierListTierPreview[] = sortedTiers.slice(0, 4).map((t, i) => {
            const fallback = FALLBACK_TIER_HEX[i % FALLBACK_TIER_HEX.length] as string;
            const placements = [...t.placements].sort((a, b) => a.sub_order - b.sub_order);
            const operators: ITierListOperatorPreview[] = placements
                .slice(0, 5)
                .map((p): ITierListOperatorPreview | null => {
                    const op = opById.get(p.operator_id);
                    if (!op) return null;
                    return {
                        id: op.id,
                        name: op.name,
                        rarity: op.rarity,
                        avatarURL: avatarURL(op.id),
                    };
                })
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
            tiers,
        };
    },
    hashParts: (data) => [
        data.title,
        data.slug,
        data.description ?? "",
        data.listType,
        data.flairLabel ?? "",
        data.flairColor ?? "",
        data.authorName ?? "",
        data.authorAvatarURL ?? "",
        data.views ?? 0,
        data.favorites ?? 0,
        data.isTrending ? 1 : 0,
        data.updatedRelative ?? "",
        data.totalOperators,
        data.tierCount,
        data.tiers.map((t) => `${t.name}:${t.color}:${t.operatorCount}:${t.operators.map((o) => o.id).join(",")}`).join("|"),
    ],
    template: (data) => TierListTemplate(data),
});

const DEFAULT_HASH_VERSION = "v6";

// Canonical id for the site-wide fallback OG image. Slugs registered in
// DEFAULT_OG_PRESETS resolve to their preset; anything else is treated as a
// literal (URL-encoded) title for one-off pages.
export const DEFAULT_OG_ID = "_root";

const defaultHandler = defineOgHandler<IDefaultOgData>({
    kind: "default",
    hashVersion: DEFAULT_HASH_VERSION,
    fetch: async (id) => {
        const preset = (DEFAULT_OG_PRESETS as Record<string, IDefaultOgData>)[id];
        if (preset) return preset;
        return { title: decodeURIComponent(id) };
    },
    hashParts: (data) => [data.title, data.subtitle ?? "", data.activeTag ?? ""],
    template: (data) => DefaultTemplate(data),
});

const TIER_LIST_BOARD_IMAGE_HASH_VERSION = "v12";

async function fetchToDataURI(url: string): Promise<string | undefined> {
    if (!url) return undefined;
    try {
        const res = await fetch(url);
        if (!res.ok) return undefined;
        const contentType = res.headers.get("content-type") ?? "image/png";
        const buf = Buffer.from(await res.arrayBuffer());
        return `data:${contentType};base64,${buf.toString("base64")}`;
    } catch {
        return undefined;
    }
}

const tierListBoardImageHandler = defineOgHandler<ITierListBoardImageData>({
    kind: "tier-list-image",
    hashVersion: TIER_LIST_BOARD_IMAGE_HASH_VERSION,
    fetch: async (slug) => {
        const enc = encodeURIComponent(slug);
        const [detailRes, opsRes] = await Promise.all([backendFetch(`/tier-lists/${enc}`), backendFetch("/operators/index")]);
        if (!detailRes.ok) return null;
        const detail = (await detailRes.json()) as IBackendTierListResponse;
        const opIndex = opsRes.ok ? ((await opsRes.json()) as IOperatorIndexEntry[]) : [];
        const opById = new Map<string, IOperatorIndexEntry>(opIndex.map((op) => [op.id, op]));

        const sortedTiers = [...detail.tiers].sort((a, b) => a.display_order - b.display_order);

        const tiers: ITierListBoardImageTier[] = sortedTiers.map((t, i) => {
            const fallback = FALLBACK_TIER_HEX[i % FALLBACK_TIER_HEX.length] as string;
            const placements = [...t.placements].sort((a, b) => a.sub_order - b.sub_order);
            const operators: ITierListBoardImageOperator[] = placements
                .map((p): ITierListBoardImageOperator | null => {
                    const op = opById.get(p.operator_id);
                    if (!op) return null;
                    return {
                        id: op.id,
                        name: op.name,
                        rarity: op.rarity,
                        avatarURL: avatarURL(op.id),
                    };
                })
                .filter((x): x is ITierListBoardImageOperator => x !== null);
            return {
                name: t.name,
                color: toHex(t.color, fallback),
                operators,
            };
        });

        return {
            title: detail.name,
            slug: detail.slug,
            tiers,
        };
    },
    hashParts: (data) => [TIER_LIST_BOARD_IMAGE_LAYOUT.width, data.title, data.slug, data.tiers.map((t) => `${t.name}:${t.color}:${t.operators.map((o) => `${o.id}.${o.rarity}`).join(",")}`).join("|")],
    template: (data) => TierListBoardImageTemplate(data),
    dimensions: (data) => tierListBoardImageDimensions(data),
});

const STAGE_HASH_VERSION = "v1";
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

const stageHandler = defineOgHandler<IStageOgData>({
    kind: "stage",
    hashVersion: STAGE_HASH_VERSION,
    fetch: async (stageId) => {
        const [stages, zones] = await Promise.all([getCachedStages().catch(() => [] as IStage[]), getCachedZones().catch(() => [] as IZone[])]);
        const stage = stages.find((s) => s.stageId === stageId);
        if (!stage) return null;

        const zone = zones.find((z) => z.zoneId === stage.zoneId);
        const previewImageURL = await resolveStagePreview(stagePreviewAssetPaths(stage));

        return buildStageOgData(stage, zone, previewImageURL);
    },
    hashParts: (data) => [data.code, data.name, data.description ?? "", data.zoneName, data.typeLabel, data.difficultyLabel ?? "", data.bossMark ? 1 : 0, data.stats.map((s) => `${s.label}=${s.value}`).join("|")],
    template: (data) => StageTemplate(data),
});

export const ogRegistry = {
    operator: operatorHandler,
    user: userHandler,
    "tier-list": tierListHandler,
    "tier-list-image": tierListBoardImageHandler,
    stage: stageHandler,
    default: defaultHandler,
};

export type OgKind = keyof typeof ogRegistry;

// biome-ignore lint/suspicious/noExplicitAny: registry erases per-kind data type at lookup
export type IAnyOgHandler = IOgHandler<any>;

export function getHandler(kind: string): IAnyOgHandler {
    return (ogRegistry as Record<string, IAnyOgHandler>)[kind] ?? ogRegistry.default;
}
