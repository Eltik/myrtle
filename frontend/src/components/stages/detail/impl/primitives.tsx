import type React from "react";
import { cn } from "#/lib/utils";

export function Kicker({ children }: { children: React.ReactNode }) {
    return <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{children}</span>;
}

export function SectionHead({ children, aside }: { children: React.ReactNode; aside?: React.ReactNode }) {
    return (
        <div className="mb-3 flex items-center gap-2.5">
            <Kicker>{children}</Kicker>
            <span className="h-px flex-1 bg-border" />
            {aside}
        </div>
    );
}

export function StatCard({ icon, label, value, accent }: { icon: React.ReactNode; label: string; value: string; accent?: string }) {
    return (
        <div className="flex items-center gap-3 rounded-[10px] border border-border bg-card p-3">
            <span
                aria-hidden="true"
                className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md [&_svg]:h-4.5 [&_svg]:w-4.5"
                style={{
                    background: accent ? `color-mix(in oklch, ${accent} 14%, transparent)` : "color-mix(in oklch, var(--muted) 50%, transparent)",
                    color: accent ?? "var(--muted-foreground)",
                }}
            >
                {icon}
            </span>
            <div className="flex min-w-0 flex-col gap-1">
                <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{label}</span>
                <span className="font-mono font-semibold text-[15px] text-foreground tabular-nums leading-none">{value}</span>
            </div>
        </div>
    );
}

export function Pill({ children, tone = "muted" }: { children: React.ReactNode; tone?: "muted" | "primary" | "warning" | "danger" }) {
    const tones: Record<string, string> = {
        muted: "border-border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)] text-muted-foreground",
        primary: "border-[color-mix(in_oklch,var(--primary)_40%,transparent)] bg-[color-mix(in_oklch,var(--primary)_12%,transparent)] text-primary",
        warning: "border-[color-mix(in_oklch,var(--warning)_45%,transparent)] bg-[color-mix(in_oklch,var(--warning)_14%,transparent)] text-warning-foreground",
        danger: "border-[color-mix(in_oklch,var(--destructive)_45%,transparent)] bg-[color-mix(in_oklch,var(--destructive)_14%,transparent)] text-destructive-foreground",
    };
    return <span className={cn("inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 font-medium font-sans text-[11.5px] leading-none", tones[tone])}>{children}</span>;
}

export function FlagRow({ label, on }: { label: string; on: boolean }) {
    return (
        <div className="flex items-center justify-between gap-2 border-border/40 border-b py-2 last:border-b-0">
            <span className="font-sans text-[12.5px] text-foreground">{label}</span>
            <span className={cn("inline-flex h-5 items-center rounded-full px-2 font-medium font-mono text-[10px] uppercase tracking-widest", on ? "bg-[color-mix(in_oklch,var(--success)_16%,transparent)] text-success-foreground" : "bg-[color-mix(in_oklch,var(--muted)_50%,transparent)] text-muted-foreground")}>
                {on ? "Yes" : "No"}
            </span>
        </div>
    );
}

export function Meta({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-md bg-[color-mix(in_oklch,var(--muted)_50%,transparent)] px-2.5 py-2">
            <div className="mb-1.25 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{label}</div>
            <div className="break-all font-medium font-mono text-[12px] text-foreground leading-tight">{value}</div>
        </div>
    );
}

interface ILegendSwatch {
    label: string;
    style: React.CSSProperties;
}

const LEGEND_SWATCHES: ILegendSwatch[] = [
    { label: "Start", style: { background: "rgba(231,45,80,0.18)", borderColor: "#e72d50" } },
    { label: "End", style: { background: "rgba(53,157,222,0.18)", borderColor: "#359dde" } },
    { label: "Ground", style: { background: "hsla(0,0%,71%,0.6)", borderColor: "hsla(0,0%,87%,0.9)" } },
    { label: "High Ground", style: { background: "hsla(0,0%,51%,0.95)", borderColor: "hsla(0,0%,78%,0.93)" } },
    { label: "Forbidden", style: { background: "hsla(0,0%,10%,0.6)", borderColor: "hsla(0,0%,87%,0.28)" } },
];

export function TileLegend() {
    return (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-[10px] border border-border bg-card px-3.5 py-2.5">
            <Kicker>Legend</Kicker>
            {LEGEND_SWATCHES.map((s) => (
                <span key={s.label} className="inline-flex items-center gap-1.5">
                    <span aria-hidden="true" className="h-3.5 w-3.5 rounded-xs border" style={s.style} />
                    <span className="font-medium font-sans text-[11.5px] text-muted-foreground leading-none">{s.label}</span>
                </span>
            ))}
        </div>
    );
}
