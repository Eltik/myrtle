--
-- Content audit log: who did what to a tier list or a grid, one readable row
-- per action, for the admin Edit history.
--
-- `audit_log` (v001/v003) already has row triggers on `tier_lists` and
-- `tier_placements`, but it holds whole rows with no actor (nothing sets
-- `app.current_user_id`) and one row per placement write, so an editor's
-- autosave reads as hundreds of anonymous rows. This table is written by the
-- route handlers instead (`services::content_audit`), which know the caller.
--
-- `action` is one of `create`, `fork`, `edit`, `publish`, `visibility`,
-- `flair`, `grant`, `revoke`, `delete`; `target_kind` is `tier_list` or
-- `grid`. Both are closed Rust enums, not CHECKs, so a new kind needs no
-- migration (the v029 tier entity precedent).
--
-- Coalescing: an `edit` folds into the actor's latest row on the same target
-- when that row is also an `edit` and was touched less than 15 minutes ago.
-- `details` then accumulates (tiers renamed, entries moved, ...),
-- `event_count` counts the requests folded in, `first_at` keeps the first and
-- `changed_at` the last. Any other action on the target ends the run, so an
-- edit after a publish starts a new row.
--
-- `target_id` has no foreign key: the row outlives a deleted list or grid,
-- and `target_name` / `target_slug` are its name and slug at the time.
-- `owner_id` is the target's owner when the row was written, so an action on
-- someone else's content (a moderator's) reads as such. `subject_id` is the
-- account a `grant` / `revoke` was about.
--
-- Account deletion: rows the account wrote go with it (`actor_id` CASCADE,
-- as `ui_message_audit_log` and `operator_notes_audit_log` do); rows that only
-- name it as owner or subject keep the action and lose the name (SET NULL).
--
CREATE TABLE content_audit_log (
    id bigserial PRIMARY KEY,
    actor_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_kind varchar(20) NOT NULL,
    target_id uuid NOT NULL,
    target_name text NOT NULL,
    target_slug text,
    owner_id uuid REFERENCES users(id) ON DELETE SET NULL,
    subject_id uuid REFERENCES users(id) ON DELETE SET NULL,
    action varchar(20) NOT NULL,
    details jsonb DEFAULT '{}'::jsonb NOT NULL,
    event_count integer DEFAULT 1 NOT NULL,
    first_at timestamp with time zone DEFAULT now() NOT NULL,
    changed_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_content_audit_changed_at ON content_audit_log (changed_at DESC);
CREATE INDEX idx_content_audit_actor ON content_audit_log (actor_id, changed_at DESC);
CREATE INDEX idx_content_audit_target ON content_audit_log (target_kind, target_id, changed_at DESC);
