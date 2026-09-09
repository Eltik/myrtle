import { SignInCalendarCard } from "frontend";

const NOW = Math.floor(Date.now() / 1000);
// EN game days roll over at 04:00 UTC-7, so the month's elapsed-day count comes
// from the current game day, not the viewer's local date.
const EN_OFFSET_SEC = (-7 - 4) * 3600;
const GAME_DAY = new Date((NOW + EN_OFFSET_SEC) * 1000).getUTCDate();

/**
 * `monthly_card_flags` has one entry per CLAIM (not per day): 1 when the
 * monthly-subscription Daily Supply came with it. `cardDays` is how many of
 * the leading claims had a card.
 */
const cardFlags = (claims: number, cardDays: number): number[] => Array.from({ length: Math.max(0, claims) }, (_, i) => (i < cardDays ? 1 : 0));

export const PerfectMonth = () => (
    <div className="w-full max-w-md">
        <SignInCalendarCard
            checkin={{
                monthly_card_flags: cardFlags(GAME_DAY, GAME_DAY),
                claimed_this_month: GAME_DAY,
                cumulative_signin: 2461,
                checkin_group_id: "signin12",
                reward_index: GAME_DAY - 1,
                can_check_in: false,
                register_ts: Math.floor(new Date("2019-05-01T12:00:00Z").getTime() / 1000),
                last_online_ts: NOW - 4 * 3600,
                updated_at: new Date((NOW - 3 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

/** Caught up through yesterday, today's slot still waiting. */
export const ClaimReady = () => (
    <div className="w-full max-w-md">
        <SignInCalendarCard
            checkin={{
                monthly_card_flags: cardFlags(GAME_DAY - 1, 3),
                claimed_this_month: Math.max(0, GAME_DAY - 1),
                cumulative_signin: 1187,
                checkin_group_id: "signin12",
                reward_index: Math.max(0, GAME_DAY - 1),
                can_check_in: true,
                register_ts: Math.floor(new Date("2022-08-11T04:20:00Z").getTime() / 1000),
                last_online_ts: NOW - 20 * 3600,
                updated_at: new Date((NOW - 18 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);

/** Well behind: open slots trail the claimed run, no monthly card. */
export const FallenBehind = () => (
    <div className="w-full max-w-md">
        <SignInCalendarCard
            checkin={{
                monthly_card_flags: cardFlags(Math.max(0, GAME_DAY - 13), 0),
                claimed_this_month: Math.max(0, GAME_DAY - 13),
                cumulative_signin: 913,
                checkin_group_id: "signin12",
                reward_index: Math.max(0, GAME_DAY - 13),
                can_check_in: true,
                register_ts: Math.floor(new Date("2021-01-16T09:00:00Z").getTime() / 1000),
                last_online_ts: NOW - 30 * 3600,
                updated_at: new Date((NOW - 26 * 3600) * 1000).toISOString(),
            }}
            server="en"
        />
    </div>
);
