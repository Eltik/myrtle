import { PreviewFallback } from "frontend";

/** The stage-group taxonomy and its accents, straight from `lib/registry/stage-groups`. */
const GROUPS = [
    { group: "story", label: "Main Story", tone: "var(--primary)" },
    { group: "events", label: "Events", tone: "#f7a452" },
    { group: "annihilation", label: "Annihilation", tone: "var(--destructive)" },
    { group: "is", label: "Integrated Strategies", tone: "#b78fe6" },
    { group: "ra", label: "Reclamation Algorithm", tone: "#5bbf86" },
    { group: "sss", label: "Stationary Security", tone: "#5a8fe8" },
    { group: "cc", label: "Contingency Contract", tone: "#e8a23c" },
    { group: "supplies", label: "Supplies", tone: "var(--muted-foreground)" },
];

export const GroupTones = () => (
    <div className="grid max-w-3xl grid-cols-2 gap-3 sm:grid-cols-4">
        {GROUPS.map((g) => (
            <div key={g.group} className="flex flex-col gap-1.5">
                <div className="relative aspect-16/10 w-full overflow-hidden rounded-lg border border-border">
                    <PreviewFallback tone={g.tone} group={g.group} />
                </div>
                <span className="font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.12em]">{g.label}</span>
            </div>
        ))}
    </div>
);

/** The 64×40 row thumbnail in the stage list, before any art resolves. */
export const RowThumbnail = () => (
    <div className="flex max-w-xl items-center gap-4 rounded-xl border border-border bg-card/40 px-4.5 py-3.5">
        <div className="relative h-10 w-16 flex-none overflow-hidden rounded-md border border-border">
            <PreviewFallback tone="var(--primary)" group="story" iconClassName="h-3.5 w-3.5" />
        </div>
        <div className="min-w-0 flex-1">
            <div className="truncate font-[650] font-sans text-[15px] text-foreground leading-[1.15] tracking-[-0.01em]">Burning Run</div>
            <div className="mt-0.5 font-mono text-[10px] text-muted-foreground tracking-[0.04em]">4-1 ~ SW-EV-4</div>
        </div>
        <span className="whitespace-nowrap font-mono text-[10.5px] text-muted-foreground">24 ops · 2 boss</span>
    </div>
);

/** The featured hero uses the same wash at 8× glyph size. */
export const HeroBanner = () => (
    <div className="relative h-60 max-w-3xl overflow-hidden rounded-2xl border border-border bg-card">
        <PreviewFallback tone="#f7a452" group="events" iconClassName="h-8 w-8" />
        <div className="absolute inset-0 flex flex-col justify-end p-6">
            <div className="text-balance font-bold font-sans text-[34px] text-foreground leading-[1.02] tracking-[-0.03em]">Vector Breakthrough #2</div>
            <div className="mt-3 font-mono text-[11px] text-muted-foreground">VEC-01 ~ VEC-SP16 · 32 operations</div>
        </div>
    </div>
);
