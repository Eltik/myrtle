import * as React from "react";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IBannerModel } from "../pulls/rates";
import type { messages } from "./PullsPlannerTab.messages";

export type PullsT = TypedT<typeof messages>;

/**
 * Odds need more resolution than `useFormatters().percent` gives: the difference
 * between 96.4% and 96.42% is the difference between agreeing with an independent
 * implementation and appearing to disagree with it. The locale is still passed
 * explicitly, which is the part that matters.
 */
export function usePct(): (fraction: number, digits?: number) => string {
    const locale = useLocale();
    return React.useCallback(
        (fraction: number, digits = 2) => {
            const clamped = Math.min(1, Math.max(0, fraction));
            // Do not round a real chance up to a certainty it is not, or down to an
            // impossibility. The epsilon has to TRACK `digits`: at two fraction
            // digits the percent formatter rounds from 0.99995 upward, so a fixed
            // 0.9999999 guard sits orders of magnitude on the wrong side of the
            // threshold and lets a real 0.99998 render as a flat 100%.
            const eps = 5 * 10 ** -(digits + 3);
            const value = clamped >= 1 ? 1 : clamped <= 0 ? 0 : Math.min(Math.max(clamped, eps), 1 - eps);
            return new Intl.NumberFormat(locale, { style: "percent", minimumFractionDigits: 0, maximumFractionDigits: digits }).format(value);
        },
        [locale],
    );
}

interface INumberFieldProps {
    id: string;
    label: string;
    value: number;
    onChange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    className?: string;
}

export function PullsNumber({ id, label, value, onChange, min = 0, max, step = 1, className }: INumberFieldProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-1">
            <Label htmlFor={id} className="font-sans text-[11.5px] text-muted-foreground">
                {label}
            </Label>
            <Input
                id={id}
                type="number"
                inputMode="numeric"
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(e) => {
                    const next = Number(e.target.value);
                    // `Number("")` is 0 and finite, so an empty box reads as zero
                    // rather than NaN. That is the wanted behaviour here, but it
                    // has to be deliberate: never trust a falsy check.
                    if (!Number.isFinite(next)) return;
                    onChange(Math.min(max ?? Number.MAX_SAFE_INTEGER, Math.max(min, next)));
                }}
                className={cn("h-8 w-28 text-right font-mono tabular-nums", className)}
            />
        </div>
    );
}

/**
 * The player-facing name of a banner archetype. `humanizeTag` title-cases the raw
 * gamedata enum, which is fine for a debug tag and wrong for a picker: the game calls
 * LINKAGE "Collaboration" and ATTAIN "Special Headhunting", and neither spelling is
 * a string a translator can reach.
 */
export type BannerArchetype = "LIMITED" | "SINGLE" | "DOUBLE" | "NORMAL" | "LINKAGE" | "CLASSIC" | "ATTAIN";

export function useBannerLabel(): (ruleType: BannerArchetype) => string {
    const t: PullsT = useT("tools");
    // Seven literal call sites rather than a table of key strings. The table reads
    // better and extracts as seven dead keys, because the extractor matches literal
    // t() arguments and cannot follow an index into a constant.
    const labels: Record<BannerArchetype, string> = {
        LIMITED: t("release.pulls.banner.LIMITED"),
        SINGLE: t("release.pulls.banner.SINGLE"),
        DOUBLE: t("release.pulls.banner.DOUBLE"),
        NORMAL: t("release.pulls.banner.NORMAL"),
        LINKAGE: t("release.pulls.banner.LINKAGE"),
        CLASSIC: t("release.pulls.banner.CLASSIC"),
        ATTAIN: t("release.pulls.banner.ATTAIN"),
    };
    return (ruleType) => labels[ruleType];
}

export function Stat({ label, value, sub, className }: { label: string; value: React.ReactNode; sub?: React.ReactNode; className?: string }): React.ReactElement {
    return (
        <div className={cn("flex min-w-0 flex-col gap-0.5", className)}>
            <span className="font-sans text-[11px] text-muted-foreground">{label}</span>
            <span className="font-bold font-mono text-[18px] text-foreground tabular-nums leading-none">{value}</span>
            {sub && <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{sub}</span>}
        </div>
    );
}

/** The plain-language reading of a banner's model, so a number is never unexplained. */
export function BannerModelNote({ model, className }: { model: IBannerModel; className?: string }): React.ReactElement {
    const t: PullsT = useT("tools");
    const lines: string[] = [t("release.pulls.rule.share", { percent: Math.round(model.shareTotal * 100), count: model.featuredCount })];

    // Both scope names are resolved unconditionally and then chosen between. A
    // ternary INSIDE the t() call would read fine and extract as a dead key, because
    // the extractor matches literal call sites and cannot see through a conditional.
    const standard = t("release.pulls.rule.scope.standard");
    const kernel = t("release.pulls.rule.scope.kernel");
    if (model.carryOver) lines.push(t("release.pulls.rule.carry", { scope: model.scope === "kernel" ? kernel : standard }));
    else lines.push(t("release.pulls.rule.isolated"));

    const g = model.guarantee;
    if (g.kind === "linkage" && g.at !== undefined) lines.push(t("release.pulls.rule.guarantee.linkage", { at: g.at }));
    if (g.kind === "selection" && g.first !== undefined && g.second !== undefined) lines.push(t("release.pulls.rule.guarantee.selection", { first: g.first, second: g.second }));
    if (g.kind === "attain") lines.push(t("release.pulls.rule.guarantee.attain"));
    if (model.spark !== null) lines.push(t("release.pulls.rule.spark", { count: model.spark }));

    return (
        <ul className={cn("m-0 flex list-none flex-col gap-0.5 p-0 font-sans text-[11.5px] text-muted-foreground leading-normal", className)}>
            {lines.map((line) => (
                <li key={line}>{line}</li>
            ))}
            {model.inferred && <li className="text-amber-500/90">{t("release.pulls.banners.inferred")}</li>}
        </ul>
    );
}
