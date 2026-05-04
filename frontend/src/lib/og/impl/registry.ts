import type { ReactNode } from "react";
import { env } from "#/env";
import type { IRosterEntry } from "#/lib/api/user";
import { backendFetch } from "#/lib/fetch";
import { rarityToNumber, toAvatarStem } from "#/lib/utils";
import type { IOperatorIndexEntry, IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";
import type { IUserProfile } from "#/types/user";
import { ogHash } from "./hash";
import { DefaultTemplate, type IDefaultOgData } from "./templates/Default";
import { type IOperatorOgData, OperatorTemplate } from "./templates/Operator";
import { type IUserOgData, type IUserSupportModule, type IUserSupportSkill, type IUserSupportUnit, UserTemplate } from "./templates/User";

export interface IOgHandler<TData> {
    fetch: (id: string) => Promise<TData | null>;
    hash: (data: TData) => string;
    template: (data: TData) => ReactNode;
    listIds?: () => Promise<string[]>;
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
const skinpackURL = (operatorId: string, skinId: string) => assetURL(`/textures/skinpack/${operatorId}/${skinId.replaceAll("@", "_").replaceAll("#", "%23")}.png`);
const campLogoURL = (id: string) => assetURL(`/textures/spritepack/ui_camp_logo_0/logo_${id.toLowerCase()}.png`);
const moduleIconURL = (uniEquipIcon: string) => assetURL(`/textures/spritepack/ui_equip_big_img_hub_0/${uniEquipIcon}.png`);
const masteryIconURL = (mastery: number) => assetURL(`/textures/arts/specialized_hub/specialized_${mastery}.png`);
const secretaryArtURL = (operatorId: string, skinId: string | null) => (skinId?.includes("@") ? skinpackURL(operatorId, skinId) : charartURL(operatorId));

const OPERATOR_HASH_VERSION = "v2";

const operatorHandler: IOgHandler<IOperatorOgData> = {
    fetch: async (id) => {
        const res = await backendFetch("/static/operators");
        if (!res.ok) return null;
        const map = (await res.json()) as IOperatorsStaticMap;
        const op = (Object.values(map) as IOperatorListItem[]).find((o) => o.id === id);
        if (!op) return null;
        return {
            name: op.name,
            appellation: op.appellation ?? "",
            profession: op.profession,
            subProfession: op.subProfessionId ?? "",
            position: op.position ?? "",
            nationId: op.nationId ?? "",
            rarity: rarityToNumber(op.rarity),
            charArtUrl: assetURL(op.skin ?? op.portrait ?? `/textures/chararts/${id}/${id}_2.png`),
            factionLogoUrl: op.nationId ? campLogoURL(op.nationId) : undefined,
        };
    },
    hash: (data) => ogHash(["operator", OPERATOR_HASH_VERSION, data.name, data.appellation, data.profession, data.rarity, data.subProfession, data.position, data.nationId]),
    template: (data) => OperatorTemplate(data),
    listIds: async () => {
        const res = await backendFetch("/operators/index");
        if (!res.ok) throw new Error(`operators/index failed: ${res.status}`);
        return ((await res.json()) as IOperatorIndexEntry[]).map((o) => o.id);
    },
};

const USER_HASH_VERSION = "v11";

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
        const iconUrl = sk.static?.image ? assetURL(sk.static.image) : iconId ? skillIconURL(iconId) : undefined;
        const mastery = masteryByIndex.get(idx) ?? 0;
        return {
            iconUrl,
            mastery,
            skillLevel: rosterEntry?.skill_level ?? 7,
            masteryIconUrl: mastery >= 1 ? masteryIconURL(mastery) : undefined,
        };
    });

    const modules: IUserSupportModule[] = (op.modules ?? [])
        .filter((m) => m.typeName1 !== "ORIGINAL")
        .slice(0, 3)
        .map((m) => ({
            iconUrl: m.image ? assetURL(m.image) : moduleIconURL(m.uniEquipIcon),
            level: moduleStateById.get(m.uniEquipId)?.level ?? 0,
        }));

    return {
        id,
        name: op.name,
        rarity: rarityToNumber(op.rarity),
        elite,
        level,
        avatarUrl: avatarURL(skinId || id),
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

const userHandler: IOgHandler<IUserOgData> = {
    fetch: async (uid) => {
        const enc = encodeURIComponent(uid);
        const [userRes, supportsRes, rosterRes, opsRes] = await Promise.all([backendFetch(`/get-user?uid=${enc}`), backendFetch(`/get-user-supports?uid=${enc}`), backendFetch(`/roster?uid=${enc}`), backendFetch("/static/operators")]);
        if (!userRes.ok) return null;
        const u = (await userRes.json()) as IUserProfile;

        const opMap = opsRes.ok ? ((await opsRes.json()) as IOperatorsStaticMap) : undefined;
        const opByIdMap = new Map<string, IOperatorListItem>();
        if (opMap) {
            for (const op of Object.values(opMap) as IOperatorListItem[]) {
                if (op.id) opByIdMap.set(op.id, op);
            }
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
            secretaryArtUrl: u.secretary ? secretaryArtURL(u.secretary, u.secretary_skin_id) : undefined,
            supportUnits,
            supportUnitsKind,
            rarityCounts: hasRarityCounts ? rarityCounts : undefined,
        };
    },
    hash: (data) =>
        ogHash([
            "user",
            USER_HASH_VERSION,
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
            data.secretaryArtUrl ?? "",
            data.supportUnitsKind ?? "",
            (data.supportUnits ?? [])
                .map((u) => {
                    const sk = (u.skills ?? []).map((s) => `${s.mastery}.${s.skillLevel}`).join(",");
                    const md = (u.modules ?? []).map((m) => m.level).join(",");
                    return `${u.id}:${u.elite}:${u.level}:s${sk}:m${md}`;
                })
                .join("|"),
            data.rarityCounts ? [6, 5, 4, 3, 2, 1].map((r) => `${r}=${data.rarityCounts?.[r] ?? 0}`).join(",") : "",
        ]),
    template: (data) => UserTemplate(data),
};

const DEFAULT_HASH_VERSION = "v2";

const defaultHandler: IOgHandler<IDefaultOgData> = {
    fetch: async (id) => ({ title: decodeURIComponent(id), subtitle: undefined }),
    hash: (data) => ogHash(["default", DEFAULT_HASH_VERSION, data.title, data.subtitle ?? ""]),
    template: (data) => DefaultTemplate(data),
};

export const ogRegistry = {
    operator: operatorHandler,
    user: userHandler,
    default: defaultHandler,
};

export type OgKind = keyof typeof ogRegistry;

// biome-ignore lint/suspicious/noExplicitAny: registry erases per-kind data type at lookup
export type IAnyOgHandler = IOgHandler<any>;

export function getHandler(kind: string): IAnyOgHandler {
    return (ogRegistry as Record<string, IAnyOgHandler>)[kind] ?? ogRegistry.default;
}
