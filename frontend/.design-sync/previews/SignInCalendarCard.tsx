import { SignInCalendarCard } from "frontend";

const NOW = Math.floor(Date.now() / 1000);
// EN game days roll over at 04:00 UTC-7, so the calendar's `history` array is
// sized against the current game day, not the viewer's local date.
const EN_OFFSET_SEC = (-7 - 4) * 3600;
const GAME_DAY = new Date((NOW + EN_OFFSET_SEC) * 1000).getUTCDate();

const monthHistory = (missedDays: number[], pendingToday = false): number[] => Array.from({ length: GAME_DAY }, (_, i) => (missedDays.includes(i + 1) || (pendingToday && i + 1 === GAME_DAY) ? 0 : 1));

export const PerfectMonth = () => (
    <div className="w-full max-w-md">
        <SignInCalendarCard
            checkin={{
                history: monthHistory([]),
                cumulative_signin: 2461,
                checkin_group_id: "signin12",
                reward_index: GAME_DAY,
                can_check_in: false,
                register_ts: Math.floor(new Date("2019-05-01T12:00:00Z").getTime() / 1000),
                last_online_ts: NOW - 4 * 3600,
                updated_at: new Date((NOW - 3 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

export const ClaimReady = () => (
    <div className="w-full max-w-md">
        <SignInCalendarCard
            checkin={{
                history: monthHistory([2, 3, 9], true),
                cumulative_signin: 1187,
                checkin_group_id: "signin12",
                reward_index: GAME_DAY - 1,
                can_check_in: true,
                register_ts: Math.floor(new Date("2022-08-11T04:20:00Z").getTime() / 1000),
                last_online_ts: NOW - 20 * 3600,
                updated_at: new Date((NOW - 18 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

export const PatchyMonth = () => (
    <div className="w-full max-w-md">
        <SignInCalendarCard
            checkin={{
                history: monthHistory([1, 2, 5, 6, 7, 10, 13, 14, 17, 18, 21, 24, 25]),
                cumulative_signin: 913,
                checkin_group_id: "signin12",
                reward_index: 6,
                can_check_in: true,
                register_ts: Math.floor(new Date("2021-01-16T09:00:00Z").getTime() / 1000),
                last_online_ts: NOW - 30 * 3600,
                updated_at: new Date((NOW - 26 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);
