import { env } from "#/env";
import { gamedataPath } from "#/lib/api/gamedata";

/**
 * One listed expression as the backend's composed thumb
 * (`GET /story/sprites/{base}/thumb/{key}`): body plus face patch, 320 px
 * high, the whole plate, so it drops into the same square box `Body` fills.
 * The key is URL-encoded (`#2$1` -> `%232%241`).
 */
export function spriteThumbUrl(base: string, key: string, server: string): string {
    return `${env.VITE_BACKEND_URL}/api${gamedataPath(server, `/story/sprites/${encodeURIComponent(base)}/thumb/${encodeURIComponent(key)}`)}`;
}
