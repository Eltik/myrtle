import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseBannersQueryOptions, releaseEventsQueryOptions, releaseSkinsQueryOptions } from "#/lib/api/release";
import { DEFAULT_LOCALE, formatMessage, sourceMessage, useGamedataServer, useT } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import { parseOperatorName } from "#/lib/utils";
import type { AutoName } from "#/types/generated/AutoName";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { LagModel } from "#/types/generated/LagModel";
import type { ReleaseBanner } from "#/types/generated/ReleaseBanner";
import type { ReleaseEvent } from "#/types/generated/ReleaseEvent";
import type { RerunForecast } from "#/types/generated/RerunForecast";
import type { Resolution } from "#/types/generated/Resolution";
import type { ReviewOutfit } from "#/types/generated/ReviewOutfit";
import type { ReviewWindow } from "#/types/generated/ReviewWindow";
import { buildOperatorLookup, type OperatorLookup } from "./components/shared";
import { resolvedEnStart } from "./helpers";
import type { messages as helperMessages } from "./helpers.messages";
import { REVIEW_NAME_CN, REVIEW_NAME_EN, reviewOutfits } from "./reviews";
import type { messages as scheduleMessages } from "./schedule.messages";
import { groupNewSkins, type INewSkinGroup } from "./skins";

/** A key in `schedule.messages.ts`; resolved by whichever component renders it. */
export type ScheduleMessageKey = keyof typeof scheduleMessages & string;

/** The `t` the row builders need: their own detail keys plus the shared helper keys. */
export type ScheduleT = TypedT<typeof scheduleMessages & typeof helperMessages>;

export const DAY_SECS = 86_400;

export type ScheduleKind = "event" | "banner" | "skin" | "rerun" | "review";

export interface IScheduleItem {
    key: string;
    kind: ScheduleKind;
    nameCn: string;
    nameEn: string | null;
    nameAuto: AutoName | null;
    tag: string;
    detail: string;
    hasStage?: boolean;
    anchor?: EventAnchor | null;
    skins?: IScheduleSkin[];
    imagePath: string | null;
    start: number;
    end: number | null;
    cnStart: number;
    cnEnd: number | null;
    resolution: Resolution;
    charIds: string[];
    estimated: boolean;
}

export interface IScheduleSkin {
    skinId: string;
    charId: string;
    skinName: string;
    skinNameEn: string | null;
    skinNameAuto: AutoName | null;
    charName: AutoName | null;
    portraitPath: string | null;
}

export const KIND_LABEL_KEYS: Record<ScheduleKind, ScheduleMessageKey> = { event: "release.kind.event", banner: "release.kind.banner", skin: "release.kind.skin", rerun: "release.kind.rerun", review: "release.kind.review" };

/** Default `t` for a caller outside an `I18nProvider`; resolves against the bundled source catalog. */
const sourceT: ScheduleT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

function cnLength(start: number, end: number | null | undefined): number | null {
    return end && end > start ? end - start : null;
}

function window(r: Resolution, cnStart: number, cnEnd: number | null): { start: number; end: number | null } | null {
    const start = resolvedEnStart(r);
    if (start === null) return null;
    if (r.status === "confirmed" || (r.status === "override" && r.enEnd)) return { start, end: r.enEnd ?? null };
    const len = cnLength(cnStart, cnEnd);
    return { start, end: len === null ? null : start + len };
}

function isEstimated(r: Resolution): boolean {
    return r.status === "estimated";
}

export function eventItem(e: ReleaseEvent): IScheduleItem | null {
    const w = window(e.resolution, e.cnStart, e.cnEnd);
    if (!w) return null;
    return {
        key: `event:${e.cnId}`,
        kind: "event",
        nameCn: e.nameCn,
        nameEn: e.nameEn,
        nameAuto: e.nameEnAuto,
        tag: e.activityType,
        detail: "",
        hasStage: e.hasStage,
        imagePath: e.imagePath,
        start: w.start,
        end: w.end,
        cnStart: e.cnStart,
        cnEnd: e.cnEnd,
        resolution: e.resolution,
        charIds: [],
        estimated: isEstimated(e.resolution),
    };
}

export function cnDay(seconds: number): string {
    return new Date((seconds + 8 * 3600) * 1000).toISOString().slice(0, 10);
}

export interface IEventArt {
    byId: Map<string, string>;
    byDay: Map<string, string>;
}

export function eventArtIndex(events: ReleaseEvent[]): IEventArt {
    const byId = new Map<string, string>();
    const byDay = new Map<string, string>();
    for (const e of events) {
        if (!e.imagePath) continue;
        byId.set(e.cnId, e.imagePath);
        if (e.hasStage && !byDay.has(cnDay(e.cnStart))) byDay.set(cnDay(e.cnStart), e.imagePath);
    }
    return { byId, byDay };
}

export function bannerItem(b: ReleaseBanner, eventArt?: IEventArt, eventTied = true): IScheduleItem | null {
    if (b.standing) return null;
    const w = window(b.resolution, b.cnOpen, b.cnEnd);
    if (!w) return null;
    const imagePath = b.imagePath ?? (eventTied ? ((b.anchorActivity ? eventArt?.byId.get(b.anchorActivity) : undefined) ?? eventArt?.byDay.get(cnDay(b.cnOpen))) : undefined) ?? null;
    const chars = [...b.enFeatured6, ...b.featured6, ...b.overrideFeatured, ...b.debutChars].filter((id, i, all) => all.indexOf(id) === i);
    return {
        key: `banner:${b.cnPoolId}`,
        kind: "banner",
        nameCn: b.nameCn,
        nameEn: null,
        nameAuto: b.nameEnAuto,
        tag: b.ruleType,
        detail: "",
        imagePath,
        start: w.start,
        end: w.end,
        cnStart: b.cnOpen,
        cnEnd: b.cnEnd,
        resolution: b.resolution,
        charIds: chars,
        estimated: isEstimated(b.resolution),
    };
}

function operatorNames(skins: { charId: string; charName: AutoName | null }[], lookup: OperatorLookup): string {
    return skins
        .map((s) => {
            const entry = lookup.get(s.charId);
            return entry ? parseOperatorName(entry.name).displayName : s.charName?.text || s.charId;
        })
        .join(", ");
}

export function newSkinItem(g: INewSkinGroup, lookup: OperatorLookup): IScheduleItem | null {
    const start = resolvedEnStart(g.resolution);
    if (start === null) return null;
    const sighted = g.resolution.status === "confirmed" ? g.resolution.enEnd : g.resolution.status === "override" ? g.resolution.enEnd : null;
    const anchorLen = g.anchor ? cnLength(g.anchor.cnStart, g.anchor.cnEnd) : null;
    const end = sighted ?? (anchorLen === null ? null : start + anchorLen);
    const art = g.skins.find((s) => s.portraitPath)?.portraitPath ?? null;
    return {
        key: `skin:new:${g.skinGroupId}`,
        kind: "skin",
        nameCn: g.skinGroupName || g.skinGroupId,
        nameEn: null,
        nameAuto: g.skinGroupNameAuto,
        tag: "NEW_SKINS",
        detail: operatorNames(g.skins, lookup),
        anchor: g.anchor,
        skins: g.skins.map((s) => ({ skinId: s.skinId, charId: s.charId, skinName: s.skinName, skinNameEn: null, skinNameAuto: s.skinNameAuto, charName: s.charName, portraitPath: s.portraitPath })),
        imagePath: art,
        start,
        end,
        cnStart: g.cnGetTime,
        cnEnd: null,
        resolution: g.resolution,
        charIds: g.skins.map((s) => s.charId),
        estimated: isEstimated(g.resolution),
    };
}

export function rerunItem(r: RerunForecast, lookup: OperatorLookup, t: ScheduleT = sourceT): IScheduleItem | null {
    const start = resolvedEnStart(r.next);
    if (start === null) return null;
    const listing = r.basis.kind === "cn_listing" ? r.basis : null;
    const sighted = r.next.status === "confirmed" ? r.next.enEnd : r.next.status === "override" ? r.next.enEnd : null;
    const cnLen = listing ? cnLength(listing.cn_start, listing.cn_end) : null;
    const end = sighted ?? (cnLen === null ? null : start + cnLen);
    return {
        key: `skin:rerun:${r.skinGroupId}:${listing?.cn_start ?? start}`,
        kind: "rerun",
        nameCn: r.skinGroupName || r.skinGroupId,
        nameEn: r.skinGroupName || null,
        nameAuto: null,
        tag: "RERUN",
        detail: operatorNames(r.skins, lookup) || t("release.detail.skinCount", { count: r.skinIds.length }),
        skins: r.skins.map((s) => ({ skinId: s.skinId, charId: s.charId, skinName: s.skinName, skinNameEn: s.skinName, skinNameAuto: null, charName: s.charName, portraitPath: s.portraitPath })),
        imagePath: r.skins.find((s) => s.portraitPath)?.portraitPath ?? null,
        start,
        end,
        cnStart: listing?.cn_start ?? 0,
        cnEnd: listing?.cn_end ?? null,
        resolution: r.next,
        anchor: listing?.anchor ?? null,
        charIds: r.skins.map((s) => s.charId),
        estimated: isEstimated(r.next),
    };
}

export function reviewItem(r: ReviewWindow, pool: ReviewOutfit[], t: ScheduleT = sourceT): IScheduleItem | null {
    const w = window(r.resolution, r.cnStart, r.cnEnd);
    if (!w) return null;
    const outfits = reviewOutfits(r, pool);
    return {
        key: `skin:review:${r.cnStart}`,
        kind: "review",
        nameCn: REVIEW_NAME_CN,
        nameEn: REVIEW_NAME_EN,
        nameAuto: null,
        tag: "FASHION REVIEW",
        detail: t("release.detail.reviewOutfits", { count: outfits.length }),
        skins: outfits.map((o) => ({ skinId: o.skinId, charId: o.charId, skinName: o.skinName, skinNameEn: o.skinName, skinNameAuto: null, charName: o.charName, portraitPath: o.portraitPath })).reverse(),
        imagePath: null,
        start: w.start,
        end: w.end,
        cnStart: r.cnStart,
        cnEnd: r.cnEnd,
        resolution: r.resolution,
        charIds: [],
        estimated: isEstimated(r.resolution),
    };
}

export interface ISchedule {
    items: IScheduleItem[];
    model: LagModel | null;
    lookup: OperatorLookup;
    isPending: boolean;
    error: Error | null;
    refetch: () => void;
}

export function useSchedule(): ISchedule {
    const t: ScheduleT = useT("tools");
    const events = useQuery(releaseEventsQueryOptions());
    const banners = useQuery(releaseBannersQueryOptions());
    const skins = useQuery(releaseSkinsQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);
    const model = events.data?.model ?? banners.data?.model ?? skins.data?.model ?? null;

    const items = React.useMemo(() => {
        const out: IScheduleItem[] = [];
        for (const e of events.data?.events ?? []) {
            const it = eventItem(e);
            if (it) out.push(it);
        }
        const eventArt = eventArtIndex(events.data?.events ?? []);
        const independent = new Set((banners.data?.ruleIndependence ?? []).filter((r) => r.independent).map((r) => r.ruleType));
        for (const b of banners.data?.banners ?? []) {
            const it = bannerItem(b, eventArt, !independent.has(b.ruleType));
            if (it) out.push(it);
        }
        for (const g of groupNewSkins(skins.data?.newSkins ?? [], model)) {
            const it = newSkinItem(g, lookup);
            if (it) out.push(it);
        }
        for (const r of skins.data?.rerunForecasts ?? []) {
            const it = rerunItem(r, lookup, t);
            if (it) out.push(it);
        }
        for (const r of skins.data?.reviews ?? []) {
            const it = reviewItem(r, skins.data?.reviewPool ?? [], t);
            if (it) out.push(it);
        }
        out.sort((a, b) => a.start - b.start || a.kind.localeCompare(b.kind));
        return out;
    }, [events.data, banners.data, skins.data, model, lookup, t]);

    const refetch = React.useCallback(() => {
        void events.refetch();
        void banners.refetch();
        void skins.refetch();
    }, [events, banners, skins]);

    return {
        items,
        model,
        lookup,
        isPending: events.isPending || banners.isPending || skins.isPending,
        error: events.error ?? banners.error ?? skins.error ?? null,
        refetch,
    };
}

export function overlaps(item: IScheduleItem, from: number, to: number): boolean {
    const end = item.end ?? item.start;
    return item.start <= to && end >= from;
}

export function dayStart(d: Date): number {
    return Math.floor(new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime() / 1000);
}

export function dayOf(seconds: number): number {
    return dayStart(new Date(seconds * 1000));
}

export function packLanes(items: IScheduleItem[], gapSecs = 0): Map<string, number> {
    const lanes: number[] = [];
    const out = new Map<string, number>();
    for (const it of [...items].sort((a, b) => a.start - b.start)) {
        const end = (it.end ?? it.start) + gapSecs;
        let lane = lanes.findIndex((laneEnd) => laneEnd < it.start);
        if (lane === -1) {
            lane = lanes.length;
            lanes.push(end);
        } else {
            lanes[lane] = end;
        }
        out.set(it.key, lane);
    }
    return out;
}
