import type { IGridImageData } from "./grid";
import { ogHash } from "./hash";
import { type IStoryOgData, storyOgHashParts } from "./story";
import type { IDefaultOgData } from "./templates/Default";
import { GRID_IMAGE_LAYOUT } from "./templates/GridImage";
import type { IOperatorOgData } from "./templates/Operator";
import type { IStageOgData } from "./templates/Stage";
import type { ITierListOgData } from "./templates/TierList";
import { type ITierListBoardImageData, TIER_LIST_BOARD_IMAGE_LAYOUT } from "./templates/TierListBoardImage";
import type { IUserOgData } from "./templates/User";

// The per-kind `?v=` hash, apart from the registry so building an og:image URL on the
// client does not bundle the templates and their server-only fetchers.

export interface IOgHasher<TData> {
    hash: (data: TData) => string;
    cacheVersion: (id: string, version?: string) => string;
}

// A kind declares its `kind` and `hashVersion` once; `hash` and `cacheVersion` are
// derived from them so neither the kind string nor the version constant is restated.
// Both feed ogHash the same leading `[kind, hashVersion, ...]`, so the resulting cache
// keys are byte-identical to hand-written signatures.
interface IOgHasherDef<TData> {
    kind: string;
    hashVersion: string;
    hashParts: (data: TData) => unknown[];
}

function defineOgHasher<TData>(def: IOgHasherDef<TData>): IOgHasher<TData> {
    const { kind, hashVersion } = def;
    return {
        hash: (data) => ogHash([kind, hashVersion, ...def.hashParts(data)]),
        cacheVersion: (id, version) => version || ogHash([kind, hashVersion, id]),
    };
}

const OPERATOR_HASH_VERSION = "v9";

const operatorHasher = defineOgHasher<IOperatorOgData>({
    kind: "operator",
    hashVersion: OPERATOR_HASH_VERSION,
    // `server` participates so an operator that graduates from CN to Global
    // re-renders against the Global asset tree instead of serving a stale card.
    hashParts: (data) => [data.name, data.appellation, data.profession, data.rarity, data.subProfession, data.position, data.nationId, data.factionLabel ?? "", data.professionIconURL ?? "", (data.stats ?? []).map((s) => `${s.label}=${s.value}`).join("|"), data.server ?? ""],
});

const USER_HASH_VERSION = "v15";

const userHasher = defineOgHasher<IUserOgData>({
    kind: "user",
    hashVersion: USER_HASH_VERSION,
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
});

const TIER_LIST_HASH_VERSION = "v3";

const tierListHasher = defineOgHasher<ITierListOgData>({
    kind: "tier-list",
    hashVersion: TIER_LIST_HASH_VERSION,
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
});

const TIER_LIST_BOARD_IMAGE_HASH_VERSION = "v12";

const tierListBoardImageHasher = defineOgHasher<ITierListBoardImageData>({
    kind: "tier-list-image",
    hashVersion: TIER_LIST_BOARD_IMAGE_HASH_VERSION,
    hashParts: (data) => [TIER_LIST_BOARD_IMAGE_LAYOUT.width, data.title, data.slug, data.tiers.map((t) => `${t.name}:${t.color}:${t.operators.map((o) => `${o.id}.${o.rarity}`).join(",")}`).join("|")],
});

const GRID_IMAGE_HASH_VERSION = "v2";

const gridImageHasher = defineOgHasher<IGridImageData>({
    kind: "grid-image",
    hashVersion: GRID_IMAGE_HASH_VERSION,
    hashParts: (data) => [GRID_IMAGE_LAYOUT.width, data.title, data.slug, data.server, data.rows, data.cols, data.cells.map((c) => `${c.label}=${c.key ?? ""}`).join("|")],
});

const STAGE_HASH_VERSION = "v1";

const stageHasher = defineOgHasher<IStageOgData>({
    kind: "stage",
    hashVersion: STAGE_HASH_VERSION,
    hashParts: (data) => [data.code, data.name, data.description ?? "", data.zoneName, data.typeLabel, data.difficultyLabel ?? "", data.bossMark ? 1 : 0, data.stats.map((s) => `${s.label}=${s.value}`).join("|")],
});

const STORY_HASH_VERSION = "v2";

const storyHasher = defineOgHasher<IStoryOgData>({
    kind: "story",
    hashVersion: STORY_HASH_VERSION,
    hashParts: (data) => storyOgHashParts(data),
});

const DEFAULT_HASH_VERSION = "v6";

const defaultHasher = defineOgHasher<IDefaultOgData>({
    kind: "default",
    hashVersion: DEFAULT_HASH_VERSION,
    // `tagLabels` are deliberately absent: they are the same five words on
    // every card in a given locale, and the locale is already in the cache key
    // (see `ogResponse`). Hashing them would change every English URL for no
    // change in the image.
    hashParts: (data) => [data.title, data.subtitle ?? "", data.activeTag ?? ""],
});

interface IOgDataByKind {
    operator: IOperatorOgData;
    user: IUserOgData;
    "tier-list": ITierListOgData;
    "tier-list-image": ITierListBoardImageData;
    "grid-image": IGridImageData;
    stage: IStageOgData;
    story: IStoryOgData;
    default: IDefaultOgData;
}

export type OgKind = keyof IOgDataByKind;

export type OgData<K extends OgKind> = IOgDataByKind[K];

// Mapped so indexing by a generic kind keeps its data type rather than a union.
export const ogHashers: { [K in OgKind]: IOgHasher<OgData<K>> } = {
    operator: operatorHasher,
    user: userHasher,
    "tier-list": tierListHasher,
    "tier-list-image": tierListBoardImageHasher,
    "grid-image": gridImageHasher,
    stage: stageHasher,
    story: storyHasher,
    default: defaultHasher,
};
