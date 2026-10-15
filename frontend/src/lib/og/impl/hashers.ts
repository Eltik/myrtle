import { archiveOgHashParts, type IArchiveOgData } from "./archive";
import { enemyOgHashParts, type IEnemyOgData } from "./enemy";
import type { IGridImageData } from "./grid";
import { ogHash } from "./hash";
import { type IMusicOgData, musicOgHashParts } from "./music";
import { type IRecruitmentOgData, recruitmentOgHashParts } from "./recruitment";
import { type ISquadOgData, squadOgHashParts } from "./squad";
import { type IStoryOgData, storyOgHashParts } from "./story";
import type { IDefaultOgData } from "./templates/Default";
import { GRID_IMAGE_LAYOUT } from "./templates/GridImage";
import type { IOperatorOgData } from "./templates/Operator";
import type { IStageOgData } from "./templates/Stage";
import type { ITierListOgData } from "./templates/TierList";
import { type ITierListBoardImageData, TIER_LIST_BOARD_IMAGE_LAYOUT } from "./templates/TierListBoardImage";
import { type IUserOgData, userOgHashParts } from "./user";

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

const OPERATOR_HASH_VERSION = "v10";

const operatorHasher = defineOgHasher<IOperatorOgData>({
    kind: "operator",
    hashVersion: OPERATOR_HASH_VERSION,
    // `server` participates so an operator that graduates from CN to Global
    // re-renders against the Global asset tree instead of serving a stale card.
    hashParts: (data) => [data.name, data.appellation, data.profession, data.professionName ?? "", data.rarity, data.subProfession, data.position, data.nationId, data.factionLabel ?? "", data.professionIconURL ?? "", (data.stats ?? []).map((s) => `${s.label}=${s.value}`).join("|"), data.server ?? ""],
});

// v17: the card is redrawn and hashes every field it draws (`userOgHashParts`);
// v16 hashed only what the route's partial data happened to carry.
const USER_HASH_VERSION = "v17";

const userHasher = defineOgHasher<IUserOgData>({
    kind: "user",
    hashVersion: USER_HASH_VERSION,
    hashParts: (data) => userOgHashParts(data),
});

const RECRUITMENT_HASH_VERSION = "v1";

const recruitmentHasher = defineOgHasher<IRecruitmentOgData>({
    kind: "recruitment",
    hashVersion: RECRUITMENT_HASH_VERSION,
    hashParts: (data) => recruitmentOgHashParts(data),
});

const TIER_LIST_HASH_VERSION = "v4";

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

const TIER_LIST_BOARD_IMAGE_HASH_VERSION = "v13";

const tierListBoardImageHasher = defineOgHasher<ITierListBoardImageData>({
    kind: "tier-list-image",
    hashVersion: TIER_LIST_BOARD_IMAGE_HASH_VERSION,
    hashParts: (data) => [TIER_LIST_BOARD_IMAGE_LAYOUT.width, data.title, data.slug, data.tiers.map((t) => `${t.name}:${t.color}:${t.operators.map((o) => `${o.id}.${o.rarity}`).join(",")}`).join("|")],
});

const GRID_IMAGE_HASH_VERSION = "v4";

const gridImageHasher = defineOgHasher<IGridImageData>({
    kind: "grid-image",
    hashVersion: GRID_IMAGE_HASH_VERSION,
    hashParts: (data) => [GRID_IMAGE_LAYOUT.width, data.title, data.slug, data.server, data.rows, data.cols, data.cells.map((c) => `${c.label}=${c.key ?? ""}@${c.artServer ?? ""}`).join("|")],
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

const ENEMY_HASH_VERSION = "v1";

const enemyHasher = defineOgHasher<IEnemyOgData>({
    kind: "enemy",
    hashVersion: ENEMY_HASH_VERSION,
    hashParts: (data) => enemyOgHashParts(data),
});

const SQUAD_HASH_VERSION = "v1";

const squadHasher = defineOgHasher<ISquadOgData>({
    kind: "squad",
    hashVersion: SQUAD_HASH_VERSION,
    hashParts: (data) => squadOgHashParts(data),
});

const ARCHIVE_HASH_VERSION = "v1";

const archiveHasher = defineOgHasher<IArchiveOgData>({
    kind: "archive",
    hashVersion: ARCHIVE_HASH_VERSION,
    hashParts: (data) => archiveOgHashParts(data),
});

const MUSIC_HASH_VERSION = "v1";

const musicHasher = defineOgHasher<IMusicOgData>({
    kind: "music",
    hashVersion: MUSIC_HASH_VERSION,
    hashParts: (data) => musicOgHashParts(data),
});

// v7: the card names its section and draws mascot art instead of the chip row.
const DEFAULT_HASH_VERSION = "v7";

const defaultHasher = defineOgHasher<IDefaultOgData>({
    kind: "default",
    hashVersion: DEFAULT_HASH_VERSION,
    // The section label is hashed as drawn, so a translator's edit to it
    // re-renders the card like an edit to the title does.
    hashParts: (data) => [data.title, data.subtitle ?? "", data.section ?? "", data.art ?? ""],
});

interface IOgDataByKind {
    operator: IOperatorOgData;
    user: IUserOgData;
    "tier-list": ITierListOgData;
    "tier-list-image": ITierListBoardImageData;
    "grid-image": IGridImageData;
    stage: IStageOgData;
    story: IStoryOgData;
    recruitment: IRecruitmentOgData;
    enemy: IEnemyOgData;
    squad: ISquadOgData;
    archive: IArchiveOgData;
    music: IMusicOgData;
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
    recruitment: recruitmentHasher,
    enemy: enemyHasher,
    squad: squadHasher,
    archive: archiveHasher,
    music: musicHasher,
    default: defaultHasher,
};
