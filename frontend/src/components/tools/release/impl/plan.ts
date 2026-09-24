import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseEventsQueryOptions, releaseSkinsQueryOptions } from "#/lib/api/release";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { AutoName } from "#/types/generated/AutoName";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { EventShop } from "#/types/generated/EventShop";
import type { FarmStage } from "#/types/generated/FarmStage";
import type { OpStage } from "#/types/generated/OpStage";
import type { ReleaseEvent } from "#/types/generated/ReleaseEvent";
import type { Resolution } from "#/types/generated/Resolution";
import type { ShopGood } from "#/types/generated/ShopGood";
import type { ShopGoodKind } from "#/types/generated/ShopGoodKind";
import type { SkinPrice } from "#/types/generated/SkinPrice";
import type { SkinTile } from "#/types/generated/SkinTile";
import type { StageClearsMap } from "#/types/stages";
import { groupNewSkins } from "./components/SkinsTab";
import { buildOperatorLookup, type OperatorLookup } from "./components/shared";
import { isPast, resolvedEnStart, sortKey } from "./helpers";
import type { messages as planMessages } from "./plan.messages";
import { REVIEW_NAME_CN, REVIEW_NAME_EN, reviewOutfits, reviewYearGroup } from "./reviews";
import type { messages as reviewMessages } from "./reviews.messages";
import { cnDay } from "./schedule";

/** A key in `plan.messages.ts`; resolved by whichever component renders it. */
export type PlanMessageKey = keyof typeof planMessages & string;

/** This module derives the store-sale name and the review year headings. */
type PlanT = TypedT<typeof planMessages & typeof reviewMessages>;

export interface IPlanSkin {
    skinId: string;
    charId: string;
    skinName: string;
    skinNameEn: string | null;
    skinNameAuto: AutoName | null;
    charName: AutoName | null;
    groupName: string;
    portraitPath: string | null;
    rerun: boolean;
    price: SkinPrice;
    colors: string[];
}

export interface IPlanRow {
    key: string;
    kind: "event" | "listing" | "review";
    cnId: string | null;
    nameCn: string;
    nameEn: string | null;
    nameAuto: AutoName | null;
    imagePath: string | null;
    enStart: number;
    resolution: Resolution;
    opStages: OpStage[];
    farmStages: FarmStage[];
    missionTokens: number;
    shop: EventShop | null;
    rerun: boolean;
    skins: IPlanSkin[];
}

export interface IPlanState {
    initial: number;
    initialManual: boolean;
    picks: Record<string, true>;
    stages: Record<string, Record<string, boolean>>;
}

const STORAGE_KEY = "release-planner:plan:v1";

export const EMPTY_STATE: IPlanState = { initial: 0, initialManual: false, picks: {}, stages: {} };

export function loadState(): IPlanState {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return EMPTY_STATE;
        const parsed = JSON.parse(raw) as Partial<IPlanState>;
        const picks: Record<string, true> = {};
        for (const id of Object.keys(parsed.picks ?? {})) picks[id] = true;
        return { initial: Number(parsed.initial) || 0, initialManual: parsed.initialManual === true, picks, stages: parsed.stages ?? {} };
    } catch {
        return EMPTY_STATE;
    }
}

export function saveState(state: IPlanState): void {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch {}
}

function isRerun(e: ReleaseEvent): boolean {
    return e.cnId.endsWith("sre");
}

function planSkin(tile: SkinTile, groupName: string, rerun: boolean, names: { skinNameEn?: string; skinNameAuto?: AutoName | null }): IPlanSkin {
    return {
        skinId: tile.skinId,
        charId: tile.charId,
        skinName: tile.skinName,
        skinNameEn: names.skinNameEn ?? null,
        skinNameAuto: names.skinNameAuto ?? null,
        charName: tile.charName,
        groupName,
        portraitPath: tile.portraitPath,
        rerun,
        price: tile.price,
        colors: tile.colors,
    };
}

export interface IPlanData {
    rows: IPlanRow[];
    lookup: OperatorLookup;
    isPending: boolean;
    error: Error | null;
}

export function usePlanData(today: Date, showPast: boolean): IPlanData {
    const t: PlanT = useT("tools");
    const events = useQuery(releaseEventsQueryOptions());
    const skins = useQuery(releaseSkinsQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);

    const rows = React.useMemo(() => {
        const model = events.data?.model ?? skins.data?.model ?? null;
        const byEvent = new Map<string, IPlanRow>();
        const listings: IPlanRow[] = [];
        for (const e of events.data?.events ?? []) {
            if (!e.hasStage && e.opStages.length === 0) continue;
            const enStart = resolvedEnStart(e.resolution);
            if (enStart === null) continue;
            byEvent.set(e.cnId, {
                key: `event:${e.cnId}`,
                kind: "event",
                cnId: e.cnId,
                nameCn: e.nameCn,
                nameEn: e.nameEn,
                nameAuto: e.nameEnAuto,
                imagePath: e.imagePath,
                enStart,
                resolution: e.resolution,
                opStages: e.opStages,
                farmStages: e.farmStages,
                missionTokens: e.missionTokens,
                shop: e.shop,
                rerun: isRerun(e),
                skins: [],
            });
        }
        const attach = (anchor: EventAnchor | null | undefined, fallback: () => IPlanRow, skin: IPlanSkin) => {
            const row = (anchor && byEvent.get(anchor.cnId)) || fallback();
            if (!row.skins.some((s) => s.skinId === skin.skinId)) row.skins.push(skin);
        };
        const saleRow = (cnStart: number, resolution: Resolution): IPlanRow => {
            const key = `sale:${cnDay(cnStart)}`;
            const existing = listings.find((l) => l.key === key);
            if (existing) return existing;
            const row: IPlanRow = { key, kind: "listing", cnId: null, nameCn: "商店上架", nameEn: t("release.plan.storeSale"), nameAuto: null, imagePath: null, enStart: resolvedEnStart(resolution) ?? 0, resolution, opStages: [], farmStages: [], missionTokens: 0, shop: null, rerun: false, skins: [] };
            listings.push(row);
            return row;
        };
        for (const g of groupNewSkins(skins.data?.newSkins ?? [], model)) {
            if (resolvedEnStart(g.resolution) === null) continue;
            for (const s of g.skins) {
                attach(g.anchor, () => saleRow(g.cnGetTime, g.resolution), planSkin(s, g.skinGroupName || g.skinGroupId, false, { skinNameAuto: s.skinNameAuto }));
            }
        }
        for (const r of skins.data?.rerunForecasts ?? []) {
            if (resolvedEnStart(r.next) === null) continue;
            const listing = r.basis.kind === "cn_listing" ? r.basis : null;
            for (const s of r.skins) {
                attach(listing?.anchor, () => saleRow(listing?.cn_start ?? resolvedEnStart(r.next) ?? 0, r.next), planSkin(s, r.skinGroupName || r.skinGroupId, true, { skinNameEn: s.skinName }));
            }
        }
        const pool = skins.data?.reviewPool ?? [];
        const reviews: IPlanRow[] = (skins.data?.reviews ?? []).flatMap((r) => {
            const enStart = resolvedEnStart(r.resolution);
            if (enStart === null) return [];
            const outfits = reviewOutfits(r, pool);
            return [
                {
                    key: `review:${r.cnStart}`,
                    kind: "review" as const,
                    cnId: null,
                    nameCn: REVIEW_NAME_CN,
                    nameEn: REVIEW_NAME_EN,
                    nameAuto: null,
                    imagePath: null,
                    enStart,
                    resolution: r.resolution,
                    opStages: [],
                    farmStages: [],
                    missionTokens: 0,
                    shop: null,
                    rerun: true,
                    skins: outfits.map((o) => planSkin(o, reviewYearGroup(o, t), true, { skinNameEn: o.skinName })).reverse(),
                },
            ];
        });
        const all = [...byEvent.values(), ...listings, ...reviews].filter((row) => showPast || !isPast(sortKey(row.resolution, row.enStart, model), today));
        all.sort((a, b) => a.enStart - b.enStart || a.key.localeCompare(b.key));
        return all;
    }, [events.data, skins.data, showPast, today, t]);

    return {
        rows,
        lookup,
        isPending: events.isPending || skins.isPending,
        error: events.error ?? skins.error ?? null,
    };
}

export interface IRowBalance {
    income: number;
    expense: number;
    balance: number;
}

export type StageClears = StageClearsMap | null;

export type StageStatus = "claimed" | "open" | "unrated" | "unknown";

export function stageStatus(stage: OpStage, clears: StageClears): StageStatus {
    const c = clears?.[stage.stageId];
    if (!c) return "unknown";
    if (c.state >= 3) return "claimed";
    if ((c.stateMax ?? c.state) < 3) return "open";
    return c.state >= 2 ? "unrated" : "unknown";
}

export function rowTouched(row: IPlanRow, clears: StageClears): boolean {
    return !!clears && row.opStages.some((st) => ["claimed", "unrated"].includes(stageStatus(st, clears)));
}

/** A stage's pick key: its id, since codes repeat within an event (奇象巡展 lists EE-01 twice). */
export function stageKey(stage: OpStage): string {
    return stage.stageId;
}

/** The code-based key plans were saved under before picks were keyed by id; still read so old picks survive. */
function legacyStageKey(stage: OpStage): string {
    return stage.challenge ? `${stage.code} CM` : stage.code;
}

function stagePick(chosen: Record<string, boolean> | undefined, stage: OpStage): boolean | undefined {
    return chosen?.[stageKey(stage)] ?? chosen?.[legacyStageKey(stage)];
}

export function stageDefault(row: IPlanRow, stage: OpStage, clears: StageClears): boolean {
    switch (stageStatus(stage, clears)) {
        case "claimed":
        case "unrated":
            return false;
        case "open":
            return true;
        default:
            if (clears && !rowTouched(row, clears)) return true;
            return !row.rerun;
    }
}

export function stageOn(row: IPlanRow, stage: OpStage, state: IPlanState, clears: StageClears): boolean {
    return stagePick(state.stages[row.key], stage) ?? stageDefault(row, stage, clears);
}

export function rowDeviates(row: IPlanRow, state: IPlanState, clears: StageClears): boolean {
    const chosen = state.stages[row.key];
    if (!chosen) return false;
    return row.opStages.some((st) => {
        const pick = stagePick(chosen, st);
        return pick !== undefined && pick !== stageDefault(row, st, clears);
    });
}

export function rowIncome(row: IPlanRow, state: IPlanState, clears: StageClears): number {
    return row.opStages.reduce((sum, st) => sum + (stageOn(row, st, state, clears) ? st.op : 0), 0);
}

export function rowPotential(row: IPlanRow): number {
    return row.opStages.reduce((sum, st) => sum + st.op, 0);
}

export function rowExpense(row: IPlanRow, state: IPlanState): number {
    return row.skins.reduce((sum, s) => sum + (state.picks[s.skinId] ? s.price.price : 0), 0);
}

export function balances(rows: IPlanRow[], state: IPlanState, clears: StageClears): Map<string, IRowBalance> {
    const out = new Map<string, IRowBalance>();
    let running = state.initial;
    for (const row of rows) {
        const income = rowIncome(row, state, clears);
        const expense = rowExpense(row, state);
        running += income - expense;
        out.set(row.key, { income, expense, balance: running });
    }
    return out;
}

export const SANITY_PER_TOKEN = 1;

export interface IShopBuyout {
    total: number;
    missions: number;
    remaining: number;
    sanity: number;
    limitedGoods: number;
}

export function shopBuyout(row: IPlanRow): IShopBuyout | null {
    if (!row.shop) return null;
    const remaining = Math.max(0, row.shop.maxPrice - row.missionTokens);
    return { total: row.shop.maxPrice, missions: row.missionTokens, remaining, sanity: remaining * SANITY_PER_TOKEN, limitedGoods: row.shop.goods.filter((g) => g.availCount > 0).length };
}

export const SHOP_KIND_LABEL_KEYS: Record<ShopGoodKind, PlanMessageKey> = {
    outfit: "release.shopKind.outfit",
    furniture: "release.shopKind.furniture",
    material: "release.shopKind.material",
    currency: "release.shopKind.currency",
    exp: "release.shopKind.exp",
    ticket: "release.shopKind.ticket",
    other: "release.shopKind.other",
};
const SHOP_KIND_ORDER: ShopGoodKind[] = ["outfit", "ticket", "material", "exp", "currency", "furniture", "other"];

export interface IShopGroup {
    kind: ShopGoodKind;
    goods: ShopGood[];
    /** Tokens to clear the group's limited stock. */
    tokens: number;
}

/** Limited goods by kind, priciest kind first within the fixed order; unlimited goods form their own trailing group. */
export function shopGroups(shop: EventShop): { limited: IShopGroup[]; unlimited: ShopGood[] } {
    const limited = SHOP_KIND_ORDER.map((kind) => {
        const goods = shop.goods.filter((g) => g.kind === kind && g.availCount > 0).sort((a, b) => b.price - a.price || a.goodId.localeCompare(b.goodId));
        return { kind, goods, tokens: goods.reduce((sum, g) => sum + g.price * g.availCount, 0) };
    }).filter((g) => g.goods.length > 0);
    const unlimited = shop.goods.filter((g) => g.availCount < 0).sort((a, b) => b.price - a.price);
    return { limited, unlimited };
}
