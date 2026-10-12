/**
 * The `server:id` card id the grid image introduced, for any kind keyed by one
 * game-data id: the bare id on the default server, `cn:char_x` elsewhere, so
 * an English URL carries no server segment.
 */

import { DEFAULT_GAMEDATA_SERVER, type GamedataServer, isGamedataServer } from "#/lib/api/gamedata";

export function serverScopedOgId(id: string, server: string = DEFAULT_GAMEDATA_SERVER): string {
    return !isGamedataServer(server) || server === DEFAULT_GAMEDATA_SERVER ? id : `${server}:${id}`;
}

/** The inverse of {@link serverScopedOgId}: an unknown prefix is part of the id. */
export function parseServerScopedOgId(raw: string): { server: GamedataServer; id: string } {
    const at = raw.indexOf(":");
    if (at > 0) {
        const prefix = raw.slice(0, at);
        if (isGamedataServer(prefix)) return { server: prefix, id: raw.slice(at + 1) };
    }
    return { server: DEFAULT_GAMEDATA_SERVER, id: raw };
}
