-- ============================================================================
-- BASELINE SCHEMA — INDEXES
--
-- Generated from `pg_dump --schema-only` of the migrated v001..v022 schema,
-- split by object type. Runs only on a fresh database (existing deployments
-- already have these objects recorded under the original migration names).
-- Generated artifact — do not hand-edit; add new changes as later migrations.
-- ============================================================================

--
-- Name: idx_audit_table_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_table_time ON public.audit_log USING btree (table_name, changed_at DESC);


--
-- Name: idx_gacha_user_rarity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_gacha_user_rarity ON public.gacha_records USING btree (user_id, rarity);


--
-- Name: idx_gacha_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_gacha_user_time ON public.gacha_records USING btree (user_id, pull_timestamp DESC);


--
-- Name: idx_lb_entries_snapshot_server; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lb_entries_snapshot_server ON public.leaderboard_snapshot_entries USING btree (snapshot_id, server_id);


--
-- Name: idx_lb_entries_user_snapshot; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lb_entries_user_snapshot ON public.leaderboard_snapshot_entries USING btree (user_id, snapshot_id DESC);


--
-- Name: idx_lb_snapshots_taken_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_lb_snapshots_taken_at ON public.leaderboard_snapshots USING btree (taken_at DESC);


--
-- Name: idx_oos_operator; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oos_operator ON public.operator_ownership_stats USING btree (operator_id);


--
-- Name: idx_operator_notes_audit_note_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_operator_notes_audit_note_id ON public.operator_notes_audit_log USING btree (note_id);


--
-- Name: idx_scores_total; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scores_total ON public.user_scores USING btree (total_score DESC);


--
-- Name: idx_tier_lists_flair; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tier_lists_flair ON public.tier_lists USING btree (flair_id) WHERE (is_active = true);


--
-- Name: idx_tier_lists_is_listed; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tier_lists_is_listed ON public.tier_lists USING btree (is_listed) WHERE (is_active = true);


--
-- Name: idx_tier_lists_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tier_lists_slug ON public.tier_lists USING btree (slug);


--
-- Name: idx_tier_lists_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tier_lists_type ON public.tier_lists USING btree (list_type) WHERE (is_active = true);


--
-- Name: idx_tl_favorites_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tl_favorites_user ON public.tier_list_favorites USING btree (user_id);


--
-- Name: idx_tl_stats_trending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tl_stats_trending ON public.tier_list_stats USING btree (is_trending, trending_score DESC) WHERE (is_trending = true);


--
-- Name: idx_tl_stats_views; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tl_stats_views ON public.tier_list_stats USING btree (view_count DESC);


--
-- Name: idx_tl_view_events_dedupe_sess; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tl_view_events_dedupe_sess ON public.tier_list_view_events USING btree (tier_list_id, session_hash, viewed_at DESC) WHERE (session_hash IS NOT NULL);


--
-- Name: idx_tl_view_events_dedupe_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tl_view_events_dedupe_user ON public.tier_list_view_events USING btree (tier_list_id, user_id, viewed_at DESC) WHERE (user_id IS NOT NULL);


--
-- Name: idx_tl_view_events_list_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tl_view_events_list_time ON public.tier_list_view_events USING btree (tier_list_id, viewed_at DESC);


--
-- Name: idx_user_medals_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_medals_user ON public.user_medals USING btree (user_id);


--
-- Name: idx_user_ops_operator; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_ops_operator ON public.user_operators USING btree (operator_id);


--
-- Name: idx_user_skins_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_skins_user ON public.user_skins USING btree (user_id);


--
-- Name: idx_user_support_units_operator; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_support_units_operator ON public.user_support_units USING btree (operator_id);


--
-- Name: idx_users_nickname; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_nickname ON public.users USING gin (nickname public.gin_trgm_ops);


--
-- Name: idx_users_server; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_server ON public.users USING btree (server_id);
