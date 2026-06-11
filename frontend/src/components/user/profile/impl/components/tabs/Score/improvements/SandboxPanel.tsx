import type { IImprovementsResponse, ISandboxCategory } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { EmptyHint, PANEL_PADDING, Pill, SectionHeader, TEXT_BADGE } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

export function SandboxPanel({ improvements, accent }: IProps) {
    const s = improvements.sandbox;
    const hasAnyData = s.total > 0 || s.categories.some((c) => c.parts.some((p) => p.current > 0));

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-3`}>
            <SectionHeader title="Reclamation Algorithm" accent={accent} />
            {!hasAnyData && <EmptyHint>No RA progress detected. Start RA in Operation Originium Dust to begin tracking.</EmptyHint>}
            <div className="grid gap-x-5 gap-y-3 sm:grid-cols-2">
                {s.categories.map((category) => (
                    <CategoryRow key={category.key} category={category} accent={accent} />
                ))}
            </div>
        </div>
    );
}

function CategoryRow({ category, accent }: { category: ISandboxCategory; accent: string }) {
    const pct = Math.min(100, Math.max(0, category.score * 100));
    const weightPct = Math.round(category.weight * 100);
    return (
        <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between gap-2">
                <span className="flex items-center gap-1.5">
                    <span className="text-[11px] text-foreground/85">{category.label}</span>
                    <Pill>{weightPct}% of RA</Pill>
                </span>
                <span className={cn(TEXT_BADGE, "text-foreground/85")}>{pct.toFixed(0)}%</span>
            </div>
            <div className="h-1 w-full overflow-hidden rounded-full bg-muted/40">
                <div
                    className="h-full rounded-full transition-[width] duration-500 ease-out"
                    style={{
                        width: `${pct}%`,
                        background: `linear-gradient(to right, color-mix(in oklch, ${accent} 55%, transparent), ${accent})`,
                    }}
                />
            </div>
            <div className="flex flex-wrap gap-x-3 gap-y-0.5">
                {category.parts.map((p) => (
                    <span key={p.label} className="text-[10px] text-muted-foreground">
                        {p.label}{" "}
                        <span className="text-foreground/70 tabular-nums">
                            {p.current.toLocaleString()}/{p.max.toLocaleString()}
                        </span>
                    </span>
                ))}
            </div>
        </div>
    );
}
