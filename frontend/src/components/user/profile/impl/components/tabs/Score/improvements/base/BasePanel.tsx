import type { CSSProperties } from "react";
import type { IUserScore } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Bar } from "../../../Stats/primitives";
import { EmptyHint, PANEL_PADDING, SectionHeader } from "../shared";
import type { messages } from "./BasePanel.messages";

/** A key in `BasePanel.messages.ts`; resolved by the panel below. */
type MessageKey = keyof typeof messages & string;

// Mirrors the backend's UTILIZATION_WEIGHT / INFRASTRUCTURE_WEIGHT constants
// in `backend/src/core/grade/base/score.rs` - change both together.
const COMPONENTS = [
    {
        key: "base_utilization",
        labelKey: "score.improvements.base.utilization.label",
        weightPct: 75,
        descriptionKey: "score.improvements.base.utilization.desc",
    },
    {
        key: "base_infrastructure",
        labelKey: "score.improvements.base.infrastructure.label",
        weightPct: 25,
        descriptionKey: "score.improvements.base.infrastructure.desc",
    },
] as const satisfies ReadonlyArray<{ key: string; labelKey: MessageKey; weightPct: number; descriptionKey: MessageKey }>;

interface IProps {
    score: IUserScore | null | undefined;
    accent: string;
}

/**
 * The Base subscore's inline panel: the grade's two stored components -
 * stationing utilization (75%) and infrastructure completeness (25%). The
 * full plan, layout and rotation tooling live in the Optimizer tab, so this
 * panel stays a pure "which term drags my score" readout.
 */
export function BasePanel({ score, accent }: IProps) {
    const t: TypedT<typeof messages> = useT("user");
    const util = score?.base_utilization ?? null;
    const infra = score?.base_infrastructure ?? null;
    if (util === null || infra === null) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>{t("score.improvements.base.empty")}</EmptyHint>
            </div>
        );
    }

    const values: Record<(typeof COMPONENTS)[number]["key"], number> = {
        base_utilization: util,
        base_infrastructure: infra,
    };

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-4`} style={{ "--imp-accent": accent } as CSSProperties}>
            <SectionHeader title={t("score.improvements.base.title")} accent={accent} />
            <div className="flex flex-col gap-3.5">
                {COMPONENTS.map((c) => {
                    const pct = Math.min(Math.max(values[c.key] * 100, 0), 100);
                    return (
                        <div key={c.key} className="flex flex-col gap-1.5">
                            <div className="flex items-baseline justify-between gap-2">
                                <span className="font-medium text-[12px] text-foreground/90">
                                    {t(c.labelKey)}
                                    <span className="ml-1.5 font-mono text-[9.5px] text-muted-foreground/70 uppercase tracking-wider">{t("score.improvements.base.weight", { pct: c.weightPct })}</span>
                                </span>
                                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{pct.toFixed(1)}%</span>
                            </div>
                            <Bar color={accent} pct={pct} />
                            <p className="text-[10.5px] text-muted-foreground/80 leading-snug">{t(c.descriptionKey)}</p>
                        </div>
                    );
                })}
            </div>
            <p className="rounded-md border border-border/40 border-dashed bg-muted/15 px-3 py-2 text-[10.5px] text-muted-foreground">{t("score.improvements.base.optimizerHint")}</p>
        </div>
    );
}
