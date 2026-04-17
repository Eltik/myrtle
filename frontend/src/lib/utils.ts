import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]): string {
    return twMerge(clsx(inputs));
}

type User = {
    secretary?: string | null;
    secretary_skin_id?: string | null;
} | null;

const DEFAULT_AVATAR = "/api/cdn/avatar/char_002_amiya";
const AVATAR_BASE_URL = "https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar";

/**
 * Normalize a skin ID for different contexts.
 */
function normalizeSkinId(skinId: string, options: { encode?: boolean } = {}): string {
    const normalized = skinId.replaceAll("@", "_").replaceAll("#", "_");

    return options.encode ? encodeURIComponent(normalized) : normalized;
}

/**
 * Normalize skin ID specifically for API routes.
 * Keeps "#" encoded instead of replaced when needed.
 */
function normalizeSkinIdForApi(skinId: string): string {
    if (skinId.includes("@")) {
        return skinId.replaceAll("@", "_").replaceAll("#", "%23");
    }

    return normalizeSkinId(skinId);
}

/**
 * Get GitHub avatar URL from a character/skin ID.
 */
export function getAvatarById(charId: string): string {
    const normalizedId = normalizeSkinId(charId, { encode: true });
    return `${AVATAR_BASE_URL}/${normalizedId}.png`;
}

/**
 * Resolve the correct skin ID for a user's secretary.
 */
function resolveSecretarySkinId(user: User): string | null {
    if (!user?.secretary) return null;

    const { secretary, secretary_skin_id = "" } = user;

    const isDefaultSkin = !secretary_skin_id?.includes("@") && secretary_skin_id?.endsWith("#1");

    return isDefaultSkin ? secretary : secretary_skin_id;
}

/**
 * Get avatar URL for a user's secretary.
 */
export function getSecretaryAvatarURL(user: User): string {
    const skinId = resolveSecretarySkinId(user);
    if (!skinId) return DEFAULT_AVATAR;

    const normalizedId = normalizeSkinIdForApi(skinId);
    return `/api/cdn/avatar/${normalizedId}`;
}

/**
 * Alias for backward compatibility.
 */
export const getAvatarSkinId = getSecretaryAvatarURL;
