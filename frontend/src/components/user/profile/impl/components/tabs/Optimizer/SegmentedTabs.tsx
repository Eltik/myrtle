import { cn } from "#/lib/utils";

export interface ISegment {
    id: string;
    label: string;
    hint?: string;
    disabled?: boolean;
}

export function SegmentedTabs({ segments, active, onChange, label }: { segments: ISegment[]; active: string; onChange: (id: string) => void; label: string }) {
    return (
        <div aria-label={label} className="inline-flex gap-0.5 rounded-lg border border-border bg-card p-0.5" role="tablist">
            {segments.map((segment) => (
                <button
                    aria-selected={active === segment.id}
                    className={cn("rounded-md px-3 py-1.5 font-medium text-[13px] transition-colors", active === segment.id ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground", segment.disabled && "pointer-events-none opacity-40")}
                    disabled={segment.disabled}
                    key={segment.id}
                    onClick={() => onChange(segment.id)}
                    role="tab"
                    type="button"
                >
                    {segment.label}
                    {segment.hint && <span className="ml-1.5 font-mono text-[11px] tabular-nums opacity-70">{segment.hint}</span>}
                </button>
            ))}
        </div>
    );
}
