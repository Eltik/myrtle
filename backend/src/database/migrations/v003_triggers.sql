-- Auto-update updated_at on any UPDATE
CREATE OR REPLACE FUNCTION fn_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_timestamp BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();
CREATE TRIGGER trg_user_settings_timestamp BEFORE UPDATE ON user_settings FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();
CREATE TRIGGER trg_tier_lists_timestamp BEFORE UPDATE ON tier_lists FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();
CREATE TRIGGER trg_tier_placements_timestamp BEFORE UPDATE ON tier_placements FOR EACH ROW EXECUTE FUNCTION fn_update_timestamp();

-- Generic audit trigger
CREATE OR REPLACE FUNCTION fn_audit_log()
RETURNS TRIGGER AS $$
DECLARE
    record_pk TEXT;
    actor UUID;
BEGIN
    -- Try to get actor from session variable (set by application)
    BEGIN
        actor := current_setting("app.current_user_id", true)::UUID;
    EXCEPTION WHEN OTHERS THEN
        actor := NULL;
    END;

    -- Extract PK (first column)
    IF TG_OP = "DELETE" THEN
        record_pk := OLD.id::TEXT;
        INSERT INTO audit_log (table_name, record_id, action, old_data, changed_by)
        VALUES (TG_TABLE_NAME, record_pk, "DELETE", to_jsonb(OLD), actor);
        RETURN OLD;
    ELSIF TG_OP = "UPDATE" THEN
        record_pk := NEW.id::TEXT;
        INSERT INTO audit_log (table_name, record_id, action, old_data, new_data, changed_by)
        VALUES (TG_TABLE_NAME, record_pk, "UPDATE", to_jsonb(OLD), to_jsonb(NEW), actor);
        RETURN NEW;
    ELSIF TG_OP = "INSERT" THEN
        record_pk := NEW.id::TEXT;
        INSERT INTO audit_log (table_name, record_id, action, new_data, changed_by)
        VALUES (TG_TABLE_NAME, record_pk, "INSERT", to_jsonb(NEW), actor);
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Attach to sensitive tables
CREATE TRIGGER trg_audit_users AFTER INSERT OR UPDATE OR DELETE ON users FOR EACH ROW EXECUTE FUNCTION fn_audit_log();
CREATE TRIGGER trg_audit_tier_lists AFTER INSERT OR UPDATE OR DELETE ON tier_lists FOR EACH ROW EXECUTE FUNCTION fn_audit_log();
CREATE TRIGGER trg_audit_tier_placements AFTER INSERT OR UPDATE OR DELETE ON tier_placements FOR EACH ROW EXECUTE FUNCTION fn_audit_log();
CREATE TRIGGER trg_audit_scores AFTER INSERT OR UPDATE OR DELETE ON user_scores FOR EACH ROW EXECUTE FUNCTION fn_audit_log();
