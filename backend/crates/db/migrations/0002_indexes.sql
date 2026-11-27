CREATE INDEX idx_plan_group_members_plan ON plan_group_members (operator_plan_id);

CREATE INDEX idx_tier_lists_created_by ON tier_lists (created_by);

CREATE INDEX idx_tl_permissions_user ON tier_list_permissions (user_id);

CREATE INDEX idx_tl_permissions_granted_by ON tier_list_permissions (granted_by)
    WHERE granted_by IS NOT NULL;

CREATE INDEX idx_tl_versions_published_by ON tier_list_versions (published_by)
    WHERE published_by IS NOT NULL;

CREATE INDEX idx_tl_view_events_user ON tier_list_view_events (user_id)
    WHERE user_id IS NOT NULL;

CREATE INDEX idx_tl_view_events_viewed_at ON tier_list_view_events (viewed_at);

CREATE INDEX idx_audit_changed_by ON audit_log (changed_by)
    WHERE changed_by IS NOT NULL;

CREATE INDEX idx_audit_table_record ON audit_log (table_name, record_id);

CREATE INDEX idx_audit_placement_tier ON audit_log ((COALESCE(new_data, old_data) ->> 'tier_id'))
    WHERE table_name = 'tier_placements';

CREATE INDEX idx_notes_audit_changed_by_at ON operator_notes_audit_log (changed_by, changed_at DESC);

CREATE INDEX idx_notes_audit_changed_at ON operator_notes_audit_log (changed_at DESC);

CREATE INDEX idx_ui_message_audit_changed_by ON ui_message_audit_log (changed_by);

CREATE INDEX idx_scores_calculated_at ON user_scores (calculated_at DESC);

CREATE INDEX idx_content_audit_owner ON content_audit_log (owner_id)
    WHERE owner_id IS NOT NULL;

CREATE INDEX idx_content_audit_subject ON content_audit_log (subject_id)
    WHERE subject_id IS NOT NULL;

CREATE INDEX idx_grids_title_trgm ON grids USING gin (title gin_trgm_ops);

CREATE INDEX idx_users_created_at ON users (created_at DESC);

CREATE INDEX idx_users_uid_trgm ON users USING gin (uid gin_trgm_ops);

CREATE INDEX idx_users_staff ON users (role) WHERE role <> 'user';

CREATE INDEX idx_users_admin_order ON users (
    (CASE role
        WHEN 'super_admin' THEN 0
        WHEN 'tier_list_admin' THEN 1
        WHEN 'tier_list_editor' THEN 2
        WHEN 'translator' THEN 3
        ELSE 4
    END),
    lower(nickname),
    id
);

DROP INDEX idx_tier_lists_slug;
