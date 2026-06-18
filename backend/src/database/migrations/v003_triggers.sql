-- ============================================================================
-- BASELINE SCHEMA — TRIGGER FUNCTIONS & TRIGGERS
--
-- Generated from `pg_dump --schema-only` of the migrated v001..v022 schema,
-- split by object type. Runs only on a fresh database (existing deployments
-- already have these objects recorded under the original migration names).
-- Generated artifact — do not hand-edit; add new changes as later migrations.
-- ============================================================================

--
-- Name: fn_audit_log(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_audit_log() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    record_pk TEXT;
    actor UUID;
BEGIN
    BEGIN
        actor := current_setting('app.current_user_id', true)::UUID;
    EXCEPTION WHEN OTHERS THEN
        actor := NULL;
    END;

    -- Extract PK: try 'id' first, fall back to 'user_id'
    IF TG_OP = 'DELETE' THEN
        record_pk := COALESCE(
            (to_jsonb(OLD)->>'id'),
            (to_jsonb(OLD)->>'user_id'),
            'unknown'
        );
        INSERT INTO audit_log (table_name, record_id, action, old_data, changed_by)
        VALUES (TG_TABLE_NAME, record_pk, 'DELETE', to_jsonb(OLD), actor);
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        record_pk := COALESCE(
            (to_jsonb(NEW)->>'id'),
            (to_jsonb(NEW)->>'user_id'),
            'unknown'
        );
        INSERT INTO audit_log (table_name, record_id, action, old_data, new_data, changed_by)
        VALUES (TG_TABLE_NAME, record_pk, 'UPDATE', to_jsonb(OLD), to_jsonb(NEW), actor);
        RETURN NEW;
    ELSIF TG_OP = 'INSERT' THEN
        record_pk := COALESCE(
            (to_jsonb(NEW)->>'id'),
            (to_jsonb(NEW)->>'user_id'),
            'unknown'
        );
        INSERT INTO audit_log (table_name, record_id, action, new_data, changed_by)
        VALUES (TG_TABLE_NAME, record_pk, 'INSERT', to_jsonb(NEW), actor);
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: fn_bump_tier_list_from_placement(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_bump_tier_list_from_placement() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    target_id UUID;
    lookup_tier UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        lookup_tier := OLD.tier_id;
    ELSE
        lookup_tier := NEW.tier_id;
    END IF;

    SELECT tier_list_id INTO target_id FROM tiers WHERE id = lookup_tier;

    IF target_id IS NOT NULL THEN
        UPDATE tier_lists SET updated_at = NOW() WHERE id = target_id;
    END IF;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: fn_bump_tier_list_from_tier(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_bump_tier_list_from_tier() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    target_id UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        target_id := OLD.tier_list_id;
    ELSE
        target_id := NEW.tier_list_id;
    END IF;

    UPDATE tier_lists SET updated_at = NOW() WHERE id = target_id;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: fn_bump_tier_list_from_version(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_bump_tier_list_from_version() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE tier_lists SET updated_at = NOW() WHERE id = NEW.tier_list_id;
    RETURN NEW;
END;
$$;


--
-- Name: fn_update_timestamp(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_update_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at THEN
        NEW.updated_at = NOW();
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: user_scores trg_audit_scores; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_audit_scores AFTER INSERT OR DELETE OR UPDATE ON public.user_scores FOR EACH ROW EXECUTE FUNCTION public.fn_audit_log();


--
-- Name: tier_lists trg_audit_tier_lists; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_audit_tier_lists AFTER INSERT OR DELETE OR UPDATE ON public.tier_lists FOR EACH ROW EXECUTE FUNCTION public.fn_audit_log();


--
-- Name: tier_placements trg_audit_tier_placements; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_audit_tier_placements AFTER INSERT OR DELETE OR UPDATE ON public.tier_placements FOR EACH ROW EXECUTE FUNCTION public.fn_audit_log();


--
-- Name: users trg_audit_users; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_audit_users AFTER INSERT OR DELETE OR UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.fn_audit_log();


--
-- Name: operator_plans trg_operator_plans_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_operator_plans_timestamp BEFORE UPDATE ON public.operator_plans FOR EACH ROW EXECUTE FUNCTION public.fn_update_timestamp();


--
-- Name: plan_groups trg_plan_groups_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_plan_groups_timestamp BEFORE UPDATE ON public.plan_groups FOR EACH ROW EXECUTE FUNCTION public.fn_update_timestamp();


--
-- Name: tier_list_versions trg_tier_list_versions_bump_tier_list; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_tier_list_versions_bump_tier_list AFTER INSERT ON public.tier_list_versions FOR EACH ROW EXECUTE FUNCTION public.fn_bump_tier_list_from_version();


--
-- Name: tier_lists trg_tier_lists_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_tier_lists_timestamp BEFORE UPDATE ON public.tier_lists FOR EACH ROW EXECUTE FUNCTION public.fn_update_timestamp();


--
-- Name: tier_placements trg_tier_placements_bump_tier_list; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_tier_placements_bump_tier_list AFTER INSERT OR DELETE OR UPDATE ON public.tier_placements FOR EACH ROW EXECUTE FUNCTION public.fn_bump_tier_list_from_placement();


--
-- Name: tier_placements trg_tier_placements_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_tier_placements_timestamp BEFORE UPDATE ON public.tier_placements FOR EACH ROW EXECUTE FUNCTION public.fn_update_timestamp();


--
-- Name: tiers trg_tiers_bump_tier_list; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_tiers_bump_tier_list AFTER INSERT OR DELETE OR UPDATE ON public.tiers FOR EACH ROW EXECUTE FUNCTION public.fn_bump_tier_list_from_tier();


--
-- Name: user_settings trg_user_settings_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_user_settings_timestamp BEFORE UPDATE ON public.user_settings FOR EACH ROW EXECUTE FUNCTION public.fn_update_timestamp();


--
-- Name: users trg_users_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_users_timestamp BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.fn_update_timestamp();
