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
import { buildOperatorLookup, type OperatorLookup } from "./components/shared";
import { isPast, resolvedEnStart, sortKey } from "./helpers";
import type { messages as planMessages } from "./plan.messages";
import { REVIEW_NAME_CN, REVIEW_NAME_EN, reviewOutfits, reviewYearGroup } from "./reviews";
import type { messages as reviewMessages } from "./reviews.messages";
import { groupNewSkins } from "./skins";

/** A key in `plan.messages.ts`; resolved by whichever component renders it. */
export type PlanMessageKey = keyof typeof planMessages & string;

/** Store-sale name and review year headings. */
type PlanT = TypedT<typeof planMessages & typeof reviewMessages>;

export interface IPlanSkin {
    skinId: string;
    charId: string;
    skinName: string;
    skinNameEn: string | null;
    skinNameAuto: AutoName | null;
    charName: AutoName | null;
    groupName: string;
    groupNameAuto: AutoName | null;
    /** The outfit series (`mh` for both Monster Hunter editions), so a card sits a set's new and rerun editions together. */
    brand: string;
    /** Sold with the row's own event, rather than on the same day as it. */
    anchored: boolean;
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
    /** EN has closed the event: its stages pay nothing more until a rerun. */
    ended: boolean;
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

interface IPlanGroup {
    name: string;
    nameAuto?: AutoName | null;
    brand: string;
}

/** A group id's series: `2026#mh` and `2023#mh` are both `mh`. */
function brandOf(groupId: string, art: Record<string, { brandId: string } | undefined> | undefined): string {
    return art?.[groupId]?.brandId ?? groupId.split("#")[1] ?? groupId;
}

/** The local calendar day an EN start falls on, the day a card shows. */
function localDay(seconds: number): string {
    const d = new Date(seconds * 1000);
    return `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`;
}

/** A card with no stages, shop or art: a store sale or a review, filled with outfits later. */
function bareRow(fields: Pick<IPlanRow, "key" | "kind" | "nameCn" | "nameEn" | "enStart" | "resolution" | "rerun">): IPlanRow {
    return { cnId: null, nameAuto: null, imagePath: null, ended: false, opStages: [], farmStages: [], missionTokens: 0, shop: null, skins: [], ...fields };
}

function planSkin(tile: SkinTile, group: IPlanGroup, rerun: boolean, names: { skinNameEn?: string; skinNameAuto?: AutoName | null }): IPlanSkin {
    return {
        skinId: tile.skinId,
        charId: tile.charId,
        skinName: tile.skinName,
        skinNameEn: names.skinNameEn ?? null,
        skinNameAuto: names.skinNameAuto ?? null,
        charName: tile.charName,
        groupName: group.name,
        groupNameAuto: group.nameAuto ?? null,
        brand: group.brand,
        anchored: false,
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
        const nowSecs = today.getTime() / 1000;
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
                ended: enEnded(e.resolution, nowSecs),
                opStages: e.opStages,
                farmStages: e.farmStages,
                missionTokens: e.missionTokens,
                shop: e.shop,
                rerun: isRerun(e),
                skins: [],
            });
        }
        const attach = (anchor: EventAnchor | null | undefined, fallback: () => IPlanRow, skin: IPlanSkin) => {
            const own = anchor ? byEvent.get(anchor.cnId) : undefined;
            const row = own ?? fallback();
            if (!row.skins.some((s) => s.skinId === skin.skinId)) row.skins.push({ ...skin, anchored: !!own });
        };
        // One store-sale card per EN day: outfits that list together are one purchase decision.
        const saleRow = (resolution: Resolution): IPlanRow => {
            const enStart = resolvedEnStart(resolution) ?? 0;
            const key = `sale:${localDay(enStart)}`;
            const existing = listings.find((l) => l.key === key);
            if (existing) return existing;
            const row = bareRow({ key, kind: "listing", nameCn: "商店上架", nameEn: t("release.plan.storeSale"), enStart, resolution, rerun: false });
            listings.push(row);
            return row;
        };
        const art = skins.data?.groupArt;
        for (const g of groupNewSkins(skins.data?.newSkins ?? [], model)) {
            if (resolvedEnStart(g.resolution) === null) continue;
            const group = { name: g.skinGroupName || g.skinGroupId, nameAuto: g.skinGroupNameAuto, brand: brandOf(g.skinGroupId, art) };
            for (const s of g.skins) {
                attach(g.anchor, () => saleRow(g.resolution), planSkin(s, group, false, { skinNameAuto: s.skinNameAuto }));
            }
        }
        for (const r of skins.data?.rerunForecasts ?? []) {
            if (resolvedEnStart(r.next) === null) continue;
            const listing = r.basis.kind === "cn_listing" ? r.basis : null;
            const group = { name: r.skinGroupName || r.skinGroupId, brand: brandOf(r.skinGroupId, art) };
            for (const s of r.skins) {
                attach(listing?.anchor, () => saleRow(r.next), planSkin(s, group, true, { skinNameEn: s.skinName }));
            }
        }
        const pool = skins.data?.reviewPool ?? [];
        const reviews: IPlanRow[] = (skins.data?.reviews ?? []).flatMap((r) => {
            // An edition EN skipped sits with the past at its CN start, with
            // nothing to pick: EN never sold it, so no plan can spend on it.
            const unlisted = r.resolution.status === "unlisted";
            const enStart = unlisted ? r.cnStart : resolvedEnStart(r.resolution);
            if (enStart === null) return [];
            const outfits = unlisted ? [] : reviewOutfits(r, pool);
            const row = bareRow({ key: `review:${r.cnStart}`, kind: "review", nameCn: REVIEW_NAME_CN, nameEn: REVIEW_NAME_EN, enStart, resolution: r.resolution, rerun: true });
            row.skins = outfits
                .map((o) => {
                    const name = reviewYearGroup(o, t);
                    return planSkin(o, { name, brand: name }, true, { skinNameEn: o.skinName });
                })
                .reverse();
            return [row];
        });
        const sales = mergeSameDay(listings, [...byEvent.values()], reviews);
        const all = [...byEvent.values(), ...sales, ...reviews].filter((row) => showPast || !isPast(sortKey(row.resolution, row.enStart, model), today));
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

export interface IPlanSkinGroup {
    name: string;
    nameAuto: AutoName | null;
    rerun: boolean;
    skins: IPlanSkin[];
}

/**
 * A card's outfits as set headings: the row's own event's sets first, then the
 * rest of the day; each series keeps its editions together, new before rerun
 * (Monster Hunter/II sits on Monster Hunter), in the order they arrived.
 */
export function planGroups(skins: IPlanSkin[]): IPlanSkinGroup[] {
    const groups = new Map<string, IPlanSkinGroup & { brand: string; anchored: boolean; at: number }>();
    const brandAt = new Map<string, { at: number; anchored: boolean }>();
    skins.forEach((s, at) => {
        const key = `${s.groupName}|${s.rerun ? "r" : "n"}`;
        const g = groups.get(key) ?? { name: s.groupName, nameAuto: s.groupNameAuto, rerun: s.rerun, skins: [], brand: s.brand, anchored: false, at };
        g.skins.push(s);
        g.anchored ||= s.anchored;
        groups.set(key, g);
        const b = brandAt.get(s.brand) ?? { at, anchored: false };
        b.anchored ||= s.anchored;
        brandAt.set(s.brand, b);
    });
    const rank = (g: { brand: string }) => brandAt.get(g.brand) ?? { at: 0, anchored: false };
    return [...groups.values()].sort((a, b) => Number(rank(b).anchored) - Number(rank(a).anchored) || rank(a).at - rank(b).at || Number(a.rerun) - Number(b.rerun) || a.at - b.at).map(({ name, nameAuto, rerun, skins: list }) => ({ name, nameAuto, rerun, skins: list }));
}

/**
 * Folds each store sale into an event or review card that opens the same EN
 * day, so the day reads as one card: a sale lands on the first such event by
 * key, else the review. The sales left over are returned.
 */
export function mergeSameDay(sales: IPlanRow[], events: IPlanRow[], reviews: IPlanRow[]): IPlanRow[] {
    const byDay = new Map<string, IPlanRow>();
    for (const row of [...reviews, ...[...events].sort((a, b) => b.key.localeCompare(a.key))]) byDay.set(localDay(row.enStart), row);
    return sales.filter((sale) => {
        const host = byDay.get(localDay(sale.enStart));
        if (!host) return true;
        for (const s of sale.skins) if (!host.skins.some((h) => h.skinId === s.skinId)) host.skins.push({ ...s, anchored: false });
        return false;
    });
}

/** Only a known EN end closes an event; an estimate has none, and one dated before today is moved to today. */
export function enEnded(r: Resolution, nowSecs: number): boolean {
    const end = r.status === "confirmed" || r.status === "override" ? r.enEnd : null;
    return end !== null && end <= nowSecs;
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

/** The legacy pick key, by stage code; still read so picks saved under it survive. */
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
    if (row.ended) return false;
    return stagePick(state.stages[row.key], stage) ?? stageDefault(row, stage, clears);
}

export function rowDeviates(row: IPlanRow, state: IPlanState, clears: StageClears): boolean {
    const chosen = state.stages[row.key];
    if (!chosen || row.ended) return false;
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

export function shopBuyout({ shop, missionTokens }: { shop: EventShop | null; missionTokens: number }): IShopBuyout | null {
    if (!shop) return null;
    const remaining = Math.max(0, shop.maxPrice - missionTokens);
    return { total: shop.maxPrice, missions: missionTokens, remaining, sanity: remaining * SANITY_PER_TOKEN, limitedGoods: shop.goods.filter((g) => g.availCount > 0).length };
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

/** Limited goods grouped by kind in a fixed kind order, priciest first within each; unlimited goods apart, priciest first. */
export function shopGroups(shop: EventShop): { limited: IShopGroup[]; unlimited: ShopGood[] } {
    const limited = SHOP_KIND_ORDER.map((kind) => {
        const goods = shop.goods.filter((g) => g.kind === kind && g.availCount > 0).sort((a, b) => b.price - a.price || a.goodId.localeCompare(b.goodId));
        return { kind, goods, tokens: goods.reduce((sum, g) => sum + g.price * g.availCount, 0) };
    }).filter((g) => g.goods.length > 0);
    const unlimited = shop.goods.filter((g) => g.availCount < 0).sort((a, b) => b.price - a.price);
    return { limited, unlimited };
}
