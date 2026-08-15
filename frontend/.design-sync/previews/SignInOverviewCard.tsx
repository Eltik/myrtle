import { SignInOverviewCard } from "frontend";

const DAY = 86_400;
const NOW = Math.floor(Date.now() / 1000);
// EN game days roll over at 04:00 UTC-7, so the current game day is what the
// calendar's `history` array is sized against.
const EN_OFFSET_SEC = (-7 - 4) * 3600;
const GAME_DAY = new Date((NOW + EN_OFFSET_SEC) * 1000).getUTCDate();

/** `history[i] = 1` when day i+1 of the current month was claimed. */
const monthHistory = (missedDays: number[], pendingToday = true): number[] => Array.from({ length: GAME_DAY }, (_, i) => (missedDays.includes(i + 1) || (pendingToday && i + 1 === GAME_DAY) ? 0 : 1));

export const Veteran = () => (
    <div className="w-full max-w-md">
        <SignInOverviewCard
            checkin={{
                history: monthHistory([4, 11]),
                cumulative_signin: 1798,
                checkin_group_id: "signin12",
                reward_index: GAME_DAY - 3,
                can_check_in: true,
                register_ts: Math.floor(new Date("2019-05-01T12:00:00Z").getTime() / 1000),
                last_online_ts: NOW - 6 * 3600,
                updated_at: new Date((NOW - 5 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

export const NewDoctor = () => (
    <div className="w-full max-w-md">
        <SignInOverviewCard
            checkin={{
                history: monthHistory([2]),
                cumulative_signin: 38,
                checkin_group_id: "signin12",
                reward_index: GAME_DAY - 2,
                can_check_in: true,
                register_ts: NOW - 41 * DAY,
                last_online_ts: NOW - 2 * 3600,
                updated_at: new Date((NOW - 90 * 60) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

export const LapsedAccount = () => (
    <div className="w-full max-w-md">
        <SignInOverviewCard
            checkin={{
                history: Array.from({ length: GAME_DAY }, () => 0),
                cumulative_signin: 913,
                checkin_group_id: "signin12",
                reward_index: 4,
                can_check_in: false,
                register_ts: Math.floor(new Date("2021-01-16T09:00:00Z").getTime() / 1000),
                last_online_ts: NOW - 96 * DAY,
                updated_at: new Date((NOW - 95 * DAY) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);
