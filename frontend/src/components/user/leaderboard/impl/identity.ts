import { getAvatarById } from "#/lib/utils";
import { DEFAULT_AVATAR_ID } from "./constants";

export interface IPlayerLike {
    uid: string;
    nickname?: string | null;
    avatar_id?: string | null;
}

/** What every row and card shows for a player: a display name that never
 * blanks, two-letter initials for the avatar fallback, and the avatar URL. */
export function playerIdentity(player: IPlayerLike): { nickname: string; initials: string; avatarSrc: string } {
    const nickname = player.nickname ?? `Player ${player.uid}`;
    return {
        nickname,
        initials: nickname.slice(0, 2).toUpperCase(),
        avatarSrc: getAvatarById(player.avatar_id ?? DEFAULT_AVATAR_ID),
    };
}
