import { StatsKpiStrip } from "frontend";

const DASH = <span className="text-muted-foreground/60">-</span>;

export const SiteHeadline = () => (
    <StatsKpiStrip
        cells={[
            { featured: true, label: "Operators indexed", value: "438", meta: <span>1,604 skills · 857 modules</span> },
            { label: "Active tier lists", value: "79", meta: <span>80 total · 34 versions</span> },
            { label: "Rosters synced", value: "1.5K", meta: <span>Yostar-linked doctors</span> },
        ]}
    />
);

export const TierListTotals = () => (
    <StatsKpiStrip
        columnsClass="grid-cols-[repeat(4,1fr)]"
        cells={[
            { label: "Total lists", value: "80", meta: <span>1 archived</span> },
            { label: "Active", value: "79", meta: <span>99% of total</span> },
            { label: "Versions saved", value: "34", meta: <span>avg 0.4 per list</span> },
            { label: "Placements", value: "6.0K", meta: <span>avg 75 per list</span> },
        ]}
    />
);

export const Loading = () => (
    <StatsKpiStrip
        cells={[
            { featured: true, label: "Operators indexed", value: DASH, meta: <span>-</span> },
            { label: "Active tier lists", value: DASH, meta: <span>-</span> },
            { label: "Rosters synced", value: DASH, meta: <span>-</span> },
        ]}
    />
);
