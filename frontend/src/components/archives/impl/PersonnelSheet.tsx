import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { ExternalLinkIcon, NetworkIcon } from "lucide-react";
import type * as React from "react";
import { useMemo, useState } from "react";
import { PolarAngleAxis, PolarGrid, PolarRadiusAxis, Radar, RadarChart } from "recharts";
import { Button } from "#/components/ui/button";
import { Sheet, SheetDescription, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { archiveOperatorQueryOptions } from "#/lib/api/archives";
import { useFormatters, useGamedataServer } from "#/lib/i18n";
import { themeStore } from "#/lib/theme/store";
import { formatProfession } from "#/lib/utils";
import type { ArchiveOperator } from "#/types/generated/ArchiveOperator";
import { type ArchivesT, AXIS_KEY, FactionLogo, INFECTION_KEY, PersonChip, peopleById, useArchivesT } from "./shared";

/** Stories listed before "Show all". */
const STORIES_SHOWN = 14;

interface IPersonnelSheetProps {
    id: string | undefined;
    onClose: () => void;
    onOpenFile: (id: string) => void;
    onOpenChart: (groupId: string) => void;
    /** Opened from the Dossiers table: the chart buttons read "Show on the network". */
    fromDossiers: boolean;
}

export function PersonnelSheet({ id, onClose, onOpenFile, onOpenChart, fromDossiers }: IPersonnelSheetProps): React.ReactElement {
    const t = useArchivesT();
    const server = useGamedataServer();
    const { data, isLoading, isError } = useQuery({ ...archiveOperatorQueryOptions(id ?? "", server), enabled: Boolean(id) });

    return (
        <Sheet open={Boolean(id)} onOpenChange={(open) => !open && onClose()}>
            <SheetPopup side="right" className="w-[min(460px,100vw)] sm:max-w-115">
                {isLoading ? <TitleOnly>{t("page.loading")}</TitleOnly> : isError ? <TitleOnly>{t("page.error")}</TitleOnly> : data ? <FileBody op={data} t={t} server={server} onOpenFile={onOpenFile} onOpenChart={onOpenChart} fromDossiers={fromDossiers} /> : <TitleOnly>{t("file.missing")}</TitleOnly>}
            </SheetPopup>
        </Sheet>
    );
}

/** The sheet with no file to show: a title saying why. */
function TitleOnly({ children }: { children: React.ReactNode }): React.ReactElement {
    return (
        <SheetHeader>
            <SheetTitle>{children}</SheetTitle>
        </SheetHeader>
    );
}

function Fact({ label, value, children }: { label: string; value?: React.ReactNode; children?: React.ReactNode }): React.ReactElement | null {
    const shown = value ?? children;
    if (shown === undefined || shown === null || shown === "") return null;
    return (
        <div className="flex items-baseline gap-2 py-1">
            <dt className="shrink-0 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{label}</dt>
            <span aria-hidden className="min-w-4 flex-1 translate-y-[-3px] border-border border-b border-dotted" />
            <dd className="m-0 text-right text-[13px] text-foreground">{shown}</dd>
        </div>
    );
}

function SectionTitle({ children }: { children: React.ReactNode }): React.ReactElement {
    return <h3 className="m-0 mt-6 mb-2 font-bold font-sans text-[0.69rem] text-primary uppercase tracking-[0.22em]">{children}</h3>;
}

function FileBody({ op, t, server, onOpenFile, onOpenChart, fromDossiers }: { op: ArchiveOperator; t: ArchivesT; server: string; onOpenFile: (id: string) => void; onOpenChart: (groupId: string) => void; fromDossiers: boolean }): React.ReactElement {
    const f = useFormatters();
    const spoilers = useSelector(themeStore, (s) => s.spoilers);
    const [allStories, setAllStories] = useState(false);
    const people = useMemo(() => peopleById(op.people), [op.people]);
    const p = op.profile;
    const home = op.factions[op.factions.length - 1];
    const homeChart = op.charts.find((c) => c.home) ?? op.charts[0];
    const otherCharts = op.charts.filter((c) => c !== homeChart);

    const radar = useMemo(
        () =>
            (Object.keys(AXIS_KEY) as (keyof typeof AXIS_KEY)[]).map((axis) => {
                const rating = p?.exam.find((r) => r.axis === axis);
                return { axis: t(AXIS_KEY[axis]), value: rating?.step != null ? rating.step + 1 : 0 };
            }),
        [p, t],
    );
    const missingRatings = radar.filter((r) => r.value === 0).length;
    const stories = allStories ? op.appearances : op.appearances.slice(0, STORIES_SHOWN);

    return (
        <>
            <SheetHeader className="relative overflow-hidden">
                {home ? <FactionLogo id={home.id} className="pointer-events-none absolute -top-2 right-10 size-28 text-foreground opacity-[0.08]" /> : null}
                <p className="m-0 font-bold font-mono text-[10.5px] text-primary uppercase tracking-[0.2em]">
                    {op.number ? `${op.number.display} · ` : ""}
                    {t("file.kicker")}
                </p>
                <SheetTitle className="font-bold text-[26px] uppercase leading-tight tracking-tight">{op.name}</SheetTitle>
                <SheetDescription>{t("file.subtitle", { profession: formatProfession(op.profession), branch: op.subProfessionName ?? op.subProfessionId })}</SheetDescription>
            </SheetHeader>
            <SheetPanel className="pb-8">
                <dl className="m-0">
                    <Fact label={t("file.codeName")} value={p?.codeName} />
                    <Fact label={t("file.gender")} value={p?.gender} />
                    <Fact label={t("file.race")} value={p?.race} />
                    <Fact label={t("file.birthplace")} value={p?.birthplace} />
                    <Fact label={t("file.birthday")} value={p?.birthday} />
                    <Fact label={t("file.height")} value={p?.height} />
                    <Fact label={t("file.combat")} value={p?.combatExperience} />
                    <Fact label={t("file.blood")} value={p?.bloodDensity} />
                    <Fact label={t("file.assimilation")} value={p?.assimilation} />
                    <Fact label={t("file.manufacturer")} value={p?.manufacturer} />
                    <Fact label={t("file.faction")} value={op.factions.map((x) => x.name).join(" · ")} />
                    {spoilers ? op.hiddenFactions.map((h) => <Fact key={h.id} label={t("file.hiddenFaction")} value={<span className="text-amber-700 dark:text-amber-300">{h.name ?? h.id}</span>} />) : null}
                </dl>

                {p?.infection ? (
                    <p className="mt-3 mb-0 border-primary/40 border-l-2 pl-3 font-serif text-[13.5px] text-foreground/90 italic leading-relaxed">
                        {p.infection} <span className="text-muted-foreground text-xs not-italic">({t(INFECTION_KEY[p.infectionState])})</span>
                    </p>
                ) : null}

                {op.number ? (
                    <p className="mt-3 mb-0 text-[12px] text-muted-foreground leading-normal">
                        {op.number.faction ? t("file.numberNote", { shared: op.number.factionHolders, holders: op.number.holders, prefix: op.number.prefix, faction: op.number.faction.name }) : t("file.numberNoteNoFaction", { holders: op.number.holders, prefix: op.number.prefix })}
                    </p>
                ) : null}

                {homeChart ? (
                    <div className="mt-4 flex flex-col gap-2">
                        <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{fromDossiers || otherCharts.length === 0 ? t("file.showOnNetwork") : t("file.alsoOn")}</span>
                        <div className="flex flex-wrap gap-1.5">
                            {(fromDossiers || otherCharts.length === 0 ? [homeChart] : otherCharts).map((c) => (
                                <Button key={c.groupId} size="sm" variant="outline" onClick={() => onOpenChart(c.groupId)}>
                                    <NetworkIcon aria-hidden className="size-3.5" />
                                    {c.groupName}
                                </Button>
                            ))}
                        </div>
                    </div>
                ) : null}

                <div className="mt-4">
                    <Button size="sm" variant="secondary" render={<Link to="/operators/$id" params={{ id: op.id }} />}>
                        <ExternalLinkIcon aria-hidden className="size-3.5" />
                        {t("file.operatorPage")}
                    </Button>
                </div>

                {p && p.exam.length > 0 ? (
                    <>
                        <SectionTitle>{t("file.exam")}</SectionTitle>
                        <div className="grid grid-cols-[minmax(0,180px)_1fr] items-center gap-3 max-[420px]:grid-cols-1">
                            <RadarChart width={180} height={170} data={radar} outerRadius={58} className="text-muted-foreground">
                                <PolarGrid stroke="var(--border)" />
                                <PolarAngleAxis dataKey="axis" tick={{ fontSize: 9, fill: "currentColor" }} />
                                <PolarRadiusAxis domain={[0, op.examSteps]} tick={false} axisLine={false} tickCount={op.examSteps + 1} />
                                <Radar dataKey="value" stroke="var(--primary)" fill="var(--primary)" fillOpacity={0.3} isAnimationActive={false} />
                            </RadarChart>
                            <dl className="m-0">
                                {p.exam.map((r) => (
                                    <Fact key={r.axis} label={t(AXIS_KEY[r.axis])} value={r.redacted && r.step == null ? <em className="text-muted-foreground">{t("file.redacted")}</em> : r.raw} />
                                ))}
                            </dl>
                        </div>
                        <p className="mt-2 mb-0 text-[12px] text-muted-foreground leading-normal">
                            {missingRatings > 0 ? `${t("file.examMissing", { count: missingRatings })} ` : ""}
                            {t("file.examNote")}
                        </p>
                    </>
                ) : null}

                {op.linked.length > 0 ? (
                    <>
                        <SectionTitle>{t("file.linked")}</SectionTitle>
                        <div className="flex flex-wrap gap-1.5">
                            {op.linked.map((cid) => (
                                <PersonChip key={cid} person={people.get(cid)} onOpen={onOpenFile} server={server} />
                            ))}
                        </div>
                    </>
                ) : null}

                {op.sharesScenes.length > 0 ? (
                    <>
                        <SectionTitle>{t("file.scenes")}</SectionTitle>
                        <div className="flex flex-wrap gap-1.5">
                            {op.sharesScenes.map((m) => (
                                <span key={m.charId} className="inline-flex items-center gap-1">
                                    <PersonChip person={people.get(m.charId)} onOpen={onOpenFile} server={server} />
                                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{f.number(m.stories)}</span>
                                </span>
                            ))}
                        </div>
                        <p className="mt-2 mb-0 text-[12px] text-muted-foreground leading-normal">{t("file.scenesNote")}</p>
                    </>
                ) : null}

                <SectionTitle>{op.appearances.length > 0 ? t("file.speaks", { count: op.appearances.length }) : t("file.speaksNone")}</SectionTitle>
                {op.appearances.length > 0 ? (
                    <>
                        <ul className="m-0 grid list-none grid-cols-[auto_1fr] gap-x-3 gap-y-1 p-0 text-[12.5px]">
                            {stories.map((a) => (
                                <li key={a.storyId} className="contents">
                                    <span className="font-mono text-muted-foreground tabular-nums">{a.code ?? ""}</span>
                                    <Tooltip>
                                        <TooltipTrigger render={<span className="truncate text-foreground" />}>
                                            {a.name}
                                            {a.tag ? <span className="text-muted-foreground"> · {a.tag}</span> : null}
                                        </TooltipTrigger>
                                        <TooltipPopup>{`${a.groupName}: ${a.name}${a.tag ? ` (${a.tag})` : ""}`}</TooltipPopup>
                                    </Tooltip>
                                </li>
                            ))}
                        </ul>
                        {op.appearances.length > STORIES_SHOWN ? (
                            <Button size="xs" variant="ghost" className="mt-1" onClick={() => setAllStories((v) => !v)}>
                                {allStories ? t("file.speaksLess") : t("file.speaksMore", { count: op.appearances.length })}
                            </Button>
                        ) : null}
                    </>
                ) : null}

                {op.records.length > 0 ? (
                    <>
                        <SectionTitle>{t("file.records")}</SectionTitle>
                        <ul className="m-0 list-none p-0 text-[12.5px] text-foreground">
                            {op.records.map((r) => (
                                <li key={r.id}>{r.name}</li>
                            ))}
                        </ul>
                    </>
                ) : null}

                {p && p.extra.length > 0 ? (
                    <>
                        <SectionTitle>{t("file.extra")}</SectionTitle>
                        <dl className="m-0">
                            {p.extra.map((x) => (
                                <Fact key={`${x.label}:${x.value}`} label={x.label} value={x.value} />
                            ))}
                        </dl>
                    </>
                ) : null}
            </SheetPanel>
        </>
    );
}
