import { useQuery } from "@tanstack/react-query";
import type * as React from "react";
import { useMemo, useRef, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
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
        <div className={cn("flex min-w-0 flex-col gap-2 rounded-xl border border-border bg-card px-4 py-3.5", className)} aria-busy={loading || undefined}>
            <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{label}</span>
            {loading ? (
                <>
                    <span className="h-6 w-16 animate-pulse rounded-md bg-muted/70" />
                    <span className="h-1 w-full rounded-full bg-muted" />
                    <span className="h-3 w-4/5 animate-pulse rounded bg-muted/50" />
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

/** A radiogroup of chips: one tab stop, arrow keys move the selection. */
function StatusPicker({ value, onChange, t }: IStatusPickerProps) {
    const refs = useRef<(HTMLButtonElement | null)[]>([]);
    const onKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
        const step = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 0;
        const edge = event.key === "Home" ? 0 : event.key === "End" ? FIELD_NOTE_STATUSES.length - 1 : null;
        if (step === 0 && edge === null) return;
        event.preventDefault();
        const current = FIELD_NOTE_STATUSES.indexOf(value);
        const next = edge ?? (current + step + FIELD_NOTE_STATUSES.length) % FIELD_NOTE_STATUSES.length;
        onChange(FIELD_NOTE_STATUSES[next]);
        refs.current[next]?.focus();
    };
    return (
        <div role="radiogroup" aria-label={t("fieldNotes.picker")} onKeyDown={onKeyDown} className="flex flex-wrap gap-1 rounded-lg border border-border bg-muted/40 p-0.5">
            {FIELD_NOTE_STATUSES.map((status, i) => {
                const checked = status === value;
                return (
                    // biome-ignore lint/a11y/useSemanticElements: a chip row with roving focus; native radio inputs would need hidden inputs plus labels for the same result
                    <button
                        key={status}
                        ref={(el) => {
                            refs.current[i] = el;
                        }}
                        type="button"
                        role="radio"
                        aria-checked={checked}
                        tabIndex={checked ? 0 : -1}
                        onClick={() => onChange(status)}
                        className={cn(
                            "rounded-md px-2.5 py-1 font-medium font-mono text-[11px] uppercase leading-none tracking-[0.08em] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                            checked ? "bg-card text-foreground shadow-xs" : "text-muted-foreground hover:text-foreground",
                        )}
                    >
                        {t("fieldNotes.status", { status })}
                    </button>
                );
            })}
        </div>
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
                <Kicker className="mb-0">{t("fieldNotes.kicker")}</Kicker>
                <StatusPicker value={status} onChange={setStatus} t={t} />
            </div>
            <div className="grid grid-cols-2 gap-3 lg:grid-cols-5">
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
