import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { env } from "#/env";

export function cn(...inputs: ClassValue[]): string {
    return twMerge(clsx(inputs));
}

type User = {
    secretary?: string | null;
    secretary_skin_id?: string | null;
} | null;

const DEFAULT_SECRETARY_ID = "char_002_amiya";

function avatarBase(): string {
    return env.VITE_BACKEND_URL ?? "";
}

// Skin IDs from game data look like `char_002_amiya@winter#1`.
// Backend asset stems use `_` where game data uses `@`, and keep `#`.
// encodeURIComponent handles `#` → `%23` so it survives the URL path.
function toAvatarStem(id: string): string {
    return encodeURIComponent(id.replaceAll("@", "_"));
}

export function getAvatarById(charId: string): string {
    return `${avatarBase()}/api/avatar/${toAvatarStem(charId)}`;
}

function resolveSecretarySkinId(user: User): string | null {
    if (!user?.secretary) return null;

    const { secretary, secretary_skin_id = "" } = user;
    const isDefaultSkin = !secretary_skin_id?.includes("@") && secretary_skin_id?.endsWith("#1");

    return isDefaultSkin ? secretary : secretary_skin_id;
}

export function getSecretaryAvatarURL(user: User): string {
    const skinId = resolveSecretarySkinId(user) ?? DEFAULT_SECRETARY_ID;
    return `${avatarBase()}/api/avatar/${toAvatarStem(skinId)}`;
}

export const getAvatarSkinId = getSecretaryAvatarURL;
