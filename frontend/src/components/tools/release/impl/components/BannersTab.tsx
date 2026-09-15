import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { Card } from "#/components/ui/card";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseBannersQueryOptions, releaseEventsQueryOptions } from "#/lib/api/release";
import { type TypedRichT, useGamedataServer, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById } from "#/lib/utils";
import type { AlignMethod } from "#/types/generated/AlignMethod";
import type { AutoName } from "#/types/generated/AutoName";
import type { ReleaseBanner } from "#/types/generated/ReleaseBanner";
import type { RuleIndependence } from "#/types/generated/RuleIndependence";
import { useAutoTranslate } from "../autoTranslate";
import { formatDate, humanizeTag, isPast, sortKey } from "../helpers";
import { cnDay } from "../schedule";
import type { messages } from "./BannersTab.messages";
import { ModelSummary } from "./ModelSummary";
import { ResolutionBadge } from "./ResolutionBadge";
import { buildOperatorLookup, CnName, ListRow, type OperatorLookup, OpRefList, ReleaseEmpty, ReleaseError, ReleaseLoading, RowImage, resolveName, Tag, ToggleField, useArt } from "./shared";

type BannersT = TypedT<typeof messages>;
type BannersRichT = TypedRichT<typeof messages>;

type CharNames = { [key in string]?: AutoName };
type EventNames = Map<string, { cn: string; en: string | null; auto: AutoName | null; imagePath: string | null }>;

function IndependenceNote({ rules }: { rules: RuleIndependence[] }): React.ReactElement | null {
    const t: BannersT = useT("tools");
    const independent = rules.filter((r) => r.independent);
    if (independent.length === 0) return null;
    const list = independent.map((r) => t("release.banners.independence.item", { type: r.ruleType.toLowerCase().replaceAll("_", " "), rate: Math.round(r.mismatchRate * 100), count: r.events })).join(", ");
    return (
        <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal" title={t("release.banners.independence.title")}>
            {t("release.banners.independence", { list })}
        </p>
    );
}

const ALIGN_LABEL_KEYS: Record<AlignMethod, (keyof typeof messages & string) | null> = {
    content: "release.banners.align.content",
    anchor: "release.banners.align.anchor",
    none: null,
};

interface IBannersTabProps {
    today: Date;
}

export function BannersTab({ today }: IBannersTabProps): React.ReactElement {
    const t: BannersT = useT("tools");
    const banners = useQuery(releaseBannersQueryOptions());
    const events = useQuery(releaseEventsQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const [showPast, setShowPast] = React.useState(false);

    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);
    const eventNames = React.useMemo(() => {
        const map: EventNames = new Map();
        for (const e of events.data?.events ?? []) map.set(e.cnId, { cn: e.nameCn, en: e.nameEn, auto: e.nameEnAuto, imagePath: e.imagePath });
        return map;
    }, [events.data]);
    const eventsByDay = React.useMemo(() => {
        const map = new Map<string, string>();
        for (const e of events.data?.events ?? []) {
            if (e.hasStage && e.imagePath && !map.has(cnDay(e.cnStart))) map.set(cnDay(e.cnStart), e.cnId);
        }
        return map;
    }, [events.data]);
    const model = banners.data?.model ?? null;
    const all: ReleaseBanner[] = banners.data?.banners ?? [];
    const charNames: CharNames = banners.data?.charNames ?? {};
    const independent = React.useMemo(() => new Set((banners.data?.ruleIndependence ?? []).filter((r) => r.independent).map((r) => r.ruleType)), [banners.data]);
    const rows = React.useMemo(() => all.filter((b) => showPast || !isPast(sortKey(b.resolution, b.cnOpen, model), today)), [all, showPast, model, today]);

    if (banners.isPending) return <ReleaseLoading />;
    if (banners.isError) return <ReleaseError error={banners.error} onRetry={() => banners.refetch()} />;

    return (
        <div className="flex flex-col gap-3">
            <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">{t("release.banners.rosterNote")}</p>
            <IndependenceNote rules={banners.data?.ruleIndependence ?? []} />
            <ModelSummary model={model} yearly={banners.data?.yearly} />
            <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                <ToggleField id="banners-show-past" label={t("release.banners.showPast")} checked={showPast} onChange={setShowPast} />
                <span className="font-medium font-mono text-[11px] text-muted-foreground">{t("release.banners.count", { shown: rows.length, total: all.length })}</span>
            </div>
            {rows.length === 0 ? (
                <ReleaseEmpty title={t("release.banners.empty.title")} description={all.length === 0 ? t("release.banners.empty.none") : t("release.banners.empty.filtered")} />
            ) : (
                <Card className="px-4 sm:px-5">
                    {rows.map((b) => (
                        <BannerRow key={b.cnPoolId} banner={b} lookup={lookup} charNames={charNames} eventNames={eventNames} eventsByDay={eventsByDay} eventTied={!independent.has(b.ruleType)} today={today} />
                    ))}
                </Card>
            )}
        </div>
    );
}

function FaceStrip({ ids, lookup, alt, t }: { ids: string[]; lookup: OperatorLookup; alt: string; t: BannersT }): React.ReactElement {
    const tight = ids.length > 2;
    return (
        <div className="flex aspect-[5/2] w-full items-center justify-center gap-1 overflow-hidden rounded-md bg-linear-to-br from-zinc-800 to-zinc-950 p-1 sm:w-60 sm:p-1.5" title={t("release.banners.faceStrip.title", { banner: alt })}>
            {ids.map((id) => (
                <FaceAvatar key={id} id={id} onEn={lookup.has(id)} tight={tight} />
            ))}
        </div>
    );
}

function FaceAvatar({ id, onEn, tight }: { id: string; onEn: boolean; tight: boolean }): React.ReactElement | null {
    const [failed, setFailed] = React.useState(false);
    if (failed) return null;
    return <img src={getAvatarById(id, onEn ? undefined : "cn")} alt="" loading="lazy" onError={() => setFailed(true)} className={cn("h-full rounded-sm object-cover", tight ? "min-w-0 flex-1" : "aspect-square flex-none")} />;
}

function BannerRow({ banner, lookup, charNames, eventNames, eventsByDay, eventTied, today }: { banner: ReleaseBanner; lookup: OperatorLookup; charNames: CharNames; eventNames: EventNames; eventsByDay: Map<string, string>; eventTied: boolean; today: Date }): React.ReactElement {
    const t: BannersT = useT("tools");
    const rt: BannersRichT = useRichT("tools");
    const locale = useLocale();
    const autoOn = useAutoTranslate();
    const anchor = banner.anchorActivity ? eventNames.get(banner.anchorActivity) : undefined;
    const anchorName = anchor ? resolveName(anchor.cn, anchor.en, anchor.auto, autoOn).text : banner.anchorActivity;
    const sameDay = eventTied && !banner.imagePath && !anchor?.imagePath ? eventNames.get(eventsByDay.get(cnDay(banner.cnOpen)) ?? "") : undefined;
    const eventArt = anchor?.imagePath ? anchor : sameDay?.imagePath ? sameDay : undefined;
    const art = useArt(banner.imagePath ?? eventArt?.imagePath ?? null);
    const eventArtName = eventArt ? resolveName(eventArt.cn, eventArt.en, eventArt.auto, autoOn).text : null;
    const artIsEvent = !banner.imagePath && !!eventArt;
    const alt = resolveName(banner.nameCn, null, banner.nameEnAuto, autoOn).text;
    const alignKey = ALIGN_LABEL_KEYS[banner.alignment.method];
    const align = alignKey ? t(alignKey) : null;
    const entered = banner.overrideFeatured.filter((id) => !banner.featured6.includes(id) && !banner.enFeatured6.includes(id));
    const rosters: [string, string[], string | undefined][] = (
        [
            [t("release.banners.roster.rateUps"), banner.featured6, undefined],
            [t("release.banners.roster.enRateUps"), banner.enFeatured6, undefined],
            [t("release.banners.roster.entered"), entered, t("release.banners.roster.entered.title")],
            [t("release.banners.roster.debut"), banner.debutChars, undefined],
        ] as [string, string[], string | undefined][]
    ).filter((r) => r[1].length > 0);
    const faces = art.src ? [] : [...banner.enFeatured6, ...banner.featured6, ...banner.overrideFeatured, ...banner.debutChars].filter((id, i, all) => all.indexOf(id) === i).slice(0, 3);
    const visual = art.src ? <RowImage src={art.src} alt={alt} title={artIsEvent && eventArtName ? t("release.banners.eventArt", { event: eventArtName }) : undefined} onError={art.onError} wide /> : faces.length > 0 ? <FaceStrip ids={faces} lookup={lookup} alt={alt} t={t} /> : null;
    return (
        <ListRow visual={visual} wide badge={<ResolutionBadge resolution={banner.resolution} today={today} standing={banner.standing} />}>
            <CnName cn={banner.nameCn} auto={banner.nameEnAuto} primaryClassName="font-sans font-semibold text-[13.5px] text-foreground">
                <Tag>{humanizeTag(banner.ruleType)}</Tag>
                <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                    <span className="mr-1 uppercase tracking-[0.06em]">{t("release.banners.cn")}</span>
                    {banner.standing ? t("release.banners.since", { date: formatDate(banner.cnOpen, locale) }) : formatDate(banner.cnOpen, locale)}
                </span>
            </CnName>
            {rosters.length > 0 && (
                <div className="flex flex-col gap-1">
                    {rosters.map(([label, ids, title]) => (
                        <div key={label} className="flex flex-wrap items-center gap-x-2 gap-y-1">
                            <span className="font-sans text-[11px] text-muted-foreground" title={title}>
                                {label}
                            </span>
                            <OpRefList ids={ids} lookup={lookup} names={charNames} />
                        </div>
                    ))}
                </div>
            )}
            {(align || banner.anchorActivity) && (
                <div className="flex flex-wrap items-center gap-x-2 font-sans text-[11px] text-muted-foreground">
                    {align && <span>{align}</span>}
                    {banner.anchorActivity && <span title={banner.anchorActivity}>{rt("release.banners.with", { event: <span className={anchor ? "text-foreground/80" : "font-mono"}>{anchorName}</span> })}</span>}
                </div>
            )}
        </ListRow>
    );
}
