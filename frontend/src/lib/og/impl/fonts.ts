import type { SatoriOptions } from "satori";
import { OG_CONFIG } from "./config";

type FontInput = NonNullable<SatoriOptions["fonts"]>[number];

const FONT_BASE = `${OG_CONFIG.siteURL.replace(/\/$/, "")}/fonts`;

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
