import type { SatoriOptions } from "satori";
import { OG_CONFIG } from "./config";

type FontInput = NonNullable<SatoriOptions["fonts"]>[number];

const FONT_BASE = `${OG_CONFIG.siteURL.replace(/\/$/, "")}/fonts`;

// Order matters: satori resolves each glyph against these in turn, so the Latin
// faces stay first and the wider-coverage ones act as fallbacks. Operator names
// come straight from gamedata, so a CN-only operator's name is Han (and a few
// are Cyrillic) -- without these the card renders tofu boxes where the name goes.
const FONTS = [
    {
        name: "Inter",
        weight: 400 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/inter-latin-400-normal.woff`,
    },
    {
        name: "Inter",
        weight: 700 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/inter-latin-700-normal.woff`,
    },
    {
        name: "Inter Cyrillic",
        weight: 400 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/inter-cyrillic-400-normal.woff`,
    },
    {
        name: "Inter Cyrillic",
        weight: 700 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/inter-cyrillic-700-normal.woff`,
    },
    {
        name: "Noto Sans SC",
        weight: 400 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/noto-sans-sc-chinese-simplified-400-normal.woff`,
    },
    {
        name: "Noto Sans SC",
        weight: 700 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/noto-sans-sc-chinese-simplified-700-normal.woff`,
    },
    {
        name: "Geist Mono",
        weight: 500 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/geist-mono-latin-500-normal.woff`,
    },
    {
        name: "Geist Mono",
        weight: 700 as const,
        style: "normal" as const,
        source: `${FONT_BASE}/geist-mono-latin-700-normal.woff`,
    },
];

let cache: Promise<FontInput[]> | null = null;

export function getFonts(): Promise<FontInput[]> {
    if (!cache) cache = loadFonts();
    return cache;
}

async function loadFonts(): Promise<FontInput[]> {
    return Promise.all(
        FONTS.map(async (font) => {
            const data = await loadSource(font.source);
            return { name: font.name, data, weight: font.weight, style: font.style };
        }),
    );
}

async function loadSource(source: string): Promise<ArrayBuffer> {
    const res = await fetch(source);
    if (!res.ok) throw new Error(`Failed to fetch font ${source}: ${res.status}`);
    return res.arrayBuffer();
}
