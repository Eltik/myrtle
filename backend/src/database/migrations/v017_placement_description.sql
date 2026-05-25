-- Per-placement editor description: a free-form blurb explaining why an
-- operator sits where it does on the tier list.
ALTER TABLE tier_placements ADD COLUMN description TEXT;
