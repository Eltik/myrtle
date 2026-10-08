import { ResolutionBadge } from "frontend";

// The EN-date verdict on a CN release row. `today` is a prop, so the stories pin
// it to 2026-09-01 and the fixtures are live /release payload rows dated around it.
const TODAY = new Date("2026-09-01T12:00:00+09:00");

// EN has scheduled it: People, A People, Sep 17 - Oct 1, 2026.
export const Confirmed = () => (
    <div className="flex w-full max-w-md justify-end p-4">
        <ResolutionBadge resolution={{ status: "confirmed", enId: "act51side", enStart: 1789570800, enEnd: 1790765999 }} today={TODAY} />
    </div>
);

// Modelled from the CN-to-EN lag: Critical Phase Transition, best guess Oct 6
// with its p25-p75 window.
export const Estimated = () => (
    <div className="flex w-full max-w-md justify-end p-4">
        <ResolutionBadge resolution={{ status: "estimated", enStart: 1791239400, lo: 1791099000, hi: 1791603900 }} today={TODAY} />
    </div>
);

// Announced by a hand-entered source ahead of the data.
export const Announced = () => (
    <div className="flex w-full max-w-md justify-end p-4">
        <ResolutionBadge resolution={{ status: "override", enId: null, enStart: 1791457200, enEnd: 1794826799, source: "EN anniversary livestream", note: "Dates read off the stream schedule" }} today={TODAY} />
    </div>
);

// An estimate whose day has passed with EN still not running it: moved to today,
// the original guess kept in the label.
export const Overdue = () => (
    <div className="flex w-full max-w-md justify-end p-4">
        <ResolutionBadge resolution={{ status: "estimated", enStart: 1788188400, lo: 1788188400, hi: 1788552000, overdue: true, estimatedStart: 1787929200 } as never} today={TODAY} />
    </div>
);

// The three undated verdicts.
export const Undated = () => (
    <div className="flex w-full max-w-md flex-col items-end gap-3 p-4">
        <ResolutionBadge resolution={{ status: "independent" }} today={TODAY} />
        <ResolutionBadge resolution={{ status: "unlisted" }} today={TODAY} />
        <ResolutionBadge resolution={{ status: "unmodelled" }} today={TODAY} note="CN-exclusive collaboration" />
    </div>
);

// A standing (permanent) row, captioned with the event it arrives alongside.
export const StandingWithCaption = () => (
    <div className="flex w-full max-w-md justify-end p-4">
        <ResolutionBadge resolution={{ status: "confirmed", enId: "act3enemyduel", enStart: 1790175600, enEnd: 1791370799 }} today={TODAY} standing caption="with Duel Channel: Ivy Vine" />
    </div>
);
