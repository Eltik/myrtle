--
-- Account Lookback: how built an account was at points through its life.
--
-- `user_lookback_snapshots`: at most one row per account per period, written
-- by the roster sync (`services::lookback::record`) the first time it runs in
-- a period, never updated after. A period starts at the launch of the latest
-- event on the account's server (an `activity_table` entry with stages that is
-- not a one-time competitive mode, see `core::lookback::is_launch`), or at the
-- Monday 00:00 UTC of the current week once that launch is more than 28 days
-- old. `activity_id` / `activity_name` name the event, NULL for a week period;
-- the name is the server's game text at the time of the snapshot.
--
-- `summary` is the headline counts (`models::lookback::LookbackSummary`,
-- about 250 bytes, read by the timeline). `build` is the compact per-operator
-- build, one array per operator: `[id, elite, level, potential, skill_level,
-- [mastery per skill], [[module_id, stage]]]`, masteries empty when none,
-- locked modules left out. Measured on the local copy (2,630 accounts, 236
-- operators on average, 412 at most): 10,886 bytes of JSON text on average,
-- 26,557 at most, 24,578 bytes as jsonb before TOAST compresses it.
--
-- Prune: a row older than 365 days is kept only when it is the first of its
-- UTC calendar month for that account, so the first year is per event and
-- older years thin to one a month. Run by the same write, for that account.
--
-- Rows live with the account: ON DELETE CASCADE, like every user_ table.
--
CREATE TABLE user_lookback_snapshots (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_start timestamp with time zone NOT NULL,
    activity_id varchar(64),
    activity_name text,
    taken_at timestamp with time zone DEFAULT now() NOT NULL,
    summary jsonb NOT NULL,
    build jsonb NOT NULL,
    PRIMARY KEY (user_id, period_start)
);
