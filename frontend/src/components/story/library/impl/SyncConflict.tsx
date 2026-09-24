import { GitMergeIcon } from "lucide-react";
import type React from "react";
import { useMemo } from "react";
import { Button } from "#/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type SyncChoice, type SyncConflictSummary, type SyncPolicy, type SyncSideSummary, setStorySyncPolicy, useStorySyncPolicy } from "#/lib/story/sync";
import type { LibIndex } from "./derive";
import type { messages } from "./SyncConflict.messages";

export interface ISyncConflictProps {
    summary: SyncConflictSummary;
    /** The library index, read only to turn the two `last` story ids into names. */
    index: LibIndex;
    onResolve: (choice: SyncChoice) => void;
}

const POLICIES: readonly SyncPolicy[] = ["ask", "merge", "account", "browser"] as const;

/**
 * The comparison a reader is shown when the browser and the account each hold
 * reading progress the other does not.
 *
 * Every number on it is a number of the document the button beside it WRITES,
 * which is the only reason a prompt is better than a silent merge: a merge
 * loses nothing and is still the first button, and the two replacements say
 * out loud what they drop before they are pressed. Nothing here writes
 * anything itself; `onResolve` carries the choice back to the engine.
 */
export function SyncConflict({ summary, index, onResolve }: ISyncConflictProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const f = useFormatters();
    const names = useMemo(() => storyNames(index), [index]);

    return (
        <div className="rounded-[14px] border border-border bg-card p-4 sm:p-5">
            <div className="flex items-start gap-3">
                <GitMergeIcon className="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                <div className="min-w-0">
                    <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("progress.sync.conflict.title")}</h3>
                    <p className="mt-0.5 mb-0 max-w-prose font-sans text-[12.5px] text-muted-foreground">{t("progress.sync.conflict.blurb")}</p>
                </div>
            </div>

            <div className="mt-4 grid grid-cols-2 gap-2.5">
                <Side heading={t("progress.sync.conflict.browser")} side={summary.local} newer={summary.newer === "local"} names={names} t={t} f={f} />
                <Side heading={t("progress.sync.conflict.account")} side={summary.remote} newer={summary.newer === "remote"} names={names} t={t} f={f} />
            </div>

            <div className="mt-4 grid gap-3 sm:grid-cols-3">
                <Choice label={t("progress.sync.conflict.merge")} hint={t("progress.sync.conflict.mergeHint", { marks: f.number(summary.merged.marks), positions: f.number(summary.merged.positions) })} variant="default" onClick={() => onResolve("merge")} />
                <Choice label={t("progress.sync.conflict.useAccount")} hint={t("progress.sync.conflict.useAccountHint", { marks: f.number(summary.dropsLocal.marks), positions: f.number(summary.dropsLocal.positions) })} variant="outline" onClick={() => onResolve("account")} />
                <Choice label={t("progress.sync.conflict.keepBrowser")} hint={t("progress.sync.conflict.keepBrowserHint", { marks: f.number(summary.dropsRemote.marks), positions: f.number(summary.dropsRemote.positions) })} variant="outline" onClick={() => onResolve("browser")} />
            </div>

            <div className="mt-4 border-border border-t pt-3">
                <SyncPolicySelect />
            </div>
        </div>
    );
}

/**
 * The standing answer to the next disagreement, offered on the conflict panel
 * and on the ordinary sync row alike.
 *
 * It sits in the quiet state on purpose: a reader who picked "Account wins"
 * once needs somewhere to take it back, and a conflict is exactly the moment
 * that surface is not available.
 */
export function SyncPolicySelect(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const policy = useStorySyncPolicy();

    return (
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <span className="font-sans text-[12.5px] text-muted-foreground">{t("progress.sync.policy")}</span>
            <Select value={policy} onValueChange={(v: string | null) => v && setStorySyncPolicy(v as SyncPolicy)}>
                <SelectTrigger size="sm" className="w-auto min-w-36 max-sm:h-11" aria-label={t("progress.sync.policy")}>
                    <SelectValue>{() => t(`progress.sync.policy.${policy}`)}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                    {POLICIES.map((name) => (
                        <SelectItem key={name} value={name}>
                            {t(`progress.sync.policy.${name}`)}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    );
}

function Side({ heading, side, newer, names, t, f }: { heading: string; side: SyncSideSummary; newer: boolean; names: Map<string, string>; t: TypedT<typeof messages>; f: ReturnType<typeof useFormatters> }): React.ReactElement {
    // An id with no name is still an ANSWER: the index carries 451 groups and
    // the account may name a story from a shelf this build does not list, so
    // the raw id is shown rather than a blank.
    const last = side.last === null ? t("progress.sync.conflict.lastNone") : (names.get(side.last) ?? side.last);

    return (
        <div className="min-w-0 rounded-[12px] border border-border bg-background p-3">
            <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                <span className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{heading}</span>
                {newer ? <span className="rounded-full bg-primary/12 px-1.5 py-0.5 font-medium font-sans text-[10px] text-primary">{t("progress.sync.conflict.newer")}</span> : null}
            </div>
            <dl className="m-0 mt-2 flex flex-col gap-1.5">
                <Line label={t("progress.sync.conflict.marks")} value={f.number(side.marks)} mono />
                <Line label={t("progress.sync.conflict.positions")} value={f.number(side.positions)} mono />
                <Line label={t("progress.sync.conflict.last")} value={last} />
            </dl>
        </div>
    );
}

function Line({ label, value, mono }: { label: string; value: string; mono?: boolean }): React.ReactElement {
    return (
        <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
            <dt className="font-sans text-[12px] text-muted-foreground">{label}</dt>
            <dd className={`m-0 min-w-0 truncate font-sans text-[12.5px] text-foreground ${mono ? "font-mono tabular-nums" : ""}`}>{value}</dd>
        </div>
    );
}

function Choice({ label, hint, variant, onClick }: { label: string; hint: string; variant: "default" | "outline"; onClick: () => void }): React.ReactElement {
    return (
        <div className="flex min-w-0 flex-col gap-1.5">
            <Button variant={variant} size="sm" className="h-11 w-full sm:h-8" onClick={onClick}>
                {label}
            </Button>
            <p className="m-0 font-sans text-[11.5px] text-muted-foreground leading-snug">{hint}</p>
        </div>
    );
}

/** Story id -> name, over every shelf and every operator record the index carries. */
function storyNames(index: LibIndex): Map<string, string> {
    const names = new Map<string, string>();
    for (const group of index.groups) for (const story of group.stories) names.set(story.id, story.name);
    for (const record of index.records) for (const story of record.stories) names.set(story.id, story.name);
    return names;
}
