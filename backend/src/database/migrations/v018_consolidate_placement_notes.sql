-- Fold any remaining `notes` value into `description`, then drop `notes`.
UPDATE tier_placements
SET description = notes
WHERE description IS NULL
  AND notes IS NOT NULL;

ALTER TABLE tier_placements DROP COLUMN notes;
