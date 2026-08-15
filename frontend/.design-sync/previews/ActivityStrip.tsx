import { ActivityStrip } from "frontend";

// One bar per day across the selected range, scaled to the busiest day. It only
// reads `date` off each commit, so the fixtures below are day-stamped shells.
const ANCHOR = Date.UTC(2024, 4, 15, 12, 0, 0);
const DAY_MS = 86_400_000;

function iso(daysAgo: number): string {
    return new Date(ANCHOR - daysAgo * DAY_MS).toISOString();
}

/** Expand a `daysAgo -> commits that day` map into the flat commit list. */
function commitsFrom(perDay: Record<number, number>) {
    const out: { date: string }[] = [];
    for (const [day, n] of Object.entries(perDay)) {
        for (let i = 0; i < n; i++) out.push({ date: iso(Number(day)) });
    }
    return out;
}

// A month of real-shaped activity: a couple of heavy landing days, a long
// weekend of nothing, steady weekday commits either side.
const MONTH = commitsFrom({ 0: 4, 1: 7, 2: 3, 3: 1, 5: 9, 6: 5, 7: 2, 9: 1, 10: 6, 11: 12, 12: 8, 13: 3, 16: 2, 17: 5, 18: 4, 19: 1, 21: 3, 22: 7, 23: 6, 24: 2, 26: 1, 27: 4, 28: 9, 29: 5 });

const QUARTER = commitsFrom({ ...{ 0: 4, 1: 7, 2: 3, 5: 9, 6: 5, 9: 1, 11: 12, 12: 8, 16: 2, 18: 4, 22: 7, 23: 6, 27: 4, 28: 9 }, ...{ 33: 5, 35: 11, 36: 2, 41: 6, 44: 3, 47: 8, 50: 4, 52: 1, 55: 7, 58: 2, 61: 5, 64: 9, 68: 3, 71: 6, 74: 2, 78: 4, 82: 7, 86: 1, 88: 3 } });

const WEEK = commitsFrom({ 0: 2, 1: 5, 3: 1, 4: 3, 6: 2 });

export const LastThirtyDays = () => (
    <div className="w-full max-w-2xl rounded-xl border border-border bg-card px-5 py-4">
        <ActivityStrip commits={MONTH} days={30} />
    </div>
);

export const LastNinetyDays = () => (
    <div className="w-full max-w-2xl rounded-xl border border-border bg-card px-5 py-4">
        <ActivityStrip commits={QUARTER} days={90} />
    </div>
);

export const ThisWeek = () => (
    <div className="w-full max-w-2xl rounded-xl border border-border bg-card px-5 py-4">
        <ActivityStrip commits={WEEK} days={7} />
    </div>
);

export const NoCommitsInRange = () => (
    <div className="w-full max-w-2xl rounded-xl border border-border bg-card px-5 py-4">
        <ActivityStrip commits={[]} days={30} />
    </div>
);
