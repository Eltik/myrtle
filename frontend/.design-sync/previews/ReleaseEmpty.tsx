import { ReleaseEmpty } from "frontend";

// The release planner's empty state: a card with the search-miss icon.
export const NoMatches = () => (
    <div className="w-full max-w-xl p-4">
        <ReleaseEmpty title="Nothing matches these filters" description="Turn a kind back on, or switch off “Stage events only”, to see more rows." />
    </div>
);

export const NothingUpcoming = () => (
    <div className="w-full max-w-xl p-4">
        <ReleaseEmpty title="No upcoming banners" description="Every CN banner in the window has already run on EN." />
    </div>
);
