--
-- The override layer learns names. An announced Global date usually arrives
-- with the event's English name, months before EN gamedata carries the id, so
-- an override row can carry one; it is shown as such
-- (AutoNameSource::Override), never as gamedata. Written by the admin PUT route.
--

ALTER TABLE public.release_overrides ADD COLUMN en_name text;
