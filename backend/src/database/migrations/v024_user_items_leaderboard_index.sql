-- The inventory leaderboard ranks every holder of ONE item by quantity, so the
-- access path is "all rows for item_id, highest quantity first". The primary
-- key (user_id, item_id) cannot serve that; without this index each page is a
-- sequential scan of the whole table (50.332 ms at 228,841 rows).
CREATE INDEX IF NOT EXISTS idx_user_items_item_quantity
    ON user_items (item_id, quantity DESC);
