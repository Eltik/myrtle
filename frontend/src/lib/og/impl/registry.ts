import type { ReactNode } from "react";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorIndexEntry, IOperatorListItem, IOperatorsStaticMap } from "#/types/operators";
import type { IUserProfile } from "#/types/user";
import { ogHash } from "./hash";
import { type DefaultOgData, DefaultTemplate } from "./templates/Default";
import { type OperatorOgData, OperatorTemplate } from "./templates/Operator";
import { type UserOgData, UserTemplate } from "./templates/User";

export interface OgHandler<TData> {
    fetch: (id: string) => Promise<TData | null>;
    hash: (data: TData) => string;
    template: (data: TData) => ReactNode;
    listIds?: () => Promise<string[]>;
}

const operatorHandler: OgHandler<OperatorOgData> = {
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
            rarity: (Number(String(op.rarity).replace("TIER_", "")) || 1) as OperatorOgData["rarity"],
        };
    },
    hash: (data) => ogHash(["operator", data.name, data.appellation, data.profession, data.rarity]),
    template: (data) => OperatorTemplate(data),
    listIds: async () => {
        const res = await backendFetch("/operators/index");
        if (!res.ok) throw new Error(`operators/index failed: ${res.status}`);
        const list = (await res.json()) as IOperatorIndexEntry[];
        return list.map((o) => o.id);
    },
};

const userHandler: OgHandler<UserOgData> = {
    fetch: async (uid) => {
        const res = await backendFetch(`/get-user?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) return null;
        const u = (await res.json()) as IUserProfile;
        return {
            nickname: u.nickname ?? "Doctor",
            level: u.level,
            grade: u.grade,
            totalScore: u.total_score,
        };
    },
    hash: (data) => ogHash(["user", data.nickname, data.level, data.grade, data.totalScore]),
    template: (data) => UserTemplate(data),
};

const defaultHandler: OgHandler<DefaultOgData> = {
    fetch: async (id) => ({ title: decodeURIComponent(id), subtitle: undefined }),
    hash: (data) => ogHash(["default", data.title, data.subtitle ?? ""]),
    template: (data) => DefaultTemplate(data),
};

export const ogRegistry = {
    operator: operatorHandler,
    user: userHandler,
    default: defaultHandler,
};

export type OgKind = keyof typeof ogRegistry;

// biome-ignore lint/suspicious/noExplicitAny: registry erases per-kind data type at lookup
export type AnyOgHandler = OgHandler<any>;

export function getHandler(kind: string): AnyOgHandler {
    return (ogRegistry as Record<string, AnyOgHandler>)[kind] ?? ogRegistry.default;
}
