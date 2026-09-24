import type React from "react";
import { useId, useMemo } from "react";
import { Input } from "#/components/ui/input";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { humanTime, MAX_MINUTES_PER_DAY, MAX_WPM, MIN_MINUTES_PER_DAY, MIN_WPM, minutesFor, paceSpan, useReadingSpeed } from "#/lib/story/reading";
import type { LibIndex } from "./derive";
import { KindBadge } from "./GroupCard";
import type { messages } from "./ReadingStats.messages";
import { readingStats } from "./stats";

type StatsT = TypedT<typeof messages>;

/** The top-N of the longest table. 25 is the reference site's own depth and it is what fits before the table stops being a ranking and starts being the index again. */
const LONGEST = 25;

export interface IReadingStatsProps {
    index: LibIndex;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
}

/**
 * HOW LONG THE LIBRARY IS, IN HOURS RATHER THAN IN WORDS.
 *
 * `1,247,905 words` is a true number nobody can picture. Divided by a reading
 * speed it becomes a span an evening can be measured against, which is the
 * whole point of the section, and every figure in it moves together when the
 * speed changes.
 *
 * The speed is a SETTING, not a measurement: nothing in the index says how fast
 * this reader reads, and the two inputs at the top are what keep that honest.
 * They are this browser's own and stay out of the synced progress document, for
 * the reason in `lib/story/reading.ts`.
 *
 * Words come through `isStoryRead`, so a chapter the Arknights client reports
 * played counts here exactly as one ticked off by hand does.
 */
export function ReadingStats({ index, progress, gameRead }: IReadingStatsProps): React.ReactElement {
    const t: StatsT = useT("story");
    const f = useFormatters();
    const { wpm, setWpm, minutesPerDay, setMinutesPerDay } = useReadingSpeed();
    const wpmId = useId();
    const perDayId = useId();

    const stats = useMemo(() => readingStats(index, progress, gameRead, LONGEST), [index, progress, gameRead]);
    const wordsLeft = Math.max(0, stats.wordsTotal - stats.wordsRead);
    const minutesLeft = minutesFor(wordsLeft, wpm);
    const pace = paceSpan(minutesLeft, minutesPerDay);

    return (
        <section className="rounded-[14px] border border-border bg-card p-4 sm:p-5">
            <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("stats.heading")}</h3>

            {!stats.counted ? (
                <p className="mt-2 mb-0 max-w-prose font-sans text-[12.5px] text-muted-foreground">{t("stats.pending")}</p>
            ) : (
                <>
                    <p className="mt-1 mb-3 max-w-prose font-sans text-[12.5px] text-muted-foreground">{t("stats.blurb")}</p>

                    <div className="flex flex-wrap gap-x-6 gap-y-3">
                        <NumberSetting id={wpmId} label={t("stats.wpm")} unit={t("stats.wpm.unit")} value={wpm} min={MIN_WPM} max={MAX_WPM} step={5} onChange={setWpm} />
                        <NumberSetting id={perDayId} label={t("stats.minutesPerDay")} unit={t("stats.minutesPerDay.unit")} value={minutesPerDay} min={MIN_MINUTES_PER_DAY} max={MAX_MINUTES_PER_DAY} step={5} onChange={setMinutesPerDay} />
                    </div>

                    <div className="mt-4 grid grid-cols-2 gap-2.5 lg:grid-cols-4">
                        <Stat label={t("stats.wordsRead")} value={t("stats.wordsReadValue", { read: f.compact(stats.wordsRead), total: f.compact(stats.wordsTotal) })} />
                        <Stat label={t("stats.timeSpent")} value={humanTime(minutesFor(stats.wordsRead, wpm))} />
                        <Stat label={t("stats.timeLeft")} value={humanTime(minutesLeft)} />
                        <Stat label={t("stats.coverToCover")} value={humanTime(minutesFor(stats.wordsTotal, wpm))} />
                    </div>

                    <p className="mt-3 mb-0 font-sans text-[12.5px] text-muted-foreground">
                        {wordsLeft === 0 ? t("stats.pace.done") : pace.unit === "day" ? t("stats.pace.days", { value: f.number(pace.value), perDay: f.number(minutesPerDay) }) : t("stats.pace.months", { value: pace.value.toFixed(1), perDay: f.number(minutesPerDay) })}
                    </p>

                    <div className="msv-scroll mt-4 max-w-full overflow-x-auto">
                        <table className="w-full min-w-95 border-collapse font-mono text-[11.5px] tabular-nums">
                            <thead>
                                <tr className="border-border border-b text-left text-[10px] text-muted-foreground uppercase tracking-[0.1em]">
                                    <th scope="col" className="py-1.5 pe-3 font-normal">
                                        {t("stats.table.category")}
                                    </th>
                                    <th scope="col" className="py-1.5 pe-3 text-right font-normal">
                                        {t("stats.table.stories")}
                                    </th>
                                    <th scope="col" className="py-1.5 pe-3 text-right font-normal">
                                        {t("stats.table.words")}
                                    </th>
                                    <th scope="col" className="py-1.5 text-right font-normal">
                                        {t("stats.table.time")}
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {stats.rows.map((row) => (
                                    <tr key={row.key} className="border-border/60 border-b">
                                        <th scope="row" className="py-1.5 pe-3 text-left font-normal font-sans text-[12px] text-foreground">
                                            {t(`stats.bucket.${row.key}`)}
                                        </th>
                                        <td className="py-1.5 pe-3 text-right text-muted-foreground">{t("stats.table.fraction", { read: f.number(row.read), total: f.number(row.total) })}</td>
                                        <td className="py-1.5 pe-3 text-right text-muted-foreground">{f.number(row.words)}</td>
                                        <td className="py-1.5 text-right text-foreground">{humanTime(minutesFor(row.words, wpm))}</td>
                                    </tr>
                                ))}
                                <tr>
                                    <th scope="row" className="py-1.5 pe-3 text-left font-sans font-semibold text-[12px] text-foreground">
                                        {t("stats.total")}
                                    </th>
                                    <td className="py-1.5 pe-3 text-right text-foreground">{t("stats.table.fraction", { read: f.number(stats.read), total: f.number(stats.total) })}</td>
                                    <td className="py-1.5 pe-3 text-right text-foreground">{f.number(stats.wordsTotal)}</td>
                                    <td className="py-1.5 text-right text-foreground">{humanTime(minutesFor(stats.wordsTotal, wpm))}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>

                    {stats.longest.length > 0 ? (
                        <>
                            <h4 className="mt-5 mb-1 font-sans font-semibold text-[13px] text-foreground">{t("stats.longest")}</h4>
                            <p className="mt-0 mb-2 max-w-prose font-sans text-[11.5px] text-muted-foreground leading-relaxed">{t("stats.longest.note", { count: stats.longest.length })}</p>
                            <ol className="m-0 flex list-none flex-col p-0">
                                {/* THE THREE FIGURES DROP UNDER THE NAME ON A PHONE. Held on
                                    one line at 390 they took 214 of the row's 294 px and
                                    left the title column 13 px, so this ranking printed
                                    "A.", "R.", "T." and was a list of nothing. The
                                    `sm:contents` wrapper dissolves above 640 and the three
                                    spans become direct children of the row again, so the
                                    desktop row is the row it was. */}
                                {stats.longest.map((row, at) => (
                                    <li key={row.id} className="flex flex-wrap items-center gap-x-2.5 gap-y-1 border-border/60 border-b py-1.5 last:border-b-0 sm:flex-nowrap">
                                        <span className="w-6 shrink-0 text-right font-mono text-[10px] text-muted-foreground tabular-nums">{at + 1}</span>
                                        <KindBadge kind={row.kind} className="shrink-0 max-sm:hidden" />
                                        <span className="min-w-0 flex-1 font-sans text-[12.5px] text-foreground sm:truncate">{row.name}</span>
                                        <span className="flex items-center gap-2.5 max-sm:w-full max-sm:justify-end max-sm:ps-8.5 sm:contents">
                                            <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("stats.table.fraction", { read: f.number(row.read), total: f.number(row.total) })}</span>
                                            <span className="w-16 shrink-0 text-right font-mono text-[10.5px] text-muted-foreground tabular-nums">{f.compact(row.words)}</span>
                                            <span className="w-14 shrink-0 text-right font-mono text-[10.5px] text-foreground tabular-nums">{humanTime(minutesFor(row.words, wpm))}</span>
                                        </span>
                                    </li>
                                ))}
                            </ol>
                        </>
                    ) : null}
                </>
            )}
        </section>
    );
}

/**
 * One bounded number the reader types.
 *
 * The clamp runs in `lib/story/reading.ts` on the way to storage rather than in
 * the field, so a half-typed "4" on the way to "400" is not rewritten to 40
 * under the cursor; `min` and `max` are on the element for the browser's own
 * stepper and for a screen reader to announce the band.
 */
function NumberSetting({ id, label, unit, value, min, max, step, onChange }: { id: string; label: string; unit: string; value: number; min: number; max: number; step: number; onChange: (next: number) => void }): React.ReactElement {
    return (
        <div className="flex min-w-0 flex-col gap-1">
            <label htmlFor={id} className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">
                {label}
            </label>
            <div className="flex items-center gap-2">
                <Input
                    id={id}
                    type="number"
                    inputMode="numeric"
                    min={min}
                    max={max}
                    step={step}
                    value={String(value)}
                    onChange={(event) => onChange(Number(event.target.value))}
                    className="pointer-coarse:h-11 w-24 font-mono tabular-nums max-sm:h-11 pointer-coarse:[&_[data-slot=input]]:h-11 pointer-coarse:[&_[data-slot=input]]:leading-11 max-sm:[&_[data-slot=input]]:h-11 max-sm:[&_[data-slot=input]]:leading-11"
                />
                <span className="font-sans text-[12px] text-muted-foreground">{unit}</span>
            </div>
        </div>
    );
}

function Stat({ label, value }: { label: string; value: string }): React.ReactElement {
    return (
        <div className="rounded-[12px] border border-border bg-background/60 p-3">
            <div className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{label}</div>
            <div className="mt-1 font-light font-mono text-[18px] text-foreground tabular-nums tracking-[-0.02em] sm:text-[22px]">{value}</div>
        </div>
    );
}
