import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseSkinsQueryOptions } from "#/lib/api/release";
import { type TypedRichT, useFormatters, useGamedataServer, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { AnniversaryStats } from "#/types/generated/AnniversaryStats";
import type { AutoName } from "#/types/generated/AutoName";
import type { BatchForecast } from "#/types/generated/BatchForecast";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { LagModel } from "#/types/generated/LagModel";
import type { NewSkin } from "#/types/generated/NewSkin";
import type { RerunBasis } from "#/types/generated/RerunBasis";
import type { RerunForecast } from "#/types/generated/RerunForecast";
import type { Resolution } from "#/types/generated/Resolution";
import type { ReviewOutfit } from "#/types/generated/ReviewOutfit";
import type { ReviewWindow } from "#/types/generated/ReviewWindow";
import type { SkinGroupArt } from "#/types/generated/SkinGroupArt";
import { daysFromToday, formatDate, formatDateRange, isPast, relativeDays, sortKey } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { REVIEW_NAME_CN, REVIEW_NAME_EN, reviewOutfits } from "../reviews";
import { ModelSummary } from "./ModelSummary";
import { ResolutionBadge } from "./ResolutionBadge";
import { AnchorCaption } from "./ScheduleShared";
import { SkinTileView } from "./SkinPopup";
import type { messages } from "./SkinsTab.messages";
import { buildOperatorLookup, CnName, ListRow, type OperatorLookup, ReleaseEmpty, ReleaseError, ReleaseLoading, SectionTitle, Tag, ToggleField, useArt } from "./shared";

/** This tab renders its own chrome plus the date wording `helpers.ts` derives. */
type SkinsT = TypedT<typeof messages & typeof helperMessages>;
type SkinsRichT = TypedRichT<typeof messages>;

type GroupArtMap = { [key in string]?: SkinGroupArt };

interface ISkinsTabProps {
    today: Date;
}

export interface INewSkinGroup {
    skinGroupId: string;
    skinGroupName: string;
    skinGroupNameAuto: AutoName | null;
    skins: NewSkin[];
    resolution: Resolution;
    cnGetTime: number;
    anchor: EventAnchor | null;
}

export function groupNewSkins(skins: NewSkin[], model: LagModel | null): INewSkinGroup[] {
    const groups = new Map<string, INewSkinGroup>();
    for (const s of skins) {
        const existing = groups.get(s.skinGroupId);
        if (existing) {
            existing.skins.push(s);
            if (sortKey(s.resolution, s.cnGetTime, model) < sortKey(existing.resolution, existing.cnGetTime, model)) {
                existing.resolution = s.resolution;
                existing.anchor = s.anchor;
            }
            if (!existing.skinGroupNameAuto && s.skinGroupNameAuto) existing.skinGroupNameAuto = s.skinGroupNameAuto;
            existing.cnGetTime = Math.min(existing.cnGetTime, s.cnGetTime);
        } else {
            groups.set(s.skinGroupId, { skinGroupId: s.skinGroupId, skinGroupName: s.skinGroupName, skinGroupNameAuto: s.skinGroupNameAuto, skins: [s], resolution: s.resolution, cnGetTime: s.cnGetTime, anchor: s.anchor });
        }
    }
    return [...groups.values()];
}

export function SkinsTab({ today }: ISkinsTabProps): React.ReactElement {
    const t: SkinsT = useT("tools");
    const locale = useLocale();
    const skins = useQuery(releaseSkinsQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const [showPast, setShowPast] = React.useState(false);
    const [showReviews, setShowReviews] = React.useState(false);

    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);
    const model = skins.data?.model ?? null;
    const newSkins: NewSkin[] = skins.data?.newSkins ?? [];
    const reruns: RerunForecast[] = skins.data?.rerunForecasts ?? [];
    const groupArt: GroupArtMap = skins.data?.groupArt ?? {};
    const reviews: ReviewWindow[] = skins.data?.reviews ?? [];
    const reviewPool: ReviewOutfit[] = skins.data?.reviewPool ?? [];

    const visibleNew = React.useMemo(() => newSkins.filter((s) => showPast || !isPast(sortKey(s.resolution, s.cnGetTime, model), today)), [newSkins, showPast, model, today]);
    const groups = React.useMemo(() => groupNewSkins(visibleNew, model), [visibleNew, model]);

    if (skins.isPending) return <ReleaseLoading />;
    if (skins.isError) return <ReleaseError error={skins.error} onRetry={() => skins.refetch()} />;

    return (
        <div className="flex flex-col gap-6">
            <SkinStats newSkins={newSkins} reruns={reruns} anniversaries={skins.data?.anniversaries ?? []} today={today} t={t} locale={locale} />

            <section className="flex flex-col gap-3">
                <ModelSummary model={model} yearly={skins.data?.yearly} />
                <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                    <ToggleField id="skins-show-past" label={t("release.skins.showPast")} checked={showPast} onChange={setShowPast} />
                    <ToggleField id="skins-show-reviews" label={t("release.skins.showReviews")} checked={showReviews} onChange={setShowReviews} />
                    <span className="font-medium font-mono text-[11px] text-muted-foreground">{t("release.skins.count", { shown: visibleNew.length, total: newSkins.length })}</span>
                </div>
                <div>
                    <SectionTitle count={groups.length}>{t("release.skins.newTitle")}</SectionTitle>
                    {groups.length === 0 ? (
                        <ReleaseEmpty title={t("release.skins.new.empty.title")} description={newSkins.length === 0 ? t("release.skins.new.empty.none") : t("release.skins.new.empty.filtered")} />
                    ) : (
                        <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
                            {groups.map((g) => (
                                <NewSkinGroupCard key={g.skinGroupId} group={g} art={groupArt[g.skinGroupId]} lookup={lookup} today={today} t={t} locale={locale} />
                            ))}
                        </div>
                    )}
                </div>
            </section>

            <section className="flex flex-col gap-3">
                <RerunSummary batches={skins.data?.batches ?? []} today={today} t={t} locale={locale} />
                <div>
                    <SectionTitle count={reruns.length}>{t("release.skins.rerunsTitle")}</SectionTitle>
                    {reruns.length === 0 ? (
                        <ReleaseEmpty title={t("release.skins.reruns.empty.title")} description={t("release.skins.reruns.empty.desc")} />
                    ) : (
                        <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
                            {reruns.map((r) => (
                                <RerunGroupCard key={rerunKey(r)} forecast={r} art={groupArt[r.skinGroupId]} lookup={lookup} today={today} t={t} locale={locale} />
                            ))}
                        </div>
                    )}
                </div>
            </section>

            {showReviews && <ReviewsSection reviews={reviews} pool={reviewPool} lookup={lookup} today={today} t={t} locale={locale} />}
        </div>
    );
}

function ReviewsSection({ reviews, pool, lookup, today, t, locale }: { reviews: ReviewWindow[]; pool: ReviewOutfit[]; lookup: OperatorLookup; today: Date; t: SkinsT; locale: string }): React.ReactElement {
    const visible = [...reviews].reverse();
    const last = reviews.at(-1);
    const gaps = reviews.slice(1).map((r, i) => (r.cnStart - reviews[i].cnStart) / 86_400);
    return (
        <section className="flex flex-col gap-3">
            <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">
                {t("release.skins.reviews.intro", {
                    cadence: gaps.length > 0 ? t("release.skins.reviews.intro.cadence", { min: Math.round(Math.min(...gaps)), max: Math.round(Math.max(...gaps)) }) : "",
                    latest: last ? t("release.skins.reviews.intro.latest", { date: formatDate(last.cnStart, locale) }) : "",
                })}
            </p>
            <div>
                <SectionTitle count={visible.length}>{t("release.skins.reviewsTitle")}</SectionTitle>
                {visible.length === 0 ? (
                    <ReleaseEmpty title={t("release.skins.reviews.empty.title")} description={t("release.skins.reviews.empty.desc")} />
                ) : (
                    <div className="rounded-xl border border-border bg-card px-3">
                        {visible.map((r) => (
                            <ReviewRow key={r.cnStart} review={r} outfits={reviewOutfits(r, pool)} lookup={lookup} today={today} t={t} locale={locale} />
                        ))}
                    </div>
                )}
            </div>
        </section>
    );
}

function ReviewRow({ review, outfits, lookup, today, t, locale }: { review: ReviewWindow; outfits: ReviewOutfit[]; lookup: OperatorLookup; today: Date; t: SkinsT; locale: string }): React.ReactElement {
    const [open, setOpen] = React.useState(false);
    return (
        <ListRow visual={null} badge={<ResolutionBadge resolution={review.resolution} today={today} />}>
            <CnName cn={REVIEW_NAME_CN} en={REVIEW_NAME_EN} auto={null} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                <Tag className="text-teal-400">{t("release.skins.review.tag")}</Tag>
            </CnName>
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                <span className="mr-1 uppercase tracking-[0.06em]">{t("release.skins.review.cn")}</span>
                {formatDateRange(review.cnStart, review.cnEnd, locale, t)}
                {t("release.skins.review.outfits", { count: outfits.length })}
                <button type="button" onClick={() => setOpen((v) => !v)} className="cursor-pointer text-primary hover:underline">
                    {open ? t("release.skins.review.hide") : t("release.skins.review.show")}
                </button>
            </span>
            {open && (
                <div className="flex flex-wrap gap-2 pt-1">
                    {[...outfits].reverse().map((o) => (
                        <SkinTileView key={o.skinId} skinId={o.skinId} charId={o.charId} charName={o.charName} skinName={o.skinName} skinNameEn={o.skinName} skinNameAuto={null} portraitPath={o.portraitPath} lookup={lookup} />
                    ))}
                </div>
            )}
        </ListRow>
    );
}

export function rerunKey(r: RerunForecast): string {
    if (r.basis.kind === "cn_listing") return `${r.skinGroupId}@${r.basis.cn_start}`;
    if (r.basis.kind === "cadence") return `${r.skinGroupId}@cadence${r.basis.en_last}`;
    return `${r.skinGroupId}@en${r.next.status === "confirmed" ? r.next.enStart : 0}`;
}

function RerunSummary({ batches, today, t, locale }: { batches: BatchForecast[]; today: Date; t: SkinsT; locale: string }): React.ReactElement | null {
    const rt: SkinsRichT = useRichT("tools");
    const next = batches[0];
    return (
        <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">
            {rt("release.skins.rerunSummary", {
                batch: next
                    ? rt("release.skins.rerunSummary.batch", {
                          name: <span className="font-medium text-foreground">{next.name}</span>,
                          resolution: <ResolutionInline resolution={next.resolution} today={today} t={t} locale={locale} />,
                      })
                    : t("release.skins.rerunSummary.noBatch"),
            })}
        </p>
    );
}

function ResolutionInline({ resolution, today, t, locale }: { resolution: RerunForecast["next"]; today: Date; t: SkinsT; locale: string }): React.ReactElement {
    if (resolution.status === "confirmed") return <>{t("release.skins.inline.confirmed", { date: formatDate(resolution.enStart, locale) })}</>;
    if (resolution.status === "estimated") return <>{t("release.skins.inline.estimated", { date: formatDate(resolution.enStart, locale), relative: relativeDays(daysFromToday(resolution.enStart, today), t) })}</>;
    return <>{t("release.skins.inline.none")}</>;
}

function basisLine(b: RerunBasis, t: SkinsT, locale: string): string | null {
    if (b.kind === "cn_listing") return t("release.skins.basis.cnListing", { dates: formatDateRange(b.cn_start, b.cn_end, locale, t) });
    if (b.kind === "cadence") return t("release.skins.basis.cadence", { date: formatDate(b.en_last, locale), median: Math.round(b.median_days), p25: Math.round(b.p25_days), p75: Math.round(b.p75_days), count: b.n });
    return null;
}

function StatBlock({ label, value, sub, format }: { label: string; value: string | number; sub: string; format: (n: number) => string }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1 rounded-xl border border-border bg-card px-3.5 py-3">
            <span className="truncate font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{label}</span>
            <span className="font-bold font-sans text-[24px] text-foreground tabular-nums leading-none tracking-tight sm:text-[28px]">{typeof value === "number" ? format(value) : value}</span>
            <span className="line-clamp-2 font-sans text-[11.5px] text-muted-foreground leading-snug">{sub}</span>
        </div>
    );
}

function SkinStats({ newSkins, reruns, anniversaries, today, t, locale }: { newSkins: NewSkin[]; reruns: RerunForecast[]; anniversaries: AnniversaryStats[]; today: Date; t: SkinsT; locale: string }): React.ReactElement {
    const f = useFormatters();
    const now = today.getTime() / 1000;
    const unconfirmed = newSkins.filter((s) => s.resolution.status !== "confirmed").length;
    const upcoming = newSkins.filter((s) => s.resolution.status === "confirmed" && s.resolution.enStart > now);
    const firstConfirmed = upcoming.reduce<number | null>((min, s) => (s.resolution.status === "confirmed" && (min === null || s.resolution.enStart < min) ? s.resolution.enStart : min), null);
    const confirmedReruns = reruns.filter((r) => r.next.status === "confirmed").length;
    const first = anniversaries.find((a) => a.year === 1);
    const second = anniversaries.find((a) => a.year === 2);
    const annValue = first && first.eligible > 0 ? `${Math.round((100 * first.observed) / first.eligible)}%` : "-";
    const annSub =
        first && first.eligible > 0
            ? `${t("release.skins.stat.anniversary.sub", { observed: first.observed, eligible: first.eligible })}${second && second.eligible > 0 ? t("release.skins.stat.anniversary.year2", { observed: second.observed, eligible: second.eligible }) : ""}`
            : t("release.skins.stat.anniversary.none");
    return (
        <div className="grid grid-cols-2 gap-2.5 md:grid-cols-4">
            <StatBlock label={t("release.skins.stat.new")} value={unconfirmed} sub={t("release.skins.stat.new.sub", { total: newSkins.length })} format={f.number} />
            <StatBlock label={t("release.skins.stat.next")} value={upcoming.length} sub={firstConfirmed === null ? t("release.skins.stat.next.none") : t("release.skins.stat.next.first", { date: formatDate(firstConfirmed, locale) })} format={f.number} />
            <StatBlock label={t("release.skins.stat.reruns")} value={reruns.length} sub={confirmedReruns > 0 ? t("release.skins.stat.reruns.some", { count: confirmedReruns }) : t("release.skins.stat.reruns.none")} format={f.number} />
            <StatBlock label={t("release.skins.stat.anniversary")} value={annValue} sub={annSub} format={f.number} />
        </div>
    );
}

function GroupCard({ art, header, badge, children }: { art: SkinGroupArt | undefined; header: React.ReactNode; badge: React.ReactNode; children: React.ReactNode }): React.ReactElement {
    const kv = useArt(art?.kvPath);
    const logo = useArt(art?.logoPath);
    return (
        <div className="flex min-w-0 flex-col overflow-hidden rounded-xl border border-border bg-card">
            <div className="relative border-border border-b">
                {kv.src && <img src={kv.src} alt="" aria-hidden loading="lazy" onError={kv.onError} className="pointer-events-none absolute inset-0 h-full w-full object-cover opacity-[0.12]" />}
                <div className="relative flex flex-col gap-1.5 px-3.5 py-2.5">
                    <div className="flex min-w-0 items-center gap-2.5">
                        {logo.src && <img src={logo.src} alt={art?.brandName ?? ""} loading="lazy" onError={logo.onError} className="h-6 w-auto max-w-18 shrink-0 object-contain" />}
                        {header}
                    </div>
                    {badge}
                </div>
            </div>
            <div className="flex gap-2.5 overflow-x-auto px-3.5 py-3">{children}</div>
        </div>
    );
}

function GroupCount({ n, extra, t }: { n: number; extra?: React.ReactNode; t: SkinsT }): React.ReactElement {
    return (
        <span className="ml-auto flex shrink-0 items-center gap-2 font-mono text-[11px] text-muted-foreground tabular-nums">
            {extra}
            <span>{t("release.skins.groupCount", { count: n })}</span>
        </span>
    );
}

function NewSkinGroupCard({ group, art, lookup, today, t, locale }: { group: INewSkinGroup; art: SkinGroupArt | undefined; lookup: OperatorLookup; today: Date; t: SkinsT; locale: string }): React.ReactElement {
    return (
        <GroupCard
            art={art}
            header={
                <>
                    <CnName cn={group.skinGroupName || group.skinGroupId} auto={group.skinGroupNameAuto} className="min-w-0 flex-1" primaryClassName="font-bold font-sans text-[14px] text-foreground" />
                    <GroupCount
                        n={group.skins.length}
                        t={t}
                        extra={
                            <span>
                                <span className="mr-1 uppercase tracking-[0.06em]">{t("release.skins.groupCn")}</span>
                                {formatDate(group.cnGetTime, locale)}
                            </span>
                        }
                    />
                </>
            }
            badge={<ResolutionBadge resolution={group.resolution} today={today} className="sm:items-start" caption={group.anchor ? <AnchorCaption anchor={group.anchor} /> : undefined} />}
        >
            {group.skins.map((s) => (
                <SkinTileView
                    key={s.skinId}
                    skinId={s.skinId}
                    charId={s.charId}
                    charName={s.charName}
                    skinName={s.skinName}
                    skinNameAuto={s.skinNameAuto}
                    portraitPath={s.portraitPath}
                    lookup={lookup}
                    title={t("release.skins.tileTitle", { date: formatDate(s.cnGetTime, locale), how: s.isBuySkin ? t("release.skins.tile.shop") : s.obtainApproach?.trim() || t("release.skins.tile.notForSale") })}
                />
            ))}
        </GroupCard>
    );
}

function RerunGroupCard({ forecast, art, lookup, today, t, locale }: { forecast: RerunForecast; art: SkinGroupArt | undefined; lookup: OperatorLookup; today: Date; t: SkinsT; locale: string }): React.ReactElement {
    const recent = forecast.windows.slice(-2);
    const reviews = forecast.windows.filter((w) => w.kind === "review").length;
    const estimated = forecast.next.status === "estimated" ? forecast.next : null;
    const overdue = estimated !== null && daysFromToday(estimated.enStart, today) < 0;
    const basis = basisLine(forecast.basis, t, locale);
    const anchor = forecast.basis.kind === "cn_listing" ? forecast.basis.anchor : null;
    const note = overdue && estimated ? t("release.skins.overdue") : null;
    const caption = (
        <>
            {basis}
            {anchor && (
                <>
                    {basis ? t("release.skins.basisAnchorJoin") : ""}
                    <AnchorCaption anchor={anchor} />
                </>
            )}
        </>
    );
    const tiles = forecast.skins.length > 0 ? forecast.skins : forecast.skinIds.map((skinId) => ({ skinId, charId: "", skinName: skinId, charName: null, portraitPath: null }));
    return (
        <GroupCard
            art={art}
            header={
                <>
                    <span className="flex min-w-0 flex-1 flex-col">
                        <span className="truncate font-bold font-sans text-[14px] text-foreground">{forecast.skinGroupName || forecast.skinGroupId}</span>
                        <span className="truncate font-mono text-[11px] text-muted-foreground tabular-nums">
                            {recent.length === 0 ? (
                                t("release.skins.noEnWindow")
                            ) : (
                                <>
                                    <span className="mr-1 uppercase tracking-[0.06em]">{t("release.skins.en")}</span>
                                    {recent.map((w) => (w.kind === "review" ? t("release.skins.window.review", { dates: formatDateRange(w.startTime, w.endTime, locale, t) }) : t("release.skins.window", { dates: formatDateRange(w.startTime, w.endTime, locale, t) }))).join(", ")}
                                    <span className="ml-1 opacity-70" title={t("release.skins.windows.title")}>
                                        {forecast.windows.length === 1 ? t("release.skins.windows.debutOnly") : t("release.skins.windows.rerunCount", { count: forecast.windows.length - 1 })}
                                        {reviews > 0 ? t("release.skins.windows.inReviews", { count: reviews }) : ""}
                                    </span>
                                </>
                            )}
                        </span>
                    </span>
                    <GroupCount n={tiles.length} t={t} />
                </>
            }
            badge={<ResolutionBadge resolution={forecast.next} today={today} className="sm:items-start" note={note} caption={caption} />}
        >
            {tiles.map((t) => (
                <SkinTileView key={t.skinId} skinId={t.skinId} charId={t.charId} charName={t.charName} skinName={t.skinName} skinNameEn={t.skinName} portraitPath={t.portraitPath} lookup={lookup} />
            ))}
        </GroupCard>
    );
}
