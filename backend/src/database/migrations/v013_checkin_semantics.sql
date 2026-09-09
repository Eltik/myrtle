-- Document what `user_checkin.history` actually holds. The column is the game's
-- raw `checkIn.checkInHistory`, and its name has twice been read as a dated
-- calendar (one entry per day of the month, 1 = claimed). It is not that:
--
--   * one entry per sign-in CLAIMED this month, in claim order - a missed day
--     adds no entry at all, so the length is the claim count and there is no
--     mapping from an entry back to a calendar date;
--   * the value is 1 when the monthly-subscription Daily Supply came with that
--     claim, 0 when it did not. Confirmed against user_status.monthly_sub_end
--     across the whole table: the final flag agrees with subscription state in
--     2193 of 2219 non-empty rows.
--
-- Counting the 1s therefore counts monthly-card days, not sign-ins. Comments
-- only - no data or shape change.

COMMENT ON COLUMN user_checkin.history IS
    'Raw checkIn.checkInHistory: one flag per sign-in CLAIMED this month, in claim order; 1 = monthly-subscription Daily Supply granted with that claim. NOT a dated calendar - length is the claim count, and no entry maps to a calendar day.';

COMMENT ON COLUMN user_checkin.reward_index IS
    'Raw checkIn.checkInRewardIndex: 0-based pointer into the month''s reward slots. Equals cardinality(history) while a claim is pending, one less just after a claim. Prefer cardinality(history) for the claim count.';

COMMENT ON COLUMN user_checkin.cumulative_signin IS
    'Lifetime cumulative sign-in days (checkIn.showCount), independent of the current month.';
