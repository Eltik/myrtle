--
-- Rate-up rosters for banners whose roster the CN client does not carry
-- (standard, joint-operation and kernel banners; only limited, pick and
-- kernel-fest banners have theirs in gamedata), entered as operator ids on
-- the override row through the admin PUT route.
--

ALTER TABLE public.release_overrides ADD COLUMN featured_chars text[];
