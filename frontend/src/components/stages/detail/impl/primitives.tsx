import type React from "react";
import { InfoHint as KitInfoHint } from "#/components/ui/info-hint";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./primitives.messages";

export function InfoHint({ children, label }: { children: React.ReactNode; label?: string }) {
    const t: TypedT<typeof messages> = useT("stages");
    return <KitInfoHint label={label ?? t("primitives.moreInfo")}>{children}</KitInfoHint>;
}

export function SectionHead({ children, aside }: { children: React.ReactNode; aside?: React.ReactNode }) {
    return (
        <div className="mb-3 flex items-center gap-2.5">
            <h2 className="m-0 font-sans font-semibold text-[15px] text-foreground leading-none">{children}</h2>
            <span className="h-px flex-1 bg-border" />
            {aside}
        </div>
    );
}

export function StatCard({ label, value, info }: { label: string; value: string; info?: React.ReactNode }) {
    return (
        <div className="flex items-center gap-3 rounded-2xl border border-border bg-card p-3">
            <div className="flex min-w-0 flex-col gap-1">
                <span className="flex items-center gap-1 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                    {label}
                    {info && <InfoHint label={label}>{info}</InfoHint>}
                </span>
                <span className="font-mono font-semibold text-[15px] text-foreground tabular-nums leading-none">{value}</span>
            </div>
        </div>
    );
}

export function Pill({ children, tone = "muted" }: { children: React.ReactNode; tone?: "muted" | "primary" | "warning" | "danger" }) {
    const tones: Record<string, string> = {
        muted: "border-border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)] text-muted-foreground",
        // Tone text on its own tint measured 3.80 (primary) and 4.36 (amber-700 on the 14% amber, light background), so the tone moves to the border and fill and the text stays foreground.
        primary: "border-[color-mix(in_oklch,var(--primary)_45%,transparent)] bg-[color-mix(in_oklch,var(--primary)_8%,transparent)] text-foreground",
        warning: "border-[color-mix(in_oklch,var(--warning)_60%,transparent)] bg-[color-mix(in_oklch,var(--warning)_8%,transparent)] text-foreground",
        danger: "border-[color-mix(in_oklch,var(--destructive)_45%,transparent)] bg-[color-mix(in_oklch,var(--destructive)_14%,transparent)] text-destructive-foreground",
    };
    return <span className={cn("inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 font-medium font-sans text-[11.5px] leading-none", tones[tone])}>{children}</span>;
}

export function FlagRow({ label, on, hint }: { label: string; on: boolean; hint?: React.ReactNode }) {
    const t: TypedT<typeof messages> = useT("stages");
    return (
        <div className="flex items-center justify-between gap-2 border-border/40 border-b py-2 last:border-b-0">
            <span className="flex items-center gap-1.5 font-sans text-[12.5px] text-foreground">
                {label}
                {hint && <InfoHint label={label}>{hint}</InfoHint>}
            </span>
            <span className={cn("inline-flex h-5 items-center rounded-full px-2 font-medium font-mono text-[10px]", on ? "bg-[color-mix(in_oklch,var(--success)_16%,transparent)] text-success-foreground" : "bg-[color-mix(in_oklch,var(--muted)_50%,transparent)] text-muted-foreground")}>
                {on ? t("primitives.flag.yes") : t("primitives.flag.no")}
            </span>
        </div>
    );
}

export function Meta({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-md bg-[color-mix(in_oklch,var(--muted)_50%,transparent)] px-2.5 py-2">
            <span className="mb-1.25 block font-medium font-sans text-[11.5px] text-muted-foreground leading-none">{label}</span>
            <div className="break-all font-medium font-mono text-[12px] text-foreground leading-tight">{value}</div>
        </div>
    );
}

interface ILegendSwatch {
    labelKey: keyof typeof messages & string;
    style: React.CSSProperties;
}

const LEGEND_SWATCHES: ILegendSwatch[] = [
    { labelKey: "legend.start", style: { background: "rgba(231,45,80,0.18)", borderColor: "#e72d50" } },
    { labelKey: "legend.end", style: { background: "rgba(53,157,222,0.18)", borderColor: "#359dde" } },
    { labelKey: "legend.ground", style: { background: "hsla(0,0%,71%,0.6)", borderColor: "hsla(0,0%,87%,0.9)" } },
    { labelKey: "legend.highGround", style: { background: "hsla(0,0%,51%,0.95)", borderColor: "hsla(0,0%,78%,0.93)" } },
    { labelKey: "legend.forbidden", style: { background: "hsla(0,0%,10%,0.6)", borderColor: "hsla(0,0%,87%,0.28)" } },
];

export function TileLegend() {
    const t: TypedT<typeof messages> = useT("stages");
    return (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-2xl border border-border bg-card px-3.5 py-2.5">
            <span className="font-medium font-sans text-[11.5px] text-foreground leading-none">{t("legend.title")}</span>
            {LEGEND_SWATCHES.map((s) => (
                <span key={s.labelKey} className="inline-flex items-center gap-1.5">
                    <span aria-hidden="true" className="h-3.5 w-3.5 rounded-xs border" style={s.style} />
                    <span className="font-medium font-sans text-[11.5px] text-muted-foreground leading-none">{t(s.labelKey)}</span>
                </span>
            ))}
        </div>
    );
}
