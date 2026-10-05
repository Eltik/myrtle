import { useQuery } from "@tanstack/react-query";
import { storySpriteQueryOptions, storySpritesQueryOptions } from "#/lib/api/story";
import { useGamedataServer } from "#/lib/i18n";

/**
 * The gallery's list, fetched on the CLIENT: it is about 1 MB of JSON on EN
 * (145 KB gzipped), and a server-side prefetch would ship it twice, once as
 * the request and once dehydrated into the HTML.
 */
export function useSpriteIndex() {
    return useQuery(storySpritesQueryOptions(useGamedataServer()));
}

/** One folder's sheet. `base` empty means "nothing open" and fetches nothing. */
export function useSpriteDetail(base: string) {
    return useQuery({ ...storySpriteQueryOptions(base, useGamedataServer()), enabled: base !== "" });
}
