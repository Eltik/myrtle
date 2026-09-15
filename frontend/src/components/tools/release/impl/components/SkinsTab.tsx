import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseSkinsQueryOptions } from "#/lib/api/release";
import type { AnniversaryStats } from "#/types/generated/AnniversaryStats";
import type { AutoName } from "#/types/generated/AutoName";
import type { BatchForecast } from "#/types/generated/BatchForecast";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { LagModel } from "#/types/generated/LagModel";
import type { NewSkin } from "#/types/generated/NewSkin";
import type { RerunBasis } from "#/types/generated/RerunBasis";
import type { RerunForecast } from "#/types/generated/RerunForecast";
import type { Resolution } from "#/types/generated/Resolution";
import type { SkinGroupArt } from "#/types/generated/SkinGroupArt";
import { daysFromToday, formatDate, formatDateRange, isPast, relativeDays, sortKey } from "../helpers";
import { ModelSummary } from "./ModelSummary";
import { ResolutionBadge } from "./ResolutionBadge";
import { AnchorCaption } from "./ScheduleShared";
import { SkinTileView } from "./SkinPopup";
import { buildOperatorLookup, CnName, type OperatorLookup, ReleaseEmpty, ReleaseError, ReleaseLoading, SectionTitle, ToggleField, useArt } from "./shared";

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
    const skins = useQuery(releaseSkinsQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions());
    const [showPast, setShowPast] = React.useState(false);

    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);
    const model = skins.data?.model ?? null;
    const newSkins: NewSkin[] = skins.data?.newSkins ?? [];
    const reruns: RerunForecast[] = skins.data?.rerunForecasts ?? [];
    const groupArt: GroupArtMap = skins.data?.groupArt ?? {};

    const visibleNew = React.useMemo(() => newSkins.filter((s) => showPast || !isPast(sortKey(s.resolution, s.cnGetTime, model), today)), [newSkins, showPast, model, today]);
    const groups = React.useMemo(() => groupNewSkins(visibleNew, model), [visibleNew, model]);

    if (skins.isPending) return <ReleaseLoading />;
    if (skins.isError) return <ReleaseError error={skins.error} onRetry={() => skins.refetch()} />;

    return (
        <div className="flex flex-col gap-6">
            <SkinStats newSkins={newSkins} reruns={reruns} anniversaries={skins.data?.anniversaries ?? []} today={today} />

            <section className="flex flex-col gap-3">
                <ModelSummary model={model} yearly={skins.data?.yearly} />
                <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                    <ToggleField id="skins-show-past" label="Show past" checked={showPast} onChange={setShowPast} />
                    <span className="font-medium font-mono text-[11px] text-muted-foreground">
                        {visibleNew.length} of {newSkins.length}
                    </span>
                </div>
                <div>
                    <SectionTitle count={groups.length}>New skins</SectionTitle>
                    {groups.length === 0 ? (
                        <ReleaseEmpty title="No new skins" description={newSkins.length === 0 ? "Every CN skin with a release time is already on EN." : "Every new skin is filtered out. Turn on Show past."} />
                    ) : (
                        <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
                            {groups.map((g) => (
                                <NewSkinGroupCard key={g.skinGroupId} group={g} art={groupArt[g.skinGroupId]} lookup={lookup} today={today} />
                            ))}
                        </div>
                    )}
                </div>
            </section>

            <section className="flex flex-col gap-3">
                <RerunSummary batches={skins.data?.batches ?? []} today={today} />
                <div>
                    <SectionTitle count={reruns.length}>Reruns</SectionTitle>
                    {reruns.length === 0 ? (
                        <ReleaseEmpty title="No reruns pending" description="CN has re-listed no skin group in the lookback that EN has not shown." />
                    ) : (
                        <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
                            {reruns.map((r) => (
                                <RerunGroupCard key={rerunKey(r)} forecast={r} art={groupArt[r.skinGroupId]} lookup={lookup} today={today} />
                            ))}
                        </div>
                    )}
                </div>
            </section>
        </div>
    );
}

export function rerunKey(r: RerunForecast): string {
    if (r.basis.kind === "cn_listing") return `${r.skinGroupId}@${r.basis.cn_start}`;
    if (r.basis.kind === "cadence") return `${r.skinGroupId}@cadence${r.basis.en_last}`;
    return `${r.skinGroupId}@en${r.next.status === "confirmed" ? r.next.enStart : 0}`;
}

function RerunSummary({ batches, today }: { batches: BatchForecast[]; today: Date }): React.ReactElement | null {
    const next = batches[0];
    return (
        <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">
            Reruns follow CN's re-listings: EN repeats them under the same event (129 of 139 within 7 d of its EN start) or at the lag (48 of 48 within 31 d), confirmed once EN game data lists the group. One EN has already shown estimates its next by EN's own re-listing cadence
            {next ? (
                <>
                    . The next rotation batch ("Multi-theme Outfit", groups not in game data) is <span className="font-medium text-foreground">{next.name}</span> <ResolutionInline resolution={next.resolution} today={today} />
                </>
            ) : (
                ". No rotation batch is pending"
            )}
            . Fashion Reviews ended on EN in Jan 2026 and are recorded, not extrapolated.
        </p>
    );
}

function ResolutionInline({ resolution, today }: { resolution: RerunForecast["next"]; today: Date }): React.ReactElement {
    if (resolution.status === "confirmed") return <>(confirmed, {formatDate(resolution.enStart)})</>;
    if (resolution.status === "estimated")
        return (
            <>
                (estimated {formatDate(resolution.enStart)}, {relativeDays(daysFromToday(resolution.enStart, today))})
            </>
        );
    return <>(no date)</>;
}

function basisLine(b: RerunBasis): string | null {
    if (b.kind === "cn_listing") return `CN re-listed ${formatDateRange(b.cn_start, b.cn_end)}`;
    if (b.kind === "cadence") return `next re-listing: last on EN ${formatDate(b.en_last)}; groups re-list every ${Math.round(b.median_days)} d (p25 ${Math.round(b.p25_days)}, p75 ${Math.round(b.p75_days)}, n=${b.n})`;
    return null;
}

function StatBlock({ label, value, sub }: { label: string; value: string | number; sub: string }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1 rounded-xl border border-border bg-card px-3.5 py-3">
            <span className="truncate font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{label}</span>
            <span className="font-bold font-sans text-[24px] text-foreground tabular-nums leading-none tracking-tight sm:text-[28px]">{typeof value === "number" ? value.toLocaleString() : value}</span>
            <span className="line-clamp-2 font-sans text-[11.5px] text-muted-foreground leading-snug">{sub}</span>
        </div>
    );
}

function SkinStats({ newSkins, reruns, anniversaries, today }: { newSkins: NewSkin[]; reruns: RerunForecast[]; anniversaries: AnniversaryStats[]; today: Date }): React.ReactElement {
    const now = today.getTime() / 1000;
    const unconfirmed = newSkins.filter((s) => s.resolution.status !== "confirmed").length;
    const upcoming = newSkins.filter((s) => s.resolution.status === "confirmed" && s.resolution.enStart > now);
    const firstConfirmed = upcoming.reduce<number | null>((min, s) => (s.resolution.status === "confirmed" && (min === null || s.resolution.enStart < min) ? s.resolution.enStart : min), null);
    const confirmedReruns = reruns.filter((r) => r.next.status === "confirmed").length;
    const first = anniversaries.find((a) => a.year === 1);
    const second = anniversaries.find((a) => a.year === 2);
    const annValue = first && first.eligible > 0 ? `${Math.round((100 * first.observed) / first.eligible)}%` : "-";
    const annSub = first && first.eligible > 0 ? `re-listed by name at year 1 (${first.observed} of ${first.eligible} EN groups)${second && second.eligible > 0 ? `, year 2 ${second.observed} of ${second.eligible}` : ""}` : "no EN group has reached its first anniversary";
    return (
        <div className="grid grid-cols-2 gap-2.5 md:grid-cols-4">
            <StatBlock label="New skins" value={unconfirmed} sub={`of ${newSkins.length} CN skins not yet confirmed on EN`} />
            <StatBlock label="Next confirmed" value={upcoming.length} sub={firstConfirmed === null ? "none scheduled in EN gamedata" : `first on ${formatDate(firstConfirmed)}`} />
            <StatBlock label="Reruns" value={reruns.length} sub={confirmedReruns > 0 ? `CN re-listings in the lookback, ${confirmedReruns} already in EN gamedata` : "CN re-listings in the lookback, none in EN gamedata yet"} />
            <StatBlock label="Anniversary reruns" value={annValue} sub={annSub} />
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

function GroupCount({ n, extra }: { n: number; extra?: React.ReactNode }): React.ReactElement {
    return (
        <span className="ml-auto flex shrink-0 items-center gap-2 font-mono text-[11px] text-muted-foreground tabular-nums">
            {extra}
            <span>
                {n} {n === 1 ? "skin" : "skins"}
            </span>
        </span>
    );
}

function NewSkinGroupCard({ group, art, lookup, today }: { group: INewSkinGroup; art: SkinGroupArt | undefined; lookup: OperatorLookup; today: Date }): React.ReactElement {
    return (
        <GroupCard
            art={art}
            header={
                <>
                    <CnName cn={group.skinGroupName || group.skinGroupId} auto={group.skinGroupNameAuto} className="min-w-0 flex-1" primaryClassName="font-bold font-sans text-[14px] text-foreground" />
                    <GroupCount
                        n={group.skins.length}
                        extra={
                            <span>
                                <span className="mr-1 uppercase tracking-[0.06em]">CN</span>
                                {formatDate(group.cnGetTime)}
                            </span>
                        }
                    />
                </>
            }
            badge={<ResolutionBadge resolution={group.resolution} today={today} className="sm:items-start" caption={group.anchor ? <AnchorCaption anchor={group.anchor} /> : undefined} />}
        >
            {group.skins.map((s) => (
                <SkinTileView key={s.skinId} skinId={s.skinId} charId={s.charId} charName={s.charName} skinName={s.skinName} skinNameAuto={s.skinNameAuto} portraitPath={s.portraitPath} lookup={lookup} title={`CN ${formatDate(s.cnGetTime)}, ${s.isBuySkin ? "shop" : s.obtainApproach?.trim() || "not for sale"}`} />
            ))}
        </GroupCard>
    );
}

function RerunGroupCard({ forecast, art, lookup, today }: { forecast: RerunForecast; art: SkinGroupArt | undefined; lookup: OperatorLookup; today: Date }): React.ReactElement {
    const recent = forecast.windows.slice(-2);
    const reviews = forecast.windows.filter((w) => w.kind === "review").length;
    const estimated = forecast.next.status === "estimated" ? forecast.next : null;
    const overdue = estimated !== null && daysFromToday(estimated.enStart, today) < 0;
    const basis = basisLine(forecast.basis);
    const anchor = forecast.basis.kind === "cn_listing" ? forecast.basis.anchor : null;
    const note = overdue && estimated ? "expected date passed, not yet listed on EN" : null;
    const caption = (
        <>
            {basis}
            {anchor && (
                <>
                    {basis ? ", " : ""}
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
                                "No EN window yet"
                            ) : (
                                <>
                                    <span className="mr-1 uppercase tracking-[0.06em]">EN</span>
                                    {recent.map((w) => `${formatDateRange(w.startTime, w.endTime)}${w.kind === "review" ? " (review)" : ""}`).join(", ")}
                                    <span className="ml-1 opacity-70" title="Every time this group was on sale on EN: its own listings (debut and reruns) and the Fashion Reviews in which every past outfit was sold">
                                        {forecast.windows.length === 1 ? "debut only" : `${forecast.windows.length - 1} rerun${forecast.windows.length === 2 ? "" : "s"}`}
                                        {reviews > 0 ? ` (${reviews} in reviews)` : ""}
                                    </span>
                                </>
                            )}
                        </span>
                    </span>
                    <GroupCount n={tiles.length} />
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
