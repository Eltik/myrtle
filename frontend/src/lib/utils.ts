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

export function getAvatarById(charId: string, server?: string): string {
    const prefix = server ? `/api/${server}` : "/api";
    return `${avatarBase()}${prefix}/avatar/${toAvatarStem(charId)}`;
}

/**
 * Operator portrait URL. Pass `server` (e.g. "cn") to read a non-default
 * server's assets via `/api/{server}/portrait/...`.
 */
export function getPortraitById(charId: string, server?: string): string {
    const prefix = server ? `/api/${server}` : "/api";
    return `${avatarBase()}${prefix}/portrait/${toAvatarStem(charId)}`;
}

function resolveSecretarySkinId(user: User): string | null {
    if (!user?.secretary) return null;
    return user.secretary_skin_id || user.secretary;
}

export function getSecretaryAvatarURL(user: User): string {
    const skinId = resolveSecretarySkinId(user) ?? DEFAULT_AVATAR_ID;
    return `${avatarBase()}/api/avatar/${toAvatarStem(skinId)}`;
}

export const getAvatarSkinId = getSecretaryAvatarURL;

export function rarityToNumber(rarity: OperatorRarityTier | OperatorRarity | string | number | null | undefined, fallback: OperatorRarity = 1): OperatorRarity {
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

/** Format a fraction in [0, 1] as a percentage, using more decimals for smaller
 *  shares so tiny values stay legible without over-reporting larger ones. */
export function formatSharePct(fraction: number): string {
    const p = fraction * 100;
    if (p >= 10) return `${p.toFixed(0)}%`;
    if (p >= 1) return `${p.toFixed(1)}%`;
    if (p > 0) return `${p.toFixed(2)}%`;
    return "0%";
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

/**
 * Bare archetype names, keyed by subProfessionId, as the game itself labels them.
 * SOURCE: assets/output/en/gamedata/excel/uniequip_table.json, SubProfDict[id].SubProfessionName,
 * with the placeholder ids (notchar*, none*) dropped. Caster names carry "Caster" in the
 * game data; every other class is bare. Regenerate this table from that file after a data
 * update. Both `supportiveranger` and `watchman` ARE in uniequip_table.SubProfDict (watchman on
 * EN and CN, supportiveranger on CN); they keep their last
 * known names so older exports still resolve.
 */
export const SUB_PROFESSION_NAMES: Record<string, string> = {
    agent: "Agent",
    alchemist: "Alchemist",
    aoesniper: "Artilleryman",
    artsfghter: "Arts Fighter",
    artsprotector: "Arts Protector",
    bard: "Bard",
    bearer: "Standard Bearer",
    blastcaster: "Blast Caster",
    blessing: "Abjurer",
    bombarder: "Flinger",
    centurion: "Centurion",
    chain: "Chain Caster",
    chainhealer: "Chain Medic",
    charger: "Charger",
    closerange: "Heavyshooter",
    corecaster: "Core Caster",
    counsellor: "Strategist",
    craftsman: "Artificer",
    crusher: "Crusher",
    dollkeeper: "Dollkeeper",
    duelist: "Duelist",
    executor: "Executor",
    fastshot: "Marksman",
    fearless: "Dreadnought",
    fighter: "Fighter",
    fortress: "Fortress",
    funnel: "Mech-accord Caster",
    geek: "Geek",
    guardian: "Guardian",
    hammer: "Earthshaker",
    healer: "Therapist",
    hookmaster: "Hookmaster",
    hunter: "Hunter",
    incantationmedic: "Incantation Medic",
    instructor: "Instructor",
    librator: "Liberator",
    longrange: "Deadeye",
    loopshooter: "Loopshooter",
    lord: "Lord",
    mercenary: "Mercenary",
    merchant: "Merchant",
    musha: "Soloblade",
    mystic: "Mystic Caster",
    phalanx: "Phalanx Caster",
    physician: "Medic",
    pioneer: "Pioneer",
    primcaster: "Primal Caster",
    primguard: "Primal Guard",
    primprotector: "Primal Protector",
    protector: "Protector",
    pusher: "Push Stroker",
    reaper: "Reaper",
    reaperrange: "Spreadshooter",
    ringhealer: "Multi-target Medic",
    ritualist: "Ritualist",
    shotprotector: "Sentry Protector",
    siegesniper: "Besieger",
    skybreaker: "Skybreaker",
    skywalker: "Skyranger",
    slower: "Decel Binder",
    soulcaster: "Shaper Caster",
    splashcaster: "Splash Caster",
    stalker: "Ambusher",
    summoner: "Summoner",
    supportiveranger: "Supportive Ranger",
    sword: "Swordmaster",
    tactician: "Tactician",
    traper: "Trapmaster",
    underminer: "Hexer",
    unyield: "Juggernaut",
    wandermedic: "Wandering Medic",
    watchman: "Watchman Medic",
};

/** The bare archetype name ("Centurion", "Medic", "Blast Caster"); the id itself when unknown. */
export const formatArchetype = (subProfession: string): string => SUB_PROFESSION_NAMES[subProfession] ?? subProfession;

/**
 * Archetype with its class appended ("Centurion Guard", "Blast Caster"). The class is
 * skipped when the game's name already ends with it, and when the name IS the class
 * (physician -> "Medic", not "Medic Medic"). Unknown ids come back unchanged so a new
 * archetype after a data update shows as a raw id instead of hiding behind a guess.
 */
export const formatSubProfession = (subProfession: string): string => {
    const name = SUB_PROFESSION_NAMES[subProfession];
    if (!name) return subProfession;
    const profession = subProfessionToProfession(subProfession);
    if (profession === "OTHER") return name;
    const cls = formatProfession(profession);
    return name === cls || name.endsWith(` ${cls}`) ? name : `${name} ${cls}`;
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
    watchman: "MEDIC",

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
    supportiveranger: "SUPPORT",

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
    if (!iso) return "-";
    const then = new Date(iso).getTime();
    if (Number.isNaN(then)) return "-";
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

/**
 * Verbose "5 min ago / 3 h ago / 2 d ago" style used by admin dashboards and
 * the settings page. Falls back to a locale date for anything older than 30
 * days. Accepts either an ISO timestamp or a unix epoch (seconds or ms).
 */
export function formatRelativeShort(input: string | number | null | undefined): string {
    if (input == null) return "-";
    let ms: number;
    if (typeof input === "number") {
        ms = input > 1e12 ? input : input * 1000;
    } else {
        ms = Date.parse(input);
    }
    if (!Number.isFinite(ms)) return typeof input === "string" ? input : "-";
    const diff = (Date.now() - ms) / 1000;
    if (diff < 0) return "just now";
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} h ago`;
    if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} d ago`;
    return new Date(ms).toLocaleDateString();
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

/** A loaded texture source: the bare element, or its pixels already decoded off-thread. */
export type DecodedImage = HTMLImageElement | ImageBitmap;

/** Load a texture image with its pixels decoded off the main thread. A browser decodes a
 *  PNG lazily at its first draw and keeps that decode only per draw size, so a loader that
 *  reads pixels inside `onload`, or a texture upload that follows it, pays the whole decode
 *  on the main thread, once per distinct draw: traced at 4x CPU throttling, one dynamic skin
 *  open carried 1176 ms of "Decode Image" on the main thread inside onload tasks of 574,
 *  572 and 235 ms (docs/DYNCHAR_GATES.md, "PERFORMANCE, FIRST RUN"). Neither
 *  `HTMLImageElement.decode()` nor `createImageBitmap(element)` moved them (both traced
 *  decoding inside the calling task). `createImageBitmap(blob)` is the platform's off-thread
 *  decode, the path PixiJS's own bitmap loader takes; the bitmap's pixels then serve canvas
 *  reads and the GPU upload without another decode. Any failure on that path falls back to
 *  the element loader, whose error is the one callers report. `?syncdecode=1` takes the
 *  element path directly for A/B measurement. */
export function loadDecoded(url: string, what: string): Promise<DecodedImage> {
    const element = () =>
        new Promise<HTMLImageElement>((resolve, reject) => {
            const img = new Image();
            img.crossOrigin = "anonymous";
            img.onload = () => resolve(img);
            img.onerror = () => reject(new Error(`Failed to load ${what}: ${url}`));
            img.src = url;
        });
    if (typeof createImageBitmap !== "function" || typeof fetch !== "function" || syncDecodeOn()) return element();
    return (
        fetch(url, { mode: "cors" })
            .then((r) => {
                if (!r.ok) throw new Error(`${r.status}`);
                return r.blob();
            })
            // Two bitmaps from one decode. The unpremultiplied one holds the PNG's own bytes,
            // so a canvas read sees what it saw from the element (a premultiplied bitmap read
            // back through a canvas is rounded twice, which moved 71 of 73 settled parity rows
            // by 1..5 levels on soft edges). WebGL uploads a bitmap's bytes as they are, ignoring
            // the premultiply unpack flag, so the upload needs the premultiplied twin, derived
            // from the first bitmap without a second decode (see uploadSource).
            .then((blob) => createImageBitmap(blob, { premultiplyAlpha: "none" }))
            .then((raw) =>
                createImageBitmap(raw, { premultiplyAlpha: "premultiply" }).then((pma) => {
                    uploadTwin.set(raw, pma);
                    return raw;
                }),
            )
            .catch(element)
    );
}

const uploadTwin = new WeakMap<ImageBitmap, ImageBitmap>();

/** The source to upload to the GPU for a decoded image: the element itself, or the
 *  premultiplied twin of a bitmap made by loadDecoded (the bitmap itself if it has none). */
export function uploadSource(img: DecodedImage): DecodedImage {
    return img instanceof HTMLImageElement ? img : (uploadTwin.get(img) ?? img);
}

/** Pixel size of a decoded source (an element reports its natural size, a bitmap its own). */
export function decodedSize(img: DecodedImage): [number, number] {
    if (img instanceof HTMLImageElement) return [img.naturalWidth || img.width, img.naturalHeight || img.height];
    return [img.width, img.height];
}

function syncDecodeOn(): boolean {
    if (typeof window === "undefined") return false;
    return new URLSearchParams(window.location.search).get("syncdecode") === "1";
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
    return RARITY_HEX_MUTED[rarity] ?? "#b5b5b5";
}
