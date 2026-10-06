import { useQuery } from "@tanstack/react-query";
import { tierEntityCatalogueQueryOptions } from "#/lib/api/tier-lists";
import { useGamedataServer } from "#/lib/i18n";

/**
 * Whether operator `id` has elite 2 art, read off the operator catalogue the art browser
 * already loads (CN's too, for an operator this server has not released). Elite 2 is
 * four stars and up: in `character_table` on 2026-10-06 every obtainable operator of four
 * stars or more lists three phases and none of three or fewer does (EN 376 and 31, CN 395
 * and 34, no exception), and the backend refuses elite 2 by the phases themselves.
 * `null` while neither catalogue knows the operator.
 */
export function useHasElite2(id: string | null): boolean | null {
    const server = useGamedataServer();
    const own = useQuery({ ...tierEntityCatalogueQueryOptions("operator", server), enabled: id !== null });
    const cn = useQuery({ ...tierEntityCatalogueQueryOptions("operator", "cn"), enabled: id !== null });
    if (id === null) return null;
    const entry = own.data?.find((e) => e.id === id) ?? cn.data?.find((e) => e.id === id);
    const rarity = entry?.facets.rarity;
    const stars = Number(Array.isArray(rarity) ? rarity[0] : rarity);
    return entry && Number.isFinite(stars) ? stars >= 4 : null;
}
