import type * as React from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionLabel } from "#/lib/registry/operator-display";
import type { ThemeKind } from "#/types/generated/ThemeKind";
import type { ThemeShare } from "#/types/generated/ThemeShare";
import type { Themes } from "#/types/generated/Themes";
import type { OperatorProfession } from "#/types/operators";
import { type IChipContext, type IPanelBaseProps, type ISeatOperator, OpChip, PanelShell, SectionHeading, StatCell, StatStrip } from "./shared";
import type { messages as sharedMessages } from "./shared.messages";
import type { messages } from "./ThemesPanel.messages";

type ThemesT = TypedT<typeof messages & typeof sharedMessages>;

interface IThemesPanelProps extends IPanelBaseProps {
    themes: Themes | undefined;
    seatOps: ReadonlyMap<number, ISeatOperator>;
}

export function ThemesPanel({ state, stale, collapsed, onToggle, themes, seatOps, ctx }: IThemesPanelProps): React.ReactElement {
    const t: ThemesT = useT("tools");
    return (
        <PanelShell title={t("squad.analysis.themes.title")} question={t("squad.analysis.themes.question")} state={state} stale={stale} collapsed={collapsed} onToggle={onToggle}>
            {themes && <ThemesBody themes={themes} seatOps={seatOps} ctx={ctx} t={t} />}
        </PanelShell>
    );
}

function ThemesBody({ themes, seatOps, ctx, t }: { themes: Themes; seatOps: ReadonlyMap<number, ISeatOperator>; ctx: IChipContext; t: ThemesT }): React.ReactElement {
    const f = useFormatters();
    const s = themes.stats;
    const missing = s.missing.map((p) => professionLabel(p as OperatorProfession));
    const missingCaption = missing.length === 0 ? t("squad.analysis.themes.classes.none") : t("squad.analysis.themes.classes.missing", { count: missing.length });

    return (
        <div className="space-y-4">
            <StatStrip>
                <StatCell label={t("squad.analysis.themes.rarity")} value={f.number(s.total_rarity)} caption={t("squad.analysis.themes.rarity.caption")} />
                <StatCell label={t("squad.analysis.themes.topRarity", { stars: s.top_rarity })} value={f.number(s.top_rarity_seats)} caption={t("squad.analysis.themes.topRarity.caption", { seats: s.seats })} />
                <StatCell label={t("squad.analysis.themes.medics")} value={f.number(s.medics)} />
                <StatCell
                    label={t("squad.analysis.themes.classes")}
                    value={t("squad.analysis.themes.classes.value", { count: s.classes, total: s.class_total })}
                    caption={
                        missing.length === 0 ? (
                            missingCaption
                        ) : (
                            <Tooltip>
                                <TooltipTrigger
                                    render={(p) => (
                                        <button
                                            {...p}
                                            type="button"
                                            className="relative cursor-help rounded text-left underline decoration-muted-foreground/40 decoration-dotted underline-offset-2 outline-none pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:-inset-y-4 focus-visible:ring-2 focus-visible:ring-ring"
                                        >
                                            {missingCaption}
                                        </button>
                                    )}
                                />
                                <TooltipPopup className="max-w-56">{missing.join(", ")}</TooltipPopup>
                            </Tooltip>
                        )
                    }
                />
            </StatStrip>
            <ShareSection title={t("squad.analysis.themes.labels")} hint={t("squad.analysis.themes.labels.hint")} empty={t("squad.analysis.themes.empty")} rows={themes.labels} seatOps={seatOps} ctx={ctx} t={t} />
            <ShareSection title={t("squad.analysis.themes.close")} hint={t("squad.analysis.themes.close.hint")} empty={t("squad.analysis.themes.empty")} rows={themes.close} seatOps={seatOps} ctx={ctx} t={t} />
            <ShareSection title={t("squad.analysis.themes.shared")} hint={t("squad.analysis.themes.shared.hint")} empty={t("squad.analysis.themes.emptyShared")} rows={themes.shared} seatOps={seatOps} ctx={ctx} t={t} />
            <p className="text-[12px] text-muted-foreground">{t("squad.analysis.themes.footer", { dub: themes.dub })}</p>
        </div>
    );
}

function ShareSection({ title, hint, empty, rows, seatOps, ctx, t }: { title: string; hint: string; empty: string; rows: ThemeShare[]; seatOps: ReadonlyMap<number, ISeatOperator>; ctx: IChipContext; t: ThemesT }): React.ReactElement {
    return (
        <section>
            <SectionHeading title={title} hint={hint} />
            {rows.length === 0 ? (
                <p className="text-[13px] text-muted-foreground">{empty}</p>
            ) : (
                <ul className="divide-y divide-border/60 rounded-xl border border-border bg-card">
                    {rows.map((row) => {
                        const value = valueText(row, t);
                        return (
                            <li key={`${row.kind}-${row.value}`} className="flex flex-col gap-1.5 px-3 py-2 sm:flex-row sm:items-center sm:gap-3">
                                <div className="min-w-0 sm:w-44 sm:shrink-0">
                                    <span className="text-[11px] text-muted-foreground">{kindLabel(row.kind, t)}</span>
                                    <div className="truncate font-medium text-[13px] text-foreground" title={value}>
                                        {value}
                                    </div>
                                </div>
                                <div className="flex min-w-0 flex-1 flex-wrap gap-1.5">
                                    {row.seats.map((seat) => {
                                        const op = seatOps.get(seat);
                                        return op ? <OpChip key={seat} charId={op.charId} seat={seat} ctx={ctx} why={value} /> : null;
                                    })}
                                </div>
                            </li>
                        );
                    })}
                </ul>
            )}
        </section>
    );
}

function valueText(row: ThemeShare, t: ThemesT): string {
    if (row.kind === "class") return professionLabel(row.value as OperatorProfession);
    if (row.kind === "rarity") {
        const stars = Number.parseInt(row.value.replace(/\D+/g, ""), 10);
        return Number.isNaN(stars) ? row.value : t("squad.analysis.themes.rarityValue", { count: stars });
    }
    return row.value;
}

function kindLabel(kind: ThemeKind, t: ThemesT): string {
    switch (kind) {
        case "gender":
            return t("squad.analysis.themes.kind.gender");
        case "race":
            return t("squad.analysis.themes.kind.race");
        case "birthplace":
            return t("squad.analysis.themes.kind.birthplace");
        case "faction":
            return t("squad.analysis.themes.kind.faction");
        case "class":
            return t("squad.analysis.themes.kind.class");
        case "branch":
            return t("squad.analysis.themes.kind.branch");
        case "position":
            return t("squad.analysis.themes.kind.position");
        case "rarity":
            return t("squad.analysis.themes.kind.rarity");
        case "artist":
            return t("squad.analysis.themes.kind.artist");
        case "voice":
            return t("squad.analysis.themes.kind.voice");
        case "birthday":
            return t("squad.analysis.themes.kind.birthday");
    }
}
