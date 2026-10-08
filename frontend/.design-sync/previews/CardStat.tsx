import { CardStat } from "frontend";
import type { ReactNode } from "react";

// One icon-plus-number stat in a tier-list browse card's footer. `value` arrives
// already compacted ("18.4K"); `title` carries the full count as a tooltip. The
// footer shows views and favourites, and adds the 24-hour trend, in the accent
// colour, only on a trending card.

/** The footer's meta line, as BrowseCard sets it: 11px muted sans, tabular numbers. */
const Meta = ({ children }: { children: ReactNode }) => (
    <div className="w-full max-w-xs p-4">
        <div className="flex items-center gap-2.5 font-sans text-[11px] text-muted-foreground tabular-nums leading-none">{children}</div>
    </div>
);

/** A trending card's footer: views, favourites and the 24h trend. */
export const TrendingFooter = () => (
    <Meta>
        <CardStat kind="views" value="18.4K" title="18,420 views" />
        <CardStat kind="favorites" value="1.3K" title="1,284 favorites" />
        <CardStat kind="views24h" value="642" title="642 views in the last 24 hours" />
    </Meta>
);

/** A quiet card's footer: views and favourites only. */
export const QuietFooter = () => (
    <Meta>
        <CardStat kind="views" value="931" title="931 views" />
        <CardStat kind="favorites" value="27" title="27 favorites" />
    </Meta>
);

/** The 24h trend alone: accent colour, bold, a chevron instead of an icon. */
export const Trend = () => (
    <Meta>
        <CardStat kind="views24h" value="2.1K" title="2,104 views in the last 24 hours" />
    </Meta>
);
