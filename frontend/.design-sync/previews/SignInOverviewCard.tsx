import { SignInOverviewCard } from "frontend";

const DAY = 86_400;
const NOW = Math.floor(Date.now() / 1000);
// EN game days roll over at 04:00 UTC-7, so the month's elapsed-day count comes
// from the current game day, not the viewer's local date.
const EN_OFFSET_SEC = (-7 - 4) * 3600;
const GAME_DAY = new Date((NOW + EN_OFFSET_SEC) * 1000).getUTCDate();

/**
 * `monthly_card_flags` has one entry per CLAIM (not per day): 1 when the
 * monthly-subscription Daily Supply came with it.
 */
const cardFlags = (claims: number, cardDays: number): number[] => Array.from({ length: Math.max(0, claims) }, (_, i) => (i < cardDays ? 1 : 0));

export const Veteran = () => (
    <div className="w-full max-w-md">
        <SignInOverviewCard
            checkin={{
                monthly_card_flags: cardFlags(GAME_DAY - 2, GAME_DAY - 2),
                claimed_this_month: Math.max(0, GAME_DAY - 2),
                cumulative_signin: 1798,
                checkin_group_id: "signin12",
                reward_index: Math.max(0, GAME_DAY - 2),
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
                monthly_card_flags: cardFlags(GAME_DAY - 1, 0),
                claimed_this_month: Math.max(0, GAME_DAY - 1),
                cumulative_signin: 38,
                checkin_group_id: "signin12",
                reward_index: Math.max(0, GAME_DAY - 1),
                can_check_in: true,
                register_ts: NOW - 41 * DAY,
                last_online_ts: NOW - 2 * 3600,
                updated_at: new Date((NOW - 90 * 60) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

/** Stopped playing three months ago - the snapshot's month is not this one. */
export const LapsedAccount = () => (
    <div className="w-full max-w-md">
        <SignInOverviewCard
            checkin={{
                monthly_card_flags: cardFlags(4, 0),
                claimed_this_month: 4,
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
