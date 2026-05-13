import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { env } from "#/env";
import type { OperatorRarity, OperatorRarityTier } from "#/types/operators";

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
export function toAvatarStem(id: string): string {
    if (id.includes("@")) {
        // Skin: char_X@winter#1 → char_X_winter#1
        return encodeURIComponent(id.replaceAll("@", "_"));
    }
    // Base art: char_X#1 → char_X, char_X#2 → char_X_2
    const stem = id.endsWith("#1") ? id.slice(0, -2) : id.replace(/#(\d+)$/, "_$1");
    return encodeURIComponent(stem);
}

export function getAvatarById(charId: string): string {
    return `${avatarBase()}/api/avatar/${toAvatarStem(charId)}`;
}

function resolveSecretarySkinId(user: User): string | null {
    if (!user?.secretary) return null;
    return user.secretary_skin_id || user.secretary;
}

export function getSecretaryAvatarURL(user: User): string {
    const skinId = resolveSecretarySkinId(user) ?? DEFAULT_SECRETARY_ID;
    return `${avatarBase()}/api/avatar/${toAvatarStem(skinId)}`;
}

export const getAvatarSkinId = getSecretaryAvatarURL;

export function rarityToNumber(rarity: OperatorRarityTier | string | null | undefined, fallback: OperatorRarity = 1): OperatorRarity {
    const n = Number(String(rarity ?? "").replace("TIER_", ""));
    return n >= 1 && n <= 6 ? (n as OperatorRarity) : fallback;
}

const NAME_SPLIT_REGEX = /( the )|\(/gi;

/** "Reed (the Flame Shadow)" → { displayName: "Reed", subtitle: "the Flame Shadow" }. */
export function parseOperatorName(name: string): { displayName: string; subtitle: string | null } {
    const parts = name.replace(/\)$/, "").split(NAME_SPLIT_REGEX);
    return { displayName: parts[0] ?? name, subtitle: parts[2] ?? null };
}

/** Locale number formatting - "1,234,567". */
export function formatNumber(n: number | null | undefined): string {
    return Number(n ?? 0).toLocaleString("en-US");
}

/** Brand-style compact: 1.2k / 2.3M (lowercase k, uppercase M). */
export function formatNumberCompact(n: number | null | undefined): string {
    const v = Number(n ?? 0);
    if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
    if (v >= 1_000) return `${(v / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
    return formatNumber(v);
}

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
            return "Juggernaut Defender";
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

export const SUB_PROFESSION_TO_PROFESSION: Record<string, string> = {
    blastcaster: "CASTER",
    chain: "CASTER",
    corecaster: "CASTER",
    funnel: "CASTER",
    mystic: "CASTER",
    phalanx: "CASTER",
    primcaster: "CASTER",
    soulcaster: "CASTER",
    splashcaster: "CASTER",

    artsprotector: "TANK",
    duelist: "TANK",
    fortress: "TANK",
    guardian: "TANK",
    unyield: "TANK",
    primprotector: "TANK",
    protector: "TANK",
    shotprotector: "TANK",

    artsfghter: "WARRIOR",
    centurion: "WARRIOR",
    crusher: "WARRIOR",
    fearless: "WARRIOR",
    hammer: "WARRIOR",
    fighter: "WARRIOR",
    instructor: "WARRIOR",
    librator: "WARRIOR",
    lord: "WARRIOR",
    mercenary: "WARRIOR",
    primguard: "WARRIOR",
    reaper: "WARRIOR",
    musha: "WARRIOR",
    sword: "WARRIOR",

    chainhealer: "MEDIC",
    incantationmedic: "MEDIC",
    physician: "MEDIC",
    ringhealer: "MEDIC",
    healer: "MEDIC",
    wandermedic: "MEDIC",

    aoesniper: "SNIPER",
    siegesniper: "SNIPER",
    longrange: "SNIPER",
    bombarder: "SNIPER",
    closerange: "SNIPER",
    hunter: "SNIPER",
    loopshooter: "SNIPER",
    fastshot: "SNIPER",
    skybreaker: "SNIPER",
    reaperrange: "SNIPER",

    alchemist: "SPECIAL",
    stalker: "SPECIAL",
    dollkeeper: "SPECIAL",
    executor: "SPECIAL",
    geek: "SPECIAL",
    hookmaster: "SPECIAL",
    merchant: "SPECIAL",
    pusher: "SPECIAL",
    skywalker: "SPECIAL",
    traper: "SPECIAL",

    blessing: "SUPPORT",
    craftsman: "SUPPORT",
    bard: "SUPPORT",
    slower: "SUPPORT",
    underminer: "SUPPORT",
    ritualist: "SUPPORT",
    summoner: "SUPPORT",

    agent: "PIONEER",
    charger: "PIONEER",
    pioneer: "PIONEER",
    bearer: "PIONEER",
    counsellor: "PIONEER",
    tactician: "PIONEER",
};

export const subProfessionToProfession = (subProfession: string): string => SUB_PROFESSION_TO_PROFESSION[subProfession] ?? "OTHER";

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

export const DEFAULT_AVATAR_ID = "char_002_amiya";

export function formatRelative(iso: string | null | undefined): string {
    if (!iso) return "—";
    const then = new Date(iso).getTime();
    if (Number.isNaN(then)) return "—";
    const diffMs = Date.now() - then;
    if (diffMs < 60_000) return "just now";
    const mins = Math.floor(diffMs / 60_000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    if (days === 1) return "yesterday";
    if (days < 7) return `${days}d ago`;
    const weeks = Math.floor(days / 7);
    if (weeks === 1) return "last week";
    if (weeks < 5) return `${weeks}w ago`;
    const months = Math.floor(days / 30);
    if (months < 12) return `${months}mo ago`;
    return `${Math.floor(days / 365)}y ago`;
}

export function lerpByLevel(level: number, maxLevel: number, base: number, max: number): number {
    if (maxLevel <= 1) return base;
    return Math.round(base + ((level - 1) * (max - base)) / (maxLevel - 1));
}

export function downloadBlob(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.style.display = "none";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 0);
}

export function loadImage(url: string, options?: { crossOrigin?: boolean }): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
        const img = new Image();
        if (options?.crossOrigin) img.crossOrigin = "anonymous";
        img.onload = () => resolve(img);
        img.onerror = () => reject(new Error(`Failed to load image: ${url}`));
        img.src = url;
    });
}

export function rarityGradient(rarity: number): string {
    switch (rarity) {
        case 6:
            return "linear-gradient(155deg,#f7d166,#f59e0b)";
        case 5:
            return "linear-gradient(155deg,#f7e79e,#d4b94a)";
        case 4:
            return "linear-gradient(155deg,#bcabdb,#8a72ad)";
        case 3:
            return "linear-gradient(155deg,#88c8e3,#5a9bbf)";
        case 2:
            return "linear-gradient(155deg,#7ef2a3,#4fc97a)";
        default:
            return "linear-gradient(155deg,#cfcfcf,#9a9a9a)";
    }
}

export const RARITY_HEX: Record<number, string> = {
    6: "#f7a452",
    5: "#f7e79e",
    4: "#bcabdb",
    3: "#88c8e3",
    2: "#7ef2a3",
    1: "#ffffff",
};

export const RARITY_HEX_MUTED: Record<number, string> = {
    ...RARITY_HEX,
    1: "#b5b5b5",
};

export const RARITY_LABELS: Record<number, string> = {
    6: "6★",
    5: "5★",
    4: "4★",
    3: "3★",
    2: "2★",
    1: "1★",
};

export const FALLBACK_TIER_COLORS = ["oklch(0.62 0.21 24)", "oklch(0.70 0.17 50)", "oklch(0.78 0.15 92)", "oklch(0.66 0.17 150)", "oklch(0.60 0.15 230)", "oklch(0.55 0.18 290)", "oklch(0.50 0.04 285)"] as const;

export function rarityStarColor(rarity: number): string {
    switch (rarity) {
        case 6:
            return "#f7a452";
        case 5:
            return "#f7e79e";
        case 4:
            return "#bcabdb";
        case 3:
            return "#88c8e3";
        case 2:
            return "#7ef2a3";
        default:
            return "#b5b5b5";
    }
}
