import { type IPreparedQuery, type IPreparedTarget, type IScoreTarget, prepareQuery, prepareTarget, scorePrepared } from "#/lib/search/fuzzy";
import type { MusicKind } from "#/types/generated/MusicKind";
import type { MusicTrack } from "#/types/generated/MusicTrack";

export type MusicKindFilter = "all" | MusicKind;

/** The filter strip, in the order the game's own soundtrack splits: fights, menus, story, maps. */
export const MUSIC_KIND_FILTERS: readonly MusicKindFilter[] = ["all", "battle", "lobby", "story", "map"];

/** The name a row leads with: the game's title, else the clip's own name. */
export function displayName(track: Pick<MusicTrack, "title" | "id">): string {
    return track.title ?? track.id;
}

/**
 * What a row is searched by. The title scores on the name tiers; the origin
 * and every named place are aliases, so "Near Light" finds the event's
 * tracks and "Operation Pyrite" its season's. The clip id and the bank names
 * sit in `extra`, which is substring only, so `act27side` still finds a
 * track without a subsequence dragging unrelated ids in.
 */
export function trackSearchTarget(track: MusicTrack): IScoreTarget {
    const aliases: string[] = [];
    const add = (name: string | undefined) => {
        if (name && !aliases.includes(name)) aliases.push(name);
    };
    add(track.origin);
    for (const place of track.places) add(place.name);
    return { name: displayName(track), aliases, extra: [track.id, ...track.banks].join(" ") };
}

export interface IMusicSearch {
    tracks: readonly MusicTrack[];
    prepared: readonly IPreparedTarget[];
}

export function prepareMusicSearch(tracks: readonly MusicTrack[]): IMusicSearch {
    return { tracks, prepared: tracks.map((t) => prepareTarget(trackSearchTarget(t))) };
}

/**
 * The fuzzy score folded into tiers, the cut points the sprite gallery uses
 * for the same scorer, so a track CALLED the query leads the ones that merely
 * play in a place called it.
 */
function tier(score: number): number {
    if (score >= 1000) return 5;
    if (score >= 600) return 4;
    if (score >= 400) return 3;
    if (score >= 250) return 2;
    if (score >= 150) return 1;
    return 0;
}

/**
 * Keep the kind the filter asks for and match the query. Inside a tier a
 * titled track comes before an untitled one, then playable before
 * unavailable, then by name.
 */
export function filterTracks(search: IMusicSearch, query: string, kind: MusicKindFilter, collator: Intl.Collator): MusicTrack[] {
    const q: IPreparedQuery | null = query.trim() === "" ? null : prepareQuery(query);
    const kept: { track: MusicTrack; tier: number }[] = [];
    search.tracks.forEach((track, at) => {
        if (kind !== "all" && !track.kinds.includes(kind)) return;
        if (q === null) {
            kept.push({ track, tier: 0 });
            return;
        }
        const target = search.prepared[at];
        const score = target ? scorePrepared(q, target) : 0;
        if (score > 0) kept.push({ track, tier: tier(score) });
    });
    const titled = (t: MusicTrack) => (t.title ? 0 : 1);
    const playable = (t: MusicTrack) => (t.loopUrl ? 0 : 1);
    kept.sort((a, b) => b.tier - a.tier || titled(a.track) - titled(b.track) || playable(a.track) - playable(b.track) || collator.compare(displayName(a.track), displayName(b.track)));
    return kept.map((k) => k.track);
}

/** The file name a downloaded clip is saved under: the game's own clip name. */
export function clipFileName(url: string): string {
    const last = url.split("/").pop() ?? "clip.ogg";
    return last.toLowerCase().endsWith(".ogg") ? last : `${last}.ogg`;
}
