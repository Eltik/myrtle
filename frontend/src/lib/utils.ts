import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import type { OperatorRarity } from "~/types/api";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

/**
 * Creates an array of numbers from start (inclusive) to end (exclusive).
 * Native replacement for lodash range function.
 *
 * @example
 * range(4) // => [0, 1, 2, 3]
 * range(1, 5) // => [1, 2, 3, 4]
 */
export function range(start: number, end?: number): number[] {
    if (end === undefined) {
        end = start;
        start = 0;
    }
    const length = Math.max(end - start, 0);
    return Array.from({ length }, (_, i) => start + i);
}

/**
 * Calculates the relative luminance of a hex color.
 * Uses the WCAG formula for calculating luminance.
 * @param hex - A hex color string (with or without #)
 * @returns A number between 0 (black) and 1 (white)
 */
export function getLuminance(hex: string): number {
    const color = hex.replace("#", "");

    const r = Number.parseInt(color.substring(0, 2), 16) / 255;
    const g = Number.parseInt(color.substring(2, 4), 16) / 255;
    const b = Number.parseInt(color.substring(4, 6), 16) / 255;

    const toLinear = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

    const rLinear = toLinear(r);
    const gLinear = toLinear(g);
    const bLinear = toLinear(b);

    return 0.2126 * rLinear + 0.7152 * gLinear + 0.0722 * bLinear;
}

/**
 * Returns the optimal text color (black or white) based on background color.
 * Uses WCAG luminance threshold for accessibility.
 * @param backgroundColor - A hex color string for the background
 * @returns "#000000" for light backgrounds, "#ffffff" for dark backgrounds
 */
export function getContrastTextColor(backgroundColor: string): string {
    const luminance = getLuminance(backgroundColor);
    // Threshold of 0.179 provides good contrast per WCAG guidelines
    return luminance > 0.179 ? "#000000" : "#ffffff";
}

export const rarityToNumber = (rarity: OperatorRarity): number => {
    switch (rarity) {
        case "TIER_6":
            return 6;
        case "TIER_5":
            return 5;
        case "TIER_4":
            return 4;
        case "TIER_3":
            return 3;
        case "TIER_2":
            return 2;
        case "TIER_1":
            return 1;
        default:
            return 0;
    }
};

export const formatProfession = (profession: string): string => {
    if (!profession) return "Guard";
    switch (profession.toLowerCase()) {
        case "pioneer":
            return "Vanguard";
        case "tank":
            return "Defender";
        case "sniper":
            return "Sniper";
        case "warrior":
            return "Guard";
        case "caster":
            return "Caster";
        case "support":
            return "Supporter";
        case "special":
            return "Specialist";
        case "medic":
            return "Medic";
        default:
            return profession;
    }
};

export const formatSubProfession = (subProfession: string): string => {
    switch (subProfession) {
        // Caster
        case "blastcaster":
            return "Blast Caster";
        case "chain":
            return "Chain Caster";
        case "corecaster":
            return "Core Caster";
        case "funnel":
            return "Mech-Accord Caster";
        case "mystic":
            return "Mystic Caster";
        case "phalanx":
            return "Phalanx Caster";
        case "primcaster":
            return "Primal Caster";
        case "soulcaster":
            return "Shaper Caster";
        case "splashcaster":
            return "Splash Caster";

        // Defender
        case "artsprotector":
            return "Arts Protector Defender";
        case "duelist":
            return "Duelist Defender";
        case "fortress":
            return "Fortress Defender";
        case "guardian":
            return "Guardian Defender";
        case "unyield":
            return "Juggernaught Defender";
        case "primprotector":
            return "Primal Protector Defender";
        case "protector":
            return "Protector Defender";
        case "shotprotector":
            return "Sentry Protector Defender";

        // Guard
        case "artsfghter":
            return "Arts Fighter Guard";
        case "centurion":
            return "Centurion Guard";
        case "crusher":
            return "Crusher Guard";
        case "fearless":
            return "Dreadnought Guard";
        case "hammer":
            return "Earthshaker Guard";
        case "fighter":
            return "Fighter Guard";
        case "instructor":
            return "Instructor Guard";
        case "librator":
            return "Liberator Guard";
        case "lord":
            return "Lord Guard";
        case "mercenary":
            return "Mercenary Guard";
        case "primguard":
            return "Primal Guard";
        case "reaper":
            return "Reaper Guard";
        case "musha":
            return "Soloblade Guard";
        case "sword":
            return "Swordmaster Guard";

        // Medic
        case "chainhealer":
            return "Chain Medic";
        case "incantationmedic":
            return "Incantation Medic";
        case "physician":
            return "Medic Medic";
        case "ringhealer":
            return "Multi-target Medic";
        case "healer":
            return "Therapist Medic";
        case "wandermedic":
            return "Wandering Medic";

        // Sniper
        case "aoesniper":
            return "Artilleryman Sniper";
        case "siegesniper":
            return "Besieger Sniper";
        case "longrange":
            return "Deadeye Sniper";
        case "bombarder":
            return "Flinger Sniper";
        case "closerange":
            return "Heavyshooter Sniper";
        case "hunter":
            return "Hunter Sniper";
        case "loopshooter":
            return "Loopshooter Sniper";
        case "fastshot":
            return "Marksman Sniper";
        case "skybreaker":
            return "Skybreaker Sniper";
        case "reaperrange":
            return "Spreadshooter Sniper";

        // Specialist
        case "alchemist":
            return "Alchemist Specialist";
        case "stalker":
            return "Ambusher Specialist";
        case "dollkeeper":
            return "Dollkeeper Specialist";
        case "executor":
            return "Executor Specialist";
        case "geek":
            return "Geek Specialist";
        case "hookmaster":
            return "Hookmaster Specialist";
        case "merchant":
            return "Merchant Specialist";
        case "pusher":
            return "Push Stroker Specialist";
        case "skywalker":
            return "Skyranger Specialist";
        case "traper":
            return "Trapmaster Specialist";

        // Supporter
        case "blessing":
            return "Abjurer Supporter";
        case "craftsman":
            return "Artificer Supporter";
        case "bard":
            return "Bard Supporter";
        case "slower":
            return "Decel Binder Supporter";
        case "underminer":
            return "Hexer Supporter";
        case "ritualist":
            return "Ritualist Supporter";
        case "summoner":
            return "Summoner Supporter";

        // Vanguard
        case "agent":
            return "Agent Vanguard";
        case "charger":
            return "Charger Vanguard";
        case "pioneer":
            return "Pioneer Vanguard";
        case "bearer":
            return "Standard Bearer Vanguard";
        case "counsellor":
            return "Strategist Vanguard";
        case "tactician":
            return "Tactician Vanguard";

        default:
            return subProfession;
    }
};

export const formatNationId = (nationId: string) => {
    switch (nationId.toLowerCase()) {
        case "rhodes":
            return "Rhodes Island";
        case "kazimierz":
            return "Kazimierz";
        case "columbia":
            return "Columbia";
        case "laterano":
            return "Laterano";
        case "victoria":
            return "Victoria";
        case "sami":
            return "Sami";
        case "bolivar":
            return "Bolivar";
        case "iberia":
            return "Iberia";
        case "siracusa":
            return "Siracusa";
        case "higashi":
            return "Higashi";
        case "sargon":
            return "Sargon";
        case "kjerag":
            return "Kjerag";
        case "minos":
            return "Minos";
        case "yan":
            return "Yan";
        case "lungmen":
            return "Lungmen";
        case "ursus":
            return "Ursus";
        case "egir":
            return "Ægir";
        case "leithanien":
            return "Leithanien";
        case "rim":
            return "Rim Billiton";
        default:
            return capitalize(nationId);
    }
};

export const formatGroupId = (groupId: string) => {
    switch (groupId.toLowerCase()) {
        case "pinus":
            return "Pinus Sylvestris";
        case "blacksteel":
            return "Blacksteel";
        case "karlan":
            return "Karlan Trade";
        case "sweep":
            return "S.W.E.E.P.";
        case "rhine":
            return "Rhine Lab";
        case "penguin":
            return "Penguin Logistics";
        case "siesta":
            return "Siesta";
        case "lgd":
            return "L.G.D.";
        case "glasgow":
            return "Glasgow Gang";
        case "abyssal":
            return "Abyssal Hunters";
        case "dublinn":
            return "Dublinn";
        case "elite":
            return "Elite Operators";
        case "sui":
            return "Yan Sui";
        default:
            return groupId;
    }
};

export const formatTeamId = (teamId: string) => {
    switch (teamId.toLowerCase()) {
        case "action4":
            return "Action Team A4";
        case "reserve1":
            return "Reserve Op Team A1";
        case "reserve4":
            return "Reserve Op Team A4";
        case "reserve6":
            return "Reserve Op Team A6";
        case "student":
            return "Ursus Student Self-Governing Group";
        case "chiave":
            return "Chiave's Gang";
        case "rainbow":
            return "Team Rainbow";
        case "followers":
            return "Followers";
        case "lee":
            return "Lee's Detective Agency";
        default:
            return capitalize(teamId);
    }
};

export function capitalize(s: string): string {
    if (!s) return "";
    return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}

/**
 * Normalizes a skin ID for use in avatar URLs.
 * Replaces "@" or "#" with "_" and URL encodes the result.
 */
export function normalizeSkinId(skinId: string): string {
    if (skinId.includes("@")) {
        return encodeURIComponent(skinId.replaceAll("@", "_"));
    }
    return encodeURIComponent(skinId.replaceAll("#", "_"));
}

/**
 * Returns the GitHub avatar URL for a given character/skin ID.
 */
export function getAvatarById(charId: string): string {
    return `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${normalizeSkinId(charId)}.png`;
}

/**
 * Gets the avatar URL for a user's secretary.
 * Handles both default skins (ending in #1) and purchased skins (containing @).
 * Uses the dynamic avatar route which looks up the correct directory.
 */
export function getAvatarSkinId(user: { status?: { secretary: string; secretarySkinId: string } } | null): string {
    return getSecretaryAvatarURL(user);
}

/**
 * Extracts the star count (1-6) from a rarity string like "TIER_6".
 * Returns 0 if the format doesn't match.
 */
export function getRarityStarCount(rarity: string): number {
    const match = rarity?.match(/TIER_(\d)/);
    return match ? Number.parseInt(match[1] ?? "", 10) : 0;
}

/**
 * Maps a profession code to its icon name for CDN paths.
 */
export function getProfessionIconName(profession: string): string {
    if (!profession) return "warrior";
    const iconMap: Record<string, string> = {
        WARRIOR: "warrior",
        SNIPER: "sniper",
        TANK: "tank",
        MEDIC: "medic",
        SUPPORT: "support",
        CASTER: "caster",
        SPECIAL: "special",
        PIONEER: "pioneer",
    };
    return iconMap[profession.toUpperCase()] ?? profession.toLowerCase() ?? "warrior";
}

/**
 * Gets the operator image URL based on character ID, skin, evolve phase, and template.
 * Handles different skin types: default skins, E2 skins, and custom skins.
 */
export function getOperatorImageURL(charId: string, skin: string, evolvePhase: number, currentTmpl?: string | null, tmpl?: Record<string, { skinId: string }> | null): string {
    let skinId = skin;

    if (currentTmpl && tmpl && tmpl[currentTmpl]) {
        skinId = tmpl[currentTmpl].skinId;
    }

    if (!skinId) {
        const suffix = evolvePhase >= 2 ? "_2" : "_1";
        return `/api/cdn/upk/chararts/${charId}/${charId}${suffix}.png`;
    }

    if (skinId.includes("@")) {
        const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "%23");
        return `/api/cdn/upk/skinpack/${charId}/${normalizedSkinId}.png`;
    }

    const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "_");
    return `/api/cdn/upk/chararts/${charId}/${normalizedSkinId}.png`;
}

/**
 * Gets the avatar URL for a user's secretary using the dynamic avatar route.
 * The backend will look up the correct directory from asset mappings.
 */
export function getSecretaryAvatarURL(user: { status?: { secretary: string; secretarySkinId: string } } | null): string {
    const DEFAULT_AVATAR = "/api/cdn/avatar/char_002_amiya";

    if (!user?.status) return DEFAULT_AVATAR;

    const secretaryId = user.status.secretary;
    const secretarySkinId = user.status.secretarySkinId;

    const skinId = !secretarySkinId.includes("@") && secretarySkinId.endsWith("#1") ? secretaryId : secretarySkinId;

    const normalizedSkinId = skinId.includes("@") ? skinId.replaceAll("@", "_").replaceAll("#", "%23") : skinId.replaceAll("@", "_").replaceAll("#", "_");

    return `/api/cdn/avatar/${normalizedSkinId}`;
}
