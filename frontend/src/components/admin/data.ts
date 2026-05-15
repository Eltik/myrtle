export interface IAdminUser {
    uid: string;
    name: string;
    server: "EN" | "JP" | "KR" | "CN";
    role: "user" | "tier_list_editor" | "tier_list_admin" | "super_admin";
    score: number;
    grade: string;
    last: string;
    level: number;
    pub: boolean;
}

export const ADMIN_USERS: IAdminUser[] = [
    { uid: "83014002", name: "Eltik", server: "EN", role: "super_admin", score: 98420, grade: "S+", last: "2 min ago", level: 120, pub: true },
    { uid: "82119910", name: "Myrrh", server: "JP", role: "tier_list_admin", score: 74012, grade: "S", last: "1 h ago", level: 118, pub: true },
    { uid: "81177321", name: "yarn_cat", server: "EN", role: "tier_list_editor", score: 68402, grade: "A+", last: "3 h ago", level: 112, pub: false },
    { uid: "82998441", name: "Saria", server: "KR", role: "tier_list_admin", score: 61108, grade: "A", last: "1 d ago", level: 109, pub: true },
    { uid: "83558211", name: "KuroNeko", server: "EN", role: "user", score: 54980, grade: "A", last: "2 d ago", level: 106, pub: true },
    { uid: "82001998", name: "PenguinLG", server: "EN", role: "user", score: 48211, grade: "A", last: "2 d ago", level: 104, pub: true },
    { uid: "81002201", name: "wisadel_2", server: "JP", role: "user", score: 41200, grade: "B+", last: "4 d ago", level: 97, pub: false },
    { uid: "83444010", name: "Mostima", server: "EN", role: "tier_list_editor", score: 33802, grade: "B", last: "1 wk ago", level: 94, pub: true },
];

export const SERVER_TINT: Record<string, string> = {
    EN: "linear-gradient(135deg, oklch(0.58 0.22 25), oklch(0.85 0.12 25))",
    JP: "linear-gradient(135deg, oklch(0.55 0.15 184), oklch(0.7 0.12 200))",
    KR: "linear-gradient(135deg, oklch(0.7 0.16 84), oklch(0.85 0.10 100))",
    CN: "linear-gradient(135deg, oklch(0.45 0.18 350), oklch(0.65 0.12 320))",
};

export interface ITierListRow {
    slug: string;
    type: string;
    active: boolean;
    owner: string;
    placements: number;
    version: number;
    perms: number;
}

export const TIER_LISTS: ITierListRow[] = [
    { slug: "meta-week-22", type: "meta", active: true, owner: "Eltik", placements: 184, version: 6, perms: 7 },
    { slug: "anniv-2026", type: "event", active: true, owner: "Saria", placements: 92, version: 12, perms: 4 },
    { slug: "is3-rogue", type: "mode", active: true, owner: "Myrrh", placements: 58, version: 3, perms: 3 },
    { slug: "cc-12", type: "challenge", active: false, owner: "yarn_cat", placements: 36, version: 8, perms: 5 },
    { slug: "low-rarity", type: "meta", active: true, owner: "KuroNeko", placements: 122, version: 2, perms: 2 },
];

export interface IPermissionRow {
    uid: string;
    name: string;
    level: "View" | "Edit" | "Publish" | "Admin";
    granted: string;
    by: string;
}

export const PERMS: IPermissionRow[] = [
    { uid: "82119910", name: "Myrrh", level: "Admin", granted: "12 days ago", by: "Eltik" },
    { uid: "81177321", name: "yarn_cat", level: "Publish", granted: "5 days ago", by: "Eltik" },
    { uid: "82998441", name: "Saria", level: "Edit", granted: "5 days ago", by: "Myrrh" },
    { uid: "83558211", name: "KuroNeko", level: "Edit", granted: "3 days ago", by: "Myrrh" },
    { uid: "82001998", name: "PenguinLG", level: "View", granted: "1 day ago", by: "Myrrh" },
];

export interface IOfficialList {
    slug: string;
    title: string;
    flair: string;
    flairLabel: string;
    version: number;
    published: string;
    placements: number;
    tiers: number;
    status: "live" | "draft";
}

export const OFFICIAL_LISTS: IOfficialList[] = [
    { slug: "official-meta", title: "Myrtle meta tier list", flair: "#D97757", flairLabel: "myrtle picks", version: 14, published: "today", placements: 412, tiers: 6, status: "live" },
    { slug: "official-cc-12", title: "CC#12 — Operation Pyrite", flair: "#2F6A4A", flairLabel: "event", version: 3, published: "3 d ago", placements: 88, tiers: 5, status: "live" },
    { slug: "official-is3", title: "IS3: Mizuki & Caerula Arbor", flair: "#4FB8B2", flairLabel: "rogue", version: 8, published: "1 wk ago", placements: 156, tiers: 7, status: "live" },
    { slug: "official-anniv-2026", title: "5th anniversary best-of", flair: "#F7A452", flairLabel: "spotlight", version: 2, published: "—", placements: 64, tiers: 5, status: "draft" },
    { slug: "official-low-rar", title: "High-impact low rarities", flair: "#88C8E3", flairLabel: "deep cut", version: 5, published: "2 wk ago", placements: 73, tiers: 5, status: "live" },
];

export interface ITierOp {
    id: string;
    name: string;
    rarity: number;
}

export interface ITierRow {
    id: string;
    name: string;
    color: string;
    ops: ITierOp[];
}

export const TIER_ROWS: ITierRow[] = [
    {
        id: "s",
        name: "S",
        color: "#dc4d56",
        ops: [
            { id: "char_4078_bdhkgt", name: "Texas2", rarity: 6 },
            { id: "char_002_amiya", name: "Amiya", rarity: 5 },
            { id: "char_1029_yato2", name: "Yato2", rarity: 6 },
            { id: "char_2024_chyue", name: "Chongyue", rarity: 6 },
            { id: "char_4144_choas", name: "Chen2", rarity: 6 },
            { id: "char_4087_eveb", name: "Eblana", rarity: 6 },
        ],
    },
    {
        id: "a",
        name: "A",
        color: "#e0834a",
        ops: [
            { id: "char_2023_ling", name: "Ling", rarity: 6 },
            { id: "char_4124_iana", name: "Iana", rarity: 5 },
            { id: "char_1023_ghost2", name: "Specter2", rarity: 6 },
            { id: "char_4014_lvdog", name: "Penance", rarity: 6 },
            { id: "char_4117_ray", name: "Ray", rarity: 6 },
        ],
    },
    {
        id: "b",
        name: "B",
        color: "#d8b54a",
        ops: [
            { id: "char_2014_nian", name: "Nian", rarity: 6 },
            { id: "char_241_panda", name: "Panda", rarity: 4 },
            { id: "char_113_cqbw", name: "W", rarity: 6 },
            { id: "char_4055_bgsnow", name: "Wisadel", rarity: 6 },
        ],
    },
    {
        id: "c",
        name: "C",
        color: "#5dbf86",
        ops: [
            { id: "char_2025_shu", name: "Shu", rarity: 6 },
            { id: "char_1037_amiya3", name: "Amiya3", rarity: 6 },
        ],
    },
    { id: "d", name: "D", color: "#5aa9d9", ops: [] },
    { id: "f", name: "F", color: "#8a8a8a", ops: [] },
];

export const POOL_OPS: ITierOp[] = [
    { id: "char_1028_texas2", name: "Texas", rarity: 6 },
    { id: "char_220_grani", name: "Grani", rarity: 5 },
    { id: "char_350_surtr", name: "Surtr", rarity: 6 },
    { id: "char_002_amiya", name: "Amiya", rarity: 5 },
    { id: "char_1013_chen2", name: "Chen", rarity: 6 },
    { id: "char_2024_chyue", name: "Chongyue", rarity: 6 },
    { id: "char_4078_bdhkgt", name: "Texas2", rarity: 6 },
    { id: "char_113_cqbw", name: "W", rarity: 6 },
    { id: "char_241_panda", name: "Panda", rarity: 4 },
    { id: "char_4087_eveb", name: "Eblana", rarity: 6 },
    { id: "char_4144_choas", name: "Chen2", rarity: 6 },
    { id: "char_4055_bgsnow", name: "Wisadel", rarity: 6 },
];

export const RARITY_BG: Record<number, string> = {
    6: "var(--rarity-6)",
    5: "var(--rarity-5)",
    4: "var(--rarity-4)",
    3: "var(--rarity-3)",
    2: "var(--rarity-2)",
    1: "var(--rarity-1)",
};

export interface IOperatorNote {
    id: string;
    op: string;
    rarity: number;
    branch: string;
    updated: string;
    by: string;
    audit: number;
    status: "published" | "draft";
}

export const NOTES: IOperatorNote[] = [
    { id: "char_4078_bdhkgt", op: "Texas the Omertosa", rarity: 6, branch: "Sword Master", updated: "2 h ago", by: "Myrrh", audit: 14, status: "published" },
    { id: "char_4124_iana", op: "Iana", rarity: 5, branch: "Heavyshooter", updated: "1 d ago", by: "yarn_cat", audit: 6, status: "published" },
    { id: "char_4087_eveb", op: "Eblana", rarity: 6, branch: "Loopshooter", updated: "1 d ago", by: "Eltik", audit: 22, status: "draft" },
    { id: "char_2024_chyue", op: "Chongyue", rarity: 6, branch: "Fighter", updated: "3 d ago", by: "Saria", audit: 31, status: "published" },
    { id: "char_4144_choas", op: "Chen Alter", rarity: 6, branch: "Swordmaster", updated: "4 d ago", by: "Myrrh", audit: 18, status: "published" },
];

export interface ICacheEntry {
    key: string;
    ttl: string;
    keys: number;
    hit: number;
    lat: number;
}

export const CACHE: ICacheEntry[] = [
    { key: "user:{uid}", ttl: "10 min", keys: 4128, hit: 94, lat: 0.8 },
    { key: "stats:global", ttl: "5 min", keys: 1, hit: 98, lat: 0.6 },
    { key: "static:{resource}:…", ttl: "30 min", keys: 14, hit: 99, lat: 0.5 },
    { key: "leaderboard:{sort}:…", ttl: "5 min", keys: 84, hit: 76, lat: 1.4 },
    { key: "search:{hash}", ttl: "2 min", keys: 312, hit: 62, lat: 0.9 },
    { key: "tierlist:{slug}", ttl: "10 min", keys: 22, hit: 91, lat: 1.0 },
    { key: "game_session:{uid}", ttl: "1 h", keys: 847, hit: 88, lat: 0.4 },
    { key: "portal_session:{uid}", ttl: "1 h", keys: 612, hit: 84, lat: 0.4 },
    { key: "gacha:global_stats", ttl: "5 min", keys: 1, hit: 99, lat: 0.7 },
];

export interface IAuditEntry {
    when: string;
    actor: string;
    action: string;
    target: string;
    level: string;
    severity: "info" | "success" | "warning" | "muted";
}

export const AUDIT_ROWS: IAuditEntry[] = [
    { when: "2 min ago", actor: "Eltik", action: "permissions.grant", target: "/tier-lists/meta-week-22 → KuroNeko", level: "Edit", severity: "info" },
    { when: "11 min ago", actor: "Myrrh", action: "operator-notes.update", target: "char_4078_bdhkgt", level: "+12 / -3", severity: "info" },
    { when: "38 min ago", actor: "Eltik", action: "permissions.revoke", target: "/tier-lists/cc-12 / yarn_cat", level: "Admin", severity: "warning" },
    { when: "1 h ago", actor: "system", action: "cache.invalidate", target: "static:operators", level: "auto", severity: "muted" },
    { when: "2 h ago", actor: "Saria", action: "tier-list.publish", target: "/tier-lists/anniv-2026 v12", level: "92 ops", severity: "success" },
    { when: "3 h ago", actor: "Eltik", action: "user.role.change", target: "Saria → tier_list_admin", level: "user → TL admin", severity: "info" },
    { when: "5 h ago", actor: "system", action: "yostar.session.evict", target: "8 sessions", level: "TTL", severity: "muted" },
    { when: "8 h ago", actor: "Eltik", action: "assets.reload", target: "v25.6.51", level: "412 operators", severity: "info" },
    { when: "yesterday", actor: "Myrrh", action: "operator-notes.publish", target: "char_4124_iana", level: "rev 6", severity: "success" },
    { when: "yesterday", actor: "yarn_cat", action: "tier-list.update", target: "/tier-lists/cc-12", level: "+4 placements", severity: "info" },
    { when: "2 d ago", actor: "Eltik", action: "user.session.revoke", target: "UID 81002201", level: "manual", severity: "warning" },
];
