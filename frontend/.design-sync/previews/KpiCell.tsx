import { KpiCell } from "frontend";

// KpiCell draws its own right/bottom dividers, so its real frame is the rounded
// strip that KpiStrip renders around it. Every story reuses that frame.
const Strip = ({ children }: { children: React.ReactNode }) => <div className="grid grid-cols-2 overflow-hidden rounded-[14px] border border-border bg-card">{children}</div>;

export const Featured = () => (
    <Strip>
        <KpiCell featured label="Operators indexed" value="438" meta={<span>1,604 skills · 857 modules</span>} />
        <KpiCell label="Enemies" value="1,590" meta={<span>across 433 zones</span>} />
    </Strip>
);

export const Standard = () => (
    <Strip>
        <KpiCell label="Versions saved" value="34" meta={<span>avg 0.4 per list</span>} />
        <KpiCell label="Placements" value="6.0K" meta={<span>avg 75 per list</span>} />
    </Strip>
);

export const WithoutMeta = () => (
    <Strip>
        <KpiCell label="Total lists" value="80" />
        <KpiCell label="Active" value="79" />
    </Strip>
);

export const Loading = () => (
    <Strip>
        <KpiCell featured label="Operators indexed" value={<span className="text-muted-foreground/60">-</span>} meta={<span>-</span>} />
        <KpiCell label="Rosters synced" value={<span className="text-muted-foreground/60">-</span>} meta={<span>-</span>} />
    </Strip>
);
