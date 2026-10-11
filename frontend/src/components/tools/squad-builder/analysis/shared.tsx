import { Link } from "@tanstack/react-router";
import { ChevronDown } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { segmentedControlItemVariants, segmentedControlRootClassName } from "#/lib/segmented-control";
import { cn, rarityStarColor } from "#/lib/utils";
import type { SourceRef } from "#/types/generated/SourceRef";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { messages } from "./shared.messages";

export type SharedT = TypedT<typeof messages>;

/** What a panel shows: nothing seated, the first answer loading, a failed answer, or the answer. */
export type PanelState = "empty" | "loading" | "error" | "ready";

/** A seat's operator as the response names it, for chips on seats the response refers to by index. */
export interface ISeatOperator {
    charId: string;
    support: boolean;
}

/** Everything a chip needs to find its operator and light its seat. */
export interface IChipContext {
    operators: ReadonlyMap<string, IOperatorIndexEntry>;
    onHoverSeat: (seat: number | null) => void;
}

interface IPanelShellProps {
    title: string;
    question: string;
    state: PanelState;
    collapsed: boolean;
    onToggle: () => void;
    /** Dims the body while a newer answer loads over the last one. */
    stale?: boolean;
    className?: string;
    children: React.ReactNode;
}

export function PanelShell({ title, question, state, collapsed, onToggle, stale = false, className, children }: IPanelShellProps): React.ReactElement {
    const t: SharedT = useT("tools");
    const bodyId = React.useId();
    return (
        <Card className={cn("min-w-0", className)}>
            <CardHeader className={cn("gap-1", collapsed ? "pb-6" : "pb-4")}>
                <div className="font-medium text-[11px] text-muted-foreground">{t("squad.analysis.eyebrow")}</div>
                <CardTitle className="text-[16px]">{title}</CardTitle>
                <CardDescription className="text-[13px]">{question}</CardDescription>
                <CardAction>
                    <Button variant="ghost" size="icon-sm" aria-expanded={!collapsed} aria-controls={bodyId} aria-label={collapsed ? t("squad.analysis.expand", { panel: title }) : t("squad.analysis.collapse", { panel: title })} onClick={onToggle} className="text-muted-foreground hover:text-foreground">
                        <ChevronDown className={cn("transition-transform", collapsed && "-rotate-90")} />
                    </Button>
                </CardAction>
            </CardHeader>
            {!collapsed && (
                <CardPanel id={bodyId} className={cn("pt-0 transition-opacity", stale && state === "ready" && "opacity-70")}>
                    {state === "empty" ? (
                        <p className="rounded-xl border border-border border-dashed px-4 py-6 text-center text-[13px] text-muted-foreground">{t("squad.analysis.empty")}</p>
                    ) : state === "loading" ? (
                        <div className="space-y-2" aria-busy="true">
                            <p className="text-[12px] text-muted-foreground">{t("squad.analysis.loading")}</p>
                            <Skeleton className="h-14 w-full rounded-xl" />
                            <Skeleton className="h-6 w-4/5 rounded-md" />
                            <Skeleton className="h-6 w-3/5 rounded-md" />
                        </div>
                    ) : state === "error" ? (
                        <p className="rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3 text-[13px] text-destructive-foreground">{t("squad.analysis.error", { panel: title })}</p>
                    ) : (
                        children
                    )}
                </CardPanel>
            )}
        </Card>
    );
}

interface IOpChipProps {
    charId: string;
    seat: number;
    ctx: IChipContext;
    /** Why the chip is here, appended to the name in the hover title. */
    why?: string;
    support?: boolean;
    className?: string;
}

/** A small linked operator chip; hovering it lights the operator's seat card. */
export function OpChip({ charId, seat, ctx, why, support = false, className }: IOpChipProps): React.ReactElement {
    const t: SharedT = useT("tools");
    const op = ctx.operators.get(charId);
    const name = op?.name ?? charId;
    return (
        <Link
            to="/operators/$id"
            params={{ id: charId }}
            title={why ? t("squad.analysis.chip.title", { name, why }) : name}
            onMouseEnter={() => ctx.onHoverSeat(seat)}
            onMouseLeave={() => ctx.onHoverSeat(null)}
            onFocus={() => ctx.onHoverSeat(seat)}
            onBlur={() => ctx.onHoverSeat(null)}
            className={cn("inline-flex pointer-coarse:min-h-11 max-w-full items-center gap-1.5 rounded-md border border-border/70 border-l-2 bg-secondary/40 py-0.5 pr-2 pl-0.5 font-medium text-[12px] text-foreground transition-colors hover:bg-accent/60", className)}
            style={{ borderLeftColor: rarityStarColor(op?.rarity ?? 1) }}
        >
            <span aria-hidden="true" className="inline-flex size-5 shrink-0 items-center justify-center overflow-hidden rounded bg-muted text-[9px]">
                <OperatorAvatar charId={charId} name={name} />
            </span>
            <span className="truncate">{name}</span>
            {support && <span className="shrink-0 rounded bg-muted px-1 text-[10px] text-muted-foreground">{t("squad.analysis.chip.support")}</span>}
        </Link>
    );
}

export type PillTone = "muted" | "accent" | "outline";

export function Pill({ tone = "muted", className, children, title }: { tone?: PillTone; className?: string; children: React.ReactNode; title?: string }): React.ReactElement {
    return (
        <span
            title={title}
            className={cn("inline-flex shrink-0 items-center gap-1 rounded-full px-2 py-0.5 font-medium text-[11px] leading-tight", tone === "muted" && "bg-muted text-muted-foreground", tone === "accent" && "bg-primary/12 text-primary", tone === "outline" && "border border-border text-muted-foreground", className)}
        >
            {children}
        </span>
    );
}

/** One cell of a panel's stat strip. */
export function StatCell({ label, value, caption, extra }: { label: string; value: React.ReactNode; caption?: React.ReactNode; extra?: React.ReactNode }): React.ReactElement {
    return (
        <div className="min-w-0 rounded-xl border border-border bg-card px-3 py-2.5">
            <div className="truncate text-[11px] text-muted-foreground">{label}</div>
            <div className="mt-0.5 flex items-center gap-2 font-mono font-semibold text-[18px] text-foreground tabular-nums">{value}</div>
            {caption && <div className="mt-0.5 text-[11px] text-muted-foreground leading-snug">{caption}</div>}
            {extra}
        </div>
    );
}

export function StatStrip({ children }: { children: React.ReactNode }): React.ReactElement {
    return <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">{children}</div>;
}

export function SectionHeading({ title, hint }: { title: string; hint?: string }): React.ReactElement {
    return (
        <div className="mb-2 flex flex-wrap items-baseline gap-x-2">
            <h3 className="font-semibold text-[13px] text-foreground">{title}</h3>
            {hint && <span className="text-[12px] text-muted-foreground">{hint}</span>}
        </div>
    );
}

/** A small segmented control for a panel's filter or view. */
export function Segmented<T extends string>({ label, value, options, onChange }: { label: string; value: T; options: ReadonlyArray<{ value: T; label: string }>; onChange: (value: T) => void }): React.ReactElement {
    return (
        <fieldset className={cn(segmentedControlRootClassName, "m-0 min-w-0 max-w-full flex-wrap justify-start border-0")}>
            <legend className="sr-only">{label}</legend>
            {options.map((o) => {
                const on = o.value === value;
                return (
                    <button key={o.value} type="button" aria-pressed={on} data-pressed={on ? "" : undefined} onClick={() => onChange(o.value)} className={cn(segmentedControlItemVariants({ size: "sm", state: "pressed" }), "pointer-coarse:min-h-11 pointer-coarse:min-w-11")}>
                        {o.label}
                    </button>
                );
            })}
        </fieldset>
    );
}

/** Where a fact came from, as the chip title words it. */
export function sourceLabel(why: SourceRef, t: SharedT): string {
    const name = why.name ?? "";
    switch (why.kind) {
        case "trait":
            return t("squad.analysis.source.trait");
        case "skill":
            return t("squad.analysis.source.skill", { n: (why.skill_index ?? 0) + 1, name });
        case "talent":
            return t("squad.analysis.source.talent", { name });
        case "module":
            return t("squad.analysis.source.module", { name });
        case "stats":
            return t("squad.analysis.source.stats");
        case "tag":
            return t("squad.analysis.source.tag");
    }
}

/** The operator's display name, falling back to the id while the index loads. */
export function opName(operators: ReadonlyMap<string, IOperatorIndexEntry>, charId: string): string {
    return operators.get(charId)?.name ?? charId;
}
