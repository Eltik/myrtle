import { useQuery } from "@tanstack/react-query";
import type * as React from "react";
import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { Skeleton } from "#/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { enemiesListQueryOptions, enemyCommunityAverageQueryOptions } from "#/lib/api/enemies";
import { operatorOwnershipQueryOptions, operatorsIndexQueryOptions, operatorsRosterQueryOptions } from "#/lib/api/operators";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./FieldNotes.messages";
import { DEFAULT_FIELD_NOTE_STATUS, enemyImmunity, FIELD_NOTE_STATUSES, type FieldNoteStatus, type IRatio, mostOwnedSixStar, operatorsInflicting, ownableOperators, rarestSixStar, share } from "./fieldNotesData";

type FieldNotesT = TypedT<typeof messages>;

const DASH = "-";

interface IFactTileProps {
    label: string;
    /** `undefined` while loading; `null` when there is no share to show (error or a zero denominator). */
    fraction: number | null | undefined;
    sentence?: React.ReactNode;
    meta?: React.ReactNode;
    className?: string;
}

function FactTile({ label, fraction, sentence, meta, className }: IFactTileProps) {
    const f = useFormatters();
    const loading = fraction === undefined;
    return (
        <div className={cn("flex min-w-0 flex-col gap-2 bg-card px-4 py-3.5", className)} aria-busy={loading || undefined}>
            <Kicker>{label}</Kicker>
            {loading ? (
                <>
                    <Skeleton className="h-6 w-16 rounded-md" />
                    <span className="h-1 w-full rounded-full bg-muted" />
                    <Skeleton className="h-3 w-4/5 rounded" />
                </>
            ) : (
                <>
                    <span className="font-bold font-sans text-[24px] text-foreground tabular-nums leading-none tracking-tight">{fraction === null ? DASH : f.percent(fraction)}</span>
                    <span className="h-1 w-full overflow-hidden rounded-full bg-muted" aria-hidden="true">
                        <span className="block h-full rounded-full bg-primary" style={{ width: `${Math.min(1, Math.max(0, fraction ?? 0)) * 100}%` }} />
                    </span>
                    {sentence ? <span className="text-pretty font-sans text-[12.5px] text-foreground/85 leading-snug">{sentence}</span> : null}
                    {meta ? <span className="mt-auto font-medium font-mono text-[11px] text-muted-foreground tabular-nums leading-[1.3]">{meta}</span> : null}
                </>
            )}
        </div>
    );
}

interface IStatusPickerProps {
    value: FieldNoteStatus;
    onChange: (status: FieldNoteStatus) => void;
    t: FieldNotesT;
}

/**
 * A single-select kit ToggleGroup: one tab stop, arrow keys move focus, the kit carries the coarse 44px hit area.
 * Unpressed items keep a border so they read as options, not text. Below sm the five statuses sit 3 + 2 on a
 * six-track grid, so no row ends with one orphaned chip ("Levitate" wrapped alone at 375).
 */
function StatusPicker({ value, onChange, t }: IStatusPickerProps) {
    return (
        <ToggleGroup aria-label={t("fieldNotes.picker")} value={[value]} onValueChange={(next) => next[0] && onChange(next[0] as FieldNoteStatus)} size="sm" className="flex-wrap max-sm:grid max-sm:w-full max-sm:grid-cols-6 max-sm:*:col-span-2 max-sm:*:nth-[n+4]:col-span-3">
            {FIELD_NOTE_STATUSES.map((status) => (
                <ToggleGroupItem key={status} value={status} className="border-border px-2.5 text-[12.5px]">
                    {t("fieldNotes.status", { status })}
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    );
}

/** `undefined` while the query loads, `null` on error or a zero denominator. */
function shareOf<T>(query: { data: T | undefined; isError: boolean }, ratio: (data: T) => IRatio | null): number | null | undefined {
    if (query.isError) return null;
    if (query.data === undefined) return undefined;
    const r = ratio(query.data);
    return r ? share(r) : null;
}

export function FieldNotes() {
    const t: FieldNotesT = useT("stats");
    const f = useFormatters();
    const server = useGamedataServer();
    const [status, setStatus] = useState<FieldNoteStatus>(DEFAULT_FIELD_NOTE_STATUS);

    const enemies = useQuery(enemiesListQueryOptions(server));
    const average = useQuery(enemyCommunityAverageQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions(server));
    const roster = useQuery(operatorsRosterQueryOptions(server));
    const ownership = useQuery(operatorOwnershipQueryOptions(server));

    const operators = useMemo(() => (index.data ? ownableOperators(index.data) : undefined), [index.data]);
    const immunity = useMemo(() => (enemies.data ? enemyImmunity(enemies.data, status) : undefined), [enemies.data, status]);
    const inflicting = useMemo(() => (operators && roster.data ? operatorsInflicting(operators, roster.data, status) : undefined), [operators, roster.data, status]);
    const owned = useMemo(() => (operators && ownership.data ? { most: mostOwnedSixStar(operators, ownership.data), rarest: rarestSixStar(operators, ownership.data) } : undefined), [operators, ownership.data]);

    const ratio = (r: IRatio) => t("fieldNotes.ratio", { n: f.number(r.n), total: f.number(r.total) });
    const pct = (r: IRatio) => {
        const s = share(r);
        return s === null ? DASH : f.percent(s);
    };

    const operatorsQuery = { data: inflicting, isError: index.isError || roster.isError };
    const ownedQuery = { data: owned, isError: index.isError || ownership.isError };

    return (
        <div className="flex flex-col gap-3 border-border border-t pt-4">
            <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
                <Kicker>{t("fieldNotes.kicker")}</Kicker>
                <StatusPicker value={status} onChange={setStatus} t={t} />
            </div>
            <div className="grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-border bg-border lg:grid-cols-5">
                <FactTile
                    label={t("fieldNotes.immune.label")}
                    fraction={shareOf(enemies, () => immunity ?? null)}
                    sentence={t("fieldNotes.immune.sentence", { status })}
                    meta={
                        immunity ? (
                            <>
                                {t("fieldNotes.immune.levels", { boss: pct(immunity.byLevel.BOSS), elite: pct(immunity.byLevel.ELITE), normal: pct(immunity.byLevel.NORMAL) })}
                                <br />
                                {ratio(immunity)}
                            </>
                        ) : null
                    }
                />
                <FactTile label={t("fieldNotes.inflict.label")} fraction={shareOf(operatorsQuery, (r) => r)} sentence={t("fieldNotes.inflict.sentence", { status })} meta={inflicting && !operatorsQuery.isError ? ratio(inflicting) : null} />
                <FactTile label={t("fieldNotes.mostOwned.label")} fraction={shareOf(ownedQuery, (o) => o.most)} sentence={owned?.most && !ownedQuery.isError ? t("fieldNotes.mostOwned.sentence", { name: owned.most.name }) : null} meta={owned?.most && !ownedQuery.isError ? ratio(owned.most) : null} />
                <FactTile label={t("fieldNotes.rarest.label")} fraction={shareOf(ownedQuery, (o) => o.rarest)} sentence={owned?.rarest && !ownedQuery.isError ? t("fieldNotes.rarest.sentence", { name: owned.rarest.name }) : null} meta={owned?.rarest && !ownedQuery.isError ? ratio(owned.rarest) : null} />
                <FactTile
                    // Spans the row while the grid is two-up, so the fifth tile is not left alone.
                    className="col-span-2 lg:col-span-1"
                    label={t("fieldNotes.progress.label")}
                    fraction={shareOf(average, (a) => ({ n: a.averageEncountered, total: a.handbookTotal }))}
                    sentence={average.data ? t("fieldNotes.progress.sentence", { met: f.number(Math.round(average.data.averageEncountered)), total: f.number(average.data.handbookTotal) }) : null}
                    meta={average.data ? t("fieldNotes.progress.meta", { count: f.number(average.data.userCount) }) : null}
                />
            </div>
        </div>
    );
}
