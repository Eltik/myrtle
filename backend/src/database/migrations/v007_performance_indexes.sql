CREATE INDEX IF NOT EXISTS idx_tier_lists_active_listed_updated
    ON public.tier_lists USING btree (is_active, is_listed, updated_at DESC)
    WHERE is_active AND is_listed;

CREATE INDEX IF NOT EXISTS idx_tier_placements_tier_sub_order
    ON public.tier_placements USING btree (tier_id, sub_order);

CREATE INDEX IF NOT EXISTS idx_gacha_records_user_id
    ON public.gacha_records USING btree (user_id);
